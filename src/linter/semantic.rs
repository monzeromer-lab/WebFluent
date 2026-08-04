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
    let pages = name_set(program, DeclKind::Page);
    let components = name_set(program, DeclKind::Component);
    let mut diags = Vec::new();

    check_duplicate_names(program, file, &mut diags);

    for decl in &program.declarations {
        let body = match decl {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            // Neither holds UI.
            Declaration::Store(_) | Declaration::Theme(_) => continue,
        };
        walk_stmts(body, &pages, &components, file, &mut diags);
    }

    check_routes(program, &pages, file, &mut diags);

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
fn check_duplicate_names(program: &Program, file: &str, diags: &mut Vec<Diagnostic>) {
    let pages = program.declarations.iter().filter_map(|d| match d {
        Declaration::Page(p) => Some((p.name.as_str(), p.header_span)),
        _ => None,
    });
    check_dupes(pages, "page", file, diags);

    let components = program.declarations.iter().filter_map(|d| match d {
        Declaration::Component(c) => Some((c.name.as_str(), c.header_span)),
        _ => None,
    });
    check_dupes(components, "component", file, diags);
}

fn check_dupes<'a>(
    items: impl Iterator<Item = (&'a str, Span)>,
    kind: &str,
    file: &str,
    diags: &mut Vec<Diagnostic>,
) {
    let mut seen: HashSet<&str> = HashSet::new();
    for (name, span) in items {
        if !seen.insert(name) {
            diags.push(
                diag(
                    format!(
                        "duplicate {kind} `{name}`: a {kind} with this name is already declared"
                    ),
                    file,
                    span,
                )
                .with_hint(format!("rename or remove one of the `{name}` {kind}s")),
            );
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
            StatementKind::UIElement(ui) => {
                check_element(ui, pages, components, file, diags);
                walk_stmts(&ui.children, pages, components, file, diags);
            }
            StatementKind::If(s) => {
                walk_stmts(&s.then_body, pages, components, file, diags);
                for (_, body) in &s.else_if_branches {
                    walk_stmts(body, pages, components, file, diags);
                }
                if let Some(body) = &s.else_body {
                    walk_stmts(body, pages, components, file, diags);
                }
            }
            StatementKind::For(s) => walk_stmts(&s.body, pages, components, file, diags),
            StatementKind::Show(s) => walk_stmts(&s.body, pages, components, file, diags),
            StatementKind::Fetch(s) => {
                if let Some(body) = &s.loading_block {
                    walk_stmts(body, pages, components, file, diags);
                }
                if let Some((_, body)) = &s.error_block {
                    walk_stmts(body, pages, components, file, diags);
                }
                if let Some(body) = &s.success_block {
                    walk_stmts(body, pages, components, file, diags);
                }
            }
            _ => {}
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
                    format!("unknown component `{name}`: no `Component {name}` is declared"),
                    file,
                    ui.span,
                )
                .with_hint(format!(
                    "declare `Component {name} {{ … }}` or check the spelling"
                )),
            );
        }
    }
}

/// Validate the `page:` target of every `Route` that codegen actually wires — the
/// direct `Route` children of the first `Router` in the `App` body. This mirrors
/// the JS codegen's `find_router_routes`, which ignores a second `Router` and any
/// `Route` nested in control flow; validating routes codegen silently drops would
/// reject programs that compile and preview fine.
fn check_routes(program: &Program, pages: &HashSet<&str>, file: &str, diags: &mut Vec<Diagnostic>) {
    let Some(app_body) = program.declarations.iter().find_map(|d| match d {
        Declaration::App(a) => Some(&a.body),
        _ => None,
    }) else {
        return;
    };
    for route in wired_routes(app_body) {
        if let Some((page, span)) = route_page(route) {
            if !pages.contains(page) {
                diags.push(
                    diag(
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
fn diag(message: String, file: &str, span: Span) -> Diagnostic {
    Diagnostic::new(message, file, span.line as usize, span.col as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn program(src: &str) -> Program {
        let toks = Lexer::new(src, "<test>").tokenize().expect("lex");
        Parser::new(toks, "<test>").parse().expect("parse")
    }

    fn check(src: &str) -> Vec<Diagnostic> {
        validate_semantics(&program(src), "<test>")
    }

    #[test]
    fn clean_program_has_no_diagnostics() {
        let src = "Page Home (path: \"/\") {\n  Container {\n    Heading(\"Hi\", h1)\n    Button(\"Go\", primary)\n  }\n}\n";
        assert!(check(src).is_empty());
    }

    #[test]
    fn undefined_component_is_flagged_with_location() {
        // ProfileCard is used but never declared. It sits on line 3.
        let src = "Page Home (path: \"/\") {\n  Container {\n    ProfileCard()\n  }\n}\n";
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
        let src = "Component ProfileCard (name: String) { Text(name, bold) }\n\
                   Page Home (path: \"/\") { Container { ProfileCard(name: \"Jo\") } }\n";
        assert!(check(src).is_empty(), "diags: {:?}", check(src));
    }

    #[test]
    fn undefined_component_inside_control_flow_is_flagged() {
        // Recursion into `if` bodies must still catch it.
        let src = "Page Home (path: \"/\") {\n  if (true) {\n    ProfileCard()\n  }\n}\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("ProfileCard"));
    }

    #[test]
    fn route_to_missing_page_is_flagged() {
        let src = "Page Home (path: \"/\") { Text(\"hi\") }\n\
                   App { Router { Route(path: \"/x\", page: Missing) } }\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1, "diags: {:?}", diags);
        assert!(
            diags[0].message.contains("Missing"),
            "msg: {}",
            diags[0].message
        );
    }

    #[test]
    fn route_to_existing_page_is_accepted() {
        let src = "Page Home (path: \"/\") { Text(\"hi\") }\n\
                   App { Router { Route(path: \"/\", page: Home) } }\n";
        assert!(check(src).is_empty(), "diags: {:?}", check(src));
    }

    #[test]
    fn route_dropped_by_codegen_is_not_flagged() {
        // Codegen wires only the FIRST Router's direct Route children; a second
        // Router's routes are dropped. Validating them would reject a program that
        // compiles and previews fine, so the gate must ignore them.
        let src = "Page Home (path: \"/\") { Text(\"hi\") }\n\
                   App {\n  Router { Route(path: \"/\", page: Home) }\n  \
                   Router { Route(path: \"/x\", page: Ghost) }\n}\n";
        assert!(
            check(src).is_empty(),
            "a dropped route must not be flagged; diags: {:?}",
            check(src)
        );
    }

    #[test]
    fn duplicate_page_name_is_flagged_once() {
        let src = "Page Home (path: \"/\") { Text(\"a\") }\n\
                   Page Home (path: \"/b\") { Text(\"b\") }\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1, "one diagnostic for the second declaration");
        assert!(diags[0].message.contains("duplicate page"));
        assert!(diags[0].message.contains("Home"));
        assert_eq!(diags[0].line, 2, "points at the second Page");
    }

    #[test]
    fn duplicate_component_name_is_flagged() {
        let src = "Component ProfileCard (x: String) { Text(x) }\n\
                   Component ProfileCard (y: String) { Text(y) }\n";
        let diags = check(src);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("duplicate component"));
        assert!(diags[0].message.contains("ProfileCard"));
    }

    #[test]
    fn page_and_component_may_share_a_name() {
        // Cross-kind name reuse is legal (separate namespaces) — no diagnostic.
        let src = "Component Profile (name: String) { Text(name) }\n\
                   Page Profile (path: \"/me\") { Text(\"me\") }\n";
        assert!(check(src).is_empty(), "diags: {:?}", check(src));
    }

    #[test]
    fn multiple_problems_are_all_reported() {
        let src = "Page Home (path: \"/\") {\n  WidgetA()\n  WidgetB()\n}\n";
        let diags = check(src);
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().any(|d| d.message.contains("WidgetA")));
        assert!(diags.iter().any(|d| d.message.contains("WidgetB")));
    }
}
