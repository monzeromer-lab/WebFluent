//! Walks over statements and expressions that collect names: what is read, written, bound and narrowed.

use super::*;

/// The value an argument carries.
pub(super) fn arg_value(arg: &Arg) -> &Expr {
    match arg {
        Arg::Positional(e) | Arg::Named(_, e) => e,
    }
}

/// The names a condition proves not null in its branch: `x`, `x != null`,
/// `x && …`.
pub(super) fn narrowed_names(cond: &Expr) -> Vec<String> {
    match cond {
        Expr::Identifier(name) => vec![name.clone()],
        Expr::BinaryOp(l, BinOp::Neq, r) => match (&**l, &**r) {
            (Expr::Identifier(name), Expr::Null) | (Expr::Null, Expr::Identifier(name)) => {
                vec![name.clone()]
            }
            _ => Vec::new(),
        },
        Expr::BinaryOp(l, BinOp::And, r) => {
            let mut names = narrowed_names(l);
            names.extend(narrowed_names(r));
            names
        }
        _ => Vec::new(),
    }
}

/// The names `stmts` assign to: `x = …`. With `top`, only the assignments
/// every run makes — not those under an `if`, a loop or a `match`.
pub(super) fn assigned(stmts: &[Statement], top: bool) -> Vec<String> {
    let mut out = Vec::new();
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::Assignment(a) => {
                if let Expr::Identifier(n) = &a.target {
                    out.push(n.clone());
                }
            }
            other if !top => {
                for body in other.bodies() {
                    out.extend(assigned(body, false));
                }
            }
            _ => {}
        }
    }
    out
}

/// Every name `stmts` read: in their expressions, an assignment's value,
/// and the base of a target that is not a bare name; `Store.member` reads
/// both `Store` and `Store.member`.
pub(super) fn read_names(stmts: &[Statement]) -> HashSet<String> {
    fn expr(e: &Expr, out: &mut HashSet<String>) {
        match e {
            Expr::Identifier(n) => {
                out.insert(n.clone());
            }
            Expr::PropertyAccess(base, member) => {
                if let Expr::Identifier(n) = base.as_ref() {
                    out.insert(format!("{n}.{member}"));
                }
                expr(base, out);
            }
            other => {
                for child in other.children() {
                    expr(child, out);
                }
            }
        }
    }
    let mut out = HashSet::new();
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::Assignment(a) => {
                expr(&a.value, &mut out);
                if !matches!(a.target, Expr::Identifier(_)) {
                    expr(&a.target, &mut out);
                }
            }
            other => {
                for e in other.exprs() {
                    expr(e, &mut out);
                }
            }
        }
        for body in stmt.kind.bodies() {
            out.extend(read_names(body));
        }
    }
    out
}

/// Every name a control in `stmts` binds: `bind: x`.
pub(super) fn bound_names(stmts: &[Statement]) -> HashSet<String> {
    fn walk(stmts: &[Statement], out: &mut HashSet<String>) {
        for stmt in stmts {
            if let StatementKind::UIElement(ui) = &stmt.kind {
                for arg in &ui.args {
                    if let Arg::Named(k, Expr::Identifier(n)) = arg
                        && k == "bind"
                    {
                        out.insert(n.clone());
                    }
                }
                walk(&ui.children, out);
                for fill in &ui.slot_fills {
                    walk(&fill.body, out);
                }
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
        }
    }
    let mut out = HashSet::new();
    walk(stmts, &mut out);
    out
}

/// The names `stmts`, at any depth, write: an assignment's target, what a
/// `bind:` names, a value a method is called on as a statement
/// (`items.push(x)`, which the compiler makes a write), and the holder
/// `optimistic` changes. A place inside a value is written through its
/// root — `form.name = v` writes `form`, and `t.done = v` on a loop's item
/// the list it came from — and a store's state is named both as the store
/// and as `Store.member`.
pub(super) fn written_names(stmts: &[Statement]) -> HashSet<String> {
    fn target(e: &Expr, out: &mut HashSet<String>) {
        let mut member = None;
        let mut at = e;
        loop {
            match at {
                Expr::PropertyAccess(base, m) | Expr::OptionalProperty(base, m) => {
                    member = Some(m);
                    at = base;
                }
                Expr::IndexAccess(base, _)
                | Expr::OptionalIndex(base, _)
                | Expr::MethodCall(base, _, _)
                | Expr::OptionalMethod(base, _, _) => {
                    member = None;
                    at = base;
                }
                Expr::Identifier(n) => {
                    if let Some(m) = member {
                        out.insert(format!("{n}.{m}"));
                    }
                    out.insert(n.clone());
                    return;
                }
                _ => return,
            }
        }
    }
    fn call(e: &Expr, out: &mut HashSet<String>) {
        match e {
            Expr::Await(inner) => call(inner, out),
            Expr::MethodCall(obj, _, _) | Expr::OptionalMethod(obj, _, _) => target(obj, out),
            Expr::FunctionCall(name, args) if name == "optimistic" => {
                if let Some(holder) = args.first() {
                    target(holder, out);
                }
            }
            _ => {}
        }
    }
    fn walk(stmts: &[Statement], out: &mut HashSet<String>) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::Assignment(a) => target(&a.target, out),
                StatementKind::MethodCall(m) => target(&m.object, out),
                StatementKind::ExprStatement(e) => call(e, out),
                StatementKind::UIElement(ui) => {
                    for arg in &ui.args {
                        if let Arg::Named(k, value) = arg
                            && k == "bind"
                        {
                            target(value, out);
                        }
                    }
                    for handler in &ui.events {
                        walk(&handler.body, out);
                    }
                    for fill in &ui.slot_fills {
                        walk(&fill.body, out);
                    }
                    walk(&ui.children, out);
                }
                // A write to an item is a write to its list.
                StatementKind::For(f) => {
                    let mut inner = HashSet::new();
                    walk(&f.body, &mut inner);
                    if inner.contains(&f.item) {
                        target(&f.iterable, out);
                    }
                    out.extend(inner);
                    continue;
                }
                _ => {}
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
        }
    }
    let mut out = HashSet::new();
    walk(stmts, &mut out);
    out
}

/// One arm of a lowered `match` expression: its subject, its case, and
/// the rest of the arms after it.
pub(super) fn match_arm(expr: &Expr) -> Option<(&Expr, String, &Expr)> {
    let Expr::MethodCall(obj, method, args) = expr else {
        return None;
    };
    let wanted = match method.as_str() {
        "__if" => "__is",
        "__iflet" => "__payload",
        _ => return None,
    };
    let Expr::MethodCall(subject, test, test_args) = obj.as_ref() else {
        return None;
    };
    match (test.as_str() == wanted, test_args.as_slice(), args.get(1)) {
        (true, [Expr::StringLiteral(case)], Some(rest)) => Some((subject, case.clone(), rest)),
        _ => None,
    }
}

/// The names a condition is true for only when they are null: `x ==
/// null`, `!x`, either side of an `||`.
pub(super) fn null_names(cond: &Expr) -> Vec<String> {
    match cond {
        Expr::BinaryOp(l, BinOp::Eq, r) => match (&**l, &**r) {
            (Expr::Identifier(name), Expr::Null) | (Expr::Null, Expr::Identifier(name)) => {
                vec![name.clone()]
            }
            _ => Vec::new(),
        },
        Expr::UnaryOp(UnaryOp::Not, inner) => match &**inner {
            Expr::Identifier(name) => vec![name.clone()],
            _ => Vec::new(),
        },
        Expr::BinaryOp(l, BinOp::Or, r) => {
            let mut names = null_names(l);
            names.extend(null_names(r));
            names
        }
        _ => Vec::new(),
    }
}

/// The lists a condition proves have items, by their text: `xs.length >
/// 0`, `xs.length >= 1`, `xs.length != 0`, `xs.length`, either side of
/// an `&&`.
pub(super) fn non_empty_lists(cond: &Expr) -> Vec<String> {
    let length_of = |e: &Expr| match e {
        Expr::PropertyAccess(base, f) if f == "length" => Some(expr_text(base)),
        _ => None,
    };
    let number = |e: &Expr| match e {
        Expr::NumberLiteral(n) => Some(*n),
        _ => None,
    };
    match cond {
        Expr::BinaryOp(l, op, r) => match op {
            BinOp::And => {
                let mut out = non_empty_lists(l);
                out.extend(non_empty_lists(r));
                out
            }
            BinOp::Gt => match (length_of(l), number(r)) {
                (Some(list), Some(n)) if n >= 0.0 => vec![list],
                _ => Vec::new(),
            },
            BinOp::Gte => match (length_of(l), number(r)) {
                (Some(list), Some(n)) if n >= 1.0 => vec![list],
                _ => Vec::new(),
            },
            BinOp::Neq => match (length_of(l), number(r)) {
                (Some(list), Some(0.0)) => vec![list],
                _ => Vec::new(),
            },
            _ => Vec::new(),
        },
        other => length_of(other).into_iter().collect(),
    }
}

/// Whether `expr` is, or reads through, a `?.` — so the rest of its chain
/// short-circuits with it.
pub(super) fn in_optional_chain(expr: &Expr) -> bool {
    match expr {
        Expr::OptionalProperty(..) | Expr::OptionalMethod(..) | Expr::OptionalIndex(..) => true,
        Expr::PropertyAccess(base, _)
        | Expr::IndexAccess(base, _)
        | Expr::MethodCall(base, _, _) => in_optional_chain(base),
        _ => false,
    }
}

/// Whether a `filter` callback keeps only the items that are there:
/// `x => x != null`, `x => null != x`.
pub(super) fn keeps_present(callback: &Expr) -> bool {
    let Expr::Lambda(param, body) = callback else {
        return false;
    };
    match body.as_ref() {
        Expr::BinaryOp(l, BinOp::Neq, r) => matches!(
            (l.as_ref(), r.as_ref()),
            (Expr::Identifier(x), Expr::Null) | (Expr::Null, Expr::Identifier(x)) if x == param
        ),
        _ => false,
    }
}

/// A short rendering of an expression, for a message.
pub fn expr_text(expr: &Expr) -> String {
    match expr {
        Expr::StringLiteral(s) => format!("\"{s}\""),
        Expr::Regex(p, f) => format!("/{p}/{f}"),
        Expr::InterpolatedString(_) => "\"…\"".to_string(),
        Expr::NumberLiteral(n) => format!("{n}"),
        Expr::BoolLiteral(b) => b.to_string(),
        Expr::Null => "null".to_string(),
        Expr::Typed(_, carrier) => expr_text(carrier),
        Expr::Identifier(n) => n.clone(),
        Expr::PropertyAccess(b, f) => format!("{}.{f}", expr_text(b)),
        Expr::IndexAccess(b, i) => format!("{}[{}]", expr_text(b), expr_text(i)),
        Expr::OptionalProperty(b, f) => format!("{}?.{f}", expr_text(b)),
        Expr::OptionalIndex(b, i) => format!("{}?.[{}]", expr_text(b), expr_text(i)),
        Expr::OptionalMethod(o, m, _) => format!("{}?.{m}(…)", expr_text(o)),
        Expr::BinaryOp(l, _, r) => format!("{} … {}", expr_text(l), expr_text(r)),
        Expr::UnaryOp(_, e) => format!("!{}", expr_text(e)),
        Expr::MethodCall(o, m, _) => format!("{}.{m}(…)", expr_text(o)),
        Expr::FunctionCall(n, _) => format!("{n}(…)"),
        Expr::ListLiteral(_) => "[…]".to_string(),
        Expr::Spread(e) => format!("...{}", expr_text(e)),
        Expr::Range(a, b, _) => format!("{}..{}", expr_text(a), expr_text(b)),
        Expr::MapLiteral(_) => "{…}".to_string(),
        Expr::Record(n, _) => format!("{n}(…)"),
        Expr::Lambda(p, _) if p.trim().is_empty() => "() => …".to_string(),
        Expr::Lambda(p, _) => format!("{p} => …"),
        Expr::EnumCase(c) => format!(".{c}"),
        Expr::CaseValue(c, args) => format!(
            ".{c}({})",
            args.iter().map(expr_text).collect::<Vec<_>>().join(", ")
        ),
        Expr::Token(t) => format!("${t}"),
        Expr::Await(e) => format!("await {}", expr_text(e)),
    }
}
