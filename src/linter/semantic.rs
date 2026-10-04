//! Semantic validation for web output — the studio's compile-gate (M4.E).
//!
//! The lexer and parser catch *syntax*, but codegen is permissive and never
//! fails: a program that parses can still reference a component that was never
//! declared, point a `Route` at a page that does not exist, or declare the same
//! page twice. All of these compile to broken output that only fails at *runtime*
//! in the browser. [`validate_semantics`] turns those silent breaks into precise
//! [`Diagnostic`]s (line/column taken from the offending node's span) so the
//! studio can gate a compile — and reject a bad AI edit — before it ever reaches
//! the preview.
//!
//! **Scope (M4.E).** Three structural checks that are precise and
//! false-positive-free:
//! 1. **Undefined component** — a `UserDefined` element whose name matches no
//!    declared `Component`.
//! 2. **Unknown route target** — a `Route(page: X)` where `X` names no declared
//!    `Page`.
//! 3. **Duplicate declarations** — two same-kind `Page`s or two same-kind
//!    `Component`s sharing a name (they collide in codegen and in the node map).
//!    A `Page` and a `Component` may legally share a name and are not flagged.
//!
//! **Out of scope (deliberately).** Unresolved *identifiers* (a `Text(missingVar)`
//! that names no state/derived/prop/param) are not checked here: `Expr` carries no
//! span (so a diagnostic could not point at the identifier), and full scope
//! resolution is false-positive-prone — wrongly rejecting a valid program at the
//! gate is worse than missing one. A runtime `ReferenceError`, surfaced by the
//! preview's runtime-error bridge, is the backstop for those.

use crate::error::Diagnostic;
use crate::parser::ast::{Arg, Expr, Span};
use crate::parser::{ComponentRef, Declaration, Program, Statement, StatementKind, UIElement};
use std::collections::HashSet;

/// Run every semantic check over a parsed program, returning one [`Diagnostic`]
/// per problem (empty = clean), in declaration order.
///
/// `file` labels each diagnostic's source. The studio compiles from a single
/// merged source and passes that name; it maps the reported (merged) line back to
/// the real file itself.
pub fn validate_semantics(program: &Program, file: &str) -> Vec<Diagnostic> {
    validate_semantics_in(program, &|_| file.to_string())
}

/// [`validate_semantics`] for a program merged from several files: `file_of`
/// names the file the declaration at that index came from.
pub fn validate_semantics_in(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<Diagnostic> {
    let pages = name_set(program, DeclKind::Page);
    let components = name_set(program, DeclKind::Component);
    let mut diags = Vec::new();

    check_duplicate_names(program, file_of, &mut diags);
    check_one_meaning(program, file_of, &mut diags);
    check_script_names(program, file_of, &mut diags);

    for (index, decl) in program.declarations.iter().enumerate() {
        let body = match decl {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            // Neither holds UI.
            Declaration::Store(_)
            | Declaration::Theme(_)
            | Declaration::Type(_)
            | Declaration::Enum(_)
            | Declaration::Api(_)
            | Declaration::Script(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => continue,
        };
        walk_stmts(body, &pages, &components, &file_of(index), &mut diags);
    }

    check_routes(program, &pages, file_of, &mut diags);

    diags
}

#[derive(Clone, Copy)]
enum DeclKind {
    Page,
    Component,
}

/// The set of declared names for one declaration kind.
fn name_set(program: &Program, kind: DeclKind) -> HashSet<&str> {
    program
        .declarations
        .iter()
        .filter_map(|d| match (kind, d) {
            (DeclKind::Page, Declaration::Page(p)) => Some(p.name.as_str()),
            (DeclKind::Component, Declaration::Component(c)) => Some(c.name.as_str()),
            _ => None,
        })
        .collect()
}

/// Flag the second and later declaration of any same-kind name.
fn check_duplicate_names(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    diags: &mut Vec<Diagnostic>,
) {
    let pages = program
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(i, d)| match d {
            Declaration::Page(p) => Some((p.name.as_str(), p.header_span, i)),
            _ => None,
        });
    check_dupes(pages, "page", file_of, diags);

    let components = program
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(i, d)| match d {
            Declaration::Component(c) => Some((c.name.as_str(), c.header_span, i)),
            _ => None,
        });
    check_dupes(components, "component", file_of, diags);
}

/// One name, one meaning. What exists when the page runs — a store, a
/// constant, a `data` file, an image, an `api` — shares one namespace, as
/// every one of them is a `const` of the bundle; a `type` and an `enum`
/// share another; and what a page, a component or a store declares — its
/// props, states, deriveds, actions, resources and connections — shares the
/// body's. Two of one name compiled to two `const`s of it, which the browser
/// refuses before the page draws, or to an object where the second silently
/// replaced the first. A component may share its name with a `type` (a
/// call reads as one or the other by where it is written); two components
/// of one name are the duplicate check's.
fn check_one_meaning(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    diags: &mut Vec<Diagnostic>,
) {
    let values = program
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(i, d)| {
            let (name, kind, span) = match d {
                Declaration::Store(s) => (&s.name, "a store", s.header_span),
                Declaration::Const(c) => (&c.name, "a `const`", c.span),
                Declaration::Data(d) if d.is_image => (&d.name, "an `image`", d.span),
                Declaration::Data(d) => (&d.name, "a `data` constant", d.span),
                Declaration::Api(a) => (&a.name, "an `api`", a.span),
                _ => return None,
            };
            Some((name.as_str(), kind, span, i))
        });
    report_clashes(values, file_of, diags);
    let types = program
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(i, d)| {
            let (name, kind, span) = match d {
                Declaration::Type(t) => (&t.name, "a `type`", t.header_span),
                Declaration::Enum(e) => (&e.name, "an `enum`", e.header_span),
                _ => return None,
            };
            Some((name.as_str(), kind, span, i))
        });
    report_clashes(types, file_of, diags);

    // A component named like a built-in would never be drawn: a call of that
    // name is the built-in's. It used to be accepted and quietly shadowed.
    for (index, decl) in program.declarations.iter().enumerate() {
        if let Declaration::Component(c) = decl {
            if crate::registry::component(&c.name).is_some() {
                diags.push(
                    diag(
                        "V11",
                        format!("`{}` is a built-in component, so a call of that name draws the built-in, never this one", c.name),
                        &file_of(index),
                        c.header_span,
                    )
                    .with_hint(format!("Rename it — `My{0}`, `Report{0}` — and its call sites", c.name)),
                );
            }
        }
    }

    for (index, decl) in program.declarations.iter().enumerate() {
        let (props, prop_kind, body): (&[crate::parser::ast::PropDecl], _, &[Statement]) =
            match decl {
                Declaration::Page(p) => (&p.params, "a parameter", &p.body),
                Declaration::Component(c) => (&c.props, "a prop", &c.body),
                Declaration::Store(s) => (&[], "", &s.body),
                Declaration::App(a) => (&[], "", &a.body),
                _ => continue,
            };
        let props = props
            .iter()
            .map(|p| (p.name.as_str(), prop_kind, p.span, index));
        let own = body.iter().filter_map(|stmt| {
            let (name, kind) = match &stmt.kind {
                StatementKind::State(s) if s.persist => (&s.name, "a `persist`"),
                StatementKind::State(s) => (&s.name, "a state"),
                StatementKind::Derived(d) => (&d.name, "a `derived`"),
                StatementKind::Action(a) => (&a.name, "an action"),
                StatementKind::Resource(r) => (&r.name, "a `resource`"),
                StatementKind::Connection(c) => (&c.name, "a connection"),
                _ => return None,
            };
            Some((name.as_str(), kind, stmt.span, index))
        });
        report_clashes(props.chain(own), file_of, diags);
    }
}

/// Flag the second and later of any name in one namespace, naming what the
/// first one was and where it is.
fn report_clashes<'a>(
    items: impl Iterator<Item = (&'a str, &'static str, Span, usize)>,
    file_of: &dyn Fn(usize) -> String,
    diags: &mut Vec<Diagnostic>,
) {
    let mut seen: std::collections::HashMap<&str, (&str, Span, usize)> =
        std::collections::HashMap::new();
    for (name, kind, span, index) in items {
        let Some(&(first_kind, first_span, first_index)) = seen.get(name) else {
            seen.insert(name, (kind, span, index));
            continue;
        };
        let file = file_of(index);
        let first_file = file_of(first_index);
        let at = if first_file == file {
            format!("line {}", first_span.line)
        } else {
            format!("{first_file}:{}", first_span.line)
        };
        let message = if first_kind == kind {
            let bare = kind.trim_start_matches("an ").trim_start_matches("a ");
            format!("duplicate {bare} `{name}`: {kind} with this name is already declared, at {at}")
        } else {
            format!("`{name}` is declared twice: as {first_kind} at {at}, and as {kind} here")
        };
        diags.push(
            diag("E102", message, &file, span)
                .with_hint(format!(
                    "A name means one thing — rename one of the two `{name}`s"
                ))
                .with_related(
                    format!("`{name}` is first declared here"),
                    first_file,
                    first_span.line as usize,
                    first_span.col as usize,
                ),
        );
    }
}

/// What the project's scripts make global must mean one thing: not two
/// files' names at once, not a name the program declares, and not one the
/// language, the browser or the compiled code already owns — the codegen
/// relies on those meaning what they mean.
fn check_script_names(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    diags: &mut Vec<Diagnostic>,
) {
    let mut declared: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    for decl in &program.declarations {
        let (name, kind) = match decl {
            Declaration::Component(c) => (c.name.as_str(), "a component"),
            Declaration::Store(s) => (s.name.as_str(), "a store"),
            Declaration::Const(c) => (c.name.as_str(), "a `const`"),
            Declaration::Data(d) => (d.name.as_str(), "a `data` constant"),
            Declaration::Api(a) => (a.name.as_str(), "an `api`"),
            Declaration::Type(t) => (t.name.as_str(), "a `type`"),
            Declaration::Enum(e) => (e.name.as_str(), "an `enum`"),
            _ => continue,
        };
        declared.insert(name, kind);
    }
    let mut seen: std::collections::HashMap<&str, String> = std::collections::HashMap::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        let Declaration::Script(script) = decl else {
            continue;
        };
        for n in &script.names {
            let at = Span {
                line: n.line as u32,
                col: n.col as u32,
                ..Span::default()
            };
            let name = n.name.as_str();
            let owned = if crate::codegen::js::BROWSER_GLOBALS.contains(&name)
                || crate::codegen::js::BROWSER_VALUES.contains(&name)
            {
                Some("the browser")
            } else if crate::sema::types::BUILT_IN_FUNCTIONS.contains(&name) {
                Some("the language")
            } else if name == "WF"
                || name == "env"
                || name.starts_with("Page_")
                || name.starts_with("__wf")
            {
                Some("the compiled pages")
            } else {
                None
            };
            let problem = if let Some(owner) = owned {
                Some((
                    format!(
                        "`{name}` in `{}` is a name {owner} already has",
                        script.path
                    ),
                    "Name it something of your own: what the page code calls must mean one thing"
                        .to_string(),
                ))
            } else if let Some(kind) = declared.get(name) {
                Some((
                    format!(
                        "`{name}` in `{}` is also {kind} the program declares",
                        script.path
                    ),
                    format!("Rename one of the two `{name}`s"),
                ))
            } else {
                seen.get(name).map(|first| {
                    (
                        format!("`{name}` is declared by both `{first}` and `{}`", script.path),
                        "Every script shares one global scope, where the second would replace the first — rename one".to_string(),
                    )
                })
            };
            if let Some((message, hint)) = problem {
                diags.push(diag("E110", message, &file_of(index), at).with_hint(hint));
            } else {
                seen.insert(name, script.path.clone());
            }
        }
    }
}

fn check_dupes<'a>(
    items: impl Iterator<Item = (&'a str, Span, usize)>,
    kind: &str,
    file_of: &dyn Fn(usize) -> String,
    diags: &mut Vec<Diagnostic>,
) {
    let mut seen: std::collections::HashMap<&str, (Span, usize)> = Default::default();
    for (name, span, index) in items {
        if let Some(&(first, first_index)) = seen.get(name) {
            diags.push(
                diag(
                    "E102",
                    format!(
                        "duplicate {kind} `{name}`: a {kind} with this name is already declared"
                    ),
                    &file_of(index),
                    span,
                )
                .with_hint(format!("rename or remove one of the `{name}` {kind}s"))
                .with_related(
                    format!("the first `{name}` is declared here"),
                    file_of(first_index),
                    first.line as usize,
                    first.col as usize,
                ),
            );
        } else {
            seen.insert(name, (span, index));
        }
    }
}

/// Walk a statement list, descending into element children and every control-flow
/// body — mirroring the node-id traversal so nothing rendered is skipped.
fn walk_stmts(
    stmts: &[Statement],
    pages: &HashSet<&str>,
    components: &HashSet<&str>,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match &stmt.kind {
            // Everywhere an element can be: its children, the slots a call
            // fills, and its handlers' bodies — which the walk used to skip.
            StatementKind::UIElement(ui) => {
                check_element(ui, pages, components, file, diags);
                walk_stmts(&ui.children, pages, components, file, diags);
                for fill in &ui.slot_fills {
                    walk_stmts(&fill.body, pages, components, file, diags);
                }
                for handler in &ui.events {
                    walk_stmts(&handler.body, pages, components, file, diags);
                }
            }
            other => {
                for body in other.bodies() {
                    walk_stmts(body, pages, components, file, diags);
                }
            }
        }
    }
}

/// Check one element for an undefined component reference. (Route targets are
/// checked separately, in [`check_routes`], so that only the routes codegen
/// actually wires are validated.)
fn check_element(
    ui: &UIElement,
    _pages: &HashSet<&str>,
    components: &HashSet<&str>,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if let ComponentRef::UserDefined(name) = &ui.component {
        if !components.contains(name.as_str()) {
            diags.push(
                diag(
                    "E101",
                    format!("unknown component `{name}`: no `component {name}` is declared"),
                    file,
                    ui.span,
                )
                .with_hint(format!(
                    "declare `component {name} {{ … }}` or check the spelling"
                )),
            );
            // The component it may have meant: the project's, or a built-in.
            let known: Vec<&str> = components
                .iter()
                .copied()
                .chain(crate::registry::components().map(|c| c.name))
                .collect();
            if let Some(near) = crate::diagnostics::fixes::nearest(name, known.iter().copied())
                && let Some(d) = diags.pop()
            {
                let hint = format!("did you mean `{near}`? Or declare `component {name} {{ … }}`");
                diags.push(d.with_hint(hint).with_plan(
                    format!("Change to `{near}`"),
                    crate::diagnostics::fixes::Plan::Rename {
                        from: name.clone(),
                        to: near.to_string(),
                    },
                ));
            }
        }
    }
}

/// Validate the `page:` target of every `Route` that codegen actually wires — the
/// direct `Route` children of the first `Router` in the `App` body. This mirrors
/// the JS codegen's `find_router_routes`, which ignores a second `Router` and any
/// `Route` nested in control flow; validating routes codegen silently drops would
/// reject programs that compile and preview fine.
fn check_routes(
    program: &Program,
    pages: &HashSet<&str>,
    file_of: &dyn Fn(usize) -> String,
    diags: &mut Vec<Diagnostic>,
) {
    let Some((index, app_body)) =
        program
            .declarations
            .iter()
            .enumerate()
            .find_map(|(i, d)| match d {
                Declaration::App(a) => Some((i, &a.body)),
                _ => None,
            })
    else {
        return;
    };
    let file = file_of(index);
    let file = file.as_str();
    for route in wired_routes(app_body) {
        if let Some((page, span)) = route_page(route) {
            if !pages.contains(page) {
                diags.push(
                    diag(
                        "R01",
                        format!(
                            "Route targets unknown page `{page}`: no `Page {page}` is declared"
                        ),
                        file,
                        span,
                    )
                    .with_hint(format!(
                        "declare `Page {page} (path: …) {{ … }}` or fix the `page:` name"
                    )),
                );
            }
        }
    }
}

/// The routes codegen wires: the direct `Route` children of the first `Router`
/// reached by a depth-first scan of element children (mirrors `find_router_routes`
/// in `codegen::js`). Only descends element children — not control-flow bodies —
/// exactly as codegen does.
fn wired_routes(body: &[Statement]) -> Vec<&UIElement> {
    for stmt in body {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
                return ui
                    .children
                    .iter()
                    .filter_map(|s| match &s.kind {
                        StatementKind::UIElement(c)
                            if matches!(&c.component, ComponentRef::BuiltIn(n) if n == "Route") =>
                        {
                            Some(c)
                        }
                        _ => None,
                    })
                    .collect();
            }
            let nested = wired_routes(&ui.children);
            if !nested.is_empty() {
                return nested;
            }
        }
    }
    Vec::new()
}

/// The identifier a `Route`'s `page:` argument names, with the span to blame
/// (the argument's own span when available, else the whole element).
fn route_page(ui: &UIElement) -> Option<(&str, Span)> {
    ui.args.iter().enumerate().find_map(|(i, arg)| {
        if let Arg::Named(name, Expr::Identifier(id)) = arg {
            if name == "page" {
                return Some((id.as_str(), ui.arg_spans.get(i).copied().unwrap_or(ui.span)));
            }
        }
        None
    })
}

/// A [`Diagnostic`] at a node's span (1-based line/column from Slice-1 spans).
fn diag(code: &'static str, message: String, file: &str, span: Span) -> Diagnostic {
    Diagnostic::coded(code, message, file, span.line as usize, span.col as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program(src: &str) -> Program {
        crate::syntax::parse_source(src, "<test>")
            .map(crate::sema::lower)
            .expect("parse")
    }

    fn check(src: &str) -> Vec<Diagnostic> {
        validate_semantics(&program(src), "<test>")
    }

    #[test]
    fn clean_program_has_no_diagnostics() {
        let src = "page Home(path: \"/\") {\n  Container {\n    Heading(\"Hi\").h1\n    Button(\"Go\").primary\n  }\n}\n";
        assert!(check(src).is_empty());
    }

    #[test]
    fn undefined_component_is_flagged_with_location() {
        // ProfileCard is used but never declared. It sits on line 3.
        let src = "page Home(path: \"/\") {\n  Container {\n    ProfileCard\n  }\n}\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("ProfileCard"),
            "msg: {}",
            diags[0].message
        );
        assert_eq!(diags[0].line, 3, "line of the undefined component");
    }

    #[test]
    fn declared_component_is_accepted() {
        let src = "component ProfileCard(name: String) { Text(name).bold }\npage Home(path: \"/\") { Container { ProfileCard(name: \"Jo\") } }\n";
        assert!(check(src).is_empty(), "diags: {:?}", check(src));
    }

    #[test]
    fn undefined_component_inside_control_flow_is_flagged() {
        // Recursion into `if` bodies must still catch it.
        let src = "page Home(path: \"/\") {\n  if (true) {\n    ProfileCard\n  }\n}\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("ProfileCard"));
    }

    #[test]
    fn route_to_existing_page_is_accepted() {
        let src = "page Home(path: \"/\") { Text(\"hi\") }\napp { Router }\n";
        assert!(check(src).is_empty(), "diags: {:?}", check(src));
    }

    #[test]
    fn route_dropped_by_codegen_is_not_flagged() {
        // Codegen wires only the FIRST Router's direct Route children; a second
        // Router's routes are dropped. Validating them would reject a program that
        // compiles and previews fine, so the gate must ignore them.
        let src = "page Home(path: \"/\") { Text(\"hi\") }\napp {\n  Router\n  Router\n}\n";
        assert!(
            check(src).is_empty(),
            "a dropped route must not be flagged; diags: {:?}",
            check(src)
        );
    }

    #[test]
    fn duplicate_page_name_is_flagged_once() {
        let src =
            "page Home(path: \"/\") { Text(\"a\") }\npage Home(path: \"/b\") { Text(\"b\") }\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1, "one diagnostic for the second declaration");
        assert!(diags[0].message.contains("duplicate page"));
        assert!(diags[0].message.contains("Home"));
        assert_eq!(diags[0].line, 2, "points at the second Page");
    }

    #[test]
    fn duplicate_component_name_is_flagged() {
        let src = "component ProfileCard(x: String) { Text(x) }\ncomponent ProfileCard(y: String) { Text(y) }\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("duplicate component"));
        assert!(diags[0].message.contains("ProfileCard"));
    }

    #[test]
    fn page_and_component_may_share_a_name() {
        // Cross-kind name reuse is legal (separate namespaces) — no diagnostic.
        let src = "component Profile(name: String) { Text(name) }\npage Profile(path: \"/me\") { Text(\"me\") }\n";
        assert!(check(src).is_empty(), "diags: {:?}", check(src));
    }

    #[test]
    fn multiple_problems_are_all_reported() {
        let src = "page Home(path: \"/\") {\n  WidgetA\n  WidgetB\n}\n";
        let diags = check(src);
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().any(|d| d.message.contains("WidgetA")));
        assert!(diags.iter().any(|d| d.message.contains("WidgetB")));
    }

    #[test]
    fn a_name_means_one_thing_across_kinds() {
        let diags = check("store K { state x = 1 }\nconst K = 2\n");
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert!(
            diags[0]
                .message
                .contains("as a store at line 1, and as a `const` here"),
            "{}",
            diags[0].message
        );
        assert_eq!(diags[0].line, 2);
        let diags = check("const L = 1\nconst L = 2\n");
        assert!(
            diags[0].message.contains("duplicate `const` `L`"),
            "{diags:?}"
        );
    }

    #[test]
    fn a_body_declares_each_name_once() {
        let page = "page P(path: \"/\") {\n  state a = 1\n  derived a = 2\n  Text(\"{a}\")\n}\n";
        let diags = check(page);
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert!(
            diags[0]
                .message
                .contains("as a state at line 2, and as a `derived` here")
        );
        let store = "store S {\n  state a = 1\n  action a() { log(1) }\n}\n";
        assert!(check(store)[0].message.contains("as an action here"));
        let comp = "component C(a: String) {\n  state a = \"x\"\n  Text(a)\n}\n";
        assert!(check(comp)[0].message.contains("as a prop"));
        let param = "page U(path: \"/u/:id\", id: String) {\n  state id = 1\n  Text(\"{id}\")\n}\n";
        assert!(check(param)[0].message.contains("as a parameter"));
    }

    #[test]
    fn a_component_may_share_its_name_with_a_type() {
        // `DeployRow(…)` placed is the component; in an expression, the record.
        let src = "type DeployRow { id: String }\ncomponent DeployRow(row: DeployRow) { Text(row.id) }\nenum Tone { calm }\nconst Tone2 = 1\n";
        assert!(check(src).is_empty(), "{:?}", check(src));
        assert!(
            check("type T { a: String }\nenum T { b }\n")[0]
                .message
                .contains("declared twice")
        );
    }

    #[test]
    fn the_same_name_in_two_bodies_is_two_names() {
        let src = "page A(path: \"/a\") { state n = 1  Text(\"{n}\") }\npage B(path: \"/b\") { state n = 2  Text(\"{n}\") }\nstore S { state n = 3 }\n";
        assert!(check(src).is_empty(), "{:?}", check(src));
    }
}
