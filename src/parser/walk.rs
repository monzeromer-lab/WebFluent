//! Walking a program's tree: a declaration's body, a statement's nested
//! bodies, and every expression with the place it is in.

use crate::parser::ast::*;

/// The statements a declaration's body holds.
pub fn body_of(decl: &Declaration) -> &[Statement] {
    match decl {
        Declaration::Page(p) => &p.body,
        Declaration::Component(c) => &c.body,
        Declaration::Store(s) => &s.body,
        Declaration::App(a) => &a.body,
        Declaration::Theme(_)
        | Declaration::Type(_)
        | Declaration::Enum(_)
        | Declaration::Api(_)
        | Declaration::Script(_)
        | Declaration::Const(_)
        | Declaration::Animation(_)
        | Declaration::Data(_) => &[],
        Declaration::Test(t) => &t.body,
    }
}

/// Every statement list nested in a statement: bodies, branches, handlers
/// — an element's children, handlers and slot fills, and every body
/// [`StatementKind::bodies`] names, a connection's handlers among them.
pub fn child_bodies(stmt: &Statement) -> Vec<&[Statement]> {
    match &stmt.kind {
        StatementKind::UIElement(el) => {
            let mut bodies: Vec<&[Statement]> = vec![&el.children];
            bodies.extend(el.events.iter().map(|e| e.body.as_slice()));
            bodies.extend(el.slot_fills.iter().map(|f| f.body.as_slice()));
            bodies
        }
        kind => kind.bodies(),
    }
}

/// Every expression in `stmts`, with the place of the statement or the
/// argument it is in.
pub(crate) fn visit(stmts: &[Statement], f: &mut dyn FnMut(&Expr, Span)) {
    fn expr(e: &Expr, span: Span, f: &mut dyn FnMut(&Expr, Span)) {
        f(e, span);
        for child in e.children() {
            expr(child, span, f);
        }
    }
    for stmt in stmts {
        // An element's arguments are visited below, each at its own place.
        if !matches!(stmt.kind, StatementKind::UIElement(_)) {
            for e in stmt.kind.exprs() {
                expr(e, stmt.span, f);
            }
        }
        if let StatementKind::UIElement(ui) = &stmt.kind {
            for (i, arg) in ui.args.iter().enumerate() {
                let at = ui.arg_spans.get(i).copied().unwrap_or(ui.span);
                match arg {
                    Arg::Positional(e) | Arg::Named(_, e) => expr(e, at, f),
                }
            }
            visit(&ui.children, f);
            for fill in &ui.slot_fills {
                visit(&fill.body, f);
            }
            for h in &ui.events {
                visit(&h.body, f);
            }
        }
        for body in stmt.kind.bodies() {
            visit(body, f);
        }
    }
}

/// The pages that are translations of one another with the page named
/// `name` — joined by `translates:`, directly or through each other — in
/// the order they are declared, the page itself among them. A page no other
/// names and that names none is alone.
pub fn translation_group<'a>(name: &str, program: &'a Program) -> Vec<&'a PageDecl> {
    let pages: Vec<&PageDecl> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Page(p) => Some(p),
            _ => None,
        })
        .collect();
    let mut group: Vec<&str> = Vec::new();
    if let Some(own) = pages.iter().find(|p| p.name == name) {
        group.push(own.name.as_str());
    }
    loop {
        let before = group.len();
        for p in &pages {
            let named = p.translates.as_deref();
            let joined =
                group.contains(&p.name.as_str()) || named.is_some_and(|t| group.contains(&t));
            if !joined {
                continue;
            }
            if !group.contains(&p.name.as_str()) {
                group.push(p.name.as_str());
            }
            if let Some(original) = pages.iter().find(|o| Some(o.name.as_str()) == named)
                && !group.contains(&original.name.as_str())
            {
                group.push(original.name.as_str());
            }
        }
        if group.len() == before {
            break;
        }
    }
    pages
        .into_iter()
        .filter(|p| group.contains(&p.name.as_str()))
        .collect()
}
