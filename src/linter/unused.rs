//! What a program declares and never reads: state and derived values no
//! expression names, actions nothing calls, components nothing places,
//! store members nothing reaches. Each is a warning, `U01`–`U05`, at the
//! declaration; a name that starts with `_` is understood to be unused on
//! purpose and is not reported.
//!
//! A read is any appearance of the name in an expression — an argument, a
//! condition, a style splice, an interpolation, a `bind:` — other than as
//! the bare target of an assignment (`count = 0` writes `count`, it does
//! not read it). Inside a store its members are read bare; outside, as
//! `Store.member`, or bare in a body that `use`s the store.

use std::collections::{HashMap, HashSet};

use crate::error::A11yWarning;
use crate::parser::ast::*;

/// The unused declarations of a program, as warnings.
pub fn lint_unused(program: &Program) -> Vec<A11yWarning> {
    lint_unused_in(program, &|_| "<unknown>".to_string())
}

/// [`lint_unused`], naming each declaration's file.
pub fn lint_unused_in(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<A11yWarning> {
    let mut out = Vec::new();

    // Every component and part the program places, by name; every layout.
    let mut placed: HashSet<String> = HashSet::new();
    for decl in &program.declarations {
        let body = match decl {
            Declaration::Page(p) => {
                if let Some(layout) = &p.layout {
                    placed.insert(layout.name.clone());
                }
                &p.body
            }
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            Declaration::Store(s) => &s.body,
            _ => continue,
        };
        placed_components(body, &mut placed);
    }

    // Every `Store.member` read anywhere, and every bare name read in a body
    // that `use`s a store, by store.
    let mut store_reads: HashMap<String, HashSet<String>> = HashMap::new();
    for decl in &program.declarations {
        let body = match decl {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            Declaration::Store(s) => &s.body,
            _ => continue,
        };
        let mut reads = Reads::default();
        read_statements(body, &mut reads);
        let used: Vec<String> = body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::Use(u) => Some(u.store_name.clone()),
                _ => None,
            })
            .collect();
        for (store, member) in reads.members {
            store_reads.entry(store).or_default().insert(member);
        }
        for store in used {
            store_reads
                .entry(store)
                .or_default()
                .extend(reads.names.iter().cloned());
        }
    }

    for (ix, decl) in program.declarations.iter().enumerate() {
        let file = file_of(ix);
        match decl {
            Declaration::Component(c) => {
                if !placed.contains(&c.name) && !c.name.starts_with('_') {
                    out.push(warn(
                        "U03",
                        format!("`{}` is declared but never placed", c.name),
                        &file,
                        c.header_span,
                        "Place it in a page, name it as a layout, or remove it",
                    ));
                }
                local_unused(&c.body, &file, &mut out);
            }
            Declaration::Page(p) => local_unused(&p.body, &file, &mut out),
            Declaration::Store(s) => {
                let outside = store_reads.get(&s.name).cloned().unwrap_or_default();
                let mut inside = Reads::default();
                read_statements(&s.body, &mut inside);
                for stmt in &s.body {
                    let (kind, name, what) = match &stmt.kind {
                        StatementKind::State(st) => ("U04", &st.name, "state"),
                        StatementKind::Derived(d) => ("U04", &d.name, "derived value"),
                        StatementKind::Action(a) => ("U04", &a.name, "action"),
                        _ => continue,
                    };
                    if name.starts_with('_')
                        || outside.contains(name)
                        || inside.names.contains(name)
                    {
                        continue;
                    }
                    out.push(warn(
                        kind,
                        format!("`{}.{name}` is declared but never read", s.name),
                        &file,
                        stmt.span,
                        &format!(
                            "Nothing reads the {what}, inside the store or as `{}.{name}`; remove it, or name it `_{name}` to keep it",
                            s.name
                        ),
                    ));
                }
            }
            _ => {}
        }
    }
    out
}

/// The state, derived values and actions of a page or component body that
/// nothing in the body reads.
fn local_unused(body: &[Statement], file: &str, out: &mut Vec<A11yWarning>) {
    let mut reads = Reads::default();
    read_statements(body, &mut reads);
    for stmt in body {
        let (code, name, what) = match &stmt.kind {
            StatementKind::State(s) => ("U01", &s.name, "state"),
            StatementKind::Derived(d) => ("U02", &d.name, "derived value"),
            StatementKind::Action(a) => ("U05", &a.name, "action"),
            _ => continue,
        };
        if name.starts_with('_') || reads.names.contains(name) {
            continue;
        }
        out.push(warn(
            code,
            format!("`{name}` is declared but never read"),
            file,
            stmt.span,
            &format!("Nothing reads the {what}; remove it, or name it `_{name}` to keep it"),
        ));
    }
}

fn warn(code: &str, message: String, file: &str, span: Span, hint: &str) -> A11yWarning {
    A11yWarning::new(
        code,
        message,
        file,
        span.line as usize,
        span.col as usize,
        hint,
    )
}

/// The names an expression tree reads.
#[derive(Default)]
struct Reads {
    /// Bare names.
    names: HashSet<String>,
    /// `Store.member` pairs.
    members: HashSet<(String, String)>,
}

fn read_statements(stmts: &[Statement], reads: &mut Reads) {
    for stmt in stmts {
        match &stmt.kind {
            // The bare target of an assignment is written, not read.
            StatementKind::Assignment(a) => {
                if !matches!(a.target, Expr::Identifier(_)) {
                    read_expr(&a.target, reads);
                }
                read_expr(&a.value, reads);
            }
            StatementKind::UIElement(el) => read_element(el, reads),
            StatementKind::MethodCall(m) => {
                if let Expr::Identifier(store) = &m.object {
                    reads.members.insert((store.clone(), m.method.clone()));
                }
                read_expr(&m.object, reads);
                for a in &m.args {
                    read_expr(a, reads);
                }
            }
            StatementKind::Emit(e) => {
                for a in &e.args {
                    read_expr(a, reads);
                }
            }
            StatementKind::For(f) => {
                read_expr(&f.iterable, reads);
                if let Some(k) = &f.key {
                    read_expr(k, reads);
                }
            }
            other => {
                for e in other.exprs() {
                    read_expr(e, reads);
                }
            }
        }
        for body in stmt.kind.bodies() {
            read_statements(body, reads);
        }
    }
}

fn read_element(el: &UIElement, reads: &mut Reads) {
    for arg in &el.args {
        match arg {
            Arg::Positional(e) | Arg::Named(_, e) => read_expr(e, reads),
        }
    }
    if let Some(style) = &el.style_block {
        for p in &style.properties {
            read_expr(&p.value, reads);
        }
        for pseudo in &style.pseudo_blocks {
            for p in &pseudo.properties {
                read_expr(&p.value, reads);
            }
        }
        for mq in &style.media_queries {
            for p in &mq.properties {
                read_expr(&p.value, reads);
            }
        }
    }
    for handler in &el.events {
        read_statements(&handler.body, reads);
    }
    for fill in &el.slot_fills {
        read_statements(&fill.body, reads);
    }
    read_statements(&el.children, reads);
}

fn read_expr(expr: &Expr, reads: &mut Reads) {
    match expr {
        Expr::Identifier(name) => {
            reads.names.insert(name.clone());
        }
        Expr::FunctionCall(name, _) => {
            reads.names.insert(name.clone());
        }
        Expr::PropertyAccess(base, member) | Expr::OptionalProperty(base, member) => {
            if let Expr::Identifier(store) = base.as_ref() {
                reads.members.insert((store.clone(), member.clone()));
            }
        }
        Expr::MethodCall(base, method, _) | Expr::OptionalMethod(base, method, _) => {
            if let Expr::Identifier(store) = base.as_ref() {
                reads.members.insert((store.clone(), method.clone()));
            }
        }
        _ => {}
    }
    for child in expr.children() {
        read_expr(child, reads);
    }
}

/// Every component and part placed in `stmts`, at any depth.
fn placed_components(stmts: &[Statement], out: &mut HashSet<String>) {
    for stmt in stmts {
        if let StatementKind::UIElement(el) = &stmt.kind {
            match &el.component {
                ComponentRef::UserDefined(name) => {
                    out.insert(name.clone());
                }
                ComponentRef::SubComponent(owner, part) => {
                    out.insert(format!("{owner}.{part}"));
                }
                ComponentRef::BuiltIn(_) => {}
            }
            placed_components(&el.children, out);
            for fill in &el.slot_fills {
                placed_components(&fill.body, out);
            }
        }
        for body in stmt.kind.bodies() {
            placed_components(body, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::v2::parse_v2;

    fn codes(src: &str) -> Vec<String> {
        let program = parse_v2(src, "<t>").expect("parse");
        lint_unused(&program)
            .iter()
            .map(|w| format!("[{}] {}", w.rule_id, w.message))
            .collect()
    }

    #[test]
    fn unread_state_derived_and_actions_are_reported() {
        let found = codes(
            "page P(path: \"/\") { state a = 1\n state b = 2\n state c = 3\n state _d = 4\n derived e = a * 2\n derived f = 1\n action go() { c = 5 }\n action stop() { log(1) }\n Button(\"x\", disabled: e > 1) { on click { go() } }\n Text(\"{b}\") }",
        );
        assert_eq!(
            found,
            vec![
                "[U01] `c` is declared but never read".to_string(),
                "[U02] `f` is declared but never read".to_string(),
                "[U05] `stop` is declared but never read".to_string(),
            ]
        );
    }

    #[test]
    fn a_bind_a_style_splice_and_an_interpolation_read_a_name() {
        let found = codes(
            "page P(path: \"/\") { state q = \"\"\n state w = 10\n state t = \"x\"\n Input(bind: q)\n Card { style { width: {w}px } }\n Text(\"a {t} b\") }",
        );
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn components_and_store_members_are_reported_when_nothing_reaches_them() {
        let found = codes(
            "component Used { Text(\"u\") }\ncomponent Unused { Text(\"n\") }\ncomponent Shell { children }\ncomponent Panel { part Header { Text(\"h\") }  Text(\"p\") }\nstore S { state a = 1\n state b = 2\n state c = 3\n derived d = a + 1\n action go() { b = 2 } }\npage P(path: \"/\", layout: Shell) { use S\n Used\n Panel { Panel.Header }\n Text(\"{S.d} {c}\") }",
        );
        assert_eq!(
            found,
            vec![
                "[U03] `Unused` is declared but never placed".to_string(),
                "[U04] `S.b` is declared but never read".to_string(),
                "[U04] `S.go` is declared but never read".to_string(),
            ]
        );
    }
}
