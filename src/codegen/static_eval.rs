//! Resolving what a page's data *is* at build time, so the static paint can
//! include it.
//!
//! The static renderer used to emit `<!--wf-for-->` wherever a list appeared and
//! leave the rows to JavaScript. For seeded data — a store's initial state, a
//! literal array in the page — the values are sitting in the AST, so the rows
//! could have been painted all along. Leaving them out meant a static site's most
//! substantial content was absent from the HTML: nothing for a crawler to index,
//! and nothing for the browser to paint until the bundle had downloaded, parsed
//! and run, which is exactly the work Largest Contentful Paint measures.
//!
//! What cannot be known at build time — a fetched collection, anything derived
//! from user input — stays a placeholder. The rule is that this module never
//! guesses: it returns `None` and the renderer falls back to the client.
//!
//! A store's `derived` values are evaluated too, in order, over its seeded
//! state; and an action a derived value calls — `rows = sorted.map(r =>
//! shape(r))` — is run at build time when it is made of what the runtime
//! would run the same way: assignments to locals, `if`/`else` with `return`,
//! `for` over a list, and the array and string methods below. Anything else
//! — a store mutation, a fetch, a browser API — stops the evaluation, and
//! the client paints that part. A fuel counter bounds the work, so a loop
//! that never ends at build time simply yields nothing.

use std::cell::Cell;
use std::collections::HashMap;

use crate::parser::ast::{
    ActionDecl, BinOp, Declaration, Expr, Program, Statement, StatementKind, UnaryOp,
};

/// How much evaluation one expression may do: enough for a few thousand rows
/// through a few methods, not enough to hang a build.
const FUEL: u32 = 500_000;
/// Nested action calls a value may go through.
const MAX_DEPTH: u32 = 32;

/// A value the compiler could work out for itself.
#[derive(Debug, Clone, PartialEq)]
pub enum Static {
    Str(String),
    Num(f64),
    Bool(bool),
    List(Vec<Static>),
    Map(Vec<(String, Static)>),
    Null,
}

impl Static {
    /// The text this value renders as.
    pub fn to_text(&self) -> String {
        match self {
            Static::Str(s) => s.clone(),
            Static::Num(n) => {
                if *n == (*n as i64) as f64 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Static::Bool(b) => format!("{}", b),
            Static::Null => String::new(),
            // A list or map has no sensible text form; the renderer should be
            // iterating it, not printing it.
            Static::List(_) | Static::Map(_) => String::new(),
        }
    }

    /// JavaScript truthiness, so a resolved `if` takes the same branch the
    /// runtime would.
    pub fn truthy(&self) -> bool {
        match self {
            Static::Bool(b) => *b,
            Static::Num(n) => *n != 0.0 && !n.is_nan(),
            Static::Str(s) => !s.is_empty(),
            Static::List(items) => !items.is_empty(),
            Static::Map(_) => true,
            Static::Null => false,
        }
    }
}

/// Names in scope at build time, innermost last, and the actions a call can
/// reach.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    frames: Vec<HashMap<String, Static>>,
    functions: HashMap<String, ActionDecl>,
    depth: u32,
}

impl Scope {
    /// The scope a page starts with: every store's seeded state, keyed
    /// `Store.field`, plus the page's own literal `state` declarations.
    ///
    /// Only initial values. A store field a user action has since changed is not
    /// knowable here, and the hydrating client will correct it.
    pub fn from_program(program: &Program, page_body: &[Statement]) -> Self {
        let mut frame = HashMap::new();

        for decl in &program.declarations {
            if let Declaration::Store(store) = decl {
                frame.insert(store.name.clone(), store_value(store));
            }
        }

        let mut scope = Scope {
            frames: vec![frame],
            functions: HashMap::new(),
            depth: 0,
        };
        scope.push_state(page_body);
        scope
    }

    /// A scope whose calls reach `actions`.
    fn with_functions(mut self, actions: &[&ActionDecl]) -> Self {
        for action in actions {
            self.functions
                .insert(action.name.clone(), (*action).clone());
        }
        self
    }

    /// Add the literal `state` declarations of a statement list.
    pub fn push_state(&mut self, stmts: &[Statement]) {
        let mut frame = HashMap::new();
        for stmt in stmts {
            if let StatementKind::State(s) = &stmt.kind {
                if let Some(v) = eval(&s.value, self) {
                    frame.insert(s.name.clone(), v);
                }
            }
        }
        self.frames.push(frame);
    }

    /// Bind one name, for a loop body.
    pub fn with(&self, name: &str, value: Static) -> Self {
        let mut next = self.clone();
        let mut frame = HashMap::new();
        frame.insert(name.to_string(), value);
        next.frames.push(frame);
        next
    }

    fn get(&self, name: &str) -> Option<&Static> {
        self.frames.iter().rev().find_map(|f| f.get(name))
    }

    fn set(&mut self, name: &str, value: Static) {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) {
                frame.insert(name.to_string(), value);
                return;
            }
        }
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), value);
        }
    }
}

/// A store as a value: its seeded state, then its derived values in order,
/// each evaluated over the fields before it and the store's own actions. A
/// derived value that cannot be worked out is simply absent.
fn store_value(store: &crate::parser::ast::StoreDecl) -> Static {
    let actions: Vec<&ActionDecl> = store
        .body
        .iter()
        .filter_map(|s| match &s.kind {
            StatementKind::Action(a) => Some(a),
            _ => None,
        })
        .collect();
    let mut scope = Scope {
        frames: vec![HashMap::new()],
        functions: HashMap::new(),
        depth: 0,
    }
    .with_functions(&actions);
    let mut fields = Vec::new();
    for stmt in &store.body {
        let (name, value) = match &stmt.kind {
            StatementKind::State(s) => (&s.name, &s.value),
            StatementKind::Derived(d) => (&d.name, &d.value),
            _ => continue,
        };
        if let Some(v) = eval(value, &scope) {
            scope.set(name, v.clone());
            fields.push((name.clone(), v));
        }
    }
    Static::Map(fields)
}

/// Bookkeeping for one evaluation: the work it has left.
struct Fuel(Cell<u32>);

impl Fuel {
    fn spend(&self) -> Option<()> {
        let left = self.0.get();
        if left == 0 {
            return None;
        }
        self.0.set(left - 1);
        Some(())
    }
}

/// Work out an expression's value, or `None` if it depends on something only the
/// running page knows.
pub fn eval(expr: &Expr, scope: &Scope) -> Option<Static> {
    eval_in(expr, scope, &Fuel(Cell::new(FUEL)))
}

fn eval_in(expr: &Expr, scope: &Scope, fuel: &Fuel) -> Option<Static> {
    fuel.spend()?;
    match expr {
        Expr::StringLiteral(s) => Some(Static::Str(s.clone())),
        Expr::NumberLiteral(n) => Some(Static::Num(*n)),
        Expr::BoolLiteral(b) => Some(Static::Bool(*b)),
        Expr::Null => Some(Static::Null),

        Expr::Identifier(name) => scope.get(name).cloned(),

        Expr::ListLiteral(items) => items
            .iter()
            .map(|e| eval_in(e, scope, fuel))
            .collect::<Option<Vec<_>>>()
            .map(Static::List),

        Expr::MapLiteral(entries) | Expr::Record(_, entries) => entries
            .iter()
            .map(|(k, v)| eval_in(v, scope, fuel).map(|v| (k.trim_matches('"').to_string(), v)))
            .collect::<Option<Vec<_>>>()
            .map(Static::Map),

        Expr::InterpolatedString(parts) => {
            use crate::parser::ast::StringPart;
            let mut out = String::new();
            for part in parts {
                match part {
                    StringPart::Literal(s) => out.push_str(s),
                    StringPart::Expression(e) => out.push_str(&eval_in(e, scope, fuel)?.to_text()),
                }
            }
            Some(Static::Str(out))
        }

        Expr::PropertyAccess(base, prop) => {
            let base = eval_in(base, scope, fuel)?;
            match (&base, prop.as_str()) {
                (Static::List(items), "length") => Some(Static::Num(items.len() as f64)),
                (Static::Str(s), "length") => Some(Static::Num(s.chars().count() as f64)),
                (Static::Map(fields), _) => fields
                    .iter()
                    .find(|(k, _)| k == prop)
                    .map(|(_, v)| v.clone()),
                _ => None,
            }
        }

        Expr::IndexAccess(base, index) => {
            let base = eval_in(base, scope, fuel)?;
            let index = eval_in(index, scope, fuel)?;
            match (base, index) {
                (Static::List(items), Static::Num(n)) => items.get(n as usize).cloned(),
                (Static::Map(fields), Static::Str(k)) => fields
                    .iter()
                    .find(|(key, _)| *key == k)
                    .map(|(_, v)| v.clone()),
                _ => None,
            }
        }

        Expr::UnaryOp(op, inner) => {
            let v = eval_in(inner, scope, fuel)?;
            match op {
                UnaryOp::Not => Some(Static::Bool(!v.truthy())),
                UnaryOp::Neg => match v {
                    Static::Num(n) => Some(Static::Num(-n)),
                    _ => None,
                },
            }
        }

        Expr::BinaryOp(left, op, right) => {
            let l = eval_in(left, scope, fuel)?;
            let r = eval_in(right, scope, fuel)?;
            binary(&l, op, &r)
        }

        // `if c { a } else { b }` as a value.
        Expr::MethodCall(cond, name, branches) if name == "__if" && branches.len() == 2 => {
            let chosen = if eval_in(cond, scope, fuel)?.truthy() {
                0
            } else {
                1
            };
            eval_in(&branches[chosen], scope, fuel)
        }

        Expr::MethodCall(base, method, args) => {
            // `Math.max(a, b)` and the like: a global, not a value.
            if let Expr::Identifier(global) = base.as_ref() {
                if scope.get(global).is_none() {
                    let args = args
                        .iter()
                        .map(|a| eval_in(a, scope, fuel))
                        .collect::<Option<Vec<_>>>()?;
                    return global_method(global, method, &args);
                }
            }
            let receiver = eval_in(base, scope, fuel)?;
            method_call(&receiver, method, args, scope, fuel)
        }

        Expr::FunctionCall(name, args) => {
            let args = args
                .iter()
                .map(|a| eval_in(a, scope, fuel))
                .collect::<Option<Vec<_>>>()?;
            match name.as_str() {
                "String" => Some(Static::Str(args.first()?.to_text())),
                "Number" => match args.first()? {
                    Static::Num(n) => Some(Static::Num(*n)),
                    Static::Str(s) => s.trim().parse::<f64>().ok().map(Static::Num),
                    Static::Bool(b) => Some(Static::Num(if *b { 1.0 } else { 0.0 })),
                    _ => None,
                },
                _ => {
                    let action = scope.functions.get(name)?.clone();
                    call_action(&action, args, scope, fuel)
                }
            }
        }

        // A bare lambda is not a value the page can show.
        Expr::Lambda(..) => None,

        // An enum case is its name at run time; a token is its custom
        // property. A request cannot be awaited at build time.
        Expr::EnumCase(case) => Some(Static::Str(case.clone())),
        Expr::Token(name) => Some(Static::Str(format!("var(--{name})"))),
        Expr::Await(_) => None,
    }
}

/// Apply a lambda to `args`, with its parameters bound over `scope`.
fn apply(lambda: &Expr, args: &[Static], scope: &Scope, fuel: &Fuel) -> Option<Static> {
    let Expr::Lambda(params, body) = lambda else {
        return None;
    };
    let mut inner = scope.clone();
    let mut frame = HashMap::new();
    for (param, arg) in params.split(',').map(str::trim).zip(args) {
        frame.insert(param.to_string(), arg.clone());
    }
    inner.frames.push(frame);
    eval_in(body, &inner, fuel)
}

/// The array and string methods the runtime would run the same way.
fn method_call(
    receiver: &Static,
    method: &str,
    args: &[Expr],
    scope: &Scope,
    fuel: &Fuel,
) -> Option<Static> {
    let value = |i: usize| args.get(i).and_then(|a| eval_in(a, scope, fuel));
    let number = |i: usize| match value(i)? {
        Static::Num(n) => Some(n),
        _ => None,
    };
    match receiver {
        Static::List(items) => match method {
            "filter" | "map" | "some" | "every" | "find" | "findIndex" => {
                let f = args.first()?;
                let mut out = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    let r = apply(f, &[item.clone(), Static::Num(i as f64)], scope, fuel)?;
                    match method {
                        "filter" => {
                            if r.truthy() {
                                out.push(item.clone());
                            }
                        }
                        "map" => out.push(r),
                        "some" => {
                            if r.truthy() {
                                return Some(Static::Bool(true));
                            }
                        }
                        "every" => {
                            if !r.truthy() {
                                return Some(Static::Bool(false));
                            }
                        }
                        "find" => {
                            if r.truthy() {
                                return Some(item.clone());
                            }
                        }
                        _ => {
                            if r.truthy() {
                                return Some(Static::Num(i as f64));
                            }
                        }
                    }
                }
                Some(match method {
                    "filter" | "map" => Static::List(out),
                    "some" => Static::Bool(false),
                    "every" => Static::Bool(true),
                    "find" => Static::Null,
                    _ => Static::Num(-1.0),
                })
            }
            "slice" => {
                let start = clamp_index(number(0).unwrap_or(0.0), items.len())?;
                let end = match args.get(1) {
                    Some(_) => clamp_index(number(1)?, items.len())?,
                    None => items.len(),
                };
                Some(Static::List(items[start..end.max(start)].to_vec()))
            }
            "concat" => {
                let mut out = items.clone();
                for i in 0..args.len() {
                    match value(i)? {
                        Static::List(more) => out.extend(more),
                        other => out.push(other),
                    }
                }
                Some(Static::List(out))
            }
            "indexOf" => {
                let needle = value(0)?;
                Some(Static::Num(
                    items
                        .iter()
                        .position(|v| *v == needle)
                        .map(|i| i as f64)
                        .unwrap_or(-1.0),
                ))
            }
            "includes" => {
                let needle = value(0)?;
                Some(Static::Bool(items.contains(&needle)))
            }
            "join" => {
                let sep = args
                    .first()
                    .map(|_| value(0))
                    .unwrap_or(Some(Static::Str(",".into())))?;
                Some(Static::Str(
                    items
                        .iter()
                        .map(Static::to_text)
                        .collect::<Vec<_>>()
                        .join(&sep.to_text()),
                ))
            }
            "reverse" => {
                let mut out = items.clone();
                out.reverse();
                Some(Static::List(out))
            }
            "sort" => Some(Static::List(sorted(items, args.first(), scope, fuel)?)),
            _ => None,
        },
        Static::Str(s) => match method {
            "toLowerCase" => Some(Static::Str(s.to_lowercase())),
            "toUpperCase" => Some(Static::Str(s.to_uppercase())),
            "trim" => Some(Static::Str(s.trim().to_string())),
            "indexOf" => {
                let needle = value(0)?.to_text();
                Some(Static::Num(
                    s.find(&needle)
                        .map(|b| s[..b].chars().count() as f64)
                        .unwrap_or(-1.0),
                ))
            }
            "includes" => Some(Static::Bool(s.contains(&value(0)?.to_text()))),
            "startsWith" => Some(Static::Bool(s.starts_with(&value(0)?.to_text()))),
            "endsWith" => Some(Static::Bool(s.ends_with(&value(0)?.to_text()))),
            "split" => {
                let sep = value(0)?.to_text();
                let parts: Vec<Static> = if sep.is_empty() {
                    s.chars().map(|c| Static::Str(c.to_string())).collect()
                } else {
                    s.split(&sep).map(|p| Static::Str(p.to_string())).collect()
                };
                Some(Static::List(parts))
            }
            "replace" => {
                let from = value(0)?.to_text();
                let to = value(1)?.to_text();
                Some(Static::Str(s.replacen(&from, &to, 1)))
            }
            "slice" | "substring" => {
                let chars: Vec<char> = s.chars().collect();
                let start = clamp_index(number(0).unwrap_or(0.0), chars.len())?;
                let end = match args.get(1) {
                    Some(_) => clamp_index(number(1)?, chars.len())?,
                    None => chars.len(),
                };
                Some(Static::Str(chars[start..end.max(start)].iter().collect()))
            }
            "charAt" => {
                let i = number(0).unwrap_or(0.0) as usize;
                Some(Static::Str(
                    s.chars().nth(i).map(|c| c.to_string()).unwrap_or_default(),
                ))
            }
            "toString" => Some(Static::Str(s.clone())),
            _ => None,
        },
        Static::Num(n) => match method {
            "toFixed" => {
                let digits = number(0).unwrap_or(0.0) as usize;
                Some(Static::Str(format!("{:.*}", digits, n)))
            }
            "toString" => Some(Static::Str(Static::Num(*n).to_text())),
            _ => None,
        },
        _ => None,
    }
}

/// A JavaScript index argument: negative counts from the end, and the
/// result is clamped to the length.
fn clamp_index(n: f64, len: usize) -> Option<usize> {
    if n.is_nan() {
        return None;
    }
    let i = if n < 0.0 {
        (len as f64 + n).max(0.0)
    } else {
        n.min(len as f64)
    };
    Some(i as usize)
}

/// `Array.prototype.sort`: stable, by the comparator when there is one,
/// else by string value.
fn sorted(
    items: &[Static],
    comparator: Option<&Expr>,
    scope: &Scope,
    fuel: &Fuel,
) -> Option<Vec<Static>> {
    let mut out = items.to_vec();
    match comparator {
        None => {
            out.sort_by_key(|a| a.to_text());
            Some(out)
        }
        Some(cmp) => {
            let mut failed = false;
            out.sort_by(|a, b| {
                if failed {
                    return std::cmp::Ordering::Equal;
                }
                match apply(cmp, &[a.clone(), b.clone()], scope, fuel) {
                    Some(Static::Num(n)) => {
                        n.partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    _ => {
                        failed = true;
                        std::cmp::Ordering::Equal
                    }
                }
            });
            (!failed).then_some(out)
        }
    }
}

/// `Math.*` and the other globals a value may go through.
fn global_method(global: &str, method: &str, args: &[Static]) -> Option<Static> {
    let n = |i: usize| match args.get(i)? {
        Static::Num(n) => Some(*n),
        _ => None,
    };
    match (global, method) {
        ("Math", "round") => Some(Static::Num(n(0)?.round())),
        ("Math", "floor") => Some(Static::Num(n(0)?.floor())),
        ("Math", "ceil") => Some(Static::Num(n(0)?.ceil())),
        ("Math", "abs") => Some(Static::Num(n(0)?.abs())),
        ("Math", "max") | ("Math", "min") => {
            let mut best: Option<f64> = None;
            for i in 0..args.len() {
                let v = n(i)?;
                best = Some(match best {
                    None => v,
                    Some(b) if method == "max" => b.max(v),
                    Some(b) => b.min(v),
                });
            }
            best.map(Static::Num)
        }
        _ => None,
    }
}

/// What running an action's statements produced.
enum Flow {
    Continue,
    Return(Static),
}

/// Run an action at build time, when every statement in it is one the
/// compiler can run: a local assignment, `if`/`else`, `for` over a list, a
/// `return`, and an in-place `sort`/`push`/`reverse` on a local. A store
/// field assignment, a fetch or anything else stops it.
fn call_action(
    action: &ActionDecl,
    args: Vec<Static>,
    scope: &Scope,
    fuel: &Fuel,
) -> Option<Static> {
    if scope.depth >= MAX_DEPTH {
        return None;
    }
    let mut inner = scope.clone();
    inner.depth += 1;
    let mut frame = HashMap::new();
    for (param, arg) in action.params.iter().zip(args) {
        frame.insert(param.name.clone(), arg);
    }
    inner.frames.push(frame);
    match run(&action.body, &mut inner, fuel)? {
        Flow::Return(v) => Some(v),
        Flow::Continue => Some(Static::Null),
    }
}

fn run(stmts: &[Statement], scope: &mut Scope, fuel: &Fuel) -> Option<Flow> {
    for stmt in stmts {
        fuel.spend()?;
        match &stmt.kind {
            StatementKind::Return(value) => {
                return Some(Flow::Return(match value {
                    Some(e) => eval_in(e, scope, fuel)?,
                    None => Static::Null,
                }));
            }
            StatementKind::Assignment(a) => {
                let Expr::Identifier(name) = &a.target else {
                    return None;
                };
                // A store field is not a local; changing it here would be a
                // guess about the page. A local of the same name shadows it.
                let is_local = scope.frames.len() >= 2
                    && scope.frames[1..].iter().any(|f| f.contains_key(name));
                let is_field = scope.frames.first().is_some_and(|f| f.contains_key(name));
                if is_field && !is_local {
                    return None;
                }
                let value = eval_in(&a.value, scope, fuel)?;
                scope.set(name, value);
            }
            StatementKind::State(s) => {
                let value = eval_in(&s.value, scope, fuel)?;
                scope.set(&s.name, value);
            }
            StatementKind::If(i) => {
                let branch = if eval_in(&i.condition, scope, fuel)?.truthy() {
                    Some(&i.then_body)
                } else {
                    let mut chosen = None;
                    for (cond, body) in &i.else_if_branches {
                        if eval_in(cond, scope, fuel)?.truthy() {
                            chosen = Some(body);
                            break;
                        }
                    }
                    chosen.or(i.else_body.as_ref())
                };
                if let Some(body) = branch {
                    if let Flow::Return(v) = run(body, scope, fuel)? {
                        return Some(Flow::Return(v));
                    }
                }
            }
            StatementKind::For(f) => {
                let Static::List(items) = eval_in(&f.iterable, scope, fuel)? else {
                    return None;
                };
                for (i, item) in items.into_iter().enumerate() {
                    let mut frame = HashMap::new();
                    frame.insert(f.item.clone(), item);
                    if let Some(index) = &f.index {
                        frame.insert(index.clone(), Static::Num(i as f64));
                    }
                    scope.frames.push(frame);
                    let flow = run(&f.body, scope, fuel);
                    scope.frames.pop();
                    if let Flow::Return(v) = flow? {
                        return Some(Flow::Return(v));
                    }
                }
            }
            // `copy.sort(cmp)`, `list.push(x)`: mutation of a local.
            StatementKind::ExprStatement(Expr::MethodCall(..)) | StatementKind::MethodCall(_) => {
                let (base, method, args): (&Expr, &str, &[Expr]) = match &stmt.kind {
                    StatementKind::ExprStatement(Expr::MethodCall(b, m, a)) => (b, m, a),
                    StatementKind::MethodCall(m) => (&m.object, &m.method, &m.args),
                    _ => unreachable!(),
                };
                let Expr::Identifier(name) = base else {
                    return None;
                };
                let Some(Static::List(items)) = scope.get(name).cloned() else {
                    return None;
                };
                let updated = match method {
                    "sort" => sorted(&items, args.first(), scope, fuel)?,
                    "reverse" => {
                        let mut out = items;
                        out.reverse();
                        out
                    }
                    "push" => {
                        let mut out = items;
                        for a in args {
                            out.push(eval_in(a, scope, fuel)?);
                        }
                        out
                    }
                    _ => return None,
                };
                scope.set(name, Static::List(updated));
            }
            StatementKind::ExprStatement(_) | StatementKind::Log(_) => {}
            _ => return None,
        }
    }
    Some(Flow::Continue)
}

fn binary(l: &Static, op: &BinOp, r: &Static) -> Option<Static> {
    use BinOp::*;
    match op {
        Add => match (l, r) {
            (Static::Num(a), Static::Num(b)) => Some(Static::Num(a + b)),
            // `+` on anything with a string is concatenation, as in JavaScript.
            (Static::Str(_), _) | (_, Static::Str(_)) => {
                Some(Static::Str(format!("{}{}", l.to_text(), r.to_text())))
            }
            _ => None,
        },
        Sub | Mul | Div | Mod => {
            let (Static::Num(a), Static::Num(b)) = (l, r) else {
                return None;
            };
            Some(Static::Num(match op {
                Sub => a - b,
                Mul => a * b,
                Div => {
                    if *b == 0.0 {
                        return None;
                    }
                    a / b
                }
                _ => {
                    if *b == 0.0 {
                        return None;
                    }
                    a % b
                }
            }))
        }
        Eq => Some(Static::Bool(l == r)),
        Neq => Some(Static::Bool(l != r)),
        Lt | Gt | Lte | Gte => {
            let (Static::Num(a), Static::Num(b)) = (l, r) else {
                return None;
            };
            Some(Static::Bool(match op {
                Lt => a < b,
                Gt => a > b,
                Lte => a <= b,
                _ => a >= b,
            }))
        }
        And => Some(Static::Bool(l.truthy() && r.truthy())),
        Or => Some(Static::Bool(l.truthy() || r.truthy())),
        NullCoalesce => Some(if matches!(l, Static::Null) {
            r.clone()
        } else {
            l.clone()
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program(src: &str) -> Program {
        crate::syntax::parse_source(src, "<t>").expect("parse")
    }

    fn page_scope(src: &str) -> (Program, Scope) {
        let p = program(src);
        let body = p
            .declarations
            .iter()
            .find_map(|d| {
                if let Declaration::Page(pg) = d {
                    Some(pg.body.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let scope = Scope::from_program(&p, &body);
        (p, scope)
    }

    fn expr_of(src: &str) -> Expr {
        // Borrow a page's first Text argument as a way to parse an expression.
        let p = program(&format!("page P(path: \"/\") {{ Text({src}) }}"));
        let Declaration::Page(page) = &p.declarations[0] else {
            unreachable!()
        };
        let StatementKind::UIElement(ui) = &page.body[0].kind else {
            unreachable!()
        };
        match &ui.args[0] {
            crate::parser::ast::Arg::Positional(e) => e.clone(),
            _ => unreachable!(),
        }
    }

    #[test]
    fn literals_resolve() {
        let s = Scope::default();
        assert_eq!(eval(&expr_of("\"hi\""), &s), Some(Static::Str("hi".into())));
        assert_eq!(eval(&expr_of("42"), &s), Some(Static::Num(42.0)));
        assert_eq!(eval(&expr_of("true"), &s), Some(Static::Bool(true)));
    }

    #[test]
    fn a_store_s_seeded_state_is_in_scope() {
        let (_, scope) =
            page_scope("store S { state rows = [1, 2, 3] }\npage P(path: \"/\") { Text(\"x\") }");
        let value = eval(&expr_of("S.rows"), &scope).expect("S.rows should resolve");
        assert_eq!(
            value,
            Static::List(vec![Static::Num(1.0), Static::Num(2.0), Static::Num(3.0)])
        );
    }

    #[test]
    fn length_resolves_so_a_count_can_be_painted() {
        let (_, scope) =
            page_scope("store S { state rows = [1, 2] }\npage P(path: \"/\") { Text(\"x\") }");
        assert_eq!(
            eval(&expr_of("S.rows.length"), &scope),
            Some(Static::Num(2.0))
        );
    }

    #[test]
    fn a_page_s_own_state_is_in_scope() {
        let (_, scope) = page_scope("page P(path: \"/\") { state n = 7\n Text(\"x\") }");
        assert_eq!(eval(&expr_of("n"), &scope), Some(Static::Num(7.0)));
    }

    #[test]
    fn map_fields_resolve_through_property_access() {
        let (_, scope) = page_scope(
            "store S { state user = { name: \"Monzer\", age: 30 } }\npage P(path: \"/\") { Text(\"x\") }",
        );
        assert_eq!(
            eval(&expr_of("S.user.name"), &scope),
            Some(Static::Str("Monzer".into()))
        );
    }

    #[test]
    fn arithmetic_and_comparison_resolve() {
        let s = Scope::default();
        assert_eq!(eval(&expr_of("2 + 3"), &s), Some(Static::Num(5.0)));
        assert_eq!(eval(&expr_of("2 > 3"), &s), Some(Static::Bool(false)));
        assert_eq!(
            eval(&expr_of("\"a\" + \"b\""), &s),
            Some(Static::Str("ab".into()))
        );
    }

    /// The compiler does not run user code, and it does not pretend to know what
    /// a call returns.
    #[test]
    fn anything_unknowable_returns_none_rather_than_a_guess() {
        let s = Scope::default();
        assert_eq!(eval(&expr_of("unknownThing"), &s), None);
        assert_eq!(eval(&expr_of("items.filter(x => x)"), &s), None);
        assert_eq!(
            eval(&expr_of("1 / 0"), &s),
            None,
            "no infinity in the output"
        );
    }

    #[test]
    fn a_loop_binding_shadows_the_outer_scope() {
        let (_, scope) = page_scope("page P(path: \"/\") { state x = 1\n Text(\"y\") }");
        let inner = scope.with("x", Static::Str("bound".into()));
        assert_eq!(
            eval(&expr_of("x"), &inner),
            Some(Static::Str("bound".into()))
        );
        assert_eq!(
            eval(&expr_of("x"), &scope),
            Some(Static::Num(1.0)),
            "outer is intact"
        );
    }

    #[test]
    fn truthiness_follows_javascript() {
        assert!(!Static::Str(String::new()).truthy());
        assert!(Static::Str("x".into()).truthy());
        assert!(!Static::Num(0.0).truthy());
        assert!(!Static::List(vec![]).truthy());
        assert!(Static::List(vec![Static::Null]).truthy());
        assert!(!Static::Null.truthy());
    }
}

#[cfg(test)]
mod interpreter_tests {
    //! Derived values, lambdas and actions the compiler can run at build time.
    use super::*;

    fn store_scope(src: &str) -> Scope {
        let program = crate::syntax::parse_source(src, "<t>").expect("parse");
        Scope::from_program(&program, &[])
    }

    fn field(scope: &Scope, store: &str, name: &str) -> Option<Static> {
        match scope.get(store)? {
            Static::Map(fields) => fields
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone()),
            _ => None,
        }
    }

    #[test]
    fn derived_values_over_lambdas_resolve_in_order() {
        let s = store_scope(
            r#"store S {
                state all = [{ n: "b", age: 2, done: false }, { n: "a", age: 1, done: true }, { n: "c", age: 3, done: false }]
                state query = "C"
                derived q = query.toLowerCase()
                derived open = all.filter(r => !r.done)
                derived names = open.map(r => r.n.toUpperCase()).join(", ")
                derived hit = all.filter(r => r.n.indexOf(q) > -1).length
                derived oldest = all.slice().sort((a, b) => b.age - a.age)[0].n
                derived label = if open.length > 1 { "many" } else { "one" }
            }"#,
        );
        assert_eq!(field(&s, "S", "q"), Some(Static::Str("c".into())));
        assert_eq!(field(&s, "S", "names"), Some(Static::Str("B, C".into())));
        assert_eq!(field(&s, "S", "hit"), Some(Static::Num(1.0)));
        assert_eq!(field(&s, "S", "oldest"), Some(Static::Str("c".into())));
        assert_eq!(field(&s, "S", "label"), Some(Static::Str("many".into())));
    }

    #[test]
    fn an_action_of_locals_ifs_and_returns_runs_at_build_time() {
        let s = store_scope(
            r#"store S {
                state all = [{ h: "x", dur: 0, st: "ready" }, { h: "y", dur: 12.345, st: "failed" }]
                state sortDir = "desc"
                derived rows = sortRows(all).map(r => shape(r))
                action shape(r: Map) {
                    return { h: r.h, status: word(r.st), duration: if r.dur == 0 { "—" } else { r.dur.toFixed(1) + " s" } }
                }
                action word(st: String) {
                    if st == "ready" { return "Ready" }
                    if st == "failed" { return "Failed" }
                    return st
                }
                action sortRows(list: [Any]) {
                    copy = list.slice()
                    dir = if sortDir == "asc" { 1 } else { -1 }
                    copy.sort((a, b) => (a.dur - b.dur) * dir)
                    return copy
                }
            }"#,
        );
        let Some(Static::List(rows)) = field(&s, "S", "rows") else {
            panic!("rows did not resolve: {:?}", s.get("S"));
        };
        assert_eq!(rows.len(), 2);
        let first = &rows[0];
        let get = |k: &str| match first {
            Static::Map(f) => f.iter().find(|(n, _)| n == k).map(|(_, v)| v.to_text()),
            _ => None,
        };
        assert_eq!(
            get("h").as_deref(),
            Some("y"),
            "sorted by duration, descending"
        );
        assert_eq!(get("status").as_deref(), Some("Failed"));
        assert_eq!(get("duration").as_deref(), Some("12.3 s"));
    }

    #[test]
    fn a_store_mutation_or_an_unknown_call_stops_the_evaluation() {
        let s = store_scope(
            r#"store S {
                state n = 1
                state items = [1, 2]
                derived bumped = bump()
                derived fetched = load()
                derived fine = items.length + n
                action bump() {
                    n = n + 1
                    return n
                }
                action load() {
                    data = fetchSomething()
                    return data
                }
            }"#,
        );
        assert_eq!(
            field(&s, "S", "bumped"),
            None,
            "writing a store field is not for build time"
        );
        assert_eq!(field(&s, "S", "fetched"), None);
        assert_eq!(field(&s, "S", "fine"), Some(Static::Num(3.0)));
    }

    #[test]
    fn runaway_recursion_runs_out_of_fuel_rather_than_hanging() {
        let s = store_scope(
            r#"store S {
                derived forever = again(0)
                action again(i: Number) { return again(i + 1) }
            }"#,
        );
        assert_eq!(field(&s, "S", "forever"), None);
    }
}
