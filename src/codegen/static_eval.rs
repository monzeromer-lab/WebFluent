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

use crate::codegen::format;
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
    /// A regular expression: its pattern and flags, compiled on use.
    Regex(String, String),
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
            Static::Regex(p, f) => format!("/{p}/{f}"),
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
            Static::Map(_) | Static::Regex(..) => true,
            Static::Null => false,
        }
    }
}

/// The compiled form of a regex value, JavaScript's flags mapped to Rust's
/// (`i`, `m`, `s`; `g` and `u` mean nothing to a single match).
/// Whether `text` matches the pattern, when the pattern compiles.
pub fn matches(text: &str, pattern: &str, flags: &str) -> Option<bool> {
    Some(compile_regex(pattern, flags)?.is_match(text))
}

fn compile_regex(pattern: &str, flags: &str) -> Option<regex::Regex> {
    let mut prefix = String::new();
    for f in flags.chars() {
        match f {
            'i' | 'm' | 's' => prefix.push(f),
            _ => {}
        }
    }
    let source = if prefix.is_empty() {
        pattern.to_string()
    } else {
        format!("(?{prefix}){pattern}")
    };
    regex::Regex::new(&source).ok()
}

/// Names in scope at build time, innermost last, and the actions a call can
/// reach.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    frames: Vec<HashMap<String, Static>>,
    functions: HashMap<String, ActionDecl>,
    depth: u32,
}

impl Static {
    /// A JSON value as a static value: what a template's data is.
    pub fn from_json(value: &serde_json::Value) -> Static {
        match value {
            serde_json::Value::Null => Static::Null,
            serde_json::Value::Bool(b) => Static::Bool(*b),
            serde_json::Value::Number(n) => Static::Num(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Static::Str(s.clone()),
            serde_json::Value::Array(items) => {
                Static::List(items.iter().map(Static::from_json).collect())
            }
            serde_json::Value::Object(map) => Static::Map(
                map.iter()
                    .map(|(k, v)| (k.clone(), Static::from_json(v)))
                    .collect(),
            ),
        }
    }

    /// The static value as JSON.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Static::Null => serde_json::Value::Null,
            Static::Bool(b) => serde_json::Value::Bool(*b),
            Static::Num(n) => serde_json::Number::from_f64(*n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Static::Str(s) => serde_json::Value::String(s.clone()),
            Static::Regex(p, f) => serde_json::Value::String(format!("/{p}/{f}")),
            Static::List(items) => {
                serde_json::Value::Array(items.iter().map(Static::to_json).collect())
            }
            Static::Map(fields) => serde_json::Value::Object(
                fields
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_json()))
                    .collect(),
            ),
        }
    }
}

impl Scope {
    /// A scope of the given names and values, for evaluating an expression
    /// over data — what the template engine hands over.
    pub fn of(values: impl IntoIterator<Item = (String, Static)>) -> Self {
        Scope {
            frames: vec![values.into_iter().collect()],
            functions: HashMap::new(),
            depth: 0,
        }
    }

    /// The scope a page starts with: every store's seeded state, keyed
    /// `Store.field`, plus the page's own literal `state` declarations.
    ///
    /// Only initial values. A store field a user action has since changed is not
    /// knowable here, and the hydrating client will correct it.
    pub fn from_program(program: &Program, page_body: &[Statement]) -> Self {
        Self::from_program_with_env(program, page_body, &Default::default())
    }

    /// [`Scope::from_program`] with the project's `env`, which a constant
    /// may read.
    pub fn from_program_with_env(
        program: &Program,
        page_body: &[Statement],
        env: &std::collections::BTreeMap<String, serde_json::Value>,
    ) -> Self {
        let mut frame = HashMap::new();
        frame.insert(
            "env".to_string(),
            Static::Map(
                env.iter()
                    .map(|(k, v)| (k.clone(), Static::from_json(v)))
                    .collect(),
            ),
        );

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
        // Constants, in order: one may read another.
        for decl in &program.declarations {
            if let Declaration::Const(c) = decl
                && let Some(v) = eval(&c.value, &scope)
            {
                scope.set(&c.name, v);
            }
        }
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

    /// Add the literal `state` declarations of a statement list, and the
    /// `derived` values that follow from them, in order.
    pub fn push_state(&mut self, stmts: &[Statement]) {
        let mut frame = HashMap::new();
        for stmt in stmts {
            if let StatementKind::State(s) = &stmt.kind
                && let Some(v) = eval(&s.value, self)
            {
                frame.insert(s.name.clone(), v);
            }
        }
        self.frames.push(frame);
        for stmt in stmts {
            if let StatementKind::Derived(d) = &stmt.kind
                && let Some(v) = eval(&d.value, self)
                && let Some(frame) = self.frames.last_mut()
            {
                frame.insert(d.name.clone(), v);
            }
        }
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

    /// The scope with the locale `format` and `ago` speak, the project's
    /// default one.
    pub fn with_locale(mut self, locale: &str) -> Self {
        if let Some(frame) = self.frames.first_mut() {
            frame.insert("__locale".to_string(), Static::Str(locale.to_string()));
        }
        self
    }

    /// The scope with the default locale's messages, so `t(…)` resolves
    /// wherever it is written. It used to resolve only as the whole of a
    /// text — `Text(t("k"))` — and anywhere else the evaluator had never
    /// heard of it: spliced into a string, added to one, or held in a prop
    /// the component then spliced, the whole value came back unknown and the
    /// static paint left the element empty until the script ran.
    pub fn with_messages(mut self, messages: &HashMap<String, String>) -> Self {
        if let Some(frame) = self.frames.first_mut() {
            frame.insert(
                "__messages".to_string(),
                Static::Map(
                    messages
                        .iter()
                        .map(|(k, v)| (k.clone(), Static::Str(v.clone())))
                        .collect(),
                ),
            );
        }
        self
    }

    /// The messages at hand, empty when the project has none.
    fn messages(&self) -> HashMap<String, String> {
        match self.get("__messages") {
            Some(Static::Map(entries)) => entries
                .iter()
                .filter_map(|(k, v)| match v {
                    Static::Str(s) => Some((k.clone(), s.clone())),
                    _ => None,
                })
                .collect(),
            _ => HashMap::new(),
        }
    }

    /// The locale at hand: the project's, a `locale` in the data, or English.
    fn locale(&self) -> String {
        match self.get("__locale").or_else(|| self.get("locale")) {
            Some(Static::Str(l)) => l.clone(),
            _ => "en".to_string(),
        }
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
/// A store as a value: its state, and the derived values computed from it.
///
/// `seed` is what a caller already knows the state to be — a test's `data`,
/// which stands in for what an action would have put there — and is
/// applied before the derived values, so they follow it.
pub fn store_as_value(
    store: &crate::parser::ast::StoreDecl,
    seed: Option<&serde_json::Value>,
) -> serde_json::Value {
    store_value_with(store, seed).to_json()
}

fn store_value(store: &crate::parser::ast::StoreDecl) -> Static {
    store_value_with(store, None)
}

fn store_value_with(
    store: &crate::parser::ast::StoreDecl,
    seed: Option<&serde_json::Value>,
) -> Static {
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
        // What the caller already knows stands in for the initial value.
        let given = match (&stmt.kind, seed) {
            (StatementKind::State(_), Some(serde_json::Value::Object(map))) => {
                map.get(name).map(Static::from_json)
            }
            _ => None,
        };
        if let Some(v) = given.or_else(|| eval(value, &scope)) {
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
        Expr::Typed(_, carrier) => eval_in(carrier, scope, fuel),
        Expr::NumberLiteral(n) => Some(Static::Num(*n)),
        Expr::BoolLiteral(b) => Some(Static::Bool(*b)),
        Expr::Null => Some(Static::Null),
        Expr::Regex(p, f) => Some(Static::Regex(p.clone(), f.clone())),

        Expr::Identifier(name) => scope.get(name).cloned(),

        Expr::ListLiteral(items) => {
            let mut out = Vec::new();
            for e in items {
                match e {
                    Expr::Spread(inner) => match eval_in(inner, scope, fuel)? {
                        Static::List(more) => out.extend(more),
                        _ => return None,
                    },
                    _ => out.push(eval_in(e, scope, fuel)?),
                }
            }
            Some(Static::List(out))
        }
        // A spread outside a list is not a value.
        Expr::Spread(_) => None,
        Expr::Range(a, b, inclusive) => {
            let (Static::Num(a), Static::Num(b)) =
                (eval_in(a, scope, fuel)?, eval_in(b, scope, fuel)?)
            else {
                return None;
            };
            let end = if *inclusive { b } else { b - 1.0 };
            let mut out = Vec::new();
            let mut i = a;
            while i <= end && out.len() < 100_000 {
                out.push(Static::Num(i));
                i += 1.0;
            }
            Some(Static::List(out))
        }

        Expr::MapLiteral(entries) | Expr::Record(_, entries) => {
            let mut out: Vec<(String, Static)> = Vec::new();
            for (k, v) in entries {
                let value = eval_in(v, scope, fuel)?;
                if k == "..." {
                    let Static::Map(more) = value else {
                        return None;
                    };
                    for (mk, mv) in more {
                        match out.iter_mut().find(|(key, _)| *key == mk) {
                            Some(slot) => slot.1 = mv,
                            None => out.push((mk, mv)),
                        }
                    }
                    continue;
                }
                let key = k.trim_matches('"').to_string();
                match out.iter_mut().find(|(existing, _)| *existing == key) {
                    Some(slot) => slot.1 = value,
                    None => out.push((key, value)),
                }
            }
            Some(Static::Map(out))
        }

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

        Expr::PropertyAccess(base_expr, prop) => {
            let base = eval_in(base_expr, scope, fuel)?;
            match (&base, prop.as_str()) {
                // The rest of a `?.` chain short-circuits with it.
                (Static::Null, _) if in_optional_chain(base_expr) => Some(Static::Null),
                (Static::List(items), "length") => Some(Static::Num(items.len() as f64)),
                (Static::Str(s), "length") => Some(Static::Num(s.chars().count() as f64)),
                (Static::Map(fields), _) => fields
                    .iter()
                    .find(|(k, _)| k == prop)
                    .map(|(_, v)| v.clone()),
                _ => None,
            }
        }

        // `?.`: null when the base is, else the plain access.
        Expr::OptionalProperty(base, prop) => match eval_in(base, scope, fuel)? {
            Static::Null => Some(Static::Null),
            _ => eval_in(
                &Expr::PropertyAccess(base.clone(), prop.clone()),
                scope,
                fuel,
            ),
        },
        Expr::OptionalIndex(base, index) => match eval_in(base, scope, fuel)? {
            Static::Null => Some(Static::Null),
            _ => eval_in(&Expr::IndexAccess(base.clone(), index.clone()), scope, fuel),
        },
        Expr::OptionalMethod(base, method, args) => match eval_in(base, scope, fuel)? {
            Static::Null => Some(Static::Null),
            _ => eval_in(
                &Expr::MethodCall(base.clone(), method.clone(), args.clone()),
                scope,
                fuel,
            ),
        },

        Expr::IndexAccess(base_expr, index) => {
            let base = eval_in(base_expr, scope, fuel)?;
            let index = eval_in(index, scope, fuel)?;
            match (base, index) {
                (Static::Null, _) if in_optional_chain(base_expr) => Some(Static::Null),
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
        // `if let x = e { a } else { b }` as a value.
        Expr::MethodCall(value, name, branches) if name == "__iflet" && branches.len() == 2 => {
            let v = eval_in(value, scope, fuel)?;
            if matches!(v, Static::Null) {
                eval_in(&branches[1], scope, fuel)
            } else {
                apply(&branches[0], &[v], scope, fuel)
            }
        }

        Expr::MethodCall(subject, name, args) if name == "__case" && args.is_empty() => {
            Some(case_of(&eval_in(subject, scope, fuel)?))
        }
        // `s is .case` and the payload where it is: what `match` lowers to.
        Expr::MethodCall(subject, name, args) if name == "__is" && args.len() == 1 => {
            let v = eval_in(subject, scope, fuel)?;
            let case = eval_in(&args[0], scope, fuel)?;
            Some(Static::Bool(case_of(&v) == case))
        }
        Expr::MethodCall(subject, name, args) if name == "__payload" && args.len() == 1 => {
            let v = eval_in(subject, scope, fuel)?;
            let case = eval_in(&args[0], scope, fuel)?;
            Some(payload_of(&v, &case))
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
            if matches!(receiver, Static::Null) && in_optional_chain(base) {
                return Some(Static::Null);
            }
            method_call(&receiver, method, args, scope, fuel)
        }

        Expr::FunctionCall(name, args) => {
            let args = args
                .iter()
                .map(|a| eval_in(a, scope, fuel))
                .collect::<Option<Vec<_>>>()?;
            match name.as_str() {
                // `t("key")` and `t("key", { count: n, name: x })`: the
                // message in the default locale, its plural picked by
                // `count` and its placeholders filled — what the top-level
                // text path already did, now wherever `t` is written.
                "t" if !scope.functions.contains_key(name) => {
                    let Some(Static::Str(key)) = args.first() else {
                        return None;
                    };
                    let params = match args.get(1) {
                        None => Vec::new(),
                        Some(Static::Map(params)) => params.clone(),
                        Some(_) => return None,
                    };
                    Some(Static::Str(crate::i18n::message(
                        &scope.messages(),
                        key,
                        &params,
                    )))
                }
                // `sanitize(html)`: the twin of the runtime's, so the
                // static paint shows exactly what hydration will.
                "sanitize" if !scope.functions.contains_key(name) => {
                    let Some(Static::Str(html)) = args.first() else {
                        return None;
                    };
                    Some(Static::Str(crate::codegen::sanitize::sanitize(html)))
                }
                // `format(value, .style, option)` and `ago(date)`, as the
                // browser would spell them, in the project's locale.
                "format" | "ago" if !scope.functions.contains_key(name) => {
                    let input = |v: &Static| match v {
                        Static::Num(n) => Some(format::Input::Number(*n)),
                        Static::Str(s) => Some(format::Input::Text(s.clone())),
                        _ => None,
                    };
                    // Money says what it is, so `format(price)` needs no
                    // style and no currency code: it carries both.
                    if let Some(Static::Map(fields)) = args.first()
                        && let (Some(amount), Some(code)) = (
                            fields.iter().find(|(k, _)| k == "amount"),
                            fields.iter().find(|(k, _)| k == "currency"),
                        )
                        && let Static::Num(minor) = amount.1
                    {
                        return format::format(
                            &format::Input::Number(minor / 100.0),
                            Some("currency"),
                            Some(&format::Input::Text(code.1.to_text())),
                            &scope.locale(),
                        )
                        .map(Static::Str);
                    }
                    let value = match args.first()? {
                        Static::Null => return Some(Static::Str(String::new())),
                        v => input(v)?,
                    };
                    if name == "ago" {
                        let now = args.get(1).and_then(input);
                        return format::ago(&value, now.as_ref()).map(Static::Str);
                    }
                    let style = match args.get(1) {
                        Some(Static::Str(s)) => Some(s.clone()),
                        _ => None,
                    };
                    let option = args.get(2).and_then(input);
                    format::format(&value, style.as_deref(), option.as_ref(), &scope.locale())
                        .map(Static::Str)
                }
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
        // A case with a payload is the case name followed by the payload.
        Expr::CaseValue(case, args) => {
            let mut items = vec![Static::Str(case.clone())];
            for a in args {
                items.push(eval_in(a, scope, fuel)?);
            }
            Some(Static::List(items))
        }
        Expr::Token(name) => Some(Static::Str(format!("var(--{name})"))),
        Expr::Await(_) => None,
    }
}

/// Whether `expr` is, or reads through, a `?.`.
fn in_optional_chain(expr: &Expr) -> bool {
    match expr {
        Expr::OptionalProperty(..) | Expr::OptionalMethod(..) | Expr::OptionalIndex(..) => true,
        Expr::PropertyAccess(base, _)
        | Expr::IndexAccess(base, _)
        | Expr::MethodCall(base, _, _) => in_optional_chain(base),
        _ => false,
    }
}

/// A JavaScript replacement string as the regex crate reads it: `$1`
/// stays `$1`, `$&` is the whole match (`$0`), a literal `$` is `$$`.
fn js_replacement(to: &str) -> String {
    to.replace("$&", "${0}")
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
    // What the language's own types can do, done here as well as in the
    // browser: a page that shows a date's arithmetic shows the answer in
    // the static paint, not a blank that fills in on hydration.
    if let Some(result) = scalars::method(receiver, method, &|i| value(i)) {
        return Some(result);
    }
    match receiver {
        Static::Regex(p, f) => {
            let re = compile_regex(p, f)?;
            let text = value(0)?.to_text();
            match method {
                "test" => Some(Static::Bool(re.is_match(&text))),
                "exec" => Some(match re.captures(&text) {
                    Some(caps) => Static::List(
                        caps.iter()
                            .map(|c| Static::Str(c.map(|m| m.as_str()).unwrap_or("").into()))
                            .collect(),
                    ),
                    None => Static::Null,
                }),
                _ => None,
            }
        }
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
            // `contains` is what WebFluent calls it; `includes` is the
            // JavaScript name the same call also answers to. Only the
            // JavaScript names were here, so a static paint of
            // `items.contains(x)` produced nothing and the element came out
            // empty until the page's script ran.
            "includes" | "contains" => {
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
            // The helpers the runtime adds to a list.
            "sortBy" => {
                let f = args.first()?;
                let mut keyed = Vec::new();
                for item in items {
                    keyed.push((
                        apply(f, std::slice::from_ref(item), scope, fuel)?,
                        item.clone(),
                    ));
                }
                keyed.sort_by(|(a, _), (b, _)| match (a, b) {
                    (Static::Num(x), Static::Num(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    _ => a.to_text().cmp(&b.to_text()),
                });
                Some(Static::List(keyed.into_iter().map(|(_, v)| v).collect()))
            }
            "groupBy" => {
                let f = args.first()?;
                let mut groups: Vec<(String, Static)> = Vec::new();
                for item in items {
                    let key = apply(f, std::slice::from_ref(item), scope, fuel)?.to_text();
                    match groups.iter_mut().find(|(k, _)| *k == key) {
                        Some((_, Static::List(members))) => members.push(item.clone()),
                        _ => groups.push((key, Static::List(vec![item.clone()]))),
                    }
                }
                Some(Static::Map(groups))
            }
            "unique" => {
                let mut out: Vec<Static> = Vec::new();
                for item in items {
                    if !out.contains(item) {
                        out.push(item.clone());
                    }
                }
                Some(Static::List(out))
            }
            "take" => {
                let n = number(0)?.max(0.0) as usize;
                Some(Static::List(items.iter().take(n).cloned().collect()))
            }
            "first" => Some(items.first().cloned().unwrap_or(Static::Null)),
            "last" => Some(items.last().cloned().unwrap_or(Static::Null)),
            "flatMap" => {
                let f = args.first()?;
                let mut out = Vec::new();
                for (i, item) in items.iter().enumerate() {
                    match apply(f, &[item.clone(), Static::Num(i as f64)], scope, fuel)? {
                        Static::List(more) => out.extend(more),
                        other => out.push(other),
                    }
                }
                Some(Static::List(out))
            }
            "sum" => Some(Static::Num(
                items
                    .iter()
                    .map(|v| match v {
                        Static::Num(n) => *n,
                        _ => 0.0,
                    })
                    .sum(),
            )),
            _ => None,
        },
        Static::Str(s) => match method {
            "toLowerCase" | "toLower" => Some(Static::Str(s.to_lowercase())),
            "toUpperCase" | "toUpper" => Some(Static::Str(s.to_uppercase())),
            "trim" => Some(Static::Str(s.trim().to_string())),
            "indexOf" => {
                let needle = value(0)?.to_text();
                Some(Static::Num(
                    s.find(&needle)
                        .map(|b| s[..b].chars().count() as f64)
                        .unwrap_or(-1.0),
                ))
            }
            "includes" | "contains" => Some(Static::Bool(s.contains(&value(0)?.to_text()))),
            "startsWith" => Some(Static::Bool(s.starts_with(&value(0)?.to_text()))),
            "endsWith" => Some(Static::Bool(s.ends_with(&value(0)?.to_text()))),
            "split" => {
                let parts: Vec<Static> = match value(0)? {
                    Static::Regex(p, f) => compile_regex(&p, &f)?
                        .split(s)
                        .map(|p| Static::Str(p.to_string()))
                        .collect(),
                    sep => {
                        let sep = sep.to_text();
                        if sep.is_empty() {
                            s.chars().map(|c| Static::Str(c.to_string())).collect()
                        } else {
                            s.split(&sep).map(|p| Static::Str(p.to_string())).collect()
                        }
                    }
                };
                Some(Static::List(parts))
            }
            "replace" | "replaceAll" => {
                let to = value(1)?.to_text();
                match value(0)? {
                    Static::Regex(p, f) => {
                        let re = compile_regex(&p, &f)?;
                        let to = js_replacement(&to);
                        let out = if f.contains('g') || method == "replaceAll" {
                            re.replace_all(s, to.as_str()).to_string()
                        } else {
                            re.replacen(s, 1, to.as_str()).to_string()
                        };
                        Some(Static::Str(out))
                    }
                    from => {
                        let from = from.to_text();
                        Some(Static::Str(if method == "replaceAll" {
                            s.replace(&from, &to)
                        } else {
                            s.replacen(&from, &to, 1)
                        }))
                    }
                }
            }
            "match" => match value(0)? {
                Static::Regex(p, f) => {
                    let re = compile_regex(&p, &f)?;
                    if f.contains('g') {
                        let all: Vec<Static> = re
                            .find_iter(s)
                            .map(|m| Static::Str(m.as_str().into()))
                            .collect();
                        Some(if all.is_empty() {
                            Static::Null
                        } else {
                            Static::List(all)
                        })
                    } else {
                        Some(match re.captures(s) {
                            Some(caps) => Static::List(
                                caps.iter()
                                    .map(|c| {
                                        Static::Str(c.map(|m| m.as_str()).unwrap_or("").into())
                                    })
                                    .collect(),
                            ),
                            None => Static::Null,
                        })
                    }
                }
                _ => None,
            },
            "search" => match value(0)? {
                Static::Regex(p, f) => Some(Static::Num(
                    compile_regex(&p, &f)?
                        .find(s)
                        .map(|m| s[..m.start()].chars().count() as f64)
                        .unwrap_or(-1.0),
                )),
                _ => None,
            },
            "dedent" => {
                let mut rows: Vec<&str> = s.split('\n').collect();
                while rows.first().is_some_and(|r| r.trim().is_empty()) {
                    rows.remove(0);
                }
                while rows.last().is_some_and(|r| r.trim().is_empty()) {
                    rows.pop();
                }
                let indent = rows
                    .iter()
                    .filter(|r| !r.trim().is_empty())
                    .map(|r| r.len() - r.trim_start().len())
                    .min()
                    .unwrap_or(0);
                Some(Static::Str(
                    rows.iter()
                        .map(|r| {
                            if r.len() >= indent {
                                &r[indent..]
                            } else {
                                r.trim_start()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                ))
            }
            "lines" => Some(Static::List(
                s.split('\n')
                    .map(|l| Static::Str(l.trim_end_matches('\r').to_string()))
                    .collect(),
            )),
            "words" => Some(Static::List(
                s.split_whitespace()
                    .map(|w| Static::Str(w.to_string()))
                    .collect(),
            )),
            "capitalize" => {
                let mut chars = s.chars();
                Some(Static::Str(match chars.next() {
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }))
            }
            "truncate" => {
                let n = number(0)? as usize;
                let text: String = s.chars().take(n).collect();
                Some(Static::Str(if s.chars().count() > n {
                    format!("{text}…")
                } else {
                    text
                }))
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
/// The case of an enum value: a bare case is its name, a case with a payload
/// the list `["case", …payload]`.
pub fn case_of(v: &Static) -> Static {
    match v {
        Static::List(items) => items.first().cloned().unwrap_or(Static::Null),
        other => other.clone(),
    }
}

/// The payload of `v` where its case is `case`: the one value of a single
/// payload, the list of a longer one, null otherwise.
pub fn payload_of(v: &Static, case: &Static) -> Static {
    match v {
        Static::List(items) if items.first() == Some(case) => match items.len() {
            2 => items[1].clone(),
            _ => Static::List(items[1..].to_vec()),
        },
        _ => Static::Null,
    }
}

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

/// The language's own types, at build time.
///
/// The twin of `src/runtime/modules/scalars.js`: the same carriers, the
/// same answers. A method this does not know falls through to the ordinary
/// ones, so a record with its own `plus` is unaffected.
mod scalars {
    use super::Static;

    /// Which carrier a value is: `date`, `time`, `datetime`, `money`,
    /// `duration`, or nothing the language knows.
    fn kind_of(value: &Static) -> Option<&'static str> {
        match value {
            Static::Num(_) => Some("duration"),
            Static::Map(fields) => {
                let has = |n: &str| fields.iter().any(|(k, _)| k == n);
                (has("amount") && has("currency")).then_some("money")
            }
            Static::Str(text) => {
                let digits = |s: &str| s.chars().all(|c| c.is_ascii_digit());
                let parts: Vec<&str> = text.split('-').collect();
                if parts.len() == 3
                    && parts[0].len() == 4
                    && parts[1].len() == 2
                    && parts[2].len() == 2
                    && parts.iter().all(|p| digits(p))
                {
                    return Some("date");
                }
                if text.len() >= 11 && text.as_bytes().get(10) == Some(&b'T') {
                    return Some("datetime");
                }
                let parts: Vec<&str> = text.split(':').collect();
                ((2..=3).contains(&parts.len())
                    && parts[0].len() == 2
                    && parts[1].len() == 2
                    && parts
                        .iter()
                        .all(|p| digits(p.split('.').next().unwrap_or(p))))
                .then_some("time")
            }
            _ => None,
        }
    }

    /// Days since 1970-01-01 for a civil date, and back — the arithmetic a
    /// calendar needs, without a calendar library.
    fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
        let y = y - i64::from(m <= 2);
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = (m + 9) % 12;
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    fn civil_from_days(z: i64) -> (i64, i64, i64) {
        let z = z + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        (y + i64::from(m <= 2), m, d)
    }

    /// A carrier as milliseconds since the epoch.
    fn epoch_ms(value: &Static) -> Option<i64> {
        let text = match value {
            Static::Str(s) => s.clone(),
            _ => return None,
        };
        let (date, time) = match kind_of(value)? {
            "date" => (text.clone(), String::new()),
            "time" => ("1970-01-01".to_string(), text.clone()),
            "datetime" => {
                let (d, t) = text.split_once('T')?;
                (d.to_string(), t.trim_end_matches('Z').to_string())
            }
            _ => return None,
        };
        let part = |s: &str, n: usize| -> Option<i64> {
            s.split(['-', ':']).nth(n)?.split('.').next()?.parse().ok()
        };
        let days = days_from_civil(part(&date, 0)?, part(&date, 1)?, part(&date, 2)?);
        let mut ms = days * 86_400_000;
        if !time.is_empty() {
            let time = time.split('+').next().unwrap_or(&time);
            ms += part(time, 0).unwrap_or(0) * 3_600_000;
            ms += part(time, 1).unwrap_or(0) * 60_000;
            ms += part(time, 2).unwrap_or(0) * 1_000;
        }
        Some(ms)
    }

    /// Milliseconds back into the carrier `like` was written in.
    fn carry(ms: i64, like: &Static) -> Static {
        let days = ms.div_euclid(86_400_000);
        let rest = ms.rem_euclid(86_400_000);
        let (y, m, d) = civil_from_days(days);
        let (hh, mm, ss) = (rest / 3_600_000, (rest / 60_000) % 60, (rest / 1000) % 60);
        Static::Str(match kind_of(like) {
            Some("date") => format!("{y:04}-{m:02}-{d:02}"),
            Some("time") => format!("{hh:02}:{mm:02}"),
            _ => format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z"),
        })
    }

    fn field(value: &Static, name: &str) -> Option<f64> {
        match value {
            Static::Map(fields) => {
                fields
                    .iter()
                    .find(|(k, _)| k == name)
                    .and_then(|(_, v)| match v {
                        Static::Num(n) => Some(*n),
                        _ => None,
                    })
            }
            _ => None,
        }
    }

    fn currency(value: &Static) -> String {
        match value {
            Static::Map(fields) => fields
                .iter()
                .find(|(k, _)| k == "currency")
                .map(|(_, v)| v.to_text())
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn money(amount: f64, currency: &str) -> Static {
        Static::Map(vec![
            ("amount".to_string(), Static::Num(amount.round())),
            ("currency".to_string(), Static::Str(currency.to_string())),
        ])
    }

    /// `#0F766E` and friends as red, green and blue.
    fn rgb_of(value: &Static) -> Option<(f64, f64, f64)> {
        let text = value.to_text();
        let hex = text.trim().strip_prefix('#')?;
        let hex: String = match hex.len() {
            3 | 4 => hex.chars().flat_map(|c| [c, c]).collect(),
            6 | 8 => hex.to_string(),
            _ => return None,
        };
        let channel = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).ok().map(f64::from);
        Some((channel(0)?, channel(2)?, channel(4)?))
    }

    fn hex(n: f64) -> String {
        format!("{:02X}", n.round().clamp(0.0, 255.0) as u8)
    }

    fn mix(a: &Static, b: &Static, t: f64) -> Option<Static> {
        let (r1, g1, b1) = rgb_of(a)?;
        let (r2, g2, b2) = rgb_of(b)?;
        Some(Static::Str(format!(
            "#{}{}{}",
            hex(r1 + (r2 - r1) * t),
            hex(g1 + (g2 - g1) * t),
            hex(b1 + (b2 - b1) * t)
        )))
    }

    fn luminance(value: &Static) -> Option<f64> {
        let (r, g, b) = rgb_of(value)?;
        let channel = |c: f64| {
            let s = c / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        Some(0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b))
    }

    /// The part of a URL after the scheme and before the path, and the path.
    fn url_parts(text: &str) -> Option<(String, String)> {
        let rest = text.split_once("://").map(|(_, r)| r).unwrap_or(text);
        let (host, path) = match rest.find('/') {
            Some(at) => (&rest[..at], &rest[at..]),
            None => (rest, "/"),
        };
        let path = path.split(['?', '#']).next().unwrap_or(path);
        Some((host.to_string(), path.to_string()))
    }

    /// A URL's search, as `URLSearchParams` reads it: `+` is a space and a
    /// `%XX` is its byte.
    fn query_pairs(text: &str) -> Vec<(String, Static)> {
        let search = text.split('#').next().unwrap_or(text);
        let Some((_, search)) = search.split_once('?') else {
            return Vec::new();
        };
        search
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
                (form_decode(key), Static::Str(form_decode(value)))
            })
            .collect()
    }

    fn form_decode(text: &str) -> String {
        let bytes = text.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'+' => out.push(b' '),
                b'%' if i + 2 < bytes.len() => {
                    let hex = |b: u8| (b as char).to_digit(16);
                    match (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                        (Some(hi), Some(lo)) => {
                            out.push((hi * 16 + lo) as u8);
                            i += 2;
                        }
                        _ => out.push(b'%'),
                    }
                }
                other => out.push(other),
            }
            i += 1;
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    /// `application/x-www-form-urlencoded`, as `URLSearchParams` writes it.
    fn form_encode(text: &str) -> String {
        let mut out = String::new();
        for byte in text.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                    out.push(byte as char)
                }
                b' ' => out.push('+'),
                other => out.push_str(&format!("%{other:02X}")),
            }
        }
        out
    }

    /// The same URL with a new path, hash or query values: a `null` value
    /// takes its key away, and every other key keeps its place.
    fn url_with(text: &str, parts: &[(String, Static)]) -> Option<Static> {
        let (scheme, rest) = text.split_once("://")?;
        let (before_hash, hash) = match rest.split_once('#') {
            Some((b, h)) => (b, Some(h.to_string())),
            None => (rest, None),
        };
        let (address, _) = before_hash.split_once('?').unwrap_or((before_hash, ""));
        let (host, path) = match address.find('/') {
            Some(at) => (&address[..at], address[at..].to_string()),
            None => (address, "/".to_string()),
        };
        let mut path = path;
        let mut hash = hash;
        let mut query = query_pairs(text);
        for (key, value) in parts {
            match (key.as_str(), value) {
                ("path", Static::Str(p)) => {
                    path = if p.starts_with('/') {
                        p.clone()
                    } else {
                        format!("/{p}")
                    }
                }
                ("hash", Static::Str(h)) => {
                    let h = h.trim_start_matches('#');
                    hash = (!h.is_empty()).then(|| h.to_string());
                }
                ("query", Static::Map(values)) => {
                    for (k, v) in values {
                        match v {
                            Static::Null => query.retain(|(existing, _)| existing != k),
                            _ => {
                                let v = Static::Str(v.to_text());
                                match query.iter_mut().find(|(existing, _)| existing == k) {
                                    Some(slot) => slot.1 = v,
                                    None => query.push((k.clone(), v)),
                                }
                            }
                        }
                    }
                }
                // A part this does not know how the runtime would treat: no
                // guess, and the element waits for the script.
                _ => return None,
            }
        }
        let search = query
            .iter()
            .map(|(k, v)| format!("{}={}", form_encode(k), form_encode(&v.to_text())))
            .collect::<Vec<_>>()
            .join("&");
        let mut out = format!("{scheme}://{host}{path}");
        if !search.is_empty() {
            out.push('?');
            out.push_str(&search);
        }
        if let Some(h) = hash {
            out.push('#');
            out.push_str(&h);
        }
        Some(Static::Str(out))
    }

    /// One method of one of the language's own types, or `None` when this
    /// is not one of them.
    pub fn method(
        receiver: &Static,
        method: &str,
        arg: &dyn Fn(usize) -> Option<Static>,
    ) -> Option<Static> {
        let kind = kind_of(receiver);
        let moment = matches!(kind, Some("date" | "time" | "datetime"));
        let number = |i: usize| match arg(i)? {
            Static::Num(n) => Some(n),
            _ => None,
        };
        let named = |name: &str| match arg(0)? {
            Static::Map(fields) => {
                fields
                    .iter()
                    .find(|(k, _)| k == name)
                    .and_then(|(_, v)| match v {
                        Static::Num(n) => Some(*n),
                        _ => None,
                    })
            }
            _ => None,
        };
        match method {
            "plus" | "minus" if moment => {
                let sign = if method == "plus" { 1 } else { -1 };
                let mut ms = epoch_ms(receiver)?;
                let by = |name: &str, unit: i64| named(name).unwrap_or(0.0) as i64 * unit;
                // Months and years move by the calendar, and land on a day
                // that exists: the 31st plus a month is the 28th, not the
                // 3rd of the next.
                let months = sign * (by("months", 1) + by("years", 12));
                if months != 0 {
                    let days = ms.div_euclid(86_400_000);
                    let rest = ms.rem_euclid(86_400_000);
                    let (y, m, d) = civil_from_days(days);
                    let total = (y * 12 + m - 1) + months;
                    let (y2, m2) = (total.div_euclid(12), total.rem_euclid(12) + 1);
                    let leap = y2 % 4 == 0 && (y2 % 100 != 0 || y2 % 400 == 0);
                    let last = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][(m2 - 1) as usize]
                        + i64::from(m2 == 2 && leap);
                    ms = days_from_civil(y2, m2, d.min(last)) * 86_400_000 + rest;
                }
                Some(carry(
                    ms + sign
                        * (by("weeks", 604_800_000)
                            + by("days", 86_400_000)
                            + by("hours", 3_600_000)
                            + by("minutes", 60_000)
                            + by("seconds", 1_000)
                            + by("ms", 1)),
                    receiver,
                ))
            }
            "year" | "month" | "day" | "weekday" | "hour" | "minute" | "second" if moment => {
                let ms = epoch_ms(receiver)?;
                let days = ms.div_euclid(86_400_000);
                let rest = ms.rem_euclid(86_400_000);
                let (y, m, d) = civil_from_days(days);
                Some(Static::Num(match method {
                    "year" => y as f64,
                    "month" => m as f64,
                    "day" => d as f64,
                    // Monday is 1 and Sunday 7, as ISO counts them.
                    "weekday" => ((days + 3).rem_euclid(7) + 1) as f64,
                    "hour" => (rest / 3_600_000) as f64,
                    "minute" => ((rest / 60_000) % 60) as f64,
                    _ => ((rest / 1000) % 60) as f64,
                }))
            }
            "isBefore" | "isAfter" | "isSame" if moment => {
                let a = epoch_ms(receiver)?;
                let b = epoch_ms(&arg(0)?)?;
                Some(Static::Bool(match method {
                    "isBefore" => a < b,
                    "isAfter" => a > b,
                    _ => a == b,
                }))
            }
            "until" if moment => Some(Static::Num(
                (epoch_ms(&arg(0)?)? - epoch_ms(receiver)?) as f64,
            )),
            "date" if kind == Some("datetime") => Some(carry(
                epoch_ms(receiver)?,
                &Static::Str("0000-00-00".into()),
            )),
            "time" if kind == Some("datetime") => {
                Some(carry(epoch_ms(receiver)?, &Static::Str("00:00".into())))
            }
            "startOfDay" | "startOfWeek" | "startOfMonth" | "endOfDay" if moment => {
                let ms = epoch_ms(receiver)?;
                let days = ms.div_euclid(86_400_000);
                let at = match method {
                    "startOfDay" => days * 86_400_000,
                    "endOfDay" => days * 86_400_000 + 86_399_000,
                    "startOfWeek" => (days - (days + 3).rem_euclid(7)) * 86_400_000,
                    _ => {
                        let (y, m, _) = civil_from_days(days);
                        days_from_civil(y, m, 1) * 86_400_000
                    }
                };
                Some(carry(at, receiver))
            }
            // A length of time, read in a unit.
            "days" | "hours" | "minutes" | "seconds" | "ms" if kind == Some("duration") => {
                let ms = match receiver {
                    Static::Num(n) => *n,
                    _ => return None,
                };
                Some(Static::Num(match method {
                    "days" => ms / 86_400_000.0,
                    "hours" => ms / 3_600_000.0,
                    "minutes" => ms / 60_000.0,
                    "seconds" => ms / 1000.0,
                    _ => ms,
                }))
            }
            // Money, in minor units, so nothing drifts.
            "plus" | "minus" if kind == Some("money") => {
                let sign = if method == "plus" { 1.0 } else { -1.0 };
                let other = arg(0)?;
                if currency(&other) != currency(receiver) {
                    return None;
                }
                Some(money(
                    field(receiver, "amount")? + sign * field(&other, "amount")?,
                    &currency(receiver),
                ))
            }
            "times" if kind == Some("money") => Some(money(
                field(receiver, "amount")? * number(0)?,
                &currency(receiver),
            )),
            "convert" if kind == Some("money") => Some(money(
                field(receiver, "amount")? * number(0)?,
                &arg(1)?.to_text(),
            )),
            // A web address, and an address.
            "host" | "path" => {
                let text = receiver.to_text();
                if !text.contains("://") {
                    return None;
                }
                let (host, path) = url_parts(&text)?;
                Some(Static::Str(if method == "host" { host } else { path }))
            }
            // `site.query()` and `site.with(query: { page: 2 })`, as the
            // runtime's `URL` and `URLSearchParams` give them: a value
            // decoded, a key replaced where it stood, the search written
            // back form-encoded. Not known here, they painted nothing and the
            // page changed as its script ran.
            "query" if receiver.to_text().contains("://") => {
                let text = receiver.to_text();
                Some(Static::Map(query_pairs(&text)))
            }
            "with" if receiver.to_text().contains("://") => {
                let Static::Map(parts) = arg(0)? else {
                    return None;
                };
                url_with(&receiver.to_text(), &parts)
            }
            "domain" => {
                let text = receiver.to_text();
                let (_, domain) = text.split_once('@')?;
                (!domain.is_empty() && domain.contains('.'))
                    .then(|| Static::Str(domain.to_string()))
            }
            // Colour.
            "mix" => mix(receiver, &arg(0)?, number(1).unwrap_or(0.5)),
            "lighten" => mix(receiver, &Static::Str("#FFFFFF".into()), number(0)?),
            "darken" => mix(receiver, &Static::Str("#000000".into()), number(0)?),
            "alpha" => {
                let (r, g, b) = rgb_of(receiver)?;
                Some(Static::Str(format!(
                    "rgba({}, {}, {}, {})",
                    r.round(),
                    g.round(),
                    b.round(),
                    number(0)?
                )))
            }
            "contrast" => {
                let a = luminance(receiver)?;
                let b = luminance(&arg(0)?)?;
                let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                Some(Static::Num((ratio * 100.0).round() / 100.0))
            }
            _ => None,
        }
    }
}
