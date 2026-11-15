//! Where the checker cannot say what a value is: each place a value
//! becomes `Any`, and why. `wf types --holes` lists them; a program with
//! none is one whose every value has a type the checker knows.
//!
//! A hole is counted where the unknown value is made — a parameter with
//! no type, a field read from a `Map`, a browser global — not at every
//! place it goes afterwards: `a.b.c` on an `Any` is one hole, at `a`. The
//! rules that let a known type through loosely (a map where a record is
//! wanted, a `Date` shown as text) are counted where they are used.

use super::{Checker, Type};
use crate::syntax::parser::ast::{Declaration, Program, Statement, StatementKind, TypeRef};

/// Why a value's type is unknown, or let through without being compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HoleKind {
    // ─── Rules that let anything through ────────────────
    /// A value of unknown type given where a type is declared.
    AnyAccepted,
    /// A map and a record agreeing without their fields being compared.
    LooseRecord,
    /// Two values of different types joined into one of unknown type.
    LooseJoin,
    /// `Any` written as a type.
    WrittenAny,
    /// `Map` written as a type.
    WrittenMap,
    /// A parameter of an action or an event with no type.
    UntypedParameter,
    /// Arithmetic on a value of unknown type.
    LooseArithmetic,
    /// A value converted without being asked: a `Date` as text, a
    /// `Duration` as a number.
    Conversion,
    // ─── Declarations whose type is never known ─────────
    /// What an action returns, to its callers.
    ActionResult,
    /// A store read before the checker has met it.
    StoreOrder,
    /// A `state` that starts `null`, with no type.
    NullState,
    /// An empty list with nothing to say what it holds.
    EmptyList,
    /// An empty map with nothing to say what it holds.
    EmptyMap,
    /// A map built with `...`, whose keys are then unknown.
    SpreadMap,
    /// A `data` file with no type.
    UntypedData,
    /// A `resource`, or an endpoint, that does not say what it returns.
    UntypedResource,
    /// A connection's handle: a socket, a stream, a channel, a peer, a worker.
    Connection,
    /// An element's handle, `ref:`.
    ElementHandle,
    /// `Name(…)` naming no type and no enum.
    UnknownRecord,
    // ─── Expressions that give up ───────────────────────
    /// A field of a `Map`, or of a value with no fields the checker knows.
    MapField,
    /// An index into something that is not a list or a string.
    LooseIndex,
    /// A lambda's parameter that nothing gives a type.
    UntypedLambda,
    /// A method of a string whose result the checker does not know.
    StringMethod,
    /// A method of a list, a `Map` or an `Any` whose result the checker
    /// does not know.
    ListMethod,
    /// A field of a list or a string other than `length`.
    ListField,
    /// What `.native()`, or a `File`'s contents, hands back.
    Native,
    /// The item of a `for` over something that is not a typed list.
    LooseLoop,
    /// A `match` arm's binding the matched value does not type.
    LooseArm,
    // ─── What comes from outside ────────────────────────
    /// `catch e`.
    CaughtError,
    /// A browser global: `window`, `document`, `JSON`, `Math`, …
    BrowserGlobal,
    /// A function the language gives a program whose result is untyped:
    /// `log`, `navigate`, `toast`, …
    BuiltinFunction,
    /// `env` and `query`, maps of unknown keys.
    EnvQuery,
    /// A DOM event's handler parameter.
    DomEvent,
    /// A value a component's event or slot hands over beyond those it
    /// declares.
    ExtraEventValue,
    /// What a project script takes or returns, where its JSDoc does not say.
    Script,
    /// A request's error body, or a resource's `error` and `items`.
    RequestError,
    /// A built-in's prop of no declared type.
    BuiltinProp,
    /// What a service's `on request`, `on response` or `on error` hook receives.
    ServiceHook,
    /// A `test` declaration, which the checker does not read.
    UncheckedTest,
}

impl HoleKind {
    /// Every kind, in the order a report lists them.
    pub const ALL: [HoleKind; 39] = [
        HoleKind::AnyAccepted,
        HoleKind::LooseRecord,
        HoleKind::LooseJoin,
        HoleKind::WrittenAny,
        HoleKind::WrittenMap,
        HoleKind::UntypedParameter,
        HoleKind::LooseArithmetic,
        HoleKind::Conversion,
        HoleKind::ActionResult,
        HoleKind::StoreOrder,
        HoleKind::NullState,
        HoleKind::EmptyList,
        HoleKind::EmptyMap,
        HoleKind::SpreadMap,
        HoleKind::UntypedData,
        HoleKind::UntypedResource,
        HoleKind::Connection,
        HoleKind::ElementHandle,
        HoleKind::UnknownRecord,
        HoleKind::MapField,
        HoleKind::LooseIndex,
        HoleKind::UntypedLambda,
        HoleKind::StringMethod,
        HoleKind::ListMethod,
        HoleKind::ListField,
        HoleKind::Native,
        HoleKind::LooseLoop,
        HoleKind::LooseArm,
        HoleKind::CaughtError,
        HoleKind::BrowserGlobal,
        HoleKind::BuiltinFunction,
        HoleKind::EnvQuery,
        HoleKind::DomEvent,
        HoleKind::ExtraEventValue,
        HoleKind::Script,
        HoleKind::RequestError,
        HoleKind::BuiltinProp,
        HoleKind::ServiceHook,
        HoleKind::UncheckedTest,
    ];

    /// The name a report and `--json` give it.
    pub fn name(self) -> &'static str {
        match self {
            HoleKind::AnyAccepted => "any-accepted",
            HoleKind::LooseRecord => "loose-record",
            HoleKind::LooseJoin => "loose-join",
            HoleKind::WrittenAny => "written-any",
            HoleKind::WrittenMap => "written-map",
            HoleKind::UntypedParameter => "untyped-parameter",
            HoleKind::LooseArithmetic => "loose-arithmetic",
            HoleKind::Conversion => "conversion",
            HoleKind::ActionResult => "action-result",
            HoleKind::StoreOrder => "store-order",
            HoleKind::NullState => "null-state",
            HoleKind::EmptyList => "empty-list",
            HoleKind::EmptyMap => "empty-map",
            HoleKind::SpreadMap => "spread-map",
            HoleKind::UntypedData => "untyped-data",
            HoleKind::UntypedResource => "untyped-resource",
            HoleKind::Connection => "connection",
            HoleKind::ElementHandle => "element-handle",
            HoleKind::UnknownRecord => "unknown-record",
            HoleKind::MapField => "map-field",
            HoleKind::LooseIndex => "loose-index",
            HoleKind::UntypedLambda => "untyped-lambda",
            HoleKind::StringMethod => "string-method",
            HoleKind::ListMethod => "list-method",
            HoleKind::ListField => "list-field",
            HoleKind::Native => "native",
            HoleKind::LooseLoop => "loose-loop",
            HoleKind::LooseArm => "loose-arm",
            HoleKind::CaughtError => "caught-error",
            HoleKind::BrowserGlobal => "browser-global",
            HoleKind::BuiltinFunction => "builtin-function",
            HoleKind::EnvQuery => "env-query",
            HoleKind::DomEvent => "dom-event",
            HoleKind::ExtraEventValue => "extra-event-value",
            HoleKind::Script => "script",
            HoleKind::RequestError => "request-error",
            HoleKind::BuiltinProp => "builtin-prop",
            HoleKind::ServiceHook => "service-hook",
            HoleKind::UncheckedTest => "unchecked-test",
        }
    }

    /// What it means, in a line.
    pub fn describe(self) -> &'static str {
        match self {
            HoleKind::AnyAccepted => "a value of unknown type given where a type is declared",
            HoleKind::LooseRecord => "a map and a record agree without their fields compared",
            HoleKind::LooseJoin => "two values of different types joined into one of unknown type",
            HoleKind::WrittenAny => "`Any` written as a type",
            HoleKind::WrittenMap => "`Map` written as a type",
            HoleKind::UntypedParameter => "a parameter with no type",
            HoleKind::LooseArithmetic => "arithmetic on a value of unknown type",
            HoleKind::Conversion => {
                "a value converted unasked: a scalar as text, a duration as a number"
            }
            HoleKind::ActionResult => "what an action returns, unknown to its callers",
            HoleKind::StoreOrder => "a store read before the checker has met it",
            HoleKind::NullState => "a `state` that starts `null` with no type",
            HoleKind::EmptyList => "an empty list with nothing to say what it holds",
            HoleKind::EmptyMap => "an empty map with nothing to say what it holds",
            HoleKind::SpreadMap => "a map built with `...`, its keys unknown",
            HoleKind::UntypedData => "a `data` file with no type",
            HoleKind::UntypedResource => "a resource or endpoint that does not say what it returns",
            HoleKind::Connection => "a connection's handle",
            HoleKind::ElementHandle => "an element's handle, `ref:`",
            HoleKind::UnknownRecord => "`Name(…)` naming no type and no enum",
            HoleKind::MapField => "a field of a `Map`, or of a value with no known fields",
            HoleKind::LooseIndex => "an index into something not a list or a string",
            HoleKind::UntypedLambda => "a lambda's parameter nothing gives a type",
            HoleKind::StringMethod => "a string method whose result is unknown",
            HoleKind::ListMethod => "a list method whose result is unknown",
            HoleKind::ListField => "a field of a list or a string other than `length`",
            HoleKind::Native => "what `.native()` or a file's contents hand back",
            HoleKind::LooseLoop => "a `for` item over something not a typed list",
            HoleKind::LooseArm => "a `match` arm's binding the value does not type",
            HoleKind::CaughtError => "`catch e`",
            HoleKind::BrowserGlobal => "a browser global",
            HoleKind::BuiltinFunction => "a function of the language's whose result is untyped",
            HoleKind::EnvQuery => "`env` or `query`, maps of unknown keys",
            HoleKind::DomEvent => "a DOM event handler's parameter",
            HoleKind::ExtraEventValue => {
                "a value an event or a slot hands over beyond those declared"
            }
            HoleKind::Script => "what a project script takes or returns, untyped by its JSDoc",
            HoleKind::RequestError => "a request's error body, or a resource's `error`",
            HoleKind::BuiltinProp => "a built-in's prop of no declared type",
            HoleKind::ServiceHook => "what a service's hook receives",
            HoleKind::UncheckedTest => "a `test` declaration the checker does not read",
        }
    }
}

/// One place a value's type is unknown.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hole {
    /// The file it is in, as the project names it.
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub kind: HoleKind,
    /// What is unknown: the expression, the name, the parameter.
    pub what: String,
}

impl Checker<'_, '_> {
    /// Record a hole of `kind` at the statement being checked, and give the
    /// unknown value: `Any`.
    pub(super) fn hole(&mut self, kind: HoleKind, what: impl Into<String>) -> Type {
        let span = self.current_span;
        self.info.holes.push(Hole {
            file: self.file.to_string(),
            line: span.line as usize,
            column: span.col as usize,
            kind,
            what: what.into(),
        });
        Type::Any
    }

    /// A value made unknown from `from`: a field of it, an item, a result.
    /// One already unknown stays as it is, and is not counted again; one an
    /// error was reported about stays an error.
    pub(super) fn loose(&mut self, from: &Type, kind: HoleKind, what: impl Into<String>) -> Type {
        match from.unwrapped() {
            Type::Error => Type::Error,
            Type::Any => Type::Any,
            _ => self.hole(kind, what),
        }
    }

    /// The type both `a` and `b` fit. Two that have nothing in common make
    /// one of unknown type: a hole at `expr`, where they meet.
    pub(super) fn join(
        &mut self,
        a: Type,
        b: Type,
        expr: &crate::syntax::parser::ast::Expr,
    ) -> Type {
        self.join_at(a, b, &|| super::expr_text(expr))
    }

    /// [`Checker::join`], naming the place as `what` says.
    pub(super) fn join_at(&mut self, a: Type, b: Type, what: &dyn Fn() -> String) -> Type {
        let known = !has_any(&a) && !has_any(&b);
        let joined = Type::join(a, b);
        if known && has_any(&joined) {
            self.hole(HoleKind::LooseJoin, what());
        }
        joined
    }

    /// What a DOM event's handler is given: an event the checker does not
    /// type, a hole where the handler names it.
    pub(super) fn dom_event(&mut self, handler: &crate::syntax::parser::ast::EventHandler) -> Type {
        match handler.params.first() {
            Some(name) => self.hole(HoleKind::DomEvent, format!("on {}({name})", handler.event)),
            None => Type::Any,
        }
    }

    /// `Any` and `Map` written in a declared type, and a parameter written
    /// with none.
    pub(super) fn written(&mut self, ty: &TypeRef, what: &str) {
        match ty {
            TypeRef::Any => {
                self.hole(HoleKind::WrittenAny, what);
            }
            TypeRef::Map => {
                self.hole(HoleKind::WrittenMap, what);
            }
            TypeRef::List(inner) | TypeRef::Optional(inner) | TypeRef::Refined(inner, _) => {
                self.written(inner, what);
            }
            TypeRef::Dict(k, v) => {
                self.written(k, what);
                self.written(v, what);
            }
            TypeRef::Func(params, ret) => {
                for p in params {
                    self.written(p, what);
                }
                self.written(ret, what);
            }
            TypeRef::Result(value, error) => {
                self.written(value, what);
                if let Some(e) = error {
                    self.written(e, what);
                }
            }
            TypeRef::String
            | TypeRef::Number
            | TypeRef::Bool
            | TypeRef::Named(_)
            | TypeRef::Unit => {}
        }
    }

    /// A parameter's type: none written is a hole of its own; `Map` or
    /// `Any` inside one that is written are the written kinds.
    pub(super) fn written_param(&mut self, ty: &TypeRef, what: &str) {
        if matches!(ty, TypeRef::Any) {
            self.hole(HoleKind::UntypedParameter, what);
        } else {
            self.written(ty, what);
        }
    }

    /// Whether a value of `given` reaches `wanted` only by a loose rule,
    /// recorded as a hole when it does. Called once a value is known to
    /// fit.
    pub(super) fn loosely(&mut self, given: &Type, wanted: &Type, what: &str) {
        let kind = match (given.unwrapped(), wanted.unwrapped()) {
            (Type::Error, _) | (_, Type::Error | Type::Any) => return,
            (Type::Any, _) => HoleKind::AnyAccepted,
            (Type::Map | Type::Shape(_), Type::Record(_))
            | (Type::Map | Type::Record(_), Type::Shape(_))
            | (Type::Record(_), Type::Map) => HoleKind::LooseRecord,
            (Type::Scalar(_), Type::String | Type::Number)
            | (Type::Number, Type::Scalar(super::Scalar::Duration)) => HoleKind::Conversion,
            _ => return,
        };
        self.hole(kind, what);
    }
}

/// The `test` declarations, which the checker passes over: each is a hole
/// of its own, at its name.
pub(super) fn unchecked_tests(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<Hole> {
    program
        .declarations
        .iter()
        .enumerate()
        .filter_map(|(i, d)| match d {
            Declaration::Test(t) => Some(Hole {
                file: file_of(i),
                line: t.span.line as usize,
                column: t.span.col as usize,
                kind: HoleKind::UncheckedTest,
                what: t.name.clone(),
            }),
            _ => None,
        })
        .collect()
}

/// Whether any statement in `stmts` is an action's or handler's `return`
/// of a value — what makes an action's result something a caller reads.
pub(super) fn returns_value(stmts: &[Statement]) -> bool {
    stmts.iter().any(|s| match &s.kind {
        StatementKind::Return(Some(_)) => true,
        StatementKind::If(i) => {
            returns_value(&i.then_body)
                || i.else_if_branches.iter().any(|(_, b)| returns_value(b))
                || i.else_body.as_deref().is_some_and(returns_value)
        }
        StatementKind::For(f) => returns_value(&f.body),
        StatementKind::Try(t) => returns_value(&t.body) || returns_value(&t.catch_body),
        _ => false,
    })
}

/// Whether `ty` is unknown anywhere in it.
fn has_any(ty: &Type) -> bool {
    match ty {
        Type::Any | Type::Error => true,
        Type::List(t) | Type::Optional(t) | Type::Promise(t) | Type::NetError(t) => has_any(t),
        Type::Dict(k, v) | Type::Result(k, v) => has_any(k) || has_any(v),
        Type::Resource(a, b) => has_any(a) || has_any(b),
        Type::Func(params, ret) => params.iter().any(has_any) || has_any(ret),
        Type::Shape(fields) => fields.iter().any(|(_, t)| has_any(t)),
        _ => false,
    }
}

/// Every hole in the project at `project_dir`, in the order of their files
/// and lines.
///
/// # Errors
///
/// What reading the project fails with: a missing `src/`, a file that
/// cannot be read or does not parse.
pub fn of_project(project_dir: &std::path::Path) -> crate::diagnostics::error::Result<Vec<Hole>> {
    let (program, files) = crate::project::sources::read_project(project_dir)?;
    let mut texts: std::collections::HashMap<&str, Option<String>> =
        std::collections::HashMap::new();
    for file in &files {
        texts
            .entry(file.as_str())
            .or_insert_with(|| std::fs::read_to_string(project_dir.join(file)).ok());
    }
    let file_of = |i: usize| files.get(i).cloned().unwrap_or_default();
    let source_of = |i: usize| {
        files
            .get(i)
            .and_then(|f| texts.get(f.as_str()))
            .and_then(|t| t.as_deref())
    };
    Ok(super::check_sources(&program, &file_of, &source_of).holes)
}

#[cfg(test)]
mod tests {
    use super::HoleKind;

    /// Each hole the checker finds in `src`, as its kind and what it names.
    fn holes(src: &str) -> Vec<(HoleKind, String)> {
        let program = crate::syntax::parse_source(src, "t.wf").expect("parses");
        super::super::check(&program, &|_| "t.wf".to_string())
            .holes
            .into_iter()
            .map(|h| (h.kind, h.what))
            .collect()
    }

    fn of(src: &str, kind: HoleKind) -> Vec<String> {
        holes(src)
            .into_iter()
            .filter(|(k, _)| *k == kind)
            .map(|(_, w)| w)
            .collect()
    }

    #[test]
    fn every_kind_has_a_name_of_its_own() {
        let mut names: Vec<&str> = HoleKind::ALL.iter().map(|k| k.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), HoleKind::ALL.len());
        assert!(HoleKind::ALL.iter().all(|k| !k.describe().is_empty()));
    }

    #[test]
    fn what_a_program_writes_loosely_is_counted_where_it_is_written() {
        let src = r#"
            component Card2(data: Map, rows: [Any]) { Text("a") }
            store S {
                state items: [String] = []
                action add(item) { items = items.concat([item]) }
            }
        "#;
        assert_eq!(of(src, HoleKind::WrittenMap), ["data"]);
        assert_eq!(of(src, HoleKind::WrittenAny), ["rows"]);
        assert_eq!(of(src, HoleKind::UntypedParameter), ["add(item)"]);
    }

    #[test]
    fn an_unknown_value_is_counted_where_it_is_made_and_not_where_it_goes() {
        let src = r#"
            page P(path: "/") {
                state r: Map = { a: 1 }
                derived deep = r.a.b.c
                Text("x")
            }
        "#;
        assert_eq!(of(src, HoleKind::MapField), ["r.a"]);
    }

    #[test]
    fn a_value_an_error_is_reported_about_is_no_hole() {
        let src = r#"
            type U { name: String }
            page P(path: "/") {
                state u = U(name: "a")
                derived x = u.nmae.first.second
                Text("x")
            }
        "#;
        assert!(of(src, HoleKind::MapField).is_empty(), "{:?}", holes(src));
    }

    #[test]
    fn two_branches_that_disagree_are_a_hole_where_they_meet() {
        let src = r#"
            page P(path: "/") {
                state ok = true
                derived x = if ok { 1 } else { "none" }
                derived y = if ok { 1 } else { 2 }
                Text("x")
            }
        "#;
        assert_eq!(of(src, HoleKind::LooseJoin), ["ok"]);
    }

    #[test]
    fn what_comes_from_outside_is_counted() {
        let src = r#"
            store S {
                state picked = null
                state text = ""
                action load() {
                    try { text = JSON.stringify(1) } catch e { text = e.message }
                }
            }
            test "it loads" { Text("a")  expect "a" }
        "#;
        assert_eq!(of(src, HoleKind::NullState), ["picked"]);
        assert_eq!(of(src, HoleKind::CaughtError), ["e"]);
        assert_eq!(of(src, HoleKind::BrowserGlobal), ["JSON"]);
        assert_eq!(of(src, HoleKind::UncheckedTest), ["it loads"]);
    }

    #[test]
    fn a_place_inferred_twice_is_one_hole() {
        let src = r#"
            page P(path: "/") {
                derived x = [].length
                Text("{x}")
            }
        "#;
        assert_eq!(of(src, HoleKind::EmptyList), ["[]"]);
    }
}
