//! The tree written back as source: an expression, an argument, an element,
//! a statement, a type, a declaration — a component with its props, events,
//! slots, parts and `///`, a page, a store, an `api` — and a whole file:
//! what an editor that changes one node writes in that node's place, and
//! what it writes for a file it makes.
//!
//! What this prints parses back to the same tree. That is the whole
//! contract, and `tests/printer_corpus.rs` holds every statement of every
//! project in the repository to it. Where the parser lowers what was
//! written — `if c { a } else { b }` as a value is a call of `__if`, a
//! `match` a chain of them, `let { a, b } = m` three locals, a call's named
//! arguments one trailing map — the printer writes the spelling a person
//! would, which parses to the very same tree.
//!
//! The layout is the formatter's (`wf fmt`): four spaces a level, a block's
//! brace one space after what it opens, one statement a line. A node keeps
//! its own place in the file; `indent` is the depth of the line it starts
//! on, so the lines after its first sit where `wf fmt` would put them.
//!
//! Comments are not in the tree, so a node printed from it has none. That
//! is why an editor prints only the node it changed — [`crate::reprint`]
//! keeps every other byte, comments and all.

use crate::parser::ast::*;

const INDENT: &str = "    ";

/// What kind of block a statement is in: a render block holds elements and
/// declarations (`state`), an imperative one — an action, a handler, an
/// effect — statements (`let`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Render,
    Imperative,
}

// ─── Expressions ─────────────────────────────────────────

/// How tightly an expression binds, loosest first: what decides where a
/// printed expression needs parentheses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    /// `x => …`, which takes everything after its arrow.
    Lambda,
    Range,
    Coalesce,
    Or,
    And,
    Equality,
    Comparison,
    Additive,
    Multiplicative,
    Unary,
    Postfix,
    Primary,
}

fn binop(op: &BinOp) -> (&'static str, Prec) {
    match op {
        BinOp::NullCoalesce => ("??", Prec::Coalesce),
        BinOp::Or => ("||", Prec::Or),
        BinOp::And => ("&&", Prec::And),
        BinOp::Eq => ("==", Prec::Equality),
        BinOp::Neq => ("!=", Prec::Equality),
        BinOp::Lt => ("<", Prec::Comparison),
        BinOp::Gt => (">", Prec::Comparison),
        BinOp::Lte => ("<=", Prec::Comparison),
        BinOp::Gte => (">=", Prec::Comparison),
        BinOp::Add => ("+", Prec::Additive),
        BinOp::Sub => ("-", Prec::Additive),
        BinOp::Mul => ("*", Prec::Multiplicative),
        BinOp::Div => ("/", Prec::Multiplicative),
        BinOp::Mod => ("%", Prec::Multiplicative),
    }
}

fn prec_of(e: &Expr) -> Prec {
    match e {
        Expr::Lambda(..) => Prec::Lambda,
        Expr::Range(..) => Prec::Range,
        Expr::BinaryOp(_, op, _) => binop(op).1,
        Expr::UnaryOp(..) | Expr::Await(_) => Prec::Unary,
        Expr::PropertyAccess(..)
        | Expr::IndexAccess(..)
        | Expr::OptionalProperty(..)
        | Expr::OptionalIndex(..)
        | Expr::OptionalMethod(..) => Prec::Postfix,
        Expr::MethodCall(_, m, _) if is_lowered(m) => Prec::Lambda,
        Expr::MethodCall(..) => Prec::Postfix,
        // `-3` is written with a minus, which is an operator.
        Expr::NumberLiteral(n) if *n < 0.0 || (*n == 0.0 && n.is_sign_negative()) => Prec::Unary,
        _ => Prec::Primary,
    }
}

/// The calls the parser writes for what has no node of its own: `if` and
/// `match` as values.
fn is_lowered(method: &str) -> bool {
    matches!(method, "__if" | "__iflet")
}

/// `e`, in parentheses when it binds looser than `at least`.
fn wrapped(e: &Expr, at_least: Prec) -> String {
    let text = expr(e);
    if prec_of(e) < at_least {
        format!("({text})")
    } else {
        text
    }
}

/// `e` as the base of `.name`, `[i]` or `(…)`: a number or a literal of
/// the language's own types is parenthesised, since `3.days` and
/// `€12.99.times` would be read as something else.
fn base(e: &Expr) -> String {
    match e {
        Expr::NumberLiteral(_) | Expr::Typed(..) => format!("({})", expr(e)),
        _ => wrapped(e, Prec::Postfix),
    }
}

/// An expression as source.
pub fn expr(e: &Expr) -> String {
    match e {
        Expr::StringLiteral(s) => string(s),
        Expr::Regex(p, f) => format!("/{p}/{f}"),
        Expr::InterpolatedString(parts) => interpolated(parts),
        Expr::NumberLiteral(n) => number(*n),
        Expr::BoolLiteral(b) => b.to_string(),
        Expr::Null => "null".to_string(),
        Expr::Identifier(n) => n.clone(),
        Expr::PropertyAccess(b, f) => format!("{}.{f}", base(b)),
        Expr::IndexAccess(b, i) => format!("{}[{}]", base(b), expr(i)),
        Expr::OptionalProperty(b, f) => format!("{}?.{f}", base(b)),
        Expr::OptionalIndex(b, i) => format!("{}?.[{}]", base(b), expr(i)),
        Expr::OptionalMethod(o, m, args) => format!("{}?.{m}({})", base(o), call_args(args, false)),
        Expr::BinaryOp(l, op, r) => {
            let (word, prec) = binop(op);
            // Left to right: the left side may bind as loosely as the
            // operator, the right must bind tighter.
            let left = wrapped(l, prec);
            let right = wrapped(r, next(prec));
            format!("{left} {word} {right}")
        }
        Expr::UnaryOp(op, inner) => {
            let word = match op {
                UnaryOp::Not => "!",
                UnaryOp::Neg => "-",
            };
            // `- -a` and `-(-1)`: never two minuses side by side.
            let text = wrapped(inner, Prec::Unary);
            if word == "-" && text.starts_with('-') {
                format!("-({text})")
            } else {
                format!("{word}{text}")
            }
        }
        Expr::Await(inner) => format!("await {}", wrapped(inner, Prec::Unary)),
        Expr::MethodCall(o, m, args) => match lowered(e) {
            Some(text) => text,
            None => format!("{}.{m}({})", base(o), call_args(args, false)),
        },
        Expr::FunctionCall(n, args) => {
            let capital = n.chars().next().is_some_and(char::is_uppercase);
            format!("{n}({})", call_args(args, capital))
        }
        Expr::ListLiteral(items) => format!(
            "[{}]",
            items.iter().map(expr).collect::<Vec<_>>().join(", ")
        ),
        Expr::MapLiteral(pairs) => map(pairs),
        Expr::Spread(inner) => format!("...{}", wrapped(inner, Prec::Coalesce)),
        Expr::Range(a, b, inclusive) => format!(
            "{}{}{}",
            wrapped(a, Prec::Coalesce),
            if *inclusive { "..=" } else { ".." },
            wrapped(b, Prec::Coalesce)
        ),
        Expr::Record(n, fields) => format!(
            "{n}({})",
            fields
                .iter()
                .map(|(k, v)| format!("{k}: {}", expr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expr::Lambda(params, body) => {
            let head = if params.is_empty() || params.contains(',') {
                format!("({params})")
            } else {
                params.clone()
            };
            format!("{head} => {}", expr(body))
        }
        Expr::EnumCase(c) => format!(".{c}"),
        Expr::CaseValue(c, args) => format!(".{c}({})", call_args(args, false)),
        Expr::Token(t) => format!("${t}"),
        Expr::Typed(kind, carrier) => typed(kind, carrier),
    }
}

fn next(p: Prec) -> Prec {
    match p {
        Prec::Lambda => Prec::Range,
        Prec::Range => Prec::Coalesce,
        Prec::Coalesce => Prec::Or,
        Prec::Or => Prec::And,
        Prec::And => Prec::Equality,
        Prec::Equality => Prec::Comparison,
        Prec::Comparison => Prec::Additive,
        Prec::Additive => Prec::Multiplicative,
        Prec::Multiplicative => Prec::Unary,
        Prec::Unary => Prec::Postfix,
        Prec::Postfix | Prec::Primary => Prec::Primary,
    }
}

/// A number as the shortest text that reads back as the same number.
pub fn number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// A string literal: quoted, with what would end it or splice escaped.
pub fn string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    escape_into(text, &mut out);
    out.push('"');
    out
}

fn escape_into(text: &str, out: &mut String) {
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            c => out.push(c),
        }
    }
}

fn interpolated(parts: &[StringPart]) -> String {
    let mut out = String::from("\"");
    for part in parts {
        match part {
            StringPart::Literal(text) => escape_into(text, &mut out),
            StringPart::Expression(e) => {
                out.push('{');
                out.push_str(&splice(e));
                out.push('}');
            }
        }
    }
    out.push('"');
    out
}

/// An expression inside `{…}` in a string: a splice is only read as one
/// when it opens with a letter, `_`, `[` or `(`, and a `:` at its top
/// level would be read as a format; anything else goes in parentheses.
fn splice(e: &Expr) -> String {
    let text = expr(e);
    let opens = text
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || matches!(c, '_' | '[' | '('));
    if opens && !colon_at_top(&text) {
        text
    } else {
        format!("({text})")
    }
}

fn colon_at_top(text: &str) -> bool {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if in_string {
            match c {
                '\\' => {
                    chars.next();
                }
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ':' if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

/// A call's arguments. A trailing map of plain names is how the parser
/// holds `name: value` arguments, and is written that way — except where
/// a capitalised name with only named arguments would be read as a record.
fn call_args(args: &[Expr], capital: bool) -> String {
    let mut out: Vec<String> = Vec::new();
    for (i, a) in args.iter().enumerate() {
        let last = i + 1 == args.len();
        match a {
            Expr::MapLiteral(pairs)
                if last
                    && !pairs.is_empty()
                    && pairs.iter().all(|(k, _)| is_name(k))
                    && !(capital && args.len() == 1) =>
            {
                for (k, v) in pairs {
                    out.push(format!("{k}: {}", expr(v)));
                }
            }
            other => out.push(expr(other)),
        }
    }
    out.join(", ")
}

fn is_name(word: &str) -> bool {
    let mut chars = word.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

fn map(pairs: &[(String, Expr)]) -> String {
    if pairs.is_empty() {
        return "{}".to_string();
    }
    let items: Vec<String> = pairs
        .iter()
        .map(|(k, v)| {
            if k == "..." {
                format!("...{}", wrapped(v, Prec::Coalesce))
            } else if let Some(inner) = k.strip_prefix('"').and_then(|k| k.strip_suffix('"')) {
                // A quoted key, held as its text: the escapes a quote or a
                // backslash needs are put back.
                let mut out = String::from("\"");
                for c in inner.chars() {
                    match c {
                        '\\' => out.push_str("\\\\"),
                        '"' => out.push_str("\\\""),
                        '\n' => out.push_str("\\n"),
                        '\t' => out.push_str("\\t"),
                        c => out.push(c),
                    }
                }
                out.push('"');
                format!("{out}: {}", expr(v))
            } else {
                format!("{k}: {}", expr(v))
            }
        })
        .collect();
    format!("{{ {} }}", items.join(", "))
}

/// A literal of the language's own types, as it is written: `@2026-03-14`,
/// `#0F766E`, `€12.99`, `3.days`.
fn typed(kind: &str, carrier: &Expr) -> String {
    match (kind, carrier) {
        ("Date" | "Time" | "DateTime" | "Instant", Expr::StringLiteral(text)) => format!("@{text}"),
        ("Color", Expr::StringLiteral(text)) if text.starts_with('#') => text.clone(),
        ("Duration", Expr::NumberLiteral(ms)) => duration(*ms),
        ("Money", Expr::MapLiteral(pairs)) => {
            let amount = pairs.iter().find_map(|(k, v)| match (k.as_str(), v) {
                ("amount", Expr::NumberLiteral(n)) => Some(*n),
                _ => None,
            });
            let currency = pairs.iter().find_map(|(k, v)| match (k.as_str(), v) {
                ("currency", Expr::StringLiteral(c)) => Some(c.as_str()),
                _ => None,
            });
            let symbol = match currency {
                Some("EUR") => Some("€"),
                Some("GBP") => Some("£"),
                Some("JPY") => Some("¥"),
                Some("USD") => Some("$"),
                _ => None,
            };
            match (amount, symbol) {
                (Some(minor), Some(symbol)) if minor >= 0.0 && minor.fract() == 0.0 => {
                    let minor = minor as u64;
                    if minor.is_multiple_of(100) {
                        format!("{symbol}{}", minor / 100)
                    } else {
                        format!("{symbol}{}.{:02}", minor / 100, minor % 100)
                    }
                }
                _ => expr(carrier),
            }
        }
        _ => expr(carrier),
    }
}

/// A length of time in the largest unit that holds it whole.
fn duration(ms: f64) -> String {
    const UNITS: &[(&str, f64)] = &[
        ("weeks", 604_800_000.0),
        ("days", 86_400_000.0),
        ("hours", 3_600_000.0),
        ("minutes", 60_000.0),
        ("seconds", 1_000.0),
    ];
    for (unit, size) in UNITS {
        let n = ms / size;
        if ms != 0.0 && n.fract() == 0.0 && n * size == ms {
            return format!("{}.{unit}", number(n));
        }
    }
    format!("{}.ms", number(ms))
}

/// `if` and `match` as values, which the parser holds as calls: written
/// back as they are written.
fn lowered(e: &Expr) -> Option<String> {
    let Expr::MethodCall(subject, method, args) = e else {
        return None;
    };
    if let Some(text) = as_match(e) {
        return Some(text);
    }
    match (method.as_str(), args.as_slice()) {
        ("__if", [then, otherwise]) => Some(format!(
            "if {} {{ {} }}{}",
            expr(subject),
            expr(then),
            else_value(otherwise)
        )),
        ("__iflet", [Expr::Lambda(name, then), otherwise]) if is_name(name) => Some(format!(
            "if let {name} = {} {{ {} }}{}",
            expr(subject),
            expr(then),
            else_value(otherwise)
        )),
        _ => None,
    }
}

fn else_value(e: &Expr) -> String {
    match e {
        Expr::Null => String::new(),
        Expr::MethodCall(_, m, _) if is_lowered(m) && as_match(e).is_none() => {
            format!(" else {}", expr(e))
        }
        other => format!(" else {{ {} }}", expr(other)),
    }
}

/// A `match` expression, from the chain of conditions it was lowered to:
/// every condition is `subject.__is("case")` (or, binding the payload,
/// `subject.__payload("case").__iflet(…)`) over one subject, and the chain
/// ends in an `else` value or `subject.__exhaustive()`.
fn as_match(e: &Expr) -> Option<String> {
    let mut arms: Vec<String> = Vec::new();
    let mut subject: Option<String> = None;
    let mut at = e;
    let same = |s: &Expr, subject: &mut Option<String>| -> bool {
        let text = expr(s);
        match subject {
            Some(known) => *known == text,
            None => {
                *subject = Some(text);
                true
            }
        }
    };
    while let Expr::MethodCall(head, method, args) = at {
        match (method.as_str(), args.as_slice()) {
            ("__if", [then, rest]) => {
                let Expr::MethodCall(s, is, case) = head.as_ref() else {
                    return None;
                };
                let [Expr::StringLiteral(case)] = case.as_slice() else {
                    return None;
                };
                if is != "__is" || !same(s, &mut subject) {
                    return None;
                }
                arms.push(format!(".{case} {{ {} }}", expr(then)));
                at = rest;
            }
            ("__iflet", [Expr::Lambda(name, then), rest]) => {
                let Expr::MethodCall(s, payload, case) = head.as_ref() else {
                    return None;
                };
                let [Expr::StringLiteral(case)] = case.as_slice() else {
                    return None;
                };
                if payload != "__payload" || !same(s, &mut subject) {
                    return None;
                }
                arms.push(format!(".{case}({name}) {{ {} }}", expr(then)));
                at = rest;
            }
            ("__exhaustive", []) => {
                if !same(head, &mut subject) {
                    return None;
                }
                return (!arms.is_empty())
                    .then(|| format!("match {} {{ {} }}", subject.unwrap(), arms.join(" ")));
            }
            _ => break,
        }
    }
    // The last value is the `else` arm.
    if arms.is_empty() {
        return None;
    }
    arms.push(format!("else {{ {} }}", expr(at)));
    Some(format!("match {} {{ {} }}", subject?, arms.join(" ")))
}

// ─── Types ───────────────────────────────────────────────

/// A type as it is written: `[Todo]`, `String?`, `Number(min: 0, max: 100)`.
pub fn type_ref(t: &TypeRef) -> String {
    match t {
        TypeRef::String => "String".into(),
        TypeRef::Number => "Number".into(),
        TypeRef::Bool => "Bool".into(),
        TypeRef::Map => "Map".into(),
        TypeRef::Any => "Any".into(),
        TypeRef::List(inner) => format!("[{}]", type_ref(inner)),
        TypeRef::Optional(inner) => format!("{}?", type_ref(inner)),
        TypeRef::Named(n) => n.clone(),
        TypeRef::Refined(base, args) => {
            // `min` and then `max` or `below` are the range they were read
            // from: `Number(1..=30)`, `Number(0..10)`.
            let (range, rest) = match args.as_slice() {
                [(lo, from), (hi, to), rest @ ..]
                    if lo == "min" && (hi == "max" || hi == "below") =>
                {
                    let range =
                        Expr::Range(Box::new(from.clone()), Box::new(to.clone()), hi == "max");
                    (Some(expr(&range)), rest)
                }
                _ => (None, args.as_slice()),
            };
            let parts: Vec<String> = range
                .into_iter()
                .chain(rest.iter().map(|(k, v)| format!("{k}: {}", expr(v))))
                .collect();
            format!("{}({})", type_ref(base), parts.join(", "))
        }
    }
}

fn typed_name(name: &str, ty: &Option<TypeRef>) -> String {
    match ty {
        Some(t) => format!("{name}: {}", type_ref(t)),
        None => name.to_string(),
    }
}

// ─── Elements ────────────────────────────────────────────

/// The name an element is called by: `Card`, `Card.Header`, `DeployRow`.
pub fn component(c: &ComponentRef) -> String {
    match c {
        ComponentRef::BuiltIn(n) | ComponentRef::UserDefined(n) => n.clone(),
        ComponentRef::SubComponent(owner, part) => format!("{owner}.{part}"),
    }
}

/// One argument of an element: `"Save"`, `gap: .md`.
pub fn arg(a: &Arg) -> String {
    match a {
        Arg::Positional(e) => expr(e),
        Arg::Named(k, e) => format!("{k}: {}", expr(e)),
    }
}

/// An element's call, without its block: `Button("Save", tone: .primary).lg`.
pub fn call(ui: &UIElement) -> String {
    let mut out = component(&ui.component);
    if !ui.args.is_empty() {
        out.push('(');
        out.push_str(&ui.args.iter().map(arg).collect::<Vec<_>>().join(", "));
        out.push(')');
    }
    for m in &ui.modifiers {
        out.push('.');
        out.push_str(m);
    }
    out
}

/// A whole element, its block and everything in it, starting on a line
/// indented `indent` levels.
pub fn element(ui: &UIElement, indent: usize) -> String {
    // What the parser makes of something that is not written as a call.
    match &ui.component {
        ComponentRef::BuiltIn(n) if n == "Children" => return slot_use(ui),
        ComponentRef::BuiltIn(n) if n == "__Head" => {
            let mut out = String::from("head {\n");
            for child in &ui.children {
                if let StatementKind::UIElement(tag) = &child.kind {
                    out.push_str(&pad(indent + 1));
                    out.push_str(&call(tag));
                    out.push('\n');
                }
            }
            out.push_str(&pad(indent));
            out.push('}');
            return out;
        }
        ComponentRef::BuiltIn(n) if n.starts_with("__") => {
            if let Some(step) = test_step(n, ui) {
                return step;
            }
        }
        _ => {}
    }
    let mut out = call(ui);
    let mut lines: Vec<String> = Vec::new();
    if let Some(style) = &ui.style_block {
        lines.push(style_block(style, indent + 1));
    }
    if let Some(t) = &ui.transition_block {
        lines.push(transition(t, indent + 1));
    }
    for h in &ui.events {
        lines.push(handler(h, indent + 1));
    }
    for fill in &ui.slot_fills {
        lines.push(slot_fill(fill, indent + 1));
    }
    lines.extend(statements(&ui.children, Block::Render, indent + 1));
    if !lines.is_empty() {
        out.push_str(" {\n");
        for line in lines {
            out.push_str(&pad(indent + 1));
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str(&pad(indent));
        out.push('}');
    }
    out
}

/// A slot filled at a call: `trailing { Badge("new") }`, `row(item) { … }`.
pub fn slot_fill(fill: &SlotFill, indent: usize) -> String {
    let params = if fill.params.is_empty() {
        String::new()
    } else {
        format!("({})", fill.params.join(", "))
    };
    format!(
        "{}{params}{}",
        fill.name,
        body(&fill.body, Block::Render, indent)
    )
}

/// `children`, `trailing`, or `row(item: t)`.
fn slot_use(ui: &UIElement) -> String {
    let name = ui.slot_name().unwrap_or("children").to_string();
    let values: Vec<String> = ui
        .args
        .iter()
        .filter(|a| !matches!(a, Arg::Named(k, _) if k == "slot"))
        .map(arg)
        .collect();
    if values.is_empty() {
        name
    } else {
        format!("{name}({})", values.join(", "))
    }
}

/// A test's step: `expect "Saved"`, `click "Save"`, `type "Ada" into "Name"`.
fn test_step(name: &str, ui: &UIElement) -> Option<String> {
    let positional: Vec<&Expr> = ui
        .args
        .iter()
        .filter_map(|a| match a {
            Arg::Positional(e) => Some(e),
            Arg::Named(..) => None,
        })
        .collect();
    Some(match (name, positional.as_slice()) {
        ("__Expect", [text]) => {
            let not = if ui.modifiers.iter().any(|m| m == "not") {
                "not "
            } else {
                ""
            };
            format!("expect {not}{}", expr(text))
        }
        ("__Click", [target]) => format!("click {}", expr(target)),
        ("__Type", [text, into]) => format!("type {} into {}", expr(text), expr(into)),
        ("__Press", [key]) => format!("press {}", expr(key)),
        ("__Press", [key, target]) => format!("press {} in {}", expr(key), expr(target)),
        _ => return None,
    })
}

/// A style block: `style { padding: 12px … }`.
pub fn style_block(s: &StyleBlock, indent: usize) -> String {
    let mut out = String::from("style {\n");
    for p in &s.properties {
        out.push_str(&pad(indent + 1));
        out.push_str(&style_property(p));
        out.push('\n');
    }
    for (selector, props) in s
        .pseudo_blocks
        .iter()
        .map(|b| (&b.state, &b.properties))
        .chain(
            s.media_queries
                .iter()
                .map(|m| (&m.condition, &m.properties)),
        )
    {
        out.push_str(&pad(indent + 1));
        out.push_str(selector);
        out.push_str(" {\n");
        for p in props {
            out.push_str(&pad(indent + 2));
            out.push_str(&style_property(p));
            out.push('\n');
        }
        out.push_str(&pad(indent + 1));
        out.push_str("}\n");
    }
    out.push_str(&pad(indent));
    out.push('}');
    out
}

/// One declaration of a style block: `padding: 12px 16px`.
pub fn style_property(p: &StyleProperty) -> String {
    format!("{}: {}", p.name, style_value(&p.value))
}

/// A style value: its text, with `$token`s and `{splices}` in their places.
pub fn style_value(e: &Expr) -> String {
    match e {
        Expr::StringLiteral(raw) => raw.clone(),
        Expr::Token(t) => format!("${t}"),
        Expr::InterpolatedString(parts) => parts
            .iter()
            .map(|p| match p {
                StringPart::Literal(text) => text.clone(),
                StringPart::Expression(Expr::Token(t)) => format!("${t}"),
                StringPart::Expression(e) => format!("{{{}}}", expr(e)),
            })
            .collect(),
        other => format!("{{{}}}", expr(other)),
    }
}

/// `transition { opacity: fast $ease-out }`.
pub fn transition(t: &TransitionBlock, indent: usize) -> String {
    let token = |word: &str| match word
        .strip_prefix("var(--")
        .and_then(|w| w.strip_suffix(')'))
    {
        Some(name) => format!("${name}"),
        None => word.to_string(),
    };
    let mut out = String::from("transition {\n");
    for p in &t.properties {
        out.push_str(&pad(indent + 1));
        out.push_str(&p.property);
        out.push_str(": ");
        out.push_str(&token(&p.duration));
        if let Some(e) = &p.easing {
            out.push(' ');
            out.push_str(&token(e));
        }
        out.push('\n');
    }
    out.push_str(&pad(indent));
    out.push('}');
    out
}

/// `on click(e) { … }`, `on key("mod+k") { … }`.
pub fn handler(h: &EventHandler, indent: usize) -> String {
    let head = match (&h.key, &h.param) {
        (Some(key), Some(p)) => format!("({}, {p})", string(key)),
        (Some(key), None) => format!("({})", string(key)),
        (None, Some(p)) => format!("({p})"),
        (None, None) => String::new(),
    };
    format!(
        "on {}{head}{}",
        h.event,
        body(&h.body, Block::Imperative, indent)
    )
}

// ─── Statements ──────────────────────────────────────────

fn pad(indent: usize) -> String {
    INDENT.repeat(indent)
}

/// ` { … }`: a block of statements, its brace on the line it opens, its
/// statements one level in, and its closing brace at `indent`.
pub fn body(stmts: &[Statement], block: Block, indent: usize) -> String {
    let lines = statements(stmts, block, indent + 1);
    if lines.is_empty() {
        return " {}".to_string();
    }
    let mut out = String::from(" {\n");
    for line in lines {
        out.push_str(&pad(indent + 1));
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(&pad(indent));
    out.push('}');
    out
}

/// The statements of a block, each as source, gathering back what the
/// parser spread out: `let { a, b } = m` is one statement written.
pub fn statements(stmts: &[Statement], block: Block, indent: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < stmts.len() {
        if let Some((text, used)) = destructure(&stmts[i..]) {
            out.push(text);
            i += used;
            continue;
        }
        out.push(statement(&stmts[i], block, indent));
        i += 1;
    }
    out
}

/// How many statements, from the first, are one destructuring `let`, and
/// how it is written: the value under a `d__N` name, then a local for
/// each of its fields or items.
pub fn destructure(stmts: &[Statement]) -> Option<(String, usize)> {
    let StatementKind::State(first) = &stmts.first()?.kind else {
        return None;
    };
    let temp = &first.name;
    if !temp
        .strip_prefix("d__")
        .is_some_and(|n| n.parse::<u32>().is_ok())
        || first.ty.is_some()
    {
        return None;
    }
    let mut names = Vec::new();
    let mut by_index = None;
    for stmt in &stmts[1..] {
        let StatementKind::State(s) = &stmt.kind else {
            break;
        };
        let (read_name, index) = match &s.value {
            Expr::PropertyAccess(b, field) if matches!(b.as_ref(), Expr::Identifier(t) if t == temp) =>
            {
                if field != &s.name {
                    break;
                }
                (true, false)
            }
            Expr::IndexAccess(b, i)
                if matches!(b.as_ref(), Expr::Identifier(t) if t == temp)
                    && matches!(i.as_ref(), Expr::NumberLiteral(n) if *n == names.len() as f64) =>
            {
                (true, true)
            }
            _ => (false, false),
        };
        if !read_name || by_index.is_some_and(|b| b != index) {
            break;
        }
        by_index = Some(index);
        names.push(s.name.clone());
    }
    if names.is_empty() {
        return None;
    }
    let (open, close) = if by_index == Some(true) {
        ("[", "]")
    } else {
        ("{ ", " }")
    };
    Some((
        format!(
            "let {open}{}{close} = {}",
            names.join(", "),
            expr(&first.value)
        ),
        names.len() + 1,
    ))
}

/// One statement, starting on a line indented `indent` levels, in a block
/// of the kind `block` says.
pub fn statement(s: &Statement, block: Block, indent: usize) -> String {
    let text = bare_statement(s, block, indent);
    // A `state`, `derived` or `action` of a body is written under its `///`.
    match s.doc() {
        Some(_) if block != Block::Imperative => {
            documented(&s.doc().map(str::to_string), &text, indent)
        }
        _ => text,
    }
}

/// One statement without the `///` above it.
fn bare_statement(s: &Statement, block: Block, indent: usize) -> String {
    match &s.kind {
        StatementKind::UIElement(ui) => element(ui, indent),
        StatementKind::State(st) => {
            if block == Block::Imperative {
                return format!("let {} = {}", typed_name(&st.name, &st.ty), expr(&st.value));
            }
            let word = if st.persist { "persist" } else { "state" };
            let mut out = format!(
                "{word} {} = {}",
                typed_name(&st.name, &st.ty),
                expr(&st.value)
            );
            if let Some(policy) = &st.policy {
                out.push_str(&persist_policy(policy, indent));
            }
            out
        }
        StatementKind::Derived(d) => {
            format!(
                "derived {} = {}",
                typed_name(&d.name, &d.ty),
                expr(&d.value)
            )
        }
        StatementKind::Effect(e) => {
            let mut lines = statements(&e.body, Block::Imperative, indent + 1);
            if !e.cleanup.is_empty() {
                lines.push(format!(
                    "cleanup{}",
                    body(&e.cleanup, Block::Imperative, indent + 1)
                ));
            }
            block_of("effect", lines, indent)
        }
        StatementKind::Timer(t) => format!(
            "{}({}){}",
            if t.every { "every" } else { "after" },
            expr(&t.interval),
            body(&t.body, Block::Imperative, indent)
        ),
        StatementKind::Action(a) => {
            let params = if a.params.is_empty() {
                String::new()
            } else {
                format!(
                    "({})",
                    a.params
                        .iter()
                        .map(|p| match &p.param_type {
                            TypeRef::Any => p.name.clone(),
                            t => format!("{}: {}", p.name, type_ref(t)),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            format!(
                "action {}{params}{}",
                a.name,
                body(&a.body, Block::Imperative, indent)
            )
        }
        StatementKind::If(i) => {
            let head = match &i.binding {
                Some(name) => format!("if let {name} = {}", expr(&i.condition)),
                None => format!("if {}", expr(&i.condition)),
            };
            let mut out = format!("{head}{}", body(&i.then_body, block, indent));
            for (cond, b) in &i.else_if_branches {
                out.push_str(&format!(
                    " else if {}{}",
                    expr(cond),
                    body(b, block, indent)
                ));
            }
            if let Some(b) = &i.else_body {
                out.push_str(&format!(" else{}", body(b, block, indent)));
            }
            out
        }
        StatementKind::For(f) => {
            let item = match &f.index {
                Some(index) => format!("{}, {index}", f.item),
                None => f.item.clone(),
            };
            let key = f
                .key
                .as_ref()
                .map(|k| format!(" by {}", expr(k)))
                .unwrap_or_default();
            format!(
                "for {item} in {}{key}{}",
                expr(&f.iterable),
                body(&f.body, block, indent)
            )
        }
        StatementKind::Show(sh) => format!(
            "show {}{}",
            expr(&sh.condition),
            body(&sh.body, Block::Render, indent)
        ),
        StatementKind::Fetch(f) => {
            let mut out = format!("fetch {} from {}", f.variable, expr(&f.url));
            if !f.options.is_empty() {
                out.push_str(&format!(
                    " ({})",
                    f.options
                        .iter()
                        .map(|o| format!("{}: {}", o.key, expr(&o.value)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            let mut lines = Vec::new();
            if let Some(b) = &f.loading_block {
                lines.push(format!("loading{}", body(b, Block::Render, indent + 1)));
            }
            if let Some((name, b)) = &f.error_block {
                lines.push(format!(
                    "error({name}){}",
                    body(b, Block::Render, indent + 1)
                ));
            }
            if let Some(b) = &f.success_block {
                lines.push(format!("success{}", body(b, Block::Render, indent + 1)));
            }
            if !lines.is_empty() {
                out.push_str(&block_of("", lines, indent));
            }
            out
        }
        StatementKind::Assignment(a) => format!("{} = {}", expr(&a.target), expr(&a.value)),
        StatementKind::MethodCall(m) => format!(
            "{}.{}({})",
            base(&m.object),
            m.method,
            call_args(&m.args, false)
        ),
        StatementKind::Use(u) => format!("use {}", u.store_name),
        StatementKind::EventHandler(h) => handler(h, indent),
        StatementKind::Navigate(e) => format!("navigate({})", expr(e)),
        StatementKind::Log(e) => format!("log({})", expr(e)),
        StatementKind::Animate(a) => {
            let mut args = vec![a.target.clone(), string(&a.animation)];
            if let Some(d) = &a.duration {
                args.push(string(d));
            }
            format!("animate({})", args.join(", "))
        }
        StatementKind::ExprStatement(e) => expr(e),
        StatementKind::Return(None) => "return".to_string(),
        StatementKind::Return(Some(e)) => format!("return {}", expr(e)),
        StatementKind::Resource(r) => {
            let value = if r.options.is_empty()
                && !matches!(r.url, Expr::StringLiteral(_) | Expr::InterpolatedString(_))
            {
                expr(&r.url)
            } else {
                let mut args = vec![expr(&r.url)];
                args.extend(
                    r.options
                        .iter()
                        .map(|o| format!("{}: {}", o.key, expr(&o.value))),
                );
                format!("fetch({})", args.join(", "))
            };
            format!("resource {} = {value}", typed_name(&r.name, &r.ty))
        }
        StatementKind::Connection(c) => connection(c, indent),
        StatementKind::Validate(v) => {
            let lines: Vec<String> = v
                .rules
                .iter()
                .map(|r| {
                    let mut out = r.name.clone();
                    if !r.args.is_empty() {
                        out.push_str(&format!(
                            "({})",
                            r.args.iter().map(expr).collect::<Vec<_>>().join(", ")
                        ));
                    }
                    if let Some(m) = &r.message {
                        out.push(' ');
                        out.push_str(&expr(m));
                    }
                    if let Some(b) = &r.body {
                        out.push_str(&format!(" {{ {} }}", expr(b)));
                    }
                    out
                })
                .collect();
            block_of(&format!("validate {}", v.name), lines, indent)
        }
        StatementKind::Match(m) => {
            let lines: Vec<String> = m
                .arms
                .iter()
                .map(|arm| {
                    let head = match &arm.pattern {
                        ArmPattern::Loading => "loading".to_string(),
                        ArmPattern::Error => "error".to_string(),
                        ArmPattern::Ready => "ready".to_string(),
                        ArmPattern::Else => "else".to_string(),
                        ArmPattern::State(s) => s.clone(),
                        ArmPattern::Case(c) => format!(".{c}"),
                    };
                    let binds = match (&arm.binding, arm.bindings.is_empty()) {
                        (Some(b), _) => format!("({b})"),
                        (None, false) => format!("({})", arm.bindings.join(", ")),
                        (None, true) => String::new(),
                    };
                    format!(
                        "{head}{binds}{}",
                        body(&arm.body, Block::Render, indent + 1)
                    )
                })
                .collect();
            block_of(&format!("match {}", expr(&m.scrutinee)), lines, indent)
        }
        StatementKind::Emit(e) => {
            if e.args.is_empty() {
                format!("emit {}", e.event)
            } else {
                format!(
                    "emit {}({})",
                    e.event,
                    e.args.iter().map(expr).collect::<Vec<_>>().join(", ")
                )
            }
        }
        StatementKind::Try(t) => {
            let param = t
                .param
                .as_ref()
                .map(|p| format!(" {p}"))
                .unwrap_or_default();
            format!(
                "try{} catch{param}{}",
                body(&t.body, Block::Imperative, indent),
                body(&t.catch_body, Block::Imperative, indent)
            )
        }
    }
}

/// `head { line … }` with its lines one level in.
fn block_of(head: &str, lines: Vec<String>, indent: usize) -> String {
    let mut out = if head.is_empty() {
        String::from(" {")
    } else {
        format!("{head} {{")
    };
    if lines.is_empty() {
        out.push('}');
        return out;
    }
    out.push('\n');
    for line in lines {
        out.push_str(&pad(indent + 1));
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(&pad(indent));
    out.push('}');
    out
}

fn persist_policy(p: &PersistPolicy, indent: usize) -> String {
    let mut lines = Vec::new();
    if let Some(s) = &p.storage {
        lines.push(format!("in: .{s}"));
    }
    if let Some(v) = p.version {
        lines.push(format!("version: {v}"));
    }
    if let Some(s) = p.sync {
        lines.push(format!("sync: {s}"));
    }
    if let Some(k) = &p.key {
        lines.push(format!("key: {}", expr(k)));
    }
    for m in &p.migrations {
        lines.push(format!(
            "migrate {} -> {} {{ {} }}",
            m.from,
            m.to,
            expr(&m.body)
        ));
    }
    format!(" {}", block_of("", lines, indent).trim_start())
}

fn connection(c: &ConnectionDecl, indent: usize) -> String {
    let (word, opener) = match c.kind {
        ConnectionKind::Socket => ("socket", "ws"),
        ConnectionKind::Stream => ("stream", "sse"),
        ConnectionKind::Channel => ("channel", "broadcast"),
        ConnectionKind::Peer => ("peer", "rtc"),
        ConnectionKind::Worker => ("worker", "spawn"),
    };
    let mut args: Vec<String> = c.url.iter().map(expr).collect();
    args.extend(c.options.iter().map(|(k, v)| format!("{k}: {}", expr(v))));
    let mut out = format!("{word} {} = {opener}({})", c.name, args.join(", "));
    let mut lines = Vec::new();
    if let Some(t) = &c.sends {
        lines.push(format!("send {}", type_ref(t)));
    }
    if let Some(t) = &c.receives {
        lines.push(format!("receive {}", type_ref(t)));
    }
    for h in &c.handlers {
        lines.push(handler(h, indent + 1));
    }
    if !lines.is_empty() {
        out.push_str(&block_of("", lines, indent));
    }
    out
}

// ─── Declarations ────────────────────────────────────────

/// A `///` comment, one line of it a line, each at `indent`; nothing when
/// there is none.
pub fn doc(doc: &Option<String>, indent: usize) -> String {
    match doc {
        Some(text) => text
            .lines()
            .map(|l| {
                if l.is_empty() {
                    format!("{}///\n", pad(indent))
                } else {
                    format!("{}/// {l}\n", pad(indent))
                }
            })
            .collect(),
        None => String::new(),
    }
}

/// A member with its `///` above it, as a block's line: the comment's
/// lines, then the member at `indent` — the first line left for the block
/// to indent.
pub fn documented(d: &Option<String>, text: &str, indent: usize) -> String {
    let lines = doc(d, indent);
    if lines.is_empty() {
        text.to_string()
    } else {
        format!("{}{}{text}", lines.trim_start(), pad(indent))
    }
}

/// A string the parser reads as text with its `{name}` splices kept as
/// written — a page's title, a path: a splice of a plain name is written
/// as it is, any other brace escaped.
fn text_with_names(text: &str) -> String {
    let mut out = String::from("\"");
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '{' {
            let close = chars[i + 1..]
                .iter()
                .position(|c| *c == '}')
                .map(|n| i + 1 + n);
            if let Some(close) = close {
                let inner: String = chars[i + 1..close].iter().collect();
                if is_name(&inner) {
                    out.push('{');
                    out.push_str(&inner);
                    out.push('}');
                    i = close + 1;
                    continue;
                }
            }
        }
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            c => out.push(c),
        }
        i += 1;
    }
    out.push('"');
    out
}

/// One prop of a component or a parameter of an endpoint:
/// `_ title: String = "x"`.
pub fn prop(p: &PropDecl) -> String {
    let mut out = String::new();
    if p.positional {
        out.push_str("_ ");
    }
    out.push_str(&format!("{}: {}", p.name, type_ref(&p.prop_type)));
    if let Some(d) = &p.default {
        out.push_str(&format!(" = {}", expr(d)));
    }
    out
}

/// `(a: T, b: U)`, or one a line when any of them has a `///` of its own.
fn prop_list(props: &[PropDecl], indent: usize) -> String {
    if props.is_empty() {
        return String::new();
    }
    if props.iter().all(|p| p.doc.is_none()) {
        return format!(
            "({})",
            props.iter().map(prop).collect::<Vec<_>>().join(", ")
        );
    }
    let mut out = String::from("(\n");
    for p in props {
        out.push_str(&doc(&p.doc, indent + 1));
        out.push_str(&pad(indent + 1));
        out.push_str(&prop(p));
        out.push_str(",\n");
    }
    out.push_str(&pad(indent));
    out.push(')');
    out
}

fn params(ps: &[ParamDecl]) -> String {
    ps.iter()
        .map(|p| match &p.param_type {
            TypeRef::Any => p.name.clone(),
            t => format!("{}: {}", p.name, type_ref(t)),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A component's header: `component Card(_ title: String, tone: Tone = .calm)`,
/// or a part's, `part Header(…)`.
pub fn component_header(c: &ComponentDecl, indent: usize) -> String {
    let (word, name) = match c.name.split_once('.') {
        Some((_, part)) => ("part", part),
        None => ("component", c.name.as_str()),
    };
    format!("{word} {name}{}", prop_list(&c.props, indent))
}

/// `event toggle(id: String)`.
pub fn event_decl(e: &EventDecl) -> String {
    if e.params.is_empty() {
        format!("event {}", e.name)
    } else {
        format!("event {}({})", e.name, params(&e.params))
    }
}

/// `slot`, `slot trailing`, `slot row(item: Todo)`.
pub fn slot_decl(s: &SlotDecl) -> String {
    match (&s.name, s.params.is_empty()) {
        (None, _) => "slot".to_string(),
        (Some(n), true) => format!("slot {n}"),
        (Some(n), false) => format!("slot {n}({})", params(&s.params)),
    }
}

/// A component whole: its header, then its block — the events, slots and
/// parts it declares, then what it renders. `parts` are its parts as the
/// program holds them, declarations of their own.
pub fn component_decl(c: &ComponentDecl, parts: &[&ComponentDecl], indent: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    for e in &c.events {
        lines.push(documented(&e.doc, &event_decl(e), indent + 1));
    }
    for s in &c.slots {
        lines.push(slot_decl(s));
    }
    for name in &c.parts {
        let full = format!("{}.{name}", c.name);
        if let Some(p) = parts.iter().find(|p| p.name == full) {
            lines.push(documented(
                &p.doc,
                &component_decl(p, &[], indent + 1),
                indent + 1,
            ));
        }
    }
    lines.extend(statements(&c.body, Block::Render, indent + 1));
    format!(
        "{}{}",
        component_header(c, indent),
        lines_block(lines, indent)
    )
}

/// ` {` the lines, one level in, `}`.
fn lines_block(lines: Vec<String>, indent: usize) -> String {
    if lines.is_empty() {
        return " {}".to_string();
    }
    let mut out = String::from(" {\n");
    for line in lines {
        out.push_str(&pad(indent + 1));
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(&pad(indent));
    out.push('}');
    out
}

/// A page's header: `page Deploy(path: "/deploys/:id", title: "…", id: String)`.
pub fn page_header(p: &PageDecl) -> String {
    let mut attrs: Vec<String> = vec![format!("path: {}", text_with_names(&p.path))];
    let text = |s: &str, e: &Option<Expr>| match e {
        Some(e) => expr(e),
        None => text_with_names(s),
    };
    if let Some(t) = &p.title {
        attrs.push(format!("title: {}", text(t, &p.title_expr)));
    }
    if let Some(d) = &p.description {
        attrs.push(format!("description: {}", text(d, &p.description_expr)));
    }
    if let Some(i) = &p.image {
        attrs.push(format!("image: {}", text_with_names(i)));
    }
    if let Some(i) = &p.image_alt {
        attrs.push(format!("image_alt: {}", text_with_names(i)));
    }
    if let Some(t) = &p.page_type {
        attrs.push(format!("type: {}", text_with_names(t)));
    }
    if let Some(l) = &p.layout {
        let args = if l.args.is_empty() {
            String::new()
        } else {
            format!(
                "({})",
                l.args.iter().map(arg).collect::<Vec<_>>().join(", ")
            )
        };
        attrs.push(format!("layout: {}{args}", l.name));
    }
    if let Some(g) = &p.guard {
        attrs.push(format!("guard: {}", expr(g)));
    }
    if let Some(r) = &p.redirect {
        attrs.push(format!("redirect: {}", text_with_names(r)));
    }
    if p.noindex {
        attrs.push("noindex: true".to_string());
    }
    if let Some(x) = &p.paths {
        attrs.push(format!("paths: {}", expr(x)));
    }
    for param in &p.params {
        attrs.push(format!("{}: {}", param.name, type_ref(&param.prop_type)));
    }
    format!("page {}({})", p.name, attrs.join(", "))
}

/// A page whole: its header, its `head { … }`, and its body.
pub fn page(p: &PageDecl, indent: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    if !p.head.is_empty() {
        let tags: Vec<String> = p
            .head
            .iter()
            .map(|t| {
                format!(
                    "{}({})",
                    t.tag,
                    t.attrs
                        .iter()
                        .map(|(k, v)| format!("{k}: {}", expr(v)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();
        lines.push(format!("head{}", lines_block(tags, indent + 1)));
    }
    lines.extend(statements(&p.body, Block::Render, indent + 1));
    format!("{}{}", page_header(p), lines_block(lines, indent))
}

/// `store Cart(scope: .route, eager: true)`.
pub fn store_header(s: &StoreDecl) -> String {
    let mut opts = Vec::new();
    if s.scope != StoreScope::App {
        opts.push(format!("scope: .{}", s.scope.as_str()));
    }
    if s.eager {
        opts.push("eager: true".to_string());
    }
    if opts.is_empty() {
        format!("store {}", s.name)
    } else {
        format!("store {}({})", s.name, opts.join(", "))
    }
}

/// One field of a record or a case's payload: `title: String = ""`.
pub fn field(f: &FieldDecl) -> String {
    match &f.default {
        Some(d) => format!("{}: {} = {}", f.name, type_ref(&f.ty), expr(d)),
        None => format!("{}: {}", f.name, type_ref(&f.ty)),
    }
}

/// `type Admin = User`.
pub fn type_header(t: &TypeDecl) -> String {
    match &t.extends {
        Some(base) => format!("type {} = {base}", t.name),
        None => format!("type {}", t.name),
    }
}

/// A `theme` token: `color-primary: #0F766E`.
pub fn token(t: &ThemeToken) -> String {
    format!("{}: {}", t.name, style_value(&t.value))
}

/// A header's name as an `api` block writes it: bare when it is words and
/// hyphens, quoted otherwise.
fn header_key(k: &str) -> String {
    if !k.is_empty() && k.split('-').all(|w| !w.is_empty() && is_name(w)) {
        k.to_string()
    } else {
        string(k)
    }
}

/// One endpoint of an `api`: `get user(id: String) at "users/:id" -> User`.
pub fn endpoint(e: &Endpoint) -> String {
    let mut out = format!("{} {}", e.method.to_lowercase(), e.name);
    if !e.params.is_empty() {
        out.push_str(&format!(
            "({})",
            e.params.iter().map(prop).collect::<Vec<_>>().join(", ")
        ));
    }
    if e.path != e.name {
        out.push_str(&format!(" at {}", text_with_names(&e.path)));
    }
    if let Some(r) = &e.returns {
        out.push_str(&format!(" -> {}", type_ref(r)));
    }
    if !e.errors.is_empty() {
        out.push_str(&format!(
            " errors {{ {} }}",
            e.errors
                .iter()
                .map(|(c, t)| format!("{c} -> {}", type_ref(t)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for (k, v) in &e.settings {
        out.push_str(&format!(" {k}: {}", expr(v)));
    }
    out
}

/// A declaration whole, with its `///` above it, at the top of a file.
/// `program` is where a component's parts are found.
pub fn declaration(d: &Declaration, program: &Program) -> String {
    let parts: Vec<&ComponentDecl> = program
        .declarations
        .iter()
        .filter_map(|x| match x {
            Declaration::Component(c) if c.name.contains('.') => Some(c),
            _ => None,
        })
        .collect();
    match d {
        Declaration::Page(p) => page(p, 0),
        Declaration::Component(c) => format!("{}{}", doc(&c.doc, 0), component_decl(c, &parts, 0)),
        Declaration::Store(s) => format!(
            "{}{}",
            store_header(s),
            lines_block(statements(&s.body, Block::Render, 1), 0)
        ),
        Declaration::App(a) => format!(
            "app{}",
            lines_block(statements(&a.body, Block::Render, 1), 0)
        ),
        Declaration::Theme(t) => format!(
            "theme {}{}",
            t.name,
            lines_block(t.tokens.iter().map(token).collect(), 0)
        ),
        Declaration::Type(t) => {
            let lines: Vec<String> = t
                .fields
                .iter()
                .map(|f| documented(&f.doc, &field(f), 1))
                .collect();
            format!(
                "{}{}{}",
                doc(&t.doc, 0),
                type_header(t),
                lines_block(lines, 0)
            )
        }
        Declaration::Enum(e) => {
            let lines: Vec<String> = e
                .cases
                .iter()
                .map(|c| {
                    if c.fields.is_empty() {
                        c.name.clone()
                    } else {
                        format!(
                            "{}({})",
                            c.name,
                            c.fields.iter().map(field).collect::<Vec<_>>().join(", ")
                        )
                    }
                })
                .collect();
            format!("{}enum {}{}", doc(&e.doc, 0), e.name, lines_block(lines, 0))
        }
        Declaration::Const(c) => format!(
            "{}const {} = {}",
            doc(&c.doc, 0),
            typed_name(&c.name, &c.ty),
            expr(&c.value)
        ),
        Declaration::Data(x) => format!(
            "{}{} {} = {}",
            doc(&x.doc, 0),
            if x.is_image { "image" } else { "data" },
            typed_name(&x.name, &x.ty),
            text_with_names(&x.file)
        ),
        Declaration::Animation(a) => {
            let lines: Vec<String> = a
                .frames
                .iter()
                .map(|f| {
                    format!(
                        "{}{}",
                        f.selector,
                        lines_block(f.properties.iter().map(style_property).collect(), 1)
                    )
                })
                .collect();
            format!(
                "{}animation {}{}",
                doc(&a.doc, 0),
                a.name,
                lines_block(lines, 0)
            )
        }
        Declaration::Test(t) => {
            let data = t
                .data
                .as_ref()
                .map(|d| format!("(data: {})", expr(d)))
                .unwrap_or_default();
            let mut lines = statements(&t.body, Block::Render, 1);
            for s in &t.steps {
                lines.push(match s {
                    Step::Expect { text, negated, .. } => {
                        format!(
                            "expect {}{}",
                            if *negated { "not " } else { "" },
                            expr(text)
                        )
                    }
                    Step::Click { target, .. } => format!("click {}", expr(target)),
                    Step::Type { text, into, .. } => {
                        format!("type {} into {}", expr(text), expr(into))
                    }
                    Step::Press {
                        key, target: None, ..
                    } => format!("press {}", expr(key)),
                    Step::Press {
                        key,
                        target: Some(t),
                        ..
                    } => {
                        format!("press {} in {}", expr(key), expr(t))
                    }
                });
            }
            format!(
                "test {}{data}{}",
                text_with_names(&t.name),
                lines_block(lines, 0)
            )
        }
        Declaration::Api(a) => {
            let mut head = format!("api {}", a.name);
            if let Some(f) = &a.from {
                head.push_str(&format!(" from {}", text_with_names(f)));
            }
            if !a.settings.is_empty() {
                head.push_str(&format!(
                    "({})",
                    a.settings
                        .iter()
                        .map(|(k, v)| format!("{k}: {}", expr(v)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            let mut lines = Vec::new();
            if !a.headers.is_empty() {
                let hs: Vec<String> = a
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{}: {}", header_key(k), expr(v)))
                    .collect();
                lines.push(format!("headers{}", lines_block(hs, 1)));
            }
            for h in &a.hooks {
                lines.push(handler(h, 1));
            }
            for e in &a.endpoints {
                lines.push(documented(&e.doc, &endpoint(e), 1));
            }
            let body = if lines.is_empty() && a.from.is_some() {
                String::new()
            } else {
                lines_block(lines, 0)
            };
            format!("{}{head}{body}", doc(&a.doc, 0))
        }
        // A project script is a file of its own, not source.
        Declaration::Script(_) => String::new(),
    }
}

/// A whole file of declarations, a blank line between each, and a
/// component's parts written inside it rather than beside it.
pub fn file_text(program: &Program) -> String {
    let mut out: Vec<String> = Vec::new();
    for d in &program.declarations {
        if matches!(d, Declaration::Component(c) if c.name.contains('.'))
            || matches!(d, Declaration::Script(_))
        {
            continue;
        }
        out.push(declaration(d, program));
    }
    let mut text = out.join("\n\n");
    text.push('\n');
    text
}

// ─── The proof ───────────────────────────────────────────

/// Every statement of `program`, nested ones too, with the kind of block
/// it is in, in source order.
pub fn each_statement<'a>(program: &'a Program, f: &mut dyn FnMut(&'a [Statement], usize, Block)) {
    for decl in &program.declarations {
        if let Some(body) = body_of(decl) {
            walk_body(body, f);
        }
    }
}

/// [`each_statement`], over one declaration's body.
pub fn walk_body<'a>(body: &'a [Statement], f: &mut dyn FnMut(&'a [Statement], usize, Block)) {
    fn walk<'a>(
        stmts: &'a [Statement],
        block: Block,
        f: &mut dyn FnMut(&'a [Statement], usize, Block),
    ) {
        for (i, s) in stmts.iter().enumerate() {
            f(stmts, i, block);
            match &s.kind {
                StatementKind::UIElement(ui) => {
                    walk(&ui.children, Block::Render, f);
                    for fill in &ui.slot_fills {
                        walk(&fill.body, Block::Render, f);
                    }
                    for h in &ui.events {
                        walk(&h.body, Block::Imperative, f);
                    }
                }
                StatementKind::If(i) => {
                    walk(&i.then_body, block, f);
                    for (_, b) in &i.else_if_branches {
                        walk(b, block, f);
                    }
                    if let Some(b) = &i.else_body {
                        walk(b, block, f);
                    }
                }
                StatementKind::For(x) => walk(&x.body, block, f),
                StatementKind::Show(x) => walk(&x.body, Block::Render, f),
                StatementKind::Match(m) => {
                    for arm in &m.arms {
                        walk(&arm.body, Block::Render, f);
                    }
                }
                StatementKind::Effect(e) => {
                    walk(&e.body, Block::Imperative, f);
                    walk(&e.cleanup, Block::Imperative, f);
                }
                StatementKind::Timer(t) => walk(&t.body, Block::Imperative, f),
                StatementKind::Action(a) => walk(&a.body, Block::Imperative, f),
                StatementKind::EventHandler(h) => walk(&h.body, Block::Imperative, f),
                StatementKind::Try(t) => {
                    walk(&t.body, Block::Imperative, f);
                    walk(&t.catch_body, Block::Imperative, f);
                }
                StatementKind::Connection(c) => {
                    for h in &c.handlers {
                        walk(&h.body, Block::Imperative, f);
                    }
                }
                StatementKind::Fetch(x) => {
                    for b in [&x.loading_block, &x.success_block].into_iter().flatten() {
                        walk(b, Block::Render, f);
                    }
                    if let Some((_, b)) = &x.error_block {
                        walk(b, Block::Render, f);
                    }
                }
                _ => {}
            }
        }
    }
    walk(body, Block::Render, f);
}

/// The statements a declaration holds: a page's, a component's, a
/// store's, the app's and a test's.
pub fn body_of(decl: &Declaration) -> Option<&[Statement]> {
    Some(match decl {
        Declaration::Page(p) => &p.body,
        Declaration::Component(c) => &c.body,
        Declaration::Store(s) => &s.body,
        Declaration::App(a) => &a.body,
        Declaration::Test(t) => &t.body,
        _ => return None,
    })
}

/// Where a declaration is written, when it keeps that.
fn span_of(decl: &Declaration) -> Option<Span> {
    match decl {
        Declaration::Page(p) => Some(p.span),
        Declaration::Component(c) => Some(c.span),
        Declaration::Store(s) => Some(s.span),
        Declaration::Test(t) => Some(t.span),
        _ => None,
    }
}

/// The tree as text with every place in the source taken out, so two
/// trees of the same program written differently compare equal.
pub fn shape(program: &Program) -> String {
    without_places(&format!("{:?}", program.declarations))
}

/// A node's `Debug` text with every place in the source taken out: each
/// span, whether one is there, and the lists of them an element keeps.
pub fn without_places(text: &str) -> String {
    use std::sync::LazyLock;
    static SPAN: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"Span \{ start: \d+, end: \d+, line: \d+, col: \d+ \}").expect("a regex")
    });
    static SOME: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"Some\(S\)").expect("a regex"));
    static NONE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(\w*span\w*): None").expect("a regex"));
    static LISTS: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(\w*_spans): \[(S(, S)*)?\]").expect("a regex"));
    let text = SPAN.replace_all(text, "S");
    let text = SOME.replace_all(&text, "S");
    let text = NONE.replace_all(&text, "$1: S");
    LISTS.replace_all(&text, "$1: _").into_owned()
}

/// A statement the printer did not write back as itself: where it is,
/// what it was, and what was printed.
#[derive(Debug, Clone)]
pub struct Mismatch {
    pub line: usize,
    pub written: String,
    pub printed: String,
    /// Why: the parse error, or `a different tree`.
    pub why: String,
}

/// Every statement of `source` printed and put back in its own place, one
/// at a time: each time the file must parse to the same tree. Returns how
/// many statements were proven, or what was not — a file that does not
/// parse at all is one `Mismatch` at line 0.
pub fn prove(source: &str, file: &str) -> Result<usize, Vec<Mismatch>> {
    let braced = if file.ends_with(".wfx") {
        match crate::layout::to_braces(source, file) {
            Ok(b) => b,
            Err(e) => {
                return Err(vec![Mismatch {
                    line: 1,
                    written: String::new(),
                    printed: String::new(),
                    why: format!("not converted to braces: {e}"),
                }]);
            }
        }
    } else {
        source.to_string()
    };
    let parse = |text: &str| crate::syntax::parse_source(text, file.trim_end_matches('x'));
    let program = match parse(&braced) {
        Ok(p) => p,
        Err(e) => {
            return Err(vec![Mismatch {
                line: 0,
                written: String::new(),
                printed: String::new(),
                why: format!("the file does not parse: {e}"),
            }]);
        }
    };
    let mut proven = 0;
    let mut wrong = Vec::new();
    let names = regex::Regex::new(r"d__\d+").expect("a regex");
    let alone = |text: &str| -> Option<String> {
        let p = parse(text).ok()?;
        // A destructuring `let` is numbered in the order the file is read,
        // which a declaration read alone does not share.
        Some(names.replace_all(&shape(&p), "d__N").into_owned())
    };
    for decl in &program.declarations {
        let Some(body) = body_of(decl) else {
            continue;
        };
        // A statement changes only the declaration it is in, so that is
        // what is parsed again: the declaration's own text, alone, before
        // and after. Where the declaration does not keep its place, or does
        // not parse alone, the whole file is.
        let (from, to) = match span_of(decl) {
            Some(sp) if (sp.end as usize) <= braced.len() && sp.start < sp.end => {
                (sp.start as usize, sp.end as usize)
            }
            _ => (0, braced.len()),
        };
        let (from, to, want) = match alone(&braced[from..to]) {
            Some(want) => (from, to, want),
            None => (0, braced.len(), alone(&braced).unwrap_or_default()),
        };
        let mut seen = std::collections::HashSet::new();
        let mut each = |stmts: &[Statement], i: usize, block: Block| {
            let s = &stmts[i];
            let (start, end) = (s.span.start as usize, s.span.end as usize);
            // A destructuring `let` is several statements over one span:
            // written once, from its first.
            if !seen.insert((start, end)) || start >= end || start < from || end > to {
                return;
            }
            let line_start = braced[..start].rfind('\n').map_or(0, |n| n + 1);
            let leading = braced[line_start..start]
                .chars()
                .take_while(|c| *c == ' ')
                .count();
            let indent = leading / INDENT.len();
            let printed = match destructure(&stmts[i..]) {
                Some((text, _)) => text,
                None => statement(s, block, indent),
            };
            let changed = format!("{}{printed}{}", &braced[from..start], &braced[end..to]);
            let why = match alone(&changed) {
                Some(got) if got == want => {
                    proven += 1;
                    return;
                }
                Some(_) => "a different tree".to_string(),
                None => match parse(&changed) {
                    Err(e) => e.to_string(),
                    Ok(_) => "a different tree".to_string(),
                },
            };
            wrong.push(Mismatch {
                line: s.span.line as usize,
                written: braced[start..end].to_string(),
                printed,
                why,
            });
        };
        walk_body(body, &mut each);
    }
    if wrong.is_empty() {
        Ok(proven)
    } else {
        Err(wrong)
    }
}

/// Where a declaration's `///` comment begins: the first of the lines
/// right above `start` that are one, or `start`'s own line.
pub fn doc_start(source: &str, start: usize) -> usize {
    let mut at = source[..start].rfind('\n').map_or(0, |n| n + 1);
    while at > 0 {
        let prev = source[..at - 1].rfind('\n').map_or(0, |n| n + 1);
        if source[prev..at - 1].trim_start().starts_with("///") {
            at = prev;
        } else {
            break;
        }
    }
    at
}

/// Where a declaration is written, from its `///` to its last byte.
pub fn declaration_span(source: &str, d: &Declaration) -> Option<(usize, usize)> {
    let span = match d {
        Declaration::Page(p) => p.span,
        Declaration::Component(c) => c.span,
        Declaration::Store(s) => s.span,
        Declaration::Theme(t) => t.span,
        Declaration::Type(t) => t.span,
        Declaration::Enum(e) => e.span,
        Declaration::Const(c) => c.span,
        Declaration::Data(x) => x.span,
        Declaration::Animation(a) => a.span,
        Declaration::Test(t) => t.span,
        Declaration::Api(a) => a.span,
        // The app keeps no place of its own: it is the `app { … }` that
        // starts a line, of which a file has one.
        Declaration::App(_) => {
            let apps: Vec<usize> = regex::Regex::new(r"(?m)^app\s*\{")
                .expect("a regex")
                .find_iter(source)
                .map(|m| m.start())
                .collect();
            // Two would leave which one this is a guess.
            let [at] = apps[..] else {
                return None;
            };
            let &(_, close) = crate::reprint::blocks_in(source, at, source.len()).first()?;
            return Some((doc_start(source, at), close + 1));
        }
        Declaration::Script(_) => return None,
    };
    let (start, end) = (span.start as usize, span.end as usize);
    (start < end && end <= source.len()).then(|| (doc_start(source, start), end))
}

/// Every declaration of `source` printed whole and put back in its own
/// place, one at a time, and then the whole file printed: each time the
/// file must parse to the same tree. Returns how many were proven, or what
/// was not — a file that does not parse is one `Mismatch` at line 0.
pub fn prove_declarations(source: &str, file: &str) -> Result<usize, Vec<Mismatch>> {
    let braced = if file.ends_with(".wfx") {
        crate::layout::to_braces(source, file).unwrap_or_default()
    } else {
        source.to_string()
    };
    let name = file.trim_end_matches('x');
    let names = regex::Regex::new(r"d__\d+").expect("a regex");
    let shape_of = |text: &str| -> Result<String, String> {
        let p = crate::syntax::parse_source(text, name).map_err(|e| e.to_string())?;
        Ok(names.replace_all(&shape(&p), "d__N").into_owned())
    };
    let program = match crate::syntax::parse_source(&braced, name) {
        Ok(p) => p,
        Err(e) => {
            return Err(vec![Mismatch {
                line: 0,
                written: String::new(),
                printed: String::new(),
                why: format!("the file does not parse: {e}"),
            }]);
        }
    };
    let want = names.replace_all(&shape(&program), "d__N").into_owned();
    let mut proven = 0;
    let mut wrong = Vec::new();
    for d in &program.declarations {
        // A part is printed inside its component.
        if matches!(d, Declaration::Component(c) if c.name.contains('.')) {
            continue;
        }
        let Some((start, end)) = declaration_span(&braced, d) else {
            continue;
        };
        let printed = declaration(d, &program);
        let changed = format!("{}{printed}{}", &braced[..start], &braced[end..]);
        match shape_of(&changed) {
            Ok(got) if got == want => proven += 1,
            other => wrong.push(Mismatch {
                line: braced[..start].matches('\n').count() + 1,
                written: braced[start..end].to_string(),
                printed,
                why: other.err().unwrap_or_else(|| "a different tree".into()),
            }),
        }
    }
    // The file printed from its tree alone. A project script is a file of
    // its own, and never in a source's tree.
    let printed = file_text(&program);
    match shape_of(&printed) {
        Ok(got) if got == want => proven += 1,
        other => wrong.push(Mismatch {
            line: 0,
            written: String::new(),
            printed: printed.lines().take(3).collect::<Vec<_>>().join("\n"),
            why: format!(
                "the whole file: {}",
                other.err().unwrap_or_else(|| "a different tree".into())
            ),
        }),
    }
    if wrong.is_empty() {
        Ok(proven)
    } else {
        Err(wrong)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_expr(source: &str) -> Expr {
        let page = format!(
            "page P(path: \"/\", title: \"P\", description: \"P.\") {{\n    derived x = {source}\n}}\n"
        );
        let program =
            crate::syntax::parse_source(&page, "t.wf").unwrap_or_else(|e| panic!("{e}\n{page}"));
        let Declaration::Page(p) = &program.declarations[0] else {
            panic!("a page")
        };
        let StatementKind::Derived(d) = &p.body[0].kind else {
            panic!("a derived")
        };
        d.value.clone()
    }

    /// What is written is printed back as a person writes it, and that
    /// reads back as the same tree.
    #[test]
    fn an_expression_prints_as_it_is_written() {
        for (written, printed) in [
            ("a + b * c", "a + b * c"),
            ("(a + b) * c", "(a + b) * c"),
            ("a - (b - c)", "a - (b - c)"),
            ("(a - b) - c", "a - b - c"),
            ("!(a && b) || c", "!(a && b) || c"),
            ("-(-a)", "-(-a)"),
            ("a ?? b ?? c", "a ?? b ?? c"),
            ("(a ?? b) || c", "(a ?? b) || c"),
            ("1..n + 1", "1..n + 1"),
            ("xs.map(x => x * 2)", "xs.map(x => x * 2)"),
            (
                "xs.reduce((a, b) => a + b, 0)",
                "xs.reduce((a, b) => a + b, 0)",
            ),
            (
                "due.plus(days: 3, months: 1)",
                "due.plus(days: 3, months: 1)",
            ),
            ("Todo(id: 1, title: \"x\")", "Todo(id: 1, title: \"x\")"),
            (
                "{ a: 1, \"b c\": 2, ...rest }",
                "{ a: 1, \"b c\": 2, ...rest }",
            ),
            ("[1, ...more]", "[1, ...more]"),
            ("a?.b?.[0]?.c(1)", "a?.b?.[0]?.c(1)"),
            (
                "if open { \"x\" } else { \"y\" }",
                "if open { \"x\" } else { \"y\" }",
            ),
            ("if open { \"x\" }", "if open { \"x\" }"),
            (
                "if a { 1 } else if b { 2 } else { 3 }",
                "if a { 1 } else if b { 2 } else { 3 }",
            ),
            (
                "if let u = user { u.name } else { \"\" }",
                "if let u = user { u.name } else { \"\" }",
            ),
            (
                "match tone { .info { 1 } .danger { 2 } else { 3 } }",
                "match tone { .info { 1 } .danger { 2 } else { 3 } }",
            ),
            (
                "match r { .failed(e) { e } else { \"ok\" } }",
                "match r { .failed(e) { e } else { \"ok\" } }",
            ),
            ("@2026-03-14", "@2026-03-14"),
            ("@09:30", "@09:30"),
            ("#0F766E", "#0F766E"),
            ("€12.99", "€12.99"),
            ("¥500", "¥500"),
            ("$3", "$3"),
            ("3.days", "3.days"),
            ("90.minutes", "90.minutes"),
            ("1.5.hours", "90.minutes"),
            ("250.ms", "250.ms"),
            ("(3).toFixed(2)", "(3).toFixed(2)"),
            ("(€1.50).times(2)", "(€1.50).times(2)"),
            (
                "\"say \\\"hi\\\" \\{not\\} a splice\"",
                "\"say \\\"hi\\\" \\{not\\} a splice\"",
            ),
            ("\"line\\nnext\\ttab\"", "\"line\\nnext\\ttab\""),
            (
                "\"Hi {name}, {count + 1} new\"",
                "\"Hi {name}, {count + 1} new\"",
            ),
            ("\"{total:.currency}\"", "\"{format(total, .currency)}\""),
            ("/^[a-z]+$/i", "/^[a-z]+$/i"),
            (".failed(\"x\")", ".failed(\"x\")"),
            ("$space-md", "$space-md"),
            ("await load()", "await load()"),
        ] {
            let e = first_expr(written);
            assert_eq!(expr(&e), printed, "{written}");
            let back = first_expr(&expr(&e));
            assert_eq!(
                format!("{back:?}"),
                format!("{e:?}"),
                "{written} → {printed}"
            );
        }
    }

    /// Every kind of statement, printed and put back, reads as itself; and
    /// the file as a whole is the formatter's layout.
    #[test]
    fn every_kind_of_statement_prints_back_as_itself() {
        let source = r##"store Cart {
    persist items: [String] = [] {
        in: .session
        version: 2
        sync: false
        migrate 1 -> 2 { old.map(i => i.id) }
    }
    derived count: Number = items.length
    action add(item: String, note) {
        let { a, b } = item.parts()
        let [x, y] = pair
        items = [...items, item]
        if a == b {
            log(a)
        } else if x {
            return
        } else {
            return y
        }
        for i, n in items {
            log(i)
        }
        try {
            await save()
        } catch e {
            navigate("/error")
        }
        emit added(item)
        emit cleared
    }
}

component Row(label: String) {
    event toggle(id: String)
    slot trailing
    Text(label)
    trailing
    children
    row(item: label)
}

page Home(path: "/", title: "Home", description: "Home.") {
    head {
        meta(name: "theme-color", content: "#fff")
    }
    use Cart
    state stage: String = "prod"
    state open = false
    validate stage {
        required "Say which"
        minLength(3) t("form.min")
        custom "Not that" { stage != "x" }
    }
    resource users: [Any] = fetch("/api/users", method: "GET")
    resource rows = Backend.rows(page: 1)
    socket chat = ws("wss://x", reconnect: true) {
        receive Any
        on message(m) {
            log(m)
        }
    }
    worker css = spawn("/w.js")
    effect {
        log(stage)
        cleanup {
            log("bye")
        }
    }
    every(1000) {
        tick()
    }
    after(2.seconds) {
        open = false
    }
    on key("mod+k", e) {
        open = !open
    }
    Container(gap: .md).fluid {
        style {
            padding: 12px $space-md
            background: {if open { "a" } else { "b" }}
            &:hover {
                opacity: 0.8
            }
            @media (max-width: 600px) {
                padding: 4px
            }
        }
        transition {
            opacity: fast $ease-out
        }
        on click(e) {
            stage = e.target.value
        }
        Heading("Home").h1
        if let u = Cart.user {
            Text(u.name)
        }
        for d in Cart.items by d {
            Row(label: d) {
                trailing {
                    Badge("new")
                }
            }
        }
        show open {
            Text("Open")
        }
        match users {
            loading {
                Spinner
            }
            error(err) {
                Alert("Failed: {err.message}").danger
            }
            ready(list) {
                Text("{list.length}")
            }
        }
        match stage {
            .prod {
                Text("p")
            }
            else {
                Text("other")
            }
        }
    }
}

test "home" {
    expect "Home"
    expect not "Gone"
    click "Save"
    type "Ada" into "Name"
    press "Escape" in "Search"
}
"##;
        let proven = prove(source, "t.wf").unwrap_or_else(|w| {
            panic!(
                "{}",
                w.iter()
                    .map(|m| format!("{}: {}\n{}\n→\n{}", m.line, m.why, m.written, m.printed))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            )
        });
        assert_eq!(proven, 56, "every statement of the sample");
    }
    /// Every kind of declaration, printed whole and put back, reads as
    /// itself; and so does the file printed from its tree alone.
    #[test]
    fn every_kind_of_declaration_prints_back_as_itself() {
        let source = r##"/// A record.
type Note {
    /// Its id.
    id: String
    title: String = ""
    score: Number(min: 0, max: 10) = 0
    tags: [String]?
}

type Pinned = Note {
    at: Date
}

/// What a request comes to.
enum Outcome {
    ok
    failed(reason: String, code: Number = 0)
}

const LIMIT: Number = 10

data posts: [Note] = "posts.json"

image hero = "hero.jpg"

animation Pulse {
    from {
        opacity: 1
    }
    50% {
        opacity: 0.4
    }
    to {
        opacity: 1
    }
}

theme Brand {
    color-primary: #0F766E
    radius-md: 14px
}

/// The service.
api Backend(base: "/api", timeout: 10.seconds) {
    headers {
        Authorization: "Bearer {token}"
        X-Request-Id: "1"
    }
    on request(r) {
        log(r)
    }
    /// One note.
    get note(id: String) at "notes/:id" -> Note errors { 404 -> Any } cache: .swr(60.seconds)
    post save(note: Note) -> Note
}

store Cart(scope: .route, eager: true) {
    state items: [String] = []
}

/// A card.
component Card(
    /// What it says.
    _ title: String,
    tone: Outcome = .ok
) {
    /// When it is opened.
    event opened(id: String)
    event closed
    slot
    slot row(item: Note)
    /// The top of it.
    part Header(_ text: String) {
        Heading(text).h3
    }
    Stack {
        Card.Header(title)
        children
        row(item: Note(id: "1", title: title))
    }
}

page Note(path: "/notes/:id", title: "{id} — Notes", description: "A note, {id}.", image: "/og.png", image_alt: "The note", type: "article", layout: Shell(crumb: "Notes"), guard: true, noindex: true, id: String) {
    head {
        meta(name: "robots", content: "x")
    }
    Text(id)
}

page Plain(path: "/", title: "Report: {Cart.items.length} items", description: "Home.") {
    Text("x")
}

app {
    Router
}

test "a note"(data: { id: "1" }) {
    Text("{id}")
    expect "1"
    click "Save"
}
"##;
        let proven = prove_declarations(source, "t.wf").unwrap_or_else(|w| {
            panic!(
                "{}",
                w.iter()
                    .map(|m| format!("{}: {}\n{}\n→\n{}", m.line, m.why, m.written, m.printed))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            )
        });
        assert_eq!(proven, 16, "fifteen declarations and the file");
        // And a component prints as it is written.
        let program = crate::syntax::parse_source(source, "t.wf").unwrap();
        let card = program
            .declarations
            .iter()
            .find(|d| matches!(d, Declaration::Component(c) if c.name == "Card"))
            .unwrap();
        let printed = declaration(card, &program);
        assert!(printed.starts_with("/// A card.\ncomponent Card(\n    /// What it says.\n    _ title: String,\n    tone: Outcome = .ok,\n) {\n    /// When it is opened.\n    event opened(id: String)\n    event closed\n    slot\n    slot row(item: Note)\n    /// The top of it.\n    part Header(_ text: String) {\n        Heading(text).h3\n    }\n"), "{printed}");
    }
}
