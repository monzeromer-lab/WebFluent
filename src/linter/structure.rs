//! What a program's shape says before any value is known: links to routes
//! no page has, a page whose route names a parameter it does not declare,
//! an `app` with no `Router`, a part outside the component it belongs to,
//! a store nothing declares, a relative URL on a nested route, a checkbox
//! that shows a state but never changes it, a select whose value is none
//! of its options.

use crate::diagnostics::Diagnostic;
use crate::parser::ast::*;
use std::collections::{HashMap, HashSet};

/// One segment of a route's path.
#[derive(Debug, Clone, PartialEq)]
enum RouteSeg {
    Lit(String),
    Param(String),
    /// `*`: the rest of the path.
    Rest,
}

/// One segment of a link: written out, or worked out at run time.
#[derive(Debug, Clone, PartialEq)]
enum LinkSeg {
    Lit(String),
    Splice,
}

fn route_segments(path: &str) -> Vec<RouteSeg> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s == "*" {
                RouteSeg::Rest
            } else if let Some(name) = s.strip_prefix(':') {
                RouteSeg::Param(name.to_string())
            } else {
                RouteSeg::Lit(s.to_string())
            }
        })
        .collect()
}

/// What a link points at, when it can be read: `Ok` with its segments, or
/// `Err` with the text when it does not begin with `/`. `None` for what is
/// not a route — another origin, a `mailto:`, a fragment, a file — or what
/// is worked out whole at run time.
fn link_of(expr: &Expr) -> Option<Result<Vec<LinkSeg>, String>> {
    const SPLICE: char = '\u{1}';
    let text = match expr {
        Expr::StringLiteral(s) => s.clone(),
        Expr::InterpolatedString(parts) => parts
            .iter()
            .map(|p| match p {
                StringPart::Literal(t) => t.clone(),
                StringPart::Expression(_) => SPLICE.to_string(),
            })
            .collect(),
        _ => return None,
    };
    if text.is_empty() || text.starts_with(SPLICE) {
        return None;
    }
    let path = text.split(['?', '#']).next().unwrap_or("");
    let lower = text.to_ascii_lowercase();
    if text.starts_with('#')
        || text.starts_with("//")
        || lower.contains("://")
        || ["mailto:", "tel:", "sms:", "ftp:", "javascript:", "data:"]
            .iter()
            .any(|s| lower.starts_with(s))
    {
        return None;
    }
    if path.is_empty() {
        return None;
    }
    if !path.starts_with('/') {
        return Some(Err(text));
    }
    let segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    // A file — `/report.pdf`, `/img/logo.png` — is not a route.
    if segs
        .last()
        .is_some_and(|last| !last.contains(SPLICE) && last.contains('.'))
    {
        return None;
    }
    Some(Ok(segs
        .into_iter()
        .map(|s| {
            if s.contains(SPLICE) {
                LinkSeg::Splice
            } else {
                LinkSeg::Lit(s.to_string())
            }
        })
        .collect()))
}

fn route_matches(route: &[RouteSeg], link: &[LinkSeg]) -> bool {
    let mut i = 0;
    for seg in route {
        match seg {
            RouteSeg::Rest => return true,
            RouteSeg::Param(_) => {
                if i >= link.len() {
                    return false;
                }
            }
            RouteSeg::Lit(r) => match link.get(i) {
                Some(LinkSeg::Lit(l)) if l == r => {}
                Some(LinkSeg::Splice) => {}
                _ => return false,
            },
        }
        i += 1;
    }
    i == link.len()
}

struct Ctx<'a> {
    file: String,
    decl: usize,
    routes: &'a [(String, Vec<RouteSeg>)],
    stores: &'a HashSet<&'a str>,
    /// The program's own components.
    components: &'a HashSet<&'a str>,
    /// The route of the page being walked, when it is a page.
    page_path: Option<&'a str>,
    /// The states the body declares with a literal value, for a select's.
    literals: HashMap<String, Expr>,
    /// How deep in `for` loops the walk is.
    loops: usize,
    /// Whether the body being walked is a component's that is placed more
    /// than once, or inside a loop: what it holds exists many times.
    many: bool,
    /// The components whose body holds state of its own.
    stateful: &'a HashSet<&'a str>,
    out: Vec<Diagnostic>,
}

/// Every structural finding in `program`.
pub fn lint_structure(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<Diagnostic> {
    let routes: Vec<(String, Vec<RouteSeg>)> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Page(p) => Some((p.path.clone(), route_segments(&p.path))),
            _ => None,
        })
        .collect();
    let stores: HashSet<&str> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Store(s) => Some(s.name.as_str()),
            _ => None,
        })
        .collect();
    let components: HashSet<&str> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some(c.name.as_str()),
            _ => None,
        })
        .collect();
    let placed = placements(program);
    let stateful: HashSet<&str> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c)
                if c.body
                    .iter()
                    .any(|s| matches!(s.kind, StatementKind::State(_))) =>
            {
                Some(c.name.as_str())
            }
            _ => None,
        })
        .collect();
    let mut out = Vec::new();

    for (index, decl) in program.declarations.iter().enumerate() {
        let file = file_of(index);
        let (body, page_path, is_app): (&[Statement], Option<&str>, bool) = match decl {
            Declaration::Page(p) => {
                route_parameters(p, &file, &mut out);
                (&p.body, Some(p.path.as_str()), false)
            }
            Declaration::Component(c) => (&c.body, None, false),
            Declaration::App(a) => (&a.body, None, true),
            Declaration::Store(s) => (&s.body, None, false),
            _ => continue,
        };
        if is_app && !routes.is_empty() {
            let routers = count_routers(body);
            // `app` itself carries no place; its first line is where it is.
            let span = body.first().map(|s| s.span).unwrap_or_default();
            if routers != 1 {
                let (message, hint) = if routers == 0 {
                    (
                        "`app` places no `Router`, so no page is ever drawn".to_string(),
                        "Put `Router` where the current page belongs: `app { Navbar { … } Router }`",
                    )
                } else {
                    (
                        format!("`app` places {routers} `Router`s; the page is drawn in one place"),
                        "Keep one `Router`",
                    )
                };
                out.push(
                    Diagnostic::coded(
                        "R03",
                        message,
                        &file,
                        span.line.max(1) as usize,
                        span.col.max(1) as usize,
                    )
                    .with_hint(hint),
                );
            }
        }
        let many = match decl {
            Declaration::Component(c) => placed
                .get(c.name.as_str())
                .is_some_and(|(count, in_loop)| *count > 1 || *in_loop),
            _ => false,
        };
        let mut cx = Ctx {
            file,
            decl: index,
            routes: &routes,
            stores: &stores,
            components: &components,
            page_path,
            literals: literal_states(body),
            loops: 0,
            many,
            stateful: &stateful,
            out: Vec::new(),
        };
        // D03: a `persist` every instance of the component shares.
        if many && let Declaration::Component(c) = decl {
            for stmt in &c.body {
                if let StatementKind::State(st) = &stmt.kind
                    && st.persist
                    && st.policy.as_ref().is_none_or(|p| p.key.is_none())
                {
                    cx.out.push(
                        Diagnostic::coded(
                            "D03",
                            format!(
                                "`persist {}` is in `{}`, which is placed more than once, so every one of them reads and writes one stored value",
                                st.name, c.name
                            ),
                            &cx.file,
                            stmt.span.line.max(1) as usize,
                            stmt.span.col.max(1) as usize,
                        )
                        .with_hint("Give each its own: `persist open = false { key: id }`, with `id` a prop that tells them apart"),
                    );
                }
            }
        }
        let mut ancestors: Vec<&UIElement> = Vec::new();
        walk(
            body,
            &mut cx,
            &mut ancestors,
            matches!(decl, Declaration::Component(_)),
        );
        out.extend(cx.out);
        let _ = cx.decl;
    }
    out
}

/// A page's `:param` segments and its declared parameters, held to each
/// other.
fn route_parameters(p: &PageDecl, file: &str, out: &mut Vec<Diagnostic>) {
    let in_path: Vec<&str> = p
        .path
        .split('/')
        .filter_map(|s| s.strip_prefix(':'))
        .collect();
    let declared: Vec<&str> = p.params.iter().map(|q| q.name.as_str()).collect();
    for name in &in_path {
        if !declared.contains(name) {
            out.push(
                Diagnostic::coded(
                    "R02",
                    format!(
                        "the route `{}` names `:{name}`, and the page declares no parameter `{name}`",
                        p.path
                    ),
                    file,
                    p.header_span.line.max(1) as usize,
                    p.header_span.col.max(1) as usize,
                )
                .with_hint(format!(
                    "Declare it: `page {}(path: \"{}\", {name}: String)`, then read `{name}`",
                    p.name, p.path
                )),
            );
        }
    }
    for q in &p.params {
        if !in_path.contains(&q.name.as_str()) {
            out.push(
                Diagnostic::coded(
                    "R02",
                    format!(
                        "`{}` is a parameter of `{}`, and its route `{}` has no `:{}` to fill it",
                        q.name, p.name, p.path, q.name
                    ),
                    file,
                    q.span.line.max(1) as usize,
                    q.span.col.max(1) as usize,
                )
                .with_hint(format!(
                    "Add `/:{}` to the path, or make it a `state`",
                    q.name
                )),
            );
        }
    }
}

/// How many times each of the program's components is placed, and whether
/// any placement is inside a loop.
fn placements(program: &Program) -> HashMap<&str, (usize, bool)> {
    fn walk<'p>(stmts: &'p [Statement], loops: usize, out: &mut HashMap<&'p str, (usize, bool)>) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::UIElement(ui) => {
                    if let ComponentRef::UserDefined(name) = &ui.component {
                        let e = out.entry(name.as_str()).or_insert((0, false));
                        e.0 += 1;
                        e.1 |= loops > 0;
                    }
                    walk(&ui.children, loops, out);
                    for fill in &ui.slot_fills {
                        walk(&fill.body, loops, out);
                    }
                }
                StatementKind::For(f) => walk(&f.body, loops + 1, out),
                other => {
                    for body in other.bodies() {
                        walk(body, loops, out);
                    }
                }
            }
        }
    }
    let mut out = HashMap::new();
    for decl in &program.declarations {
        let body = match decl {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            _ => continue,
        };
        walk(body, 0, &mut out);
    }
    out
}

/// What in a loop's body holds state of its own, said for a finding: a
/// control the reader types or clicks into, or a component with state.
fn stateful_item(stmts: &[Statement], stateful: &HashSet<&str>) -> Option<String> {
    const CONTROLS: &[&str] = &[
        "Input",
        "Select",
        "Textarea",
        "Checkbox",
        "Radio",
        "Switch",
        "Slider",
        "DatePicker",
        "FileUpload",
    ];
    for stmt in stmts {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            match &ui.component {
                ComponentRef::BuiltIn(n) if CONTROLS.contains(&n.as_str()) => {
                    return Some(format!("a `{n}`"));
                }
                ComponentRef::UserDefined(n) if stateful.contains(n.as_str()) => {
                    return Some(format!("a `{n}` with state of its own"));
                }
                _ => {}
            }
            if let Some(found) = stateful_item(&ui.children, stateful) {
                return Some(found);
            }
        }
        for body in stmt.kind.bodies() {
            if let Some(found) = stateful_item(body, stateful) {
                return Some(found);
            }
        }
    }
    None
}

fn count_routers(stmts: &[Statement]) -> usize {
    let mut n = 0;
    for stmt in stmts {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            if matches!(&ui.component, ComponentRef::BuiltIn(name) if name == "Router") {
                n += 1;
            }
            n += count_routers(&ui.children);
            for fill in &ui.slot_fills {
                n += count_routers(&fill.body);
            }
        }
        for body in stmt.kind.bodies() {
            n += count_routers(body);
        }
    }
    n
}

/// `state x = "literal"` at the top of a body.
fn literal_states(body: &[Statement]) -> HashMap<String, Expr> {
    body.iter()
        .filter_map(|s| match &s.kind {
            StatementKind::State(st)
                if matches!(
                    st.value,
                    Expr::StringLiteral(_) | Expr::NumberLiteral(_) | Expr::BoolLiteral(_)
                ) =>
            {
                Some((st.name.clone(), st.value.clone()))
            }
            _ => None,
        })
        .collect()
}

fn walk<'a>(
    stmts: &'a [Statement],
    cx: &mut Ctx,
    ancestors: &mut Vec<&'a UIElement>,
    in_component: bool,
) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(ui) => {
                element(ui, cx, ancestors, in_component);
                ancestors.push(ui);
                walk(&ui.children, cx, ancestors, in_component);
                for fill in &ui.slot_fills {
                    walk(&fill.body, cx, ancestors, in_component);
                }
                for handler in &ui.events {
                    walk(&handler.body, cx, ancestors, in_component);
                }
                ancestors.pop();
            }
            StatementKind::Navigate(target) => link(target, stmt.span, cx),
            StatementKind::For(f) => {
                // U09: with no `by`, a change to the list redraws every
                // item, and what an item holds — a field being typed in, a
                // component's own state — starts again.
                if f.key.is_none()
                    && let Some(what) = stateful_item(&f.body, cx.stateful)
                {
                    cx.out.push(
                        Diagnostic::coded(
                            "U09",
                            format!("this `for` has no `by`, and each item holds {what}, which starts again whenever the list changes"),
                            &cx.file,
                            stmt.span.line.max(1) as usize,
                            stmt.span.col.max(1) as usize,
                        )
                        .with_hint(format!("Key it: `for {} in … by {}.id`", f.item, f.item)),
                    );
                }
                cx.loops += 1;
                walk(&f.body, cx, ancestors, in_component);
                cx.loops -= 1;
            }
            StatementKind::Use(u) if !cx.stores.contains(u.store_name.as_str()) => {
                let mut names: Vec<&&str> = cx.stores.iter().collect();
                names.sort();
                let hint = if names.is_empty() {
                    "Declare it: `store Name { state … }`".to_string()
                } else {
                    format!(
                        "The stores are {}",
                        names
                            .iter()
                            .map(|n| format!("`{n}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                cx.out.push(
                    Diagnostic::coded(
                        "X05",
                        format!("`use {}` names a store nothing declares", u.store_name),
                        &cx.file,
                        stmt.span.line.max(1) as usize,
                        stmt.span.col.max(1) as usize,
                    )
                    .with_hint(hint),
                );
            }
            _ => {
                for body in stmt.kind.bodies() {
                    walk(body, cx, ancestors, in_component);
                }
            }
        }
    }
}

fn named<'a>(ui: &'a UIElement, key: &str) -> Option<&'a Expr> {
    ui.args.iter().find_map(|a| match a {
        Arg::Named(k, v) if k == key => Some(v),
        _ => None,
    })
}

fn element(ui: &UIElement, cx: &mut Ctx, ancestors: &[&UIElement], in_component: bool) {
    let at = |ui: &UIElement| (ui.span.line.max(1) as usize, ui.span.col.max(1) as usize);

    // C03: a part outside the component it belongs to. Inside a component's
    // body, or below one of your components in a page, the part may well be
    // placed inside its owner at run time, so only what is certain is said.
    if let ComponentRef::SubComponent(owner, part) = &ui.component
        && !in_component
        // `Unsafe.Html` is a name under a namespace, not a part of a
        // component: only a built-in or one of yours is an owner.
        && (crate::registry::component(owner)
            .is_some_and(|c| matches!(c.children, crate::registry::Children::Elements))
            || cx.components.contains(owner.as_str()))
        && !ancestors.iter().any(|a| match &a.component {
            ComponentRef::BuiltIn(n) => n == owner,
            ComponentRef::SubComponent(o, _) => o == owner,
            // One of your components may place the part inside its owner.
            ComponentRef::UserDefined(_) => true,
        })
    {
        let (line, col) = at(ui);
        cx.out.push(
            Diagnostic::coded(
                "C03",
                format!("`{owner}.{part}` is a part of `{owner}`, and it is not inside one"),
                &cx.file,
                line,
                col,
            )
            .with_hint(format!("Place it in a `{owner} {{ … }}`")),
        );
    }

    // R01: a link to a route no page has.
    if let Some(target) = named(ui, "to") {
        link(target, ui.span, cx);
    }

    // A16: one literal `id` on an element drawn many times.
    if let Some(Expr::StringLiteral(id)) = named(ui, "id")
        && (cx.loops > 0 || cx.many)
    {
        let (line, col) = at(ui);
        let why = if cx.loops > 0 {
            "inside a `for`"
        } else {
            "in a component placed more than once"
        };
        cx.out.push(
            Diagnostic::coded(
                "A16",
                format!("`id: \"{id}\"` is {why}, so the page has several elements with one id"),
                &cx.file,
                line,
                col,
            )
            .with_hint("Make it unique — `id: \"row-{item.id}\"` — or drop it; a label's `for` and `aria-*` find one element by id"),
        );
    }

    // R04: a relative URL on a nested route.
    if let Some(path) = cx.page_path {
        let depth = path.split('/').filter(|s| !s.is_empty()).count();
        if depth >= 2 {
            for key in ["src", "href", "poster", "captions", "transcript"] {
                if let Some(Expr::StringLiteral(url)) = named(ui, key) {
                    let lower = url.to_ascii_lowercase();
                    let relative = !url.is_empty()
                        && !url.starts_with('/')
                        && !url.starts_with('#')
                        && !url.starts_with('.')
                        && !lower.contains(':');
                    if relative {
                        let base = path.rsplit_once('/').map(|(b, _)| b).unwrap_or("");
                        let (line, col) = at(ui);
                        cx.out.push(
                            Diagnostic::coded(
                                "R04",
                                format!(
                                    "`{key}: \"{url}\"` is relative, so on `{path}` it is fetched from `{base}/{url}`"
                                ),
                                &cx.file,
                                line,
                                col,
                            )
                            .with_hint(format!("Write it from the site's root: `\"/{url}\"`")),
                        );
                    }
                }
            }
        }
    }

    let name = match &ui.component {
        ComponentRef::BuiltIn(n) => n.as_str(),
        _ => "",
    };
    // F04: shows a state, never changes it.
    if matches!(name, "Checkbox" | "Switch")
        && named(ui, "bind").is_none()
        && named(ui, "checked").is_some()
        && !ui
            .events
            .iter()
            .any(|h| matches!(h.event.as_str(), "change" | "input" | "click"))
    {
        let (line, col) = at(ui);
        cx.out.push(
            Diagnostic::coded(
                "F04",
                format!("this `{name}` shows `checked:` but nothing changes it, so a click toggles the box and not the state"),
                &cx.file,
                line,
                col,
            )
            .with_hint("Write `bind:` to keep the box and the state together, or handle `on change`"),
        );
    }
    // F05: a select whose value is none of its options.
    if name == "Select"
        && let Some(Expr::Identifier(bound)) = named(ui, "bind")
        && let Some(initial) = cx.literals.get(bound)
    {
        let mut values: Vec<String> = Vec::new();
        let mut known = true;
        for child in &ui.children {
            let StatementKind::UIElement(opt) = &child.kind else {
                known = false;
                break;
            };
            let value = named(opt, "value").or_else(|| {
                opt.args.iter().find_map(|a| match a {
                    Arg::Positional(e) => Some(e),
                    _ => None,
                })
            });
            match value {
                Some(Expr::StringLiteral(s)) => values.push(s.clone()),
                Some(Expr::NumberLiteral(n)) => values.push(n.to_string()),
                Some(Expr::BoolLiteral(b)) => values.push(b.to_string()),
                _ => {
                    known = false;
                    break;
                }
            }
        }
        let initial_text = match initial {
            Expr::StringLiteral(s) => s.clone(),
            Expr::NumberLiteral(n) => n.to_string(),
            Expr::BoolLiteral(b) => b.to_string(),
            _ => String::new(),
        };
        if known && !values.is_empty() && !values.contains(&initial_text) {
            let (line, col) = at(ui);
            cx.out.push(
                Diagnostic::coded(
                    "F05",
                    format!(
                        "`{bound}` starts as `{initial_text}`, which is none of this select's options"
                    ),
                    &cx.file,
                    line,
                    col,
                )
                .with_hint(format!(
                    "Start it as one of them ({}), so what is shown is what is held",
                    values.iter().map(|v| format!("`{v}`")).collect::<Vec<_>>().join(", ")
                )),
            );
        }
    }
}

/// R01 for one link's target.
fn link(target: &Expr, span: Span, cx: &mut Ctx) {
    if cx.routes.is_empty() {
        return;
    }
    let (line, col) = (span.line.max(1) as usize, span.col.max(1) as usize);
    match link_of(target) {
        None => {}
        Some(Err(text)) => cx.out.push(
            Diagnostic::coded(
                "R01",
                format!("`{text}` does not begin with `/`, so where it leads depends on the page it is on"),
                &cx.file,
                line,
                col,
            )
            .with_hint(format!("Write the route from the site's root: `\"/{text}\"`")),
        ),
        Some(Ok(segs)) => {
            if cx.routes.iter().any(|(_, r)| route_matches(r, &segs)) {
                return;
            }
            let shown: String = segs
                .iter()
                .map(|s| match s {
                    LinkSeg::Lit(t) => t.clone(),
                    LinkSeg::Splice => "{…}".to_string(),
                })
                .collect::<Vec<_>>()
                .join("/");
            let shown = format!("/{shown}");
            let nearest = cx
                .routes
                .iter()
                .map(|(p, _)| (crate::linter::vocabulary::levenshtein(&shown, p), p))
                .min()
                .map(|(_, p)| p.clone())
                .unwrap_or_default();
            cx.out.push(
                Diagnostic::coded(
                    "R01",
                    format!("`{shown}` is not a route: no page's `path` matches it"),
                    &cx.file,
                    line,
                    col,
                )
                .with_hint(format!("The nearest is `{nearest}`")),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_matches_a_route_by_its_segments() {
        let route = route_segments("/team/:slug");
        let link = |s: &str| link_of(&Expr::StringLiteral(s.into())).unwrap().unwrap();
        assert!(route_matches(&route, &link("/team/ops")));
        assert!(!route_matches(&route, &link("/team")));
        assert!(!route_matches(&route, &link("/teams/ops")));
        assert!(route_matches(
            &route_segments("/docs/*"),
            &link("/docs/a/b")
        ));
        assert!(route_matches(&route_segments("/"), &link("/")));
        assert!(link_of(&Expr::StringLiteral("https://x.dev/a".into())).is_none());
        assert!(link_of(&Expr::StringLiteral("/report.pdf".into())).is_none());
        assert!(matches!(
            link_of(&Expr::StringLiteral("about".into())),
            Some(Err(_))
        ));
    }
}
