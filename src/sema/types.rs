//! The type checker: what every name is, inferred from what it is declared
//! as or given, and the mistakes that follow from mixing them up.
//!
//! Types come from declarations — `state x: T`, props, params, `type`
//! fields, `enum` cases, route parameters, `resource x: [T]` — and from
//! values: literals, records, lists, the built-in methods of strings,
//! numbers and lists, the members of stores. Anything that cannot be
//! resolved is `Any`, and `Any` agrees with everything, so a program that
//! declares no types checks as it always did; every annotation narrows
//! what the checker can say.
//!
//! The checks report what is certainly wrong, each with a hint: a value
//! of the wrong type given to a prop, a state, a field or a parameter; a
//! case an enum does not have; a field a record does not have; a member a
//! store does not have; a value that may be `null` read as if it were
//! not; a call with the wrong number of arguments; an `emit` that does
//! not match its event; a `bind:` on a control of the wrong type; a `for`
//! over something that is not a list; a `match` on something with no
//! arms to match; a list used as a condition.

use std::collections::HashMap;

use super::Findings;
use crate::error::Diagnostic;
use crate::parser::ast::*;
use crate::registry::{self, PropType};

/// A type the checker reasons about.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Unknown, or anything: agrees with every type.
    Any,
    String,
    Number,
    Bool,
    Null,
    /// A design token, `$name`.
    Token,
    /// A regular expression, `/…/`.
    Regex,
    List(Box<Type>),
    /// A map with unknown keys.
    Map,
    /// A map literal's own shape: the fields it was written with. Reads of
    /// a field it lacks are faults; extra fields at an assignment are not.
    Shape(Vec<(String, Type)>),
    /// A record of a declared `type`.
    Record(String),
    /// A value of a declared `enum`.
    Enum(String),
    /// A case whose enum is not yet known: `.danger` on its own.
    Case(String),
    /// A value that may be `null`.
    Optional(Box<Type>),
    /// An action, a lambda or a store's action.
    Func(Vec<Type>, Box<Type>),
    /// A `resource`, matched with `loading`, `error`, `ready`.
    Resource(Box<Type>),
    /// A store brought into scope with `use`.
    Store(String),
    /// A type the language knows and the program did not declare: a date,
    /// a duration, money, a URL. Each is a plain JSON value at run time.
    Scalar(Scalar),
    /// A service declared with `api`.
    Api(String),
    /// What a request can fail with: `.offline`, `.timeout`, `.aborted`,
    /// `.parse`, `.network`, `.status(code, body)` — matched like an enum,
    /// and read like a record.
    NetError,
}

/// The types the language brings with it.
///
/// Each is carried by a plain JSON value — a string, a number, a small map
/// — so it crosses `fetch`, `persist`, the static paint and the template
/// engine unchanged, and a library that wants the platform's own object
/// gets one from `.native()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scalar {
    /// `"2026-03-14"`.
    Date,
    /// `"09:30"`, or `"09:30:15"`.
    Time,
    /// ISO 8601: `"2026-03-14T09:30:00Z"`.
    DateTime,
    /// Milliseconds.
    Duration,
    /// `{ amount: 1299, currency: "EUR" }` — minor units, so no float drift.
    Money,
    /// An absolute URL.
    Url,
    /// An address.
    Email,
    /// `"#0F766E"`, or any CSS colour.
    Color,
    /// `"3f2b…"` — the canonical 8-4-4-4-12 spelling.
    Uuid,
    /// What a `FileUpload` yields: the browser's own `File`.
    File,
    /// A string that must not be persisted, logged, put in a URL or shown.
    Secret,
}

impl Scalar {
    /// The name it is written with.
    pub fn name(self) -> &'static str {
        match self {
            Scalar::Date => "Date",
            Scalar::Time => "Time",
            Scalar::DateTime => "DateTime",
            Scalar::Duration => "Duration",
            Scalar::Money => "Money",
            Scalar::Url => "Url",
            Scalar::Email => "Email",
            Scalar::Color => "Color",
            Scalar::Uuid => "Uuid",
            Scalar::File => "File",
            Scalar::Secret => "Secret",
        }
    }

    /// The type that name means, when the program declares no type of its own.
    pub fn of_name(name: &str) -> Option<Scalar> {
        Some(match name {
            "Date" => Scalar::Date,
            "Time" => Scalar::Time,
            "DateTime" => Scalar::DateTime,
            "Duration" => Scalar::Duration,
            "Money" => Scalar::Money,
            "Url" => Scalar::Url,
            "Email" => Scalar::Email,
            "Color" => Scalar::Color,
            "Uuid" => Scalar::Uuid,
            "File" => Scalar::File,
            "Secret" => Scalar::Secret,
            _ => return None,
        })
    }

    /// Whether a string carries it, so a literal may be read as one and the
    /// value shown wherever text is shown.
    pub fn is_text(self) -> bool {
        matches!(
            self,
            Scalar::Date
                | Scalar::Time
                | Scalar::DateTime
                | Scalar::Url
                | Scalar::Email
                | Scalar::Color
                | Scalar::Uuid
                | Scalar::Secret
        )
    }
}

impl Type {
    pub fn from_ref(ty: &TypeRef) -> Type {
        match ty {
            TypeRef::String => Type::String,
            TypeRef::Number => Type::Number,
            TypeRef::Bool => Type::Bool,
            TypeRef::Map => Type::Map,
            TypeRef::Any => Type::Any,
            TypeRef::List(inner) => Type::List(Box::new(Type::from_ref(inner))),
            TypeRef::Optional(inner) => Type::Optional(Box::new(Type::from_ref(inner))),
            TypeRef::Named(name) => Type::Record(name.clone()),
            // A condition narrows the values, not the type.
            TypeRef::Refined(inner, _) => Type::from_ref(inner),
        }
    }

    fn list(inner: Type) -> Type {
        Type::List(Box::new(inner))
    }

    fn optional(inner: Type) -> Type {
        match inner {
            Type::Optional(_) | Type::Null => inner,
            other => Type::Optional(Box::new(other)),
        }
    }

    /// The type without its `null`.
    fn unwrapped(&self) -> Type {
        match self {
            Type::Optional(inner) => (**inner).clone(),
            other => other.clone(),
        }
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any)
    }

    /// Whether a value of `self` may be given where `to` is expected.
    fn assignable_to(&self, to: &Type) -> bool {
        match (self, to) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Case(_), Type::Enum(_)) => true,
            (Type::Null, Type::Optional(_)) => true,
            (Type::Null, Type::Null) => true,
            (Type::Optional(a), Type::Optional(b)) => a.assignable_to(b),
            (from, Type::Optional(b)) => from.assignable_to(b),
            (Type::List(a), Type::List(b)) => a.assignable_to(b),
            (Type::Map, Type::Record(_)) | (Type::Record(_), Type::Map) => true,
            (Type::Shape(_), Type::Map | Type::Record(_))
            | (Type::Map | Type::Record(_), Type::Shape(_)) => true,
            // A shape fits another when the fields they share agree.
            (Type::Shape(from), Type::Shape(to)) => from.iter().all(|(name, ty)| {
                to.iter()
                    .find(|(n, _)| n == name)
                    .is_none_or(|(_, wanted)| ty.assignable_to(wanted))
            }),
            (Type::Func(..), Type::Func(..)) => true,
            // A scalar is a plain value at run time, so it shows wherever
            // text shows — except a `Secret`, which must not be shown at
            // all, and `Duration`, which is a number of milliseconds.
            (Type::Scalar(s), Type::String) => s.is_text() && *s != Scalar::Secret,
            (Type::Scalar(Scalar::Duration), Type::Number) => true,
            (Type::Number, Type::Scalar(Scalar::Duration)) => true,
            (Type::Resource(a), Type::Resource(b)) => a.assignable_to(b),
            (a, b) => a == b,
        }
    }

    /// The type both `a` and `b` fit: their common type, or `Any`.
    fn join(a: Type, b: Type) -> Type {
        match (a, b) {
            (a, b) if a == b => a,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Null, other) | (other, Type::Null) => Type::optional(other),
            (Type::Optional(a), b) | (b, Type::Optional(a)) => Type::optional(Type::join(*a, b)),
            (Type::List(a), Type::List(b)) => Type::list(Type::join(*a, *b)),
            // Two shapes join to every field either has; a field only one
            // of them has may be missing.
            (Type::Shape(a), Type::Shape(b)) => {
                let mut fields: Vec<(String, Type)> = Vec::new();
                for (name, ty) in a.iter().chain(b.iter()) {
                    if fields.iter().any(|(n, _)| n == name) {
                        continue;
                    }
                    let in_a = a.iter().find(|(n, _)| n == name).map(|(_, t)| t);
                    let in_b = b.iter().find(|(n, _)| n == name).map(|(_, t)| t);
                    let joined = match (in_a, in_b) {
                        (Some(x), Some(y)) => Type::join(x.clone(), y.clone()),
                        _ => Type::optional(ty.clone()),
                    };
                    fields.push((name.clone(), joined));
                }
                Type::Shape(fields)
            }
            (Type::Shape(_), Type::Map | Type::Record(_))
            | (Type::Map | Type::Record(_), Type::Shape(_)) => Type::Map,
            (Type::Case(_), Type::Enum(e)) | (Type::Enum(e), Type::Case(_)) => Type::Enum(e),
            (Type::Case(_), Type::Case(_)) => Type::Any,
            _ => Type::Any,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Any => write!(f, "Any"),
            Type::String => write!(f, "String"),
            Type::Number => write!(f, "Number"),
            Type::Bool => write!(f, "Bool"),
            Type::Null => write!(f, "null"),
            Type::Token => write!(f, "$token"),
            Type::Regex => write!(f, "Regex"),
            Type::List(inner) => write!(f, "[{inner}]"),
            Type::Map => write!(f, "Map"),
            Type::Shape(fields) => {
                let inner: Vec<String> = fields.iter().map(|(n, t)| format!("{n}: {t}")).collect();
                write!(f, "{{ {} }}", inner.join(", "))
            }
            Type::Record(name) | Type::Enum(name) | Type::Store(name) => write!(f, "{name}"),
            Type::Case(name) => write!(f, ".{name}"),
            Type::Optional(inner) => write!(f, "{inner}?"),
            Type::Func(params, ret) => {
                let params: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                if **ret == Type::Null {
                    write!(f, "action({})", params.join(", "))
                } else {
                    write!(f, "action({}) -> {ret}", params.join(", "))
                }
            }
            Type::Resource(inner) => write!(f, "resource<{inner}>"),
            Type::Scalar(s) => write!(f, "{}", s.name()),
            Type::Api(name) => write!(f, "{name}"),
            Type::NetError => write!(f, "NetworkError"),
        }
    }
}

/// What the program declares, resolved once.
struct World<'p> {
    types: HashMap<&'p str, &'p TypeDecl>,
    enums: HashMap<&'p str, &'p EnumDecl>,
    components: HashMap<&'p str, &'p ComponentDecl>,
    /// Each store's members, typed.
    stores: HashMap<&'p str, HashMap<String, Type>>,
    /// The program's constants, typed.
    consts: HashMap<String, Type>,
    /// The services the program declares.
    apis: HashMap<&'p str, &'p ApiDecl>,
}

impl<'p> World<'p> {
    /// One endpoint of a service, by name.
    fn endpoint(&self, api: &str, name: &str) -> Option<&'p Endpoint> {
        self.apis
            .get(api)?
            .endpoints
            .iter()
            .find(|e| e.name == name)
    }

    fn record_field(&self, record: &str, field: &str) -> Option<Type> {
        self.record_fields(record)?
            .into_iter()
            .find(|f| f.name == field)
            .map(|f| Type::from_ref(&f.ty))
    }

    /// Every field of a record, the extended record's included.
    fn record_fields(&self, record: &str) -> Option<Vec<&'p FieldDecl>> {
        let decl = self.types.get(record)?;
        Some(decl.all_fields(&|name| self.types.get(name).copied()))
    }

    /// A `TypeRef::Named` may name an enum rather than a record.
    fn resolve(&self, ty: Type) -> Type {
        match ty {
            Type::Record(name) if self.enums.contains_key(name.as_str()) => Type::Enum(name),
            // A name the language knows, unless the program declares a type
            // of its own by that name — which wins, so nothing the language
            // adds can take a name away.
            Type::Record(name)
                if !self.types.contains_key(name.as_str()) && Scalar::of_name(&name).is_some() =>
            {
                Type::Scalar(Scalar::of_name(&name).expect("just checked"))
            }
            Type::List(inner) => Type::list(self.resolve(*inner)),
            Type::Optional(inner) => Type::optional(self.resolve(*inner)),
            other => other,
        }
    }
}

/// A name's type, recorded where it is declared, for the editor.
#[derive(Debug, Clone)]
pub struct Typed {
    /// The index of the declaration it belongs to.
    pub decl: usize,
    pub name: String,
    /// The span of the statement, prop or parameter that declares it.
    pub span: Span,
    pub ty: Type,
}

/// The types of every name declared in the program, in the order they
/// were met, beside the findings.
#[derive(Default)]
pub struct TypeInfo {
    pub findings: Findings,
    pub bindings: Vec<Typed>,
}

impl TypeInfo {
    /// The type of the name declared by the binding at `span` in `decl`.
    pub fn type_at(&self, decl: usize, name: &str, span: Span) -> Option<&Type> {
        self.bindings
            .iter()
            .find(|t| t.decl == decl && t.name == name && t.span.start == span.start)
            .map(|t| &t.ty)
    }

    /// The type of a store's member.
    pub fn member(&self, store_decl: usize, name: &str) -> Option<&Type> {
        self.bindings
            .iter()
            .find(|t| t.decl == store_decl && t.name == name)
            .map(|t| &t.ty)
    }
}

/// Check every declaration of the program.
pub fn check(program: &Program, file_of: &dyn Fn(usize) -> String) -> TypeInfo {
    check_in(program, file_of, &|_| None)
}

/// [`check`], with the source text of each declaration's file at hand:
/// an error is then placed at the expression it names inside its
/// statement, not at the statement's start.
pub fn check_in(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    source_of: &dyn Fn(usize) -> Option<String>,
) -> TypeInfo {
    let mut world = World {
        types: HashMap::new(),
        enums: HashMap::new(),
        components: HashMap::new(),
        stores: HashMap::new(),
        consts: HashMap::new(),
        apis: HashMap::new(),
    };
    for decl in &program.declarations {
        match decl {
            Declaration::Type(t) => {
                world.types.insert(t.name.as_str(), t);
            }
            Declaration::Enum(e) => {
                world.enums.insert(e.name.as_str(), e);
            }
            Declaration::Api(a) => {
                world.apis.insert(a.name.as_str(), a);
            }
            Declaration::Component(c) => {
                world.components.insert(c.name.as_str(), c);
            }
            _ => {}
        }
    }
    let mut info = TypeInfo::default();

    // Constants first: plain values, read everywhere. A `data` declaration
    // the build has not resolved yet is what it declares, or `Any`.
    for (index, decl) in program.declarations.iter().enumerate() {
        if let Declaration::Data(d) = decl {
            let ty =
                d.ty.as_ref()
                    .map(|t| world.resolve(Type::from_ref(t)))
                    .unwrap_or(Type::Any);
            info.bindings.push(Typed {
                decl: index,
                name: d.name.clone(),
                span: d.span,
                ty: ty.clone(),
            });
            world.consts.insert(d.name.clone(), ty);
        }
        if let Declaration::Const(c) = decl {
            let file = file_of(index);
            let source = source_of(index);
            let mut cx = Checker::new(&world, &file, index, &mut info, source);
            cx.current_span = c.span;
            let declared = c.ty.as_ref().map(|t| cx.world.resolve(Type::from_ref(t)));
            let given = cx.infer(&c.value, declared.as_ref());
            if let Some(declared) = &declared {
                let what = format!("`const {}`", c.name);
                cx.expect(&given, declared, c.span, &what);
            }
            let ty = declared.unwrap_or(given);
            cx.info.bindings.push(Typed {
                decl: index,
                name: c.name.clone(),
                span: c.span,
                ty: ty.clone(),
            });
            world.consts.insert(c.name.clone(), ty);
        }
    }

    // Stores next: their members are read everywhere else. A store's
    // derived values and actions may read other stores, which are `Any`
    // until met.
    for (index, decl) in program.declarations.iter().enumerate() {
        if let Declaration::Store(s) = decl {
            let file = file_of(index);
            let source = source_of(index);
            let mut cx = Checker::new(&world, &file, index, &mut info, source);
            cx.push_scope();
            cx.declare_hoisted(&s.body);
            let members = cx.scopes.last().cloned().unwrap_or_default();
            cx.statements(&s.body, Body::Store);
            cx.pop_scope();
            world.stores.insert(s.name.as_str(), members);
        }
    }

    for (index, decl) in program.declarations.iter().enumerate() {
        let file = file_of(index);
        let source = source_of(index);
        let mut cx = Checker::new(&world, &file, index, &mut info, source);
        match decl {
            Declaration::Page(p) => {
                cx.push_scope();
                for param in &p.params {
                    let ty = cx.world.resolve(Type::from_ref(&param.prop_type));
                    cx.bind(&param.name, ty, param.span);
                }
                if let Some(layout) = &p.layout {
                    cx.current_span = layout.span;
                    cx.layout(layout);
                }
                cx.declare_hoisted(&p.body);
                cx.statements(&p.body, Body::Page);
                cx.pop_scope();
            }
            Declaration::Component(c) => {
                cx.push_scope();
                for prop in &c.props {
                    let mut ty = cx.world.resolve(Type::from_ref(&prop.prop_type));
                    if prop.optional {
                        ty = Type::optional(ty);
                    }
                    if let Some(default) = &prop.default {
                        cx.current_span = prop.span;
                        let given = cx.infer(default, Some(&ty));
                        cx.expect(
                            &given,
                            &ty,
                            prop.span,
                            &format!("the default of `{}`", prop.name),
                        );
                    }
                    cx.bind(&prop.name, ty, prop.span);
                }
                cx.component = Some(c);
                cx.declare_hoisted(&c.body);
                cx.statements(&c.body, Body::Component);
                cx.pop_scope();
            }
            Declaration::App(a) => {
                cx.push_scope();
                cx.declare_hoisted(&a.body);
                cx.statements(&a.body, Body::Page);
                cx.pop_scope();
            }
            Declaration::Type(t) => {
                for field in &t.fields {
                    if let Some(default) = &field.default {
                        cx.current_span = field.span;
                        let ty = cx.world.resolve(Type::from_ref(&field.ty));
                        let given = cx.infer(default, Some(&ty));
                        cx.expect(
                            &given,
                            &ty,
                            field.span,
                            &format!("the default of `{}`", field.name),
                        );
                    }
                }
            }
            // A service's own code runs like any other: the settings and
            // headers are expressions, and a hook is a body. The hook's
            // parameter is the request, the reply or the error, whose shape
            // the runtime gives it, so it is `Any`.
            Declaration::Api(a) => {
                for (_, e) in a.settings.iter().chain(a.headers.iter()) {
                    cx.current_span = a.span;
                    cx.infer(e, None);
                }
                for endpoint in &a.endpoints {
                    for (_, e) in &endpoint.settings {
                        cx.current_span = endpoint.span;
                        cx.infer(e, None);
                    }
                }
                for hook in &a.hooks {
                    cx.push_scope();
                    if let Some(param) = &hook.param {
                        cx.bind(param, Type::Any, hook.span);
                    }
                    cx.returns.push(Vec::new());
                    cx.statements(&hook.body, Body::Imperative);
                    cx.returns.pop();
                    cx.pop_scope();
                }
            }
            Declaration::Store(_)
            | Declaration::Theme(_)
            | Declaration::Enum(_)
            | Declaration::External(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => {}
        }
    }
    info
}

/// The kind of body being checked, for what a statement may be.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    Page,
    Component,
    Store,
    /// An action, handler or effect.
    Imperative,
}

struct Checker<'a, 'p> {
    world: &'a World<'p>,
    file: &'a str,
    decl: usize,
    info: &'a mut TypeInfo,
    scopes: Vec<HashMap<String, Type>>,
    /// The component being checked, for `emit`.
    component: Option<&'p ComponentDecl>,
    /// The `return` types met in the action being checked.
    returns: Vec<Vec<Type>>,
    /// The span of the statement or argument being checked: where an
    /// expression's error lands, since expressions carry no span.
    current_span: Span,
    /// The file's text, when the caller has it: where an error is placed
    /// at the expression it names.
    source: Option<String>,
    /// Whether an element's arguments are being checked, where the
    /// resolver already reports a case the prop's enum lacks.
    in_args: bool,
}

impl<'a, 'p> Checker<'a, 'p> {
    fn new(
        world: &'a World<'p>,
        file: &'a str,
        decl: usize,
        info: &'a mut TypeInfo,
        source: Option<String>,
    ) -> Self {
        Self {
            world,
            file,
            decl,
            info,
            scopes: Vec::new(),
            component: None,
            returns: Vec::new(),
            current_span: Span::dummy(),
            in_args: false,
            source,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind(&mut self, name: &str, ty: Type, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty.clone());
        }
        self.info.bindings.push(Typed {
            decl: self.decl,
            name: name.to_string(),
            span,
            ty,
        });
    }

    /// Narrow a name within the current scope, without recording a binding.
    fn narrow(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }

    fn error(&mut self, span: Span, code: &str, message: String, hint: &str) {
        let d = Diagnostic::new(
            format!("[{code}] {message}"),
            self.file,
            span.line as usize,
            span.col as usize,
        );
        self.info.findings.errors.push(if hint.is_empty() {
            d
        } else {
            d.with_hint(hint)
        });
    }

    /// Report `given` where `wanted` was expected, at `span`, naming `what`.
    fn expect(&mut self, given: &Type, wanted: &Type, span: Span, what: &str) {
        if given.assignable_to(wanted) {
            return;
        }
        let hint = match (given, wanted) {
            (Type::Optional(_), _) => {
                "Unwrap it first: `if let x = value { … }`, `value ?? fallback`, or a check for `!= null`".to_string()
            }
            (Type::String, Type::Number) => "Convert it: `Number(value)`".to_string(),
            (Type::Number, Type::String) => "Convert it: `String(value)` or `\"{value}\"`".to_string(),
            (_, Type::Enum(e)) => match self.world.enums.get(e.as_str()) {
                Some(decl) => format!(
                    "`{e}` takes {}",
                    decl.case_names()
                        .iter()
                        .map(|c| format!(".{c}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                None => String::new(),
            },
            _ => String::new(),
        };
        self.error(
            span,
            "T01",
            format!("{what} is `{given}`, but `{wanted}` is wanted"),
            &hint,
        );
    }

    // ─── Declarations first ──────────────────────────────

    /// Bind the names a body declares before checking it, as the compiler
    /// resolves them regardless of order: actions may call each other, a
    /// derived value may read a later state.
    fn declare_hoisted(&mut self, stmts: &[Statement]) {
        // Two passes: state and resources first (their types are declared
        // or read off their values), then derived values and actions, whose
        // types may depend on them.
        for stmt in stmts {
            self.current_span = stmt.span;
            match &stmt.kind {
                StatementKind::State(s) => {
                    let ty = match &s.ty {
                        Some(t) => self.world.resolve(Type::from_ref(t)),
                        // A state that starts as `null` will hold something
                        // later: a value that may be null.
                        None => match self.infer(&s.value, None) {
                            Type::Null => Type::optional(Type::Any),
                            other => other,
                        },
                    };
                    self.bind(&s.name, ty, stmt.span);
                }
                StatementKind::Resource(r) => {
                    let inner = match &r.ty {
                        Some(t) => self.world.resolve(Type::from_ref(t)),
                        // An endpoint says what it returns, so a resource
                        // over one is typed without an annotation.
                        None => match &r.url {
                            Expr::MethodCall(obj, method, _) => {
                                match (&**obj, self.world.apis.is_empty()) {
                                    (Expr::Identifier(api), false) => self
                                        .world
                                        .endpoint(api, method)
                                        .and_then(|e| e.returns.as_ref())
                                        .map(|t| self.world.resolve(Type::from_ref(t)))
                                        .unwrap_or(Type::Any),
                                    _ => Type::Any,
                                }
                            }
                            _ => Type::Any,
                        },
                    };
                    self.bind(&r.name, Type::Resource(Box::new(inner)), stmt.span);
                }
                StatementKind::Use(u) => {
                    self.bind(&u.store_name, Type::Store(u.store_name.clone()), stmt.span);
                }
                _ => {}
            }
            // `ref: name` anywhere in the body declares `name`: a handle on
            // the element, read like the element itself.
            for name in ref_names(std::slice::from_ref(stmt)) {
                self.bind(&name, Type::Any, stmt.span);
            }
            // `Form(bind: form)` declares `form`: its validity and values.
            for name in form_names(std::slice::from_ref(stmt)) {
                self.bind(&name, form_type(), stmt.span);
            }
        }
        for stmt in stmts {
            self.current_span = stmt.span;
            match &stmt.kind {
                StatementKind::Action(a) => {
                    let params: Vec<Type> = a
                        .params
                        .iter()
                        .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                        .collect();
                    // The return type is found when the body is checked.
                    self.bind(&a.name, Type::Func(params, Box::new(Type::Any)), stmt.span);
                }
                StatementKind::Derived(d) => {
                    let ty = self.infer(&d.value, None);
                    self.bind(&d.name, ty, stmt.span);
                }
                _ => {}
            }
        }
    }

    // ─── Statements ──────────────────────────────────────

    fn statements(&mut self, stmts: &[Statement], body: Body) {
        for stmt in stmts {
            self.statement(stmt, body);
        }
    }

    fn statement(&mut self, stmt: &Statement, body: Body) {
        let span = stmt.span;
        self.current_span = span;
        match &stmt.kind {
            StatementKind::State(s) => {
                let declared = s.ty.as_ref().map(|t| self.world.resolve(Type::from_ref(t)));
                let given = self.infer(&s.value, declared.as_ref());
                if let Some(declared) = &declared {
                    self.expect(&given, declared, span, &format!("`{}`", s.name));
                }
                // A secret must not outlive the visit.
                if s.persist
                    && declared.as_ref().map(|t| t.unwrapped())
                        == Some(Type::Scalar(Scalar::Secret))
                {
                    self.error(
                        span,
                        "T12",
                        format!("`{}` is a `Secret`, and `persist` would keep it in the browser", s.name),
                        "Hold it in `state`, which the visit ends; what must outlive it belongs on the server",
                    );
                }
                // And what the type says its values must be.
                if let Some(ty) = &s.ty
                    && let Some(fault) = refinement_fault(ty, &s.value)
                {
                    self.error(
                        span,
                        "T01",
                        format!("`{}` {fault}", s.name),
                        "The type says what its values may be; this one is outside it",
                    );
                }
                if body == Body::Imperative {
                    // A local: bound here, in order.
                    self.bind(&s.name, declared.unwrap_or(given), span);
                }
            }
            StatementKind::Derived(d) => {
                let ty = self.infer(&d.value, None);
                self.narrow(&d.name, ty);
            }
            StatementKind::Resource(r) => {
                // An `api` endpoint is a call, not an address.
                if !matches!(&r.url, Expr::MethodCall(..) | Expr::FunctionCall(..)) {
                    let url = self.infer(&r.url, Some(&Type::String));
                    self.expect(&url, &Type::String, span, "a resource's URL");
                } else {
                    self.infer(&r.url, None);
                }
                for option in &r.options {
                    self.infer(&option.value, None);
                }
            }
            // The rules a value must meet, checked against what it is: a
            // length on a number, or a match against a name that is not
            // there, is a fault where it is written.
            StatementKind::Validate(v) => {
                let guarded = self.lookup(&v.name);
                if guarded.is_none() {
                    self.error(
                        span,
                        "T05",
                        format!("`{}` is not a name this page declares", v.name),
                        "A `validate` block guards a `state` of the page or component it is in",
                    );
                }
                let ty = guarded.unwrap_or(Type::Any).unwrapped();
                for rule in &v.rules {
                    for e in rule.args.iter().chain(rule.message.iter()) {
                        self.infer(e, None);
                    }
                    if let Some(body) = &rule.body {
                        let checked = self.infer(body, Some(&Type::Bool));
                        self.expect(&checked, &Type::Bool, rule.span, "a rule's check");
                    }
                    let Some(wanted) = rule_wants(&rule.name) else {
                        self.error(
                            rule.span,
                            "T05",
                            format!("`{}` is not a rule", rule.name),
                            "The rules are `required`, `email`, `url`, `minLength`, `maxLength`, `min`, `max`, `pattern`, `matches`, `oneOf`, `custom` and `async`",
                        );
                        continue;
                    };
                    if !ty.is_any() && !wanted.iter().any(|w| ty.assignable_to(w)) {
                        let names: Vec<String> = wanted.iter().map(|w| w.to_string()).collect();
                        self.error(
                            rule.span,
                            "T01",
                            format!(
                                "`{}` is `{ty}`, and `{}` is a rule for {}",
                                v.name,
                                rule.name,
                                names.join(" or ")
                            ),
                            "",
                        );
                    }
                }
            }
            StatementKind::Connection(c) => {
                let url = self.infer(&c.url, Some(&Type::String));
                self.expect(&url, &Type::String, span, "a connection's address");
                for (_, value) in &c.options {
                    self.infer(value, None);
                }
                for handler in &c.handlers {
                    self.push_scope();
                    if let Some(param) = &handler.param {
                        let ty = match &c.receives {
                            Some(t) => self.world.resolve(Type::from_ref(t)),
                            None => Type::Any,
                        };
                        self.bind(param, ty, span);
                    }
                    self.statements(&handler.body, Body::Imperative);
                    self.pop_scope();
                }
            }
            StatementKind::Use(_) => {}
            StatementKind::Action(a) => {
                self.push_scope();
                for p in &a.params {
                    let ty = self.world.resolve(Type::from_ref(&p.param_type));
                    self.bind(&p.name, ty, span);
                }
                self.returns.push(Vec::new());
                self.statements(&a.body, Body::Imperative);
                let returned = self.returns.pop().unwrap_or_default();
                self.pop_scope();
                let ret = returned
                    .into_iter()
                    .reduce(Type::join)
                    .unwrap_or(Type::Null);
                let params: Vec<Type> = a
                    .params
                    .iter()
                    .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                    .collect();
                let func = Type::Func(params, Box::new(ret));
                self.narrow(&a.name, func.clone());
                if let Some(t) =
                    self.info.bindings.iter_mut().rev().find(|t| {
                        t.decl == self.decl && t.name == a.name && t.span.start == span.start
                    })
                {
                    t.ty = func;
                }
            }
            StatementKind::Effect(e) => {
                self.push_scope();
                self.statements(&e.body, Body::Imperative);
                self.statements(&e.cleanup, Body::Imperative);
                self.pop_scope();
            }
            StatementKind::Timer(t) => {
                let ty = self.infer(&t.interval, Some(&Type::Number));
                if !ty.assignable_to(&Type::Number) {
                    self.error_at_current(
                        "T01",
                        format!(
                            "`{}` takes milliseconds, but `{}` is `{ty}`",
                            if t.every { "every" } else { "after" },
                            expr_text(&t.interval)
                        ),
                        "",
                    );
                }
                self.push_scope();
                self.statements(&t.body, Body::Imperative);
                self.pop_scope();
            }
            StatementKind::EventHandler(h) => {
                self.push_scope();
                if let Some(param) = &h.param {
                    self.bind(param, Type::Any, h.span);
                }
                self.statements(&h.body, Body::Imperative);
                self.pop_scope();
            }
            StatementKind::UIElement(el) => self.element(el, span),
            StatementKind::If(i) => self.if_statement(i, span, body),
            StatementKind::Try(t) => {
                self.push_scope();
                self.statements(&t.body, body);
                self.pop_scope();
                self.push_scope();
                if let Some(param) = &t.param {
                    self.bind(param, Type::Any, span);
                }
                self.statements(&t.catch_body, body);
                self.pop_scope();
            }
            StatementKind::For(f) => {
                let iterable = self.infer(&f.iterable, None);
                let item = match &iterable {
                    Type::List(inner) => (**inner).clone(),
                    Type::Any | Type::Map => Type::Any,
                    other => {
                        self.error(
                            span,
                            "T08",
                            format!(
                                "`for` loops over a list, but `{}` is `{other}`",
                                expr_text(&f.iterable)
                            ),
                            "Give it a list, or `.split(…)` a string first",
                        );
                        Type::Any
                    }
                };
                self.push_scope();
                self.bind(&f.item, item, span);
                if let Some(index) = &f.index {
                    self.bind(index, Type::Number, span);
                }
                if let Some(key) = &f.key {
                    self.infer(key, None);
                }
                self.statements(&f.body, body);
                self.pop_scope();
            }
            StatementKind::Show(s) => {
                let cond = self.infer(&s.condition, Some(&Type::Bool));
                self.condition(&cond, &s.condition, span, "show");
                self.statements(&s.body, body);
            }
            StatementKind::Match(m) => self.match_statement(m, span, body),
            StatementKind::Assignment(a) => {
                let target = match &a.target {
                    // An undeclared name is a plain variable; the compiler
                    // has always let it be.
                    Expr::Identifier(name) => self.lookup(name).unwrap_or(Type::Any),
                    other => self.infer(other, None),
                };
                let value = self.infer(&a.value, Some(&target));
                let what = format!("`{}`", expr_text(&a.target));
                self.expect(&value, &target, span, &what);
            }
            StatementKind::MethodCall(mc) => {
                self.infer(
                    &Expr::MethodCall(
                        Box::new(mc.object.clone()),
                        mc.method.clone(),
                        mc.args.clone(),
                    ),
                    None,
                );
            }
            StatementKind::ExprStatement(e) => {
                self.infer(e, None);
            }
            StatementKind::Navigate(e) => {
                let ty = self.infer(e, Some(&Type::String));
                self.expect(&ty, &Type::String, span, "`navigate`'s path");
            }
            StatementKind::Log(e) => {
                let ty = self.infer(e, None);
                if ty.unwrapped() == Type::Scalar(Scalar::Secret) {
                    self.error(
                        span,
                        "T12",
                        format!("`{}` is a `Secret`, and this would log it", expr_text(e)),
                        "A log line is read by whoever can open the console, and often shipped to a service",
                    );
                }
            }
            StatementKind::Return(e) => {
                let ty = match e {
                    Some(e) => self.infer(e, None),
                    None => Type::Null,
                };
                if let Some(returns) = self.returns.last_mut() {
                    returns.push(ty);
                }
            }
            StatementKind::Emit(e) => self.emit(e, span),
            StatementKind::Animate(_) => {}
            StatementKind::Fetch(f) => {
                self.infer(&f.url, Some(&Type::String));
                if let Some(b) = &f.loading_block {
                    self.statements(b, body);
                }
                if let Some((name, b)) = &f.error_block {
                    self.push_scope();
                    self.bind(name, Type::Any, span);
                    self.statements(b, body);
                    self.pop_scope();
                }
                if let Some(b) = &f.success_block {
                    self.push_scope();
                    self.bind(&f.variable, Type::Any, span);
                    self.statements(b, body);
                    self.pop_scope();
                }
            }
        }
    }

    /// A condition: a `Bool`, or anything JavaScript reads as one — a
    /// number, a string, a value that may be null — but not a list or a
    /// record, which are always true.
    fn condition(&mut self, ty: &Type, expr: &Expr, span: Span, keyword: &str) {
        match ty {
            Type::List(_) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but a list is always true",
                    expr_text(expr)
                ),
                "Ask about its length: `items.length > 0`",
            ),
            Type::Record(name) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but a `{name}` is always true",
                    expr_text(expr)
                ),
                "Compare one of its fields",
            ),
            Type::Func(..) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but an action is always true",
                    expr_text(expr)
                ),
                "Call it: `{}()`",
            ),
            _ => {}
        }
    }

    fn if_statement(&mut self, i: &IfStmt, span: Span, body: Body) {
        self.push_scope();
        match &i.binding {
            Some(name) => {
                let ty = self.infer(&i.condition, None);
                self.bind(name, ty.unwrapped(), span);
            }
            None => {
                let ty = self.infer(&i.condition, Some(&Type::Bool));
                self.condition(&ty, &i.condition, span, "if");
                // `if x != null { }`, `if x { }`: `x` is not null inside.
                for name in narrowed_names(&i.condition) {
                    if let Some(Type::Optional(inner)) = self.lookup(&name) {
                        self.narrow(&name, *inner);
                    }
                }
            }
        }
        self.statements(&i.then_body, body);
        self.pop_scope();
        for (cond, branch) in &i.else_if_branches {
            self.push_scope();
            let ty = self.infer(cond, Some(&Type::Bool));
            self.condition(&ty, cond, span, "else if");
            for name in narrowed_names(cond) {
                if let Some(Type::Optional(inner)) = self.lookup(&name) {
                    self.narrow(&name, *inner);
                }
            }
            self.statements(branch, body);
            self.pop_scope();
        }
        if let Some(b) = &i.else_body {
            self.push_scope();
            self.statements(b, body);
            self.pop_scope();
        }
    }

    fn match_statement(&mut self, m: &MatchStmt, span: Span, body: Body) {
        let scrutinee = self.infer(&m.scrutinee, None);
        match &scrutinee {
            Type::Resource(inner) => {
                for arm in &m.arms {
                    self.push_scope();
                    match (&arm.pattern, &arm.binding) {
                        (ArmPattern::Ready, Some(name)) => {
                            self.bind(name, (**inner).clone(), arm.span)
                        }
                        (ArmPattern::Error, Some(name)) => {
                            self.bind(name, Type::NetError, arm.span)
                        }
                        (ArmPattern::Case(case), _) => self.error(
                            arm.span,
                            "T11",
                            format!(
                                "`{}` is a resource; `.{case}` is not one of its states",
                                expr_text(&m.scrutinee)
                            ),
                            "A resource matches `loading`, `error(e)`, `ready(value)` and `else`",
                        ),
                        _ => {}
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            Type::Enum(name) => {
                let cases: Vec<String> = self
                    .world
                    .enums
                    .get(name.as_str())
                    .map(|e| e.case_names())
                    .unwrap_or_default();
                for arm in &m.arms {
                    match &arm.pattern {
                        ArmPattern::Case(case) if !cases.contains(case) => self.error(
                            arm.span,
                            "T02",
                            format!("`{name}` has no case `.{case}`"),
                            &format!(
                                "`{name}` takes {}",
                                cases.iter().map(|c| format!(".{c}")).collect::<Vec<_>>().join(", ")
                            ),
                        ),
                        // `.failed(r)` binds the payload, one name per part.
                        ArmPattern::Case(case) => {
                            let fields = self.payload_of(name, case).unwrap_or_default();
                            if !arm.bindings.is_empty() && arm.bindings.len() != fields.len() {
                                let hint = if fields.is_empty() {
                                    format!("`.{case}` carries nothing: write `.{case} {{ … }}`")
                                } else {
                                    format!("Bind every part: `{}`", case_signature(case, &fields))
                                };
                                self.error(
                                    arm.span,
                                    "T10",
                                    format!(
                                        "`.{case}` carries {} value{}, but {} {} bound",
                                        fields.len(),
                                        if fields.len() == 1 { "" } else { "s" },
                                        arm.bindings.len(),
                                        if arm.bindings.len() == 1 { "is" } else { "are" }
                                    ),
                                    &hint,
                                );
                            }
                        }
                        ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready => self.error(
                            arm.span,
                            "T11",
                            format!(
                                "`{}` is a `{name}`, not a resource; it has no `loading`, `error` or `ready`",
                                expr_text(&m.scrutinee)
                            ),
                            "Match its cases: `.case { … }`",
                        ),
                        _ => {}
                    }
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind(name, Type::Any, arm.span);
                    }
                    if let ArmPattern::Case(case) = &arm.pattern {
                        let fields = self.payload_of(name, case).unwrap_or_default();
                        for (i, bound) in arm.bindings.iter().enumerate() {
                            let ty = fields.get(i).map(|(_, t)| t.clone()).unwrap_or(Type::Any);
                            self.bind(bound, ty, arm.span);
                        }
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            Type::Any | Type::Case(_) => {
                for arm in &m.arms {
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind(name, Type::Any, arm.span);
                    }
                    for bound in &arm.bindings {
                        self.bind(bound, Type::Any, arm.span);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            // What a request can fail with, matched by its case.
            Type::NetError => {
                for arm in &m.arms {
                    self.push_scope();
                    if let ArmPattern::Case(case) = &arm.pattern
                        && !matches!(
                            case.as_str(),
                            "offline" | "timeout" | "aborted" | "parse" | "network" | "status"
                        )
                    {
                        self.error(
                            arm.span,
                            "T02",
                            format!("a request cannot fail with `.{case}`"),
                            "It fails with `.offline`, `.timeout`, `.aborted`, `.parse`, `.network` or `.status(code, body)`",
                        );
                    }
                    if let Some(name) = &arm.binding {
                        let bound = match &arm.pattern {
                            ArmPattern::Case(case) if case == "status" => Type::Any,
                            _ => Type::Any,
                        };
                        self.bind(name, bound, arm.span);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            other => {
                self.error(
                    span,
                    "T11",
                    format!(
                        "`match` needs a resource or an enum, but `{}` is `{other}`",
                        expr_text(&m.scrutinee)
                    ),
                    "Use `if` for a condition; `match` chooses among a resource's states or an enum's cases",
                );
                for arm in &m.arms {
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind(name, Type::Any, arm.span);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
        }
    }

    fn emit(&mut self, e: &EmitStmt, span: Span) {
        let Some(component) = self.component else {
            return;
        };
        let Some(event) = component.events.iter().find(|ev| ev.name == e.event) else {
            return; // sema reports the undeclared event
        };
        if event.params.len() != e.args.len() {
            self.error(
                span,
                "T09",
                format!(
                    "`emit {}` passes {} argument{}, but the event takes {}",
                    e.event,
                    e.args.len(),
                    if e.args.len() == 1 { "" } else { "s" },
                    event.params.len()
                ),
                &format!(
                    "`event {}({})`",
                    event.name,
                    event
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, Type::from_ref(&p.param_type)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }
        for (arg, param) in e.args.iter().zip(&event.params) {
            let wanted = self.world.resolve(Type::from_ref(&param.param_type));
            let given = self.infer(arg, Some(&wanted));
            self.expect(
                &given,
                &wanted,
                span,
                &format!("`{}` of `emit {}`", param.name, e.event),
            );
        }
    }

    fn layout(&mut self, layout: &LayoutRef) {
        let Some(component) = self.world.components.get(layout.name.as_str()) else {
            return;
        };
        self.component_args(&layout.args, &[], component, layout.span);
    }

    // ─── Elements ────────────────────────────────────────

    fn element(&mut self, el: &UIElement, span: Span) {
        match &el.component {
            ComponentRef::UserDefined(name) => {
                if let Some(component) = self.world.components.get(name.as_str()) {
                    self.component_args(&el.args, &el.arg_spans, component, span);
                } else {
                    for arg in &el.args {
                        self.infer(arg_value(arg), None);
                    }
                }
            }
            // A scoped slot's values, against what its declaration names.
            ComponentRef::BuiltIn(_) if el.slot_name().is_some() => {
                let slot = el.slot_name().unwrap_or("children").to_string();
                let params: Vec<(String, Type)> = self
                    .component
                    .and_then(|c| {
                        c.slots
                            .iter()
                            .find(|s| s.name.as_deref().unwrap_or("children") == slot)
                    })
                    .map(|s| {
                        s.params
                            .iter()
                            .map(|p| {
                                (
                                    p.name.clone(),
                                    self.world.resolve(Type::from_ref(&p.param_type)),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                for arg in &el.args {
                    let Arg::Named(key, value) = arg else {
                        continue;
                    };
                    if key == "slot" {
                        continue;
                    }
                    let wanted = params
                        .iter()
                        .find(|(n, _)| n == key)
                        .map(|(_, t)| t.clone());
                    let given = self.infer(value, wanted.as_ref());
                    if let Some(wanted) = wanted
                        && !given.assignable_to(&wanted)
                    {
                        self.error_at_current(
                            "T01",
                            format!("`{key}` of `{slot}` is `{given}`, but `{wanted}` is wanted"),
                            "",
                        );
                    }
                }
            }
            ComponentRef::BuiltIn(name) => {
                let sig = registry::component(name);
                self.builtin_args(el, sig, span);
            }
            ComponentRef::SubComponent(owner, part) => {
                let qualified = format!("{owner}.{part}");
                if let Some(component) = self.world.components.get(qualified.as_str()) {
                    self.component_args(&el.args, &el.arg_spans, component, span);
                } else {
                    let sig = registry::part(owner, part);
                    self.builtin_args(el, sig, span);
                }
            }
        }
        if let Some(style) = &el.style_block {
            for prop in &style.properties {
                self.current_span = prop.span;
                self.infer(&prop.value, None);
            }
        }
        self.current_span = span;
        for handler in &el.events {
            self.push_scope();
            if let Some(param) = &handler.param {
                // A declared event's handler receives its first parameter;
                // a DOM handler receives the event.
                let ty = match &el.component {
                    ComponentRef::UserDefined(name) => self
                        .world
                        .components
                        .get(name.as_str())
                        .and_then(|c| c.events.iter().find(|e| e.name == handler.event))
                        .and_then(|e| e.params.first())
                        .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                        .unwrap_or(Type::Any),
                    _ => Type::Any,
                };
                self.bind(param, ty, handler.span);
            }
            self.statements(&handler.body, Body::Imperative);
            self.pop_scope();
        }
        for fill in &el.slot_fills {
            // A fill's names take the types the slot's declaration gives
            // its values, in order.
            let params: Vec<Type> = match &el.component {
                ComponentRef::UserDefined(name) => self
                    .world
                    .components
                    .get(name.as_str())
                    .and_then(|c| {
                        c.slots
                            .iter()
                            .find(|s| s.name.as_deref() == Some(fill.name.as_str()))
                    })
                    .map(|s| {
                        s.params
                            .iter()
                            .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                            .collect()
                    })
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
            self.push_scope();
            for (i, name) in fill.params.iter().enumerate() {
                let ty = params.get(i).cloned().unwrap_or(Type::Any);
                self.bind(name, ty, fill.span);
            }
            self.statements(&fill.body, Body::Page);
            self.pop_scope();
        }
        self.statements(&el.children, Body::Page);
    }

    /// The arguments of a call to a user component, against its props.
    fn component_args(
        &mut self,
        args: &[Arg],
        spans: &[Span],
        component: &ComponentDecl,
        span: Span,
    ) {
        self.in_args = true;
        for (i, arg) in args.iter().enumerate() {
            let at = spans.get(i).copied().unwrap_or(span);
            self.current_span = at;
            let prop = match arg {
                Arg::Positional(_) => component
                    .props
                    .iter()
                    .find(|p| p.positional)
                    .or(component.props.first()),
                Arg::Named(key, _) => component.props.iter().find(|p| &p.name == key),
            };
            let value = arg_value(arg);
            let Some(prop) = prop else {
                self.infer(value, None);
                continue;
            };
            let mut wanted = self.world.resolve(Type::from_ref(&prop.prop_type));
            if prop.optional {
                wanted = Type::optional(wanted);
            }
            let given = self.infer(value, Some(&wanted));
            self.expect(
                &given,
                &wanted,
                at,
                &format!("`{}` of `{}`", prop.name, component.name),
            );
        }
        self.in_args = false;
    }

    /// The arguments of a built-in, against the registry's prop types.
    fn builtin_args(
        &mut self,
        el: &UIElement,
        sig: Option<&'static registry::ComponentSig>,
        span: Span,
    ) {
        let name = match &el.component {
            ComponentRef::BuiltIn(n) => n.clone(),
            ComponentRef::SubComponent(o, p) => format!("{o}.{p}"),
            ComponentRef::UserDefined(n) => n.clone(),
        };
        self.in_args = true;
        for (i, arg) in el.args.iter().enumerate() {
            let at = el.arg_spans.get(i).copied().unwrap_or(span);
            let at_span = at;
            self.current_span = at;
            let prop = match (arg, sig) {
                (Arg::Positional(_), Some(sig)) => sig.positional.as_ref(),
                (Arg::Named(key, _), Some(sig)) => sig.prop(key),
                _ => None,
            };
            let value = arg_value(arg);
            // `columns: { base: 1, md: 2 }` is one value per breakpoint,
            // each checked as the prop itself; the widths are the
            // stylesheet's, not a type's.
            if crate::codegen::scoped_css::is_responsive(value)
                && let Some(prop) = prop
            {
                let wanted = match prop.ty {
                    PropType::Num => Some(Type::Number),
                    PropType::Str => Some(Type::String),
                    PropType::Bool => Some(Type::Bool),
                    _ => None,
                };
                if let Expr::MapLiteral(pairs) = value {
                    for (step, at) in pairs {
                        let given = self.infer(at, wanted.as_ref());
                        if let Some(wanted) = &wanted
                            && !given.assignable_to(wanted)
                        {
                            self.error(
                                at_span,
                                "T01",
                                format!(
                                    "`{}:` at `{}` on `{name}` is `{given}`, but `{wanted}` is wanted",
                                    prop.name,
                                    step.trim_matches('"')
                                ),
                                "",
                            );
                        }
                    }
                }
                continue;
            }
            // Nothing the engine draws takes a secret: it would be shown,
            // or written into an attribute, which is the same thing.
            if let Expr::Identifier(_) | Expr::PropertyAccess(..) = value
                && self.infer(value, None).unwrapped() == Type::Scalar(Scalar::Secret)
            {
                self.error(
                    at,
                    "T12",
                    format!(
                        "`{}` is a `Secret`, and `{name}` would show it",
                        expr_text(value)
                    ),
                    "A secret must not reach the page; send it as a value, or show one field of what it unlocks",
                );
            }
            let Some(prop) = prop else {
                self.infer(value, None);
                continue;
            };
            // Nothing the engine draws takes a secret: it would be shown,
            // or written into an attribute, which is the same thing.
            if matches!(value, Expr::Identifier(_) | Expr::PropertyAccess(..))
                && self.infer(value, None).unwrapped() == Type::Scalar(Scalar::Secret)
            {
                self.error(
                    at,
                    "T12",
                    format!(
                        "`{}` is a `Secret`, and `{name}` would show it",
                        expr_text(value)
                    ),
                    "A secret must not reach the page; send it as a value, or show one field of what it unlocks",
                );
            }
            match (prop.ty, prop.name) {
                // `bind:` — the control's value type.
                (PropType::State, "ref") => {
                    if !matches!(value, Expr::Identifier(_)) {
                        self.error(
                            at,
                            "T01",
                            format!(
                                "`ref:` on `{name}` names the handle, but `{}` is given",
                                expr_text(value)
                            ),
                            "Write `ref: nameInput`, then read `nameInput` as the element",
                        );
                    }
                }
                (PropType::State, "bind") => {
                    let wanted = bound_type(&name);
                    let given = self.infer(value, wanted.as_ref());
                    if let Some(wanted) = wanted {
                        // A state that starts as `null` may hold the value later.
                        let given = match given {
                            Type::Optional(inner) => *inner,
                            other => other,
                        };
                        self.expect(&given, &wanted, at, &format!("`bind:` on `{name}`"));
                    }
                }
                (PropType::Str | PropType::Path, _) => {
                    let given = self.infer(value, Some(&Type::String));
                    // Text-like props take a number or a bool as text.
                    if !matches!(given, Type::Number | Type::Bool) {
                        self.expect(
                            &given,
                            &Type::String,
                            at,
                            &format!("`{}:` on `{name}`", prop.name),
                        );
                    }
                }
                (PropType::Num, _) => {
                    let given = self.infer(value, Some(&Type::Number));
                    self.expect(
                        &given,
                        &Type::Number,
                        at,
                        &format!("`{}:` on `{name}`", prop.name),
                    );
                }
                (PropType::Bool, _) => {
                    let given = self.infer(value, Some(&Type::Bool));
                    self.expect(
                        &given,
                        &Type::Bool,
                        at,
                        &format!("`{}:` on `{name}`", prop.name),
                    );
                }
                _ => {
                    self.infer(value, None);
                }
            }
        }
        self.in_args = false;
    }

    // ─── Expressions ─────────────────────────────────────

    /// The type of `expr`; `expected` guides a case or a lambda.
    fn infer(&mut self, expr: &Expr, expected: Option<&Type>) -> Type {
        match expr {
            // A literal written where one of the language's own types is
            // wanted is read as one, and held to its shape — so a `Url`
            // that is not a URL is caught where it is written, not where
            // it is used.
            Expr::StringLiteral(text) => match expected.map(|t| t.unwrapped()) {
                Some(Type::Scalar(s)) if s.is_text() => {
                    if let Some(shape) = ill_formed(s, text) {
                        let name = s.name();
                        // "an Email", but "a Url" and "a Uuid": the letter is not the sound.
                        let article = if name.starts_with('E') { "an" } else { "a" };
                        self.error_at_current(
                            "T01",
                            format!("`\"{text}\"` is not {article} `{name}`"),
                            &shape,
                        );
                    }
                    Type::Scalar(s)
                }
                _ => Type::String,
            },
            Expr::InterpolatedString(parts) => {
                for part in parts {
                    if let StringPart::Expression(e) = part {
                        let ty = self.infer(e, None);
                        // A `Secret` in a string is a secret in a URL, in a
                        // log line, in the page — wherever that string goes.
                        if ty.unwrapped() == Type::Scalar(Scalar::Secret) {
                            self.error_at_current(
                                "T12",
                                format!("`{}` is a `Secret`, and this puts it in text", expr_text(e)),
                                "A secret must not be shown, logged or put in a URL; send it as a value, or read one field of what it unlocks",
                            );
                        }
                    }
                }
                Type::String
            }
            Expr::NumberLiteral(_) => Type::Number,
            // A literal of one of the language's own types. The carrier is
            // checked as itself, and the value is what the name says.
            Expr::Typed(name, carrier) => {
                self.infer(carrier, None);
                self.world.resolve(Type::Record(name.clone()))
            }
            Expr::BoolLiteral(_) => Type::Bool,
            Expr::Null => Type::Null,
            Expr::Token(_) => Type::Token,
            Expr::Regex(..) => Type::Regex,
            Expr::EnumCase(case) => match expected.map(|t| t.unwrapped()) {
                Some(Type::Enum(name)) => {
                    let known = self.world.enums.get(name.as_str()).map(|e| e.case_names());
                    if let Some(cases) = known
                        && !cases.contains(case)
                        && !self.in_args
                    {
                        self.error_at_current(
                            "T02",
                            format!("`{name}` has no case `.{case}`"),
                            &format!(
                                "`{name}` takes {}",
                                cases
                                    .iter()
                                    .map(|c| format!(".{c}"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                    } else if let Some(fields) = self.payload_of(&name, case)
                        && !fields.is_empty()
                    {
                        self.error_at_current(
                            "T02",
                            format!(
                                "`.{case}` carries a payload; `.{case}` alone is not a `{name}`"
                            ),
                            &format!("Give it: `{}`", case_signature(case, &fields)),
                        );
                    }
                    Type::Enum(name)
                }
                _ => Type::Case(case.clone()),
            },
            // `.failed("x")`: the case with its payload, checked against the
            // enum's declaration when the position names one.
            Expr::CaseValue(case, args) => match expected.map(|t| t.unwrapped()) {
                Some(Type::Enum(name)) => {
                    let known = self.world.enums.get(name.as_str()).map(|e| e.case_names());
                    match self.payload_of(&name, case) {
                        Some(fields) if !fields.is_empty() => {
                            let params: Vec<Type> = fields.iter().map(|(_, t)| t.clone()).collect();
                            self.call_args(&params, args, &format!("`.{case}`"));
                        }
                        Some(_) => {
                            for a in args {
                                self.infer(a, None);
                            }
                            self.error_at_current(
                                "T02",
                                format!(
                                    "`.{case}` carries nothing; `.{case}(…)` is not a `{name}`"
                                ),
                                &format!("Write `.{case}`"),
                            );
                        }
                        None => {
                            for a in args {
                                self.infer(a, None);
                            }
                            if let Some(cases) = known
                                && !self.in_args
                            {
                                self.error_at_current(
                                    "T02",
                                    format!("`{name}` has no case `.{case}`"),
                                    &format!(
                                        "`{name}` takes {}",
                                        cases
                                            .iter()
                                            .map(|c| format!(".{c}"))
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    ),
                                );
                            }
                        }
                    }
                    Type::Enum(name)
                }
                _ => {
                    for a in args {
                        self.infer(a, None);
                    }
                    Type::Case(case.clone())
                }
            },
            Expr::Identifier(name) => self
                .lookup(name)
                .or_else(|| self.world.consts.get(name).cloned())
                // A service, and a store, are reachable by name from
                // anywhere: the bundle declares each once, at the top. `use`
                // says a body depends on a store, it does not make the name
                // visible — so `Store.member` is checked either way.
                .or_else(|| {
                    self.world
                        .apis
                        .contains_key(name.as_str())
                        .then(|| Type::Api(name.clone()))
                })
                .or_else(|| {
                    self.world
                        .stores
                        .contains_key(name.as_str())
                        .then(|| Type::Store(name.clone()))
                })
                .unwrap_or_else(|| global_type(name)),
            // A plain access after a `?.` in the same chain is short-circuited
            // with it: `a?.b.c` is null when `a` is, never a fault.
            Expr::PropertyAccess(base, field) if in_optional_chain(base) => {
                let base_ty = self.infer(base, None).unwrapped();
                Type::optional(self.property(&base_ty, base, field))
            }
            Expr::PropertyAccess(base, field) => {
                let base_ty = self.infer(base, None);
                self.property(&base_ty, base, field)
            }
            Expr::IndexAccess(base, index) => {
                let optional = in_optional_chain(base);
                let base_ty = self.infer(base, None);
                let base_ty = if optional {
                    base_ty.unwrapped()
                } else {
                    base_ty
                };
                self.infer(index, None);
                let ty = match base_ty.unwrapped() {
                    Type::List(inner) => *inner,
                    Type::String => Type::String,
                    _ => Type::Any,
                };
                if optional { Type::optional(ty) } else { ty }
            }
            // `?.`: the base may be null, and the result may be too.
            Expr::OptionalProperty(base, field) => {
                let base_ty = self.infer(base, None).unwrapped();
                Type::optional(self.property(&base_ty, base, field))
            }
            Expr::OptionalIndex(base, index) => {
                let base_ty = self.infer(base, None).unwrapped();
                self.infer(index, None);
                Type::optional(match base_ty {
                    Type::List(inner) => *inner,
                    Type::String => Type::String,
                    _ => Type::Any,
                })
            }
            Expr::OptionalMethod(obj, method, args) => {
                // Checked as the plain call on the unwrapped base: the same
                // methods, the same arguments, a result that may be null.
                let ty = self.method_call_on(obj, method, args, true);
                Type::optional(ty)
            }
            Expr::BinaryOp(l, op, r) => {
                let lt = self.infer(l, None);
                let rt = self.infer(r, None);
                match op {
                    BinOp::Add => {
                        if lt.unwrapped() == Type::String || rt.unwrapped() == Type::String {
                            Type::String
                        } else if lt == Type::Number && rt == Type::Number {
                            Type::Number
                        } else {
                            Type::Any
                        }
                    }
                    BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => Type::Number,
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => {
                        Type::Bool
                    }
                    BinOp::And | BinOp::Or => Type::Bool,
                    BinOp::NullCoalesce => Type::join(lt.unwrapped(), rt),
                }
            }
            Expr::UnaryOp(op, e) => {
                self.infer(e, None);
                match op {
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Neg => Type::Number,
                }
            }
            Expr::MethodCall(obj, method, args) if in_optional_chain(obj) => {
                Type::optional(self.method_call_on(obj, method, args, true))
            }
            Expr::MethodCall(obj, method, args) => self.method_call(obj, method, args),
            Expr::FunctionCall(name, args) => self.function_call(name, args),
            Expr::ListLiteral(items) => {
                let expected_item = match expected {
                    Some(Type::List(inner)) => Some((**inner).clone()),
                    _ => None,
                };
                let mut item = None;
                for e in items {
                    let ty = self.infer(e, expected_item.as_ref());
                    item = Some(match item {
                        None => ty,
                        Some(prev) => Type::join(prev, ty),
                    });
                }
                Type::list(item.unwrap_or(Type::Any))
            }
            // A map literal has the shape it was written with; a spread inside
            // it makes the keys unknown.
            Expr::MapLiteral(pairs) => {
                let wanted = match expected {
                    Some(Type::Record(name)) => self.world.record_fields(name).map(|fields| {
                        fields
                            .into_iter()
                            .map(|f| (f.name.clone(), self.world.resolve(Type::from_ref(&f.ty))))
                            .collect::<Vec<_>>()
                    }),
                    Some(Type::Shape(fields)) => Some(fields.clone()),
                    _ => None,
                };
                let mut shape: Vec<(String, Type)> = Vec::new();
                let mut spread = false;
                for (k, v) in pairs {
                    if k == "..." {
                        self.infer(v, None);
                        spread = true;
                        continue;
                    }
                    let key = k.trim_matches('"').to_string();
                    let want = wanted
                        .as_ref()
                        .and_then(|w| w.iter().find(|(n, _)| *n == key).map(|(_, t)| t.clone()));
                    let ty = self.infer(v, want.as_ref());
                    if let Some(want) = &want
                        && !ty.assignable_to(want)
                    {
                        self.error_at_current(
                            "T01",
                            format!("`{key}` is `{ty}`, but `{want}` is wanted"),
                            "",
                        );
                    }
                    match shape.iter().position(|(n, _)| *n == key) {
                        Some(at) => shape[at].1 = ty,
                        None => shape.push((key, ty)),
                    }
                }
                // An empty literal says nothing about its keys.
                if spread || shape.is_empty() {
                    Type::Map
                } else {
                    Type::Shape(shape)
                }
            }
            // `...items` in a list: its items; the list's own type is theirs.
            Expr::Spread(inner) => match self.infer(inner, None).unwrapped() {
                Type::List(item) => *item,
                Type::Any => Type::Any,
                other => {
                    self.error_at_current(
                        "T08",
                        format!(
                            "`...` spreads a list, but `{}` is `{other}`",
                            expr_text(inner)
                        ),
                        "",
                    );
                    Type::Any
                }
            },
            Expr::Range(a, b, _) => {
                for e in [a, b] {
                    let ty = self.infer(e, Some(&Type::Number));
                    if !ty.assignable_to(&Type::Number) {
                        self.error_at_current(
                            "T01",
                            format!(
                                "a range's end `{}` is `{ty}`, but `Number` is wanted",
                                expr_text(e)
                            ),
                            "",
                        );
                    }
                }
                Type::list(Type::Number)
            }
            Expr::Record(name, fields) => {
                if let Some(all) = self.world.record_fields(name) {
                    for (key, value) in fields {
                        match all.iter().find(|f| &f.name == key) {
                            Some(field) => {
                                let wanted = self.world.resolve(Type::from_ref(&field.ty));
                                self.infer(value, Some(&wanted));
                            }
                            None => {
                                self.infer(value, None);
                            }
                        };
                    }
                    Type::Record(name.clone())
                } else if self.world.enums.contains_key(name.as_str()) {
                    Type::Enum(name.clone())
                } else {
                    for (_, v) in fields {
                        self.infer(v, None);
                    }
                    Type::Any
                }
            }
            Expr::Lambda(params, body) => {
                let names: Vec<&str> = params.split(',').map(str::trim).collect();
                let param_types: Vec<Type> = match expected {
                    Some(Type::Func(types, _)) => names
                        .iter()
                        .enumerate()
                        .map(|(i, _)| types.get(i).cloned().unwrap_or(Type::Any))
                        .collect(),
                    _ => names.iter().map(|_| Type::Any).collect(),
                };
                self.push_scope();
                for (name, ty) in names.iter().zip(&param_types) {
                    self.narrow(name, ty.clone());
                }
                let ret = self.infer(body, None);
                self.pop_scope();
                Type::Func(param_types, Box::new(ret))
            }
            Expr::Await(e) => {
                let ty = self.infer(e, None);
                match ty {
                    Type::Resource(inner) => *inner,
                    other => other,
                }
            }
        }
    }

    fn property(&mut self, base_ty: &Type, base: &Expr, field: &str) -> Type {
        match base_ty {
            // What a failed request says about itself.
            Type::NetError => match field {
                "message" => Type::String,
                "status" => Type::Number,
                "kind" => Type::String,
                "body" => Type::Any,
                "headers" => Type::Map,
                _ => {
                    self.error_at_current(
                        "T05",
                        format!("a request's error has no field `{field}`"),
                        "It has `.message`, `.status`, `.kind`, `.body` and `.headers`, and matches `.offline`, `.timeout`, `.aborted`, `.parse`, `.network` and `.status(code, body)`",
                    );
                    Type::Any
                }
            },
            // An endpoint of a service.
            Type::Api(name) => match self.world.endpoint(name, field) {
                Some(endpoint) => Type::Func(
                    endpoint
                        .params
                        .iter()
                        .map(|p| self.world.resolve(Type::from_ref(&p.prop_type)))
                        .collect(),
                    Box::new(match &endpoint.returns {
                        Some(ty) => self.world.resolve(Type::from_ref(ty)),
                        None => Type::Null,
                    }),
                ),
                None => {
                    let names: Vec<String> = self
                        .world
                        .apis
                        .get(name.as_str())
                        .map(|a| {
                            a.endpoints
                                .iter()
                                .map(|e| format!("`{}`", e.name))
                                .collect()
                        })
                        .unwrap_or_default();
                    self.error_at_current(
                        "T06",
                        format!("`{name}` has no endpoint `{field}`"),
                        &format!("Its endpoints are {}", names.join(", ")),
                    );
                    Type::Any
                }
            },
            // A scalar's fields are its carrier's: money is an amount and a
            // currency, a file a name, a size and a type. Everything the
            // language computes is a call, so nothing here is invented.
            Type::Scalar(scalar) => match (*scalar, field) {
                (Scalar::Money, "amount") => Type::Number,
                (Scalar::Money, "currency") => Type::String,
                (Scalar::File, "name" | "type") => Type::String,
                (Scalar::File, "size") => Type::Number,
                (s, "length") if s.is_text() => Type::Number,
                (s, _) => {
                    self.error_at_current(
                        "T05",
                        format!("`{}` has no field `{field}`", s.name()),
                        &scalar_methods_hint(s),
                    );
                    Type::Any
                }
            },
            Type::Shape(fields) => match fields.iter().find(|(n, _)| n == field) {
                Some((_, ty)) => ty.clone(),
                None => {
                    let names: Vec<String> = fields.iter().map(|(n, _)| format!("`{n}`")).collect();
                    self.error_at_current(
                        "T05",
                        format!("`{}` has no field `{field}`", expr_text(base)),
                        &format!("It was written with {}", names.join(", ")),
                    );
                    Type::Any
                }
            },
            Type::Record(name) => match self.world.record_field(name, field) {
                Some(ty) => self.world.resolve(ty),
                None => {
                    let fields: Vec<String> = self
                        .world
                        .record_fields(name)
                        .map(|all| all.iter().map(|f| format!("`{}`", f.name)).collect())
                        .unwrap_or_default();
                    self.error_at_current(
                        "T05",
                        format!("`{name}` has no field `{field}`"),
                        &format!("Its fields are {}", fields.join(", ")),
                    );
                    Type::Any
                }
            },
            Type::Store(name) => match self.world.stores.get(name.as_str()) {
                Some(members) => match members.get(field) {
                    Some(ty) => ty.clone(),
                    None => {
                        let mut names: Vec<&String> = members.keys().collect();
                        names.sort();
                        self.error_at_current(
                            "T06",
                            format!("`{name}` has no member `{field}`"),
                            &format!(
                                "Its members are {}",
                                names
                                    .iter()
                                    .map(|n| format!("`{n}`"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                        Type::Any
                    }
                },
                None => Type::Any,
            },
            Type::Optional(inner) => {
                self.error_at_current(
                    "T04",
                    format!(
                        "`{}` may be null, so `.{field}` may fail",
                        expr_text(base)
                    ),
                    "Unwrap it first: `if let x = value { … }`, `value ?? fallback`, `value?.field`, or a check for `!= null`",
                );
                self.property(inner, base, field)
            }
            Type::List(_) => match field {
                "length" => Type::Number,
                _ => Type::Any,
            },
            Type::String => match field {
                "length" => Type::Number,
                _ => Type::Any,
            },
            Type::Resource(inner) => match field {
                "data" => Type::optional((**inner).clone()),
                "state" => Type::String,
                "error" => Type::optional(Type::Any),
                _ => Type::Any,
            },
            // `save.pending`: whether a call of the action is under way.
            Type::Func(..) => match field {
                "pending" => Type::Bool,
                _ => {
                    self.error_at_current(
                        "T05",
                        format!(
                            "`{}` is an action; it has no field `{field}`",
                            expr_text(base)
                        ),
                        "An action has `pending`, true while a call of it runs",
                    );
                    Type::Any
                }
            },
            _ => Type::Any,
        }
    }

    fn method_call(&mut self, obj: &Expr, method: &str, args: &[Expr]) -> Type {
        self.method_call_on(obj, method, args, false)
    }

    /// A method call; `optional` when written `?.`, so a base that may be
    /// null is what the operator is for, not a fault.
    fn method_call_on(&mut self, obj: &Expr, method: &str, args: &[Expr], optional: bool) -> Type {
        // `if let x = e { a } else { b }` as a value: `x` is the non-null
        // value in `a`.
        if method == "__iflet"
            && args.len() == 2
            && let Expr::Lambda(name, then_expr) = &args[0]
        {
            let value_ty = self.infer(obj, None);
            self.push_scope();
            self.narrow(name, value_ty.unwrapped());
            let then_ty = self.infer(then_expr, None);
            self.pop_scope();
            let else_ty = self.infer(&args[1], None);
            return Type::join(then_ty, else_ty);
        }
        // `match` over an enum as a value: `__is` asks for a case, `__payload`
        // for the payload where the case is the one asked for.
        if method == "__case" && args.is_empty() {
            self.infer(obj, None);
            return Type::String;
        }
        if (method == "__is" || method == "__payload")
            && args.len() == 1
            && let Expr::StringLiteral(case) = &args[0]
        {
            let subject = self.infer(obj, None);
            let fields = match subject.unwrapped() {
                Type::Enum(name) => {
                    let known = self.world.enums.get(name.as_str()).map(|e| e.case_names());
                    let fields = self.payload_of(&name, case);
                    if fields.is_none()
                        && let Some(cases) = known
                    {
                        self.error_at_current(
                            "T02",
                            format!("`{name}` has no case `.{case}`"),
                            &format!(
                                "`{name}` takes {}",
                                cases
                                    .iter()
                                    .map(|c| format!(".{c}"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                    }
                    fields
                }
                _ => None,
            };
            return if method == "__is" {
                Type::Bool
            } else {
                match fields {
                    Some(fields) if fields.len() == 1 => Type::optional(fields[0].1.clone()),
                    Some(fields) if fields.len() > 1 => Type::optional(Type::Map),
                    _ => Type::Any,
                }
            };
        }
        // `if c { a } else { b }` as a value: `c` narrows `a`, as it does a
        // branch; the value is what both arms fit.
        if method == "__if" && args.len() == 2 {
            let cond_ty = self.infer(obj, Some(&Type::Bool));
            self.condition(&cond_ty, obj, self.current_span, "if");
            self.push_scope();
            for name in narrowed_names(obj) {
                if let Some(Type::Optional(inner)) = self.lookup(&name) {
                    self.narrow(&name, *inner);
                }
            }
            let then_ty = self.infer(&args[0], None);
            self.pop_scope();
            let else_ty = self.infer(&args[1], None);
            return Type::join(then_ty, else_ty);
        }
        let obj_ty = self.infer(obj, None);
        // `Backend.users(page: 2)`: an endpoint of a service, whose
        // arguments are its parameters and whose value is what it returns.
        if let Type::Api(name) = &obj_ty {
            let api = name.clone();
            return match self.world.endpoint(&api, method) {
                Some(endpoint) => {
                    let wanted: Vec<(String, Type)> = endpoint
                        .params
                        .iter()
                        .map(|p| {
                            (
                                p.name.clone(),
                                self.world.resolve(Type::from_ref(&p.prop_type)),
                            )
                        })
                        .collect();
                    let returns = match &endpoint.returns {
                        Some(ty) => self.world.resolve(Type::from_ref(ty)),
                        None => Type::Null,
                    };
                    for arg in args {
                        // The arguments arrive as one map of names.
                        if let Expr::MapLiteral(pairs) = arg {
                            for (key, value) in pairs {
                                let key = key.trim_matches('"');
                                let want = wanted.iter().find(|(n, _)| n == key).map(|(_, t)| t);
                                let given = self.infer(value, want);
                                match want {
                                    Some(want) if !given.assignable_to(want) => {
                                        self.error_at_current(
                                            "T01",
                                            format!("`{key}:` on `{api}.{method}` is `{given}`, but `{want}` is wanted"),
                                            "",
                                        );
                                    }
                                    // `cache`, `on`, `timeout` and `signal`
                                    // are the call's own, not the service's.
                                    None if !matches!(
                                        key,
                                        "cache"
                                            | "on"
                                            | "timeout"
                                            | "signal"
                                            | "headers"
                                            | "as"
                                            | "retry"
                                    ) =>
                                    {
                                        let names: Vec<String> =
                                            wanted.iter().map(|(n, _)| format!("`{n}`")).collect();
                                        self.error_at_current(
                                            "T10",
                                            format!("`{api}.{method}` has no parameter `{key}`"),
                                            &format!("It takes {}", names.join(", ")),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        } else {
                            self.infer(arg, wanted.first().map(|(_, t)| t));
                        }
                    }
                    returns
                }
                None => {
                    let names: Vec<String> = self
                        .world
                        .apis
                        .get(api.as_str())
                        .map(|a| {
                            a.endpoints
                                .iter()
                                .map(|e| format!("`{}`", e.name))
                                .collect()
                        })
                        .unwrap_or_default();
                    self.error_at_current(
                        "T06",
                        format!("`{api}` has no endpoint `{method}`"),
                        &format!("Its endpoints are {}", names.join(", ")),
                    );
                    Type::Any
                }
            };
        }
        if let Type::Optional(_) = obj_ty
            && !optional
        {
            self.error_at_current(
                "T04",
                format!("`{}` may be null, so `.{method}()` may fail", expr_text(obj)),
                "Unwrap it first: `if let x = value { … }`, `value ?? fallback`, `value?.{method}()`, or a check for `!= null`",
            );
        }
        let obj_ty = obj_ty.unwrapped();
        match &obj_ty {
            // A shape's field that is a function: `form.reset()`.
            Type::Shape(fields)
                if matches!(
                    fields.iter().find(|(n, _)| n == method),
                    Some((_, Type::Func(..)))
                ) =>
            {
                let Some((_, Type::Func(params, ret))) =
                    fields.iter().find(|(n, _)| n == method).cloned()
                else {
                    unreachable!("matched a function field")
                };
                self.call_args(&params, args, &format!("`{}`", expr_text(obj)));
                *ret
            }
            Type::Store(name) => {
                let member = self
                    .world
                    .stores
                    .get(name.as_str())
                    .and_then(|m| m.get(method).cloned());
                match member {
                    Some(Type::Func(params, ret)) => {
                        self.call_args(&params, args, &format!("`{name}.{method}`"));
                        *ret
                    }
                    Some(_) => {
                        for a in args {
                            self.infer(a, None);
                        }
                        Type::Any
                    }
                    None => {
                        if self.world.stores.contains_key(name.as_str()) {
                            self.error_at_current(
                                "T06",
                                format!("`{name}` has no action `{method}`"),
                                "",
                            );
                        }
                        for a in args {
                            self.infer(a, None);
                        }
                        Type::Any
                    }
                }
            }
            Type::List(item) => {
                let item = (**item).clone();
                let lambda =
                    |ret: Type| Type::Func(vec![item.clone(), Type::Number], Box::new(ret));
                match method {
                    "map" => {
                        let f = self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        let ret = match f {
                            Type::Func(_, ret) => *ret,
                            _ => Type::Any,
                        };
                        Type::list(ret)
                    }
                    "filter" | "slice" | "concat" | "reverse" | "sort" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, Some(&lambda(Type::Bool)));
                        }
                        Type::list(item)
                    }
                    "find" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Bool)));
                        Type::optional(item)
                    }
                    "some" | "every" | "includes" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Bool)));
                        Type::Bool
                    }
                    // What is pushed must fit the list.
                    "push" => {
                        for (i, arg) in args.iter().enumerate() {
                            let ty = self.infer_arg(args, i, Some(&item));
                            if !ty.assignable_to(&item) {
                                self.error_at_current(
                                    "T01",
                                    format!(
                                        "`{}` is `{ty}`, but `{item}` is wanted",
                                        expr_text(arg)
                                    ),
                                    "",
                                );
                            }
                        }
                        Type::Number
                    }
                    "findIndex" | "indexOf" | "sum" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, Some(&lambda(Type::Bool)));
                        }
                        Type::Number
                    }
                    "join" => {
                        self.infer_arg(args, 0, None);
                        Type::String
                    }
                    "reduce" => {
                        let init = self.infer_arg(args, 1, None);
                        self.infer_arg(
                            args,
                            0,
                            Some(&Type::Func(
                                vec![init.clone(), item],
                                Box::new(init.clone()),
                            )),
                        );
                        init
                    }
                    "toString" => Type::String,
                    "sortBy" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        Type::list(item)
                    }
                    "groupBy" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        Type::Map
                    }
                    "unique" | "take" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, None);
                        }
                        Type::list(item)
                    }
                    "first" | "last" => Type::optional(item),
                    "flatMap" => {
                        let f = self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        match f {
                            Type::Func(_, ret) => match *ret {
                                Type::List(inner) => Type::list(*inner),
                                _ => Type::list(Type::Any),
                            },
                            _ => Type::list(Type::Any),
                        }
                    }
                    _ => {
                        for a in args {
                            self.infer(a, None);
                        }
                        Type::Any
                    }
                }
            }
            Type::String => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "toLowerCase" | "toUpperCase" | "trim" | "trimStart" | "trimEnd"
                    | "replace" | "replaceAll" | "slice" | "substring" | "charAt" | "at"
                    | "toString" | "padStart" | "padEnd" | "repeat" | "normalize"
                    | "capitalize" | "truncate" | "dedent" => Type::String,
                    "indexOf" | "lastIndexOf" | "length" | "charCodeAt" | "search"
                    | "localeCompare" => Type::Number,
                    "includes" | "startsWith" | "endsWith" => Type::Bool,
                    "split" | "lines" | "words" => Type::list(Type::String),
                    // `"x".match(/…/)`: the matches, or null.
                    "match" => Type::optional(Type::list(Type::String)),
                    _ => Type::Any,
                }
            }
            // The language's own types: a date's arithmetic, money's, a
            // URL's parts, a colour's mixing.
            Type::Scalar(scalar) => {
                for a in args {
                    self.infer(a, None);
                }
                let scalar = *scalar;
                match scalar_method(scalar, method) {
                    Some(ty) => ty,
                    None => {
                        self.error_at_current(
                            "T05",
                            format!("`{}` has no method `{method}`", scalar.name()),
                            &scalar_methods_hint(scalar),
                        );
                        Type::Any
                    }
                }
            }
            Type::Regex => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "test" => Type::Bool,
                    "exec" => Type::optional(Type::list(Type::String)),
                    _ => {
                        self.error_at_current(
                            "T05",
                            format!("a regular expression has no method `{method}`"),
                            "It has `test(text)` and `exec(text)`; a string has `match`, `replace`, `split` and `search`",
                        );
                        Type::Any
                    }
                }
            }
            Type::Number => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "toFixed" | "toString" => Type::String,
                    _ => Type::Any,
                }
            }
            Type::Record(name) => {
                for a in args {
                    self.infer(a, None);
                }
                self.error_at_current(
                    "T05",
                    format!("`{name}` has no method `{method}`; a record holds fields"),
                    "",
                );
                Type::Any
            }
            _ => {
                for a in args {
                    self.infer(a, None);
                }
                Type::Any
            }
        }
    }

    /// The payload of `case` on the enum `name`: its fields and their types,
    /// empty for a bare case, `None` for a case the enum lacks.
    fn payload_of(&self, name: &str, case: &str) -> Option<Vec<(String, Type)>> {
        let decl = self.world.enums.get(name)?;
        let case = decl.case(case)?;
        Some(
            case.fields
                .iter()
                .map(|f| (f.name.clone(), self.world.resolve(Type::from_ref(&f.ty))))
                .collect(),
        )
    }

    fn infer_arg(&mut self, args: &[Expr], i: usize, expected: Option<&Type>) -> Type {
        match args.get(i) {
            Some(e) => self.infer(e, expected),
            None => Type::Any,
        }
    }

    fn function_call(&mut self, name: &str, args: &[Expr]) -> Type {
        match self.lookup(name) {
            Some(Type::Func(params, ret)) => {
                self.call_args(&params, args, &format!("`{name}`"));
                *ret
            }
            Some(_) => {
                for a in args {
                    self.infer(a, None);
                }
                Type::Any
            }
            None if name == "format" => self.format_call(args),
            None if name == "setTheme" => {
                if args.len() != 1 {
                    self.error_at_current(
                        "T10",
                        format!("`setTheme` takes one of `\"light\"`, `\"dark\"` or `\"system\"`, but {} arguments are given", args.len()),
                        "",
                    );
                }
                for a in args {
                    let ty = self.infer(a, Some(&Type::String));
                    if !ty.assignable_to(&Type::String) {
                        self.error_at_current(
                            "T01",
                            format!(
                                "`setTheme` takes a `String`, but `{}` is `{ty}`",
                                expr_text(a)
                            ),
                            "One of `\"light\"`, `\"dark\"` or `\"system\"`",
                        );
                    }
                }
                Type::Null
            }
            None if name == "ago" => {
                if args.is_empty() || args.len() > 2 {
                    self.error_at_current(
                        "T10",
                        format!("`ago` takes a date, but {} arguments are given", args.len()),
                        "`ago(date)`, or `ago(date, now)` against a moment of your own",
                    );
                }
                for a in args {
                    self.infer(a, None);
                }
                Type::String
            }
            None => {
                for a in args {
                    self.infer(a, None);
                }
                match name {
                    "String" | "t" => Type::String,
                    "Number" => Type::Number,
                    "Bool" | "Boolean" => Type::Bool,
                    // Where a `Uuid` comes from.
                    "uuid" => Type::Scalar(Scalar::Uuid),
                    _ => Type::Any,
                }
            }
        }
    }

    /// `format(value, .style, option)`: the style is one the runtime knows,
    /// or a date pattern.
    fn format_call(&mut self, args: &[Expr]) -> Type {
        const STYLES: [&str; 10] = [
            "number", "integer", "decimal", "currency", "percent", "compact", "date", "time",
            "datetime", "relative",
        ];
        if args.is_empty() || args.len() > 3 {
            self.error_at_current(
                "T10",
                format!(
                    "`format` takes 1 to 3 arguments, but {} are given",
                    args.len()
                ),
                "`format(value)`, `format(value, .style)` or `format(value, .style, option)`",
            );
        }
        for (i, a) in args.iter().enumerate() {
            if i == 1 {
                match a {
                    Expr::EnumCase(style) if !STYLES.contains(&style.as_str()) => {
                        self.error_at_current(
                            "T02",
                            format!("`format` has no style `.{style}`"),
                            &format!(
                                "It takes {}, or a date pattern such as \"yyyy-MM-dd\"",
                                STYLES
                                    .iter()
                                    .map(|s| format!(".{s}"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                    }
                    Expr::EnumCase(_) | Expr::StringLiteral(_) => {}
                    other => {
                        let ty = self.infer(other, None);
                        if !ty.assignable_to(&Type::String) && !matches!(ty, Type::Case(_)) {
                            self.error_at_current(
                                "T01",
                                format!("the style of `format` is `{ty}`, but a `.style` or a date pattern is wanted"),
                                "",
                            );
                        }
                    }
                }
                continue;
            }
            self.infer(a, None);
        }
        Type::String
    }

    fn call_args(&mut self, params: &[Type], args: &[Expr], what: &str) {
        if params.len() != args.len() {
            self.error_at_current(
                "T10",
                format!(
                    "{what} takes {} argument{}, but {} {} given",
                    params.len(),
                    if params.len() == 1 { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "is" } else { "are" }
                ),
                "",
            );
        }
        for (i, arg) in args.iter().enumerate() {
            let wanted = params.get(i).cloned().unwrap_or(Type::Any);
            let given = self.infer(arg, Some(&wanted));
            if !given.assignable_to(&wanted) {
                self.error_at_current(
                    "T01",
                    format!(
                        "argument {} of {what} is `{given}`, but `{wanted}` is wanted",
                        i + 1
                    ),
                    "",
                );
            }
        }
    }

    /// An error at the statement or argument being checked. Expressions
    /// carry no span of their own; the nearest enclosing one is used.
    fn error_at_current(&mut self, code: &str, message: String, hint: &str) {
        let span = self.located(self.current_span, &message);
        self.error(span, code, message, hint);
    }

    /// `span`, moved to the first thing the message names in backticks
    /// that the statement's text holds — `sel?.title`, `.angry`, `nam` —
    /// when the source is at hand.
    fn located(&self, span: Span, message: &str) -> Span {
        let Some(source) = &self.source else {
            return span;
        };
        let (start, end) = (span.start as usize, span.end as usize);
        if end <= start
            || end > source.len()
            || !source.is_char_boundary(start)
            || !source.is_char_boundary(end)
        {
            return span;
        }
        let text = &source[start..end];
        let mut rest = message;
        while let Some(open) = rest.find('`') {
            let after = &rest[open + 1..];
            let Some(close) = after.find('`') else {
                break;
            };
            let needle = &after[..close];
            rest = &after[close + 1..];
            if needle.is_empty() {
                continue;
            }
            if let Some(at) = text.find(needle) {
                let before = &text[..at];
                let line = span.line as usize + before.matches('\n').count();
                let col = match before.rfind('\n') {
                    Some(nl) => before[nl + 1..].chars().count() + 1,
                    None => span.col as usize + before.chars().count(),
                };
                let mut located = span;
                located.line = line as u32;
                located.col = col as u32;
                located.start = (start + at) as u32;
                located.end = (start + at + needle.len()) as u32;
                return located;
            }
        }
        span
    }
}

/// The value an argument carries.
fn arg_value(arg: &Arg) -> &Expr {
    match arg {
        Arg::Positional(e) | Arg::Named(_, e) => e,
    }
}

/// The type a control's `bind:` state must hold.
fn bound_type(component: &str) -> Option<Type> {
    match component {
        "Checkbox" | "Switch" => Some(Type::Bool),
        "Slider" => Some(Type::Number),
        // A date picker picks a date, which is a `Date` — not a string
        // that happens to look like one. Empty, it holds nothing.
        "DatePicker" => Some(Type::optional(Type::Scalar(Scalar::Date))),
        "Input" => None, // text or number, by its `type`
        _ => None,
    }
}

/// The names a condition proves not null in its branch: `x`, `x != null`,
/// `x && …`.
fn narrowed_names(cond: &Expr) -> Vec<String> {
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

/// Whether `expr` is, or reads through, a `?.` — so the rest of its chain
/// short-circuits with it.
fn in_optional_chain(expr: &Expr) -> bool {
    match expr {
        Expr::OptionalProperty(..) | Expr::OptionalMethod(..) | Expr::OptionalIndex(..) => true,
        Expr::PropertyAccess(base, _)
        | Expr::IndexAccess(base, _)
        | Expr::MethodCall(base, _, _) => in_optional_chain(base),
        _ => false,
    }
}

/// The types of the names every program can read.
fn global_type(name: &str) -> Type {
    match name {
        "event" | "params" | "window" | "document" | "console" | "localStorage"
        | "sessionStorage" | "JSON" | "Math" | "Date" | "navigator" | "location" | "fetch"
        | "setTimeout" | "clearTimeout" | "setInterval" | "clearInterval" | "Object" | "Array"
        | "Promise" | "locale" | "dir" => Type::Any,
        // The project's `env`: a map of what the config set.
        "env" => Type::Map,
        // The browser as values.
        "viewport" => Type::Shape(vec![
            ("width".to_string(), Type::Number),
            ("height".to_string(), Type::Number),
            ("sm".to_string(), Type::Bool),
            ("md".to_string(), Type::Bool),
            ("lg".to_string(), Type::Bool),
            ("xl".to_string(), Type::Bool),
        ]),
        "query" => Type::Map,
        "hash" => Type::String,
        // What the reader chose: `light`, `dark` or `system`.
        "theme" => Type::String,
        // The moment the page is at, kept current.
        "now" => Type::Scalar(Scalar::DateTime),
        // The connection, as the page can read it.
        "network" => Type::Shape(vec![
            ("online".to_string(), Type::Bool),
            ("effectiveType".to_string(), Type::String),
            ("saveData".to_string(), Type::Bool),
            ("downlink".to_string(), Type::Number),
        ]),
        _ => Type::Any,
    }
}

/// What `Form(bind: form)` binds: whether every control is valid, the
/// values by field name, and `reset()`.
pub fn form_type() -> Type {
    Type::Shape(vec![
        ("valid".to_string(), Type::Bool),
        // Whether an async rule is still asking.
        ("pending".to_string(), Type::Bool),
        // The message of every field that has one, and the fields the
        // reader has left, each by the control's name.
        ("errors".to_string(), Type::Map),
        ("touched".to_string(), Type::Map),
        ("values".to_string(), Type::Map),
        ("element".to_string(), Type::Any),
        (
            "reset".to_string(),
            Type::Func(Vec::new(), Box::new(Type::Null)),
        ),
        (
            "submit".to_string(),
            Type::Func(Vec::new(), Box::new(Type::Null)),
        ),
        // What a 422 said, by field name.
        (
            "apply".to_string(),
            Type::Func(vec![Type::Map], Box::new(Type::Null)),
        ),
    ])
}

/// The names every `Form(bind: name)` in `stmts` declares, at any depth.
pub fn form_names(stmts: &[Statement]) -> Vec<String> {
    fn walk(stmts: &[Statement], out: &mut Vec<String>) {
        for stmt in stmts {
            if let StatementKind::UIElement(el) = &stmt.kind {
                if matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Form") {
                    for arg in &el.args {
                        if let Arg::Named(k, Expr::Identifier(name)) = arg
                            && k == "bind"
                            && !out.contains(name)
                        {
                            out.push(name.clone());
                        }
                    }
                }
                walk(&el.children, out);
                for fill in &el.slot_fills {
                    walk(&fill.body, out);
                }
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(stmts, &mut out);
    out
}

/// The names every `ref: name` in `stmts` declares, at any depth.
pub fn ref_names(stmts: &[Statement]) -> Vec<String> {
    fn walk(stmts: &[Statement], out: &mut Vec<String>) {
        for stmt in stmts {
            if let StatementKind::UIElement(el) = &stmt.kind {
                for arg in &el.args {
                    if let Arg::Named(k, Expr::Identifier(name)) = arg
                        && k == "ref"
                        && !out.contains(name)
                    {
                        out.push(name.clone());
                    }
                }
                walk(&el.children, out);
                for fill in &el.slot_fills {
                    walk(&fill.body, out);
                }
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(stmts, &mut out);
    out
}

/// `.failed(reason: String)`: a case with its payload, for a hint.
fn case_signature(case: &str, fields: &[(String, Type)]) -> String {
    let parts: Vec<String> = fields.iter().map(|(n, t)| format!("{n}: {t}")).collect();
    format!(".{case}({})", parts.join(", "))
}

/// What a rule is a rule for, or `None` when there is no such rule.
fn rule_wants(name: &str) -> Option<Vec<Type>> {
    Some(match name {
        // A value of any type is either there or not.
        "required" | "custom" | "async" | "matches" | "oneOf" => {
            vec![Type::Any, Type::String, Type::Number, Type::Bool]
        }
        "email" => vec![Type::String, Type::Scalar(Scalar::Email)],
        "url" => vec![Type::String, Type::Scalar(Scalar::Url)],
        "minLength" | "maxLength" | "pattern" => vec![Type::String],
        "min" | "max" => vec![
            Type::Number,
            Type::Scalar(Scalar::Date),
            Type::Scalar(Scalar::DateTime),
            Type::Scalar(Scalar::Time),
        ],
        _ => return None,
    })
}

/// Why a literal is not the scalar it was written for, or `None` when it
/// is one. The message says how the type is written.
fn ill_formed(scalar: Scalar, text: &str) -> Option<String> {
    let hex = |s: &str| s.chars().all(|c| c.is_ascii_hexdigit());
    let fits = match scalar {
        Scalar::Date => {
            let p: Vec<&str> = text.split('-').collect();
            p.len() == 3
                && p[0].len() == 4
                && p[1].len() == 2
                && p[2].len() == 2
                && p.iter().all(|s| s.chars().all(|c| c.is_ascii_digit()))
        }
        Scalar::Time => {
            let head = text.split(['+', 'Z']).next().unwrap_or(text);
            let p: Vec<&str> = head.split(':').collect();
            (2..=3).contains(&p.len())
                && p[0].len() == 2
                && p[1].len() == 2
                && p.iter().all(|s| {
                    s.split('.')
                        .next()
                        .unwrap_or(s)
                        .chars()
                        .all(|c| c.is_ascii_digit())
                })
        }
        Scalar::DateTime => match text.split_once('T') {
            Some((d, t)) => {
                ill_formed(Scalar::Date, d).is_none() && ill_formed(Scalar::Time, t).is_none()
            }
            None => false,
        },
        Scalar::Url => text.contains("://") && text.len() > 8,
        Scalar::Email => match text.split_once('@') {
            Some((user, host)) => !user.is_empty() && host.contains('.') && !host.starts_with('.'),
            None => false,
        },
        Scalar::Color => {
            let body = text.trim();
            match body.strip_prefix('#') {
                Some(digits) => matches!(digits.len(), 3 | 4 | 6 | 8) && hex(digits),
                // A CSS colour of any other spelling: a function, or a name.
                None => body.contains('(') || body.chars().all(|c| c.is_ascii_alphabetic()),
            }
        }
        Scalar::Uuid => {
            let p: Vec<&str> = text.split('-').collect();
            p.len() == 5
                && [8, 4, 4, 4, 12] == [p[0].len(), p[1].len(), p[2].len(), p[3].len(), p[4].len()]
                && p.iter().all(|s| hex(s))
        }
        // Anything may be a secret; a file is never a literal.
        Scalar::Secret => true,
        _ => true,
    };
    if fits {
        return None;
    }
    Some(
        match scalar {
            Scalar::Date => "A date is written `2026-03-14`, or `@2026-03-14`",
            Scalar::Time => "A time is written `09:30`, or `@09:30`",
            Scalar::DateTime => {
                "A moment is written `2026-03-14T09:30:00Z`, or `@2026-03-14T09:30Z`"
            }
            Scalar::Url => "A URL has a scheme: `https://example.com`",
            Scalar::Email => "An address is `name@example.com`",
            Scalar::Color => "A colour is `#0F766E`, `rgb(15, 118, 110)` or a CSS name",
            Scalar::Uuid => "A UUID is `3f2b1c4d-5e6f-7081-92a3-b4c5d6e7f809`",
            _ => "",
        }
        .to_string(),
    )
}

/// Why a value does not meet its type's condition, when it plainly does
/// not: `Number(0..=100)` given `120`, `String(minLength: 8)` given `"abc"`.
///
/// Only a value the compiler can read is checked — a literal, or an
/// expression over literals. Anything that arrives at run time is
/// validation's to refuse, from the same condition.
pub fn refinement_fault(ty: &TypeRef, value: &Expr) -> Option<String> {
    use crate::codegen::static_eval::{Scope, Static, eval};
    let args = ty.refinement();
    if args.is_empty() {
        return None;
    }
    let scope = Scope::default();
    let got = eval(value, &scope)?;
    let number = |v: &Static| match v {
        Static::Num(n) => Some(*n),
        _ => None,
    };
    for (name, bound) in args {
        let want = eval(bound, &scope)?;
        let fault = match name.as_str() {
            "min" => number(&got)? < number(&want)?,
            "max" => number(&got)? > number(&want)?,
            "below" => number(&got)? >= number(&want)?,
            "minLength" => (got.to_text().chars().count() as f64) < number(&want)?,
            "maxLength" => (got.to_text().chars().count() as f64) > number(&want)?,
            "after" => got.to_text().as_str() <= want.to_text().as_str(),
            "before" => got.to_text().as_str() >= want.to_text().as_str(),
            "pattern" => match &want {
                Static::Regex(p, f) => !crate::codegen::static_eval::matches(&got.to_text(), p, f)?,
                _ => false,
            },
            _ => false,
        };
        if fault {
            let said = match name.as_str() {
                "min" => format!("is below {}", want.to_text()),
                "max" => format!("is above {}", want.to_text()),
                "below" => format!("is not below {}", want.to_text()),
                "minLength" => format!("is shorter than {}", want.to_text()),
                "maxLength" => format!("is longer than {}", want.to_text()),
                "after" => format!("is not after {}", want.to_text()),
                "before" => format!("is not before {}", want.to_text()),
                _ => format!("does not match {}", want.to_text()),
            };
            return Some(said);
        }
    }
    None
}

/// What a method of one of the language's own types gives back, and
/// whether it has that method at all.
fn scalar_method(scalar: Scalar, method: &str) -> Option<Type> {
    use Scalar::*;
    let moment = matches!(scalar, Date | Time | DateTime);
    Some(match method {
        // Reading a moment apart.
        "year" | "month" | "day" | "weekday" | "hour" | "minute" | "second" if moment => {
            Type::Number
        }
        "native" if moment => Type::Any,
        "date" if scalar == DateTime => Type::Scalar(Date),
        "time" if scalar == DateTime => Type::Scalar(Time),
        "inZone" if moment => Type::Scalar(DateTime),
        "startOfDay" | "startOfWeek" | "startOfMonth" | "endOfDay" if moment => {
            Type::Scalar(scalar)
        }
        "isBefore" | "isAfter" | "isSame" if moment => Type::Bool,
        "until" if moment => Type::Scalar(Duration),
        // Arithmetic: a moment and money both take `plus` and `minus`, and
        // each gives back what it was.
        "plus" | "minus" if moment || matches!(scalar, Money | Duration) => Type::Scalar(scalar),
        // A length of time, read in a unit.
        "days" | "hours" | "minutes" | "seconds" | "ms" if scalar == Duration => Type::Number,
        // Money.
        "times" if scalar == Money => Type::Scalar(Money),
        "convert" if scalar == Money => Type::Scalar(Money),
        // A web address.
        "host" | "path" if scalar == Url => Type::String,
        "query" if scalar == Url => Type::Map,
        "with" if scalar == Url => Type::Scalar(Url),
        "domain" if scalar == Email => Type::String,
        // Colour.
        "mix" | "lighten" | "darken" if scalar == Color => Type::Scalar(Color),
        "alpha" if scalar == Color => Type::String,
        "contrast" if scalar == Color => Type::Number,
        // A file the reader chose.
        "preview" if scalar == File => Type::String,
        // Everything a string can do, a string-carried scalar can do.
        _ if scalar.is_text() => return string_method(method),
        _ => return None,
    })
}

/// What a `String`'s method gives back, `None` when it has none.
fn string_method(method: &str) -> Option<Type> {
    Some(match method {
        "toLowerCase" | "toUpperCase" | "trim" | "trimStart" | "trimEnd" | "replace"
        | "replaceAll" | "slice" | "substring" | "charAt" | "at" | "toString" | "padStart"
        | "padEnd" | "repeat" | "normalize" | "capitalize" | "truncate" | "dedent" => Type::String,
        "indexOf" | "lastIndexOf" | "length" | "charCodeAt" | "search" | "localeCompare" => {
            Type::Number
        }
        "includes" | "startsWith" | "endsWith" => Type::Bool,
        "split" | "lines" | "words" => Type::list(Type::String),
        "match" => Type::optional(Type::list(Type::String)),
        _ => return None,
    })
}

/// The methods a scalar has, for the message when one is misspelled.
fn scalar_methods_hint(scalar: Scalar) -> String {
    use Scalar::*;
    let listed: &str = match scalar {
        Date | Time | DateTime => {
            "`.year()`, `.month()`, `.day()`, `.weekday()`, `.hour()`, `.minute()`,              `.plus(days: 1)`, `.minus(…)`, `.isBefore(d)`, `.isAfter(d)`, `.until(d)`,              `.startOfDay()`, `.startOfWeek()`, `.startOfMonth()`, `.endOfDay()`,              `.inZone(\"Europe/Berlin\")`, `.native()`"
        }
        Duration => "`.days()`, `.hours()`, `.minutes()`, `.seconds()`, `.ms()`, `.plus(d)`",
        Money => {
            "`.plus(m)`, `.minus(m)`, `.times(n)`, `.convert(rate, \"USD\")`, and the fields `.amount` and `.currency`"
        }
        Url => "`.host()`, `.path()`, `.query()`, `.with(query: { page: 2 })`",
        Email => "`.domain()`",
        Color => {
            "`.mix(other, 0.2)`, `.lighten(0.1)`, `.darken(0.1)`, `.alpha(0.5)`, `.contrast(other)`"
        }
        File => "`.preview()`, and the fields `.name`, `.size` and `.type`",
        Uuid | Secret => "the methods of a string",
    };
    format!("It has {listed}")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::v2::parse_v2;

    fn errors(src: &str) -> Vec<String> {
        let program = parse_v2(src, "<t>").expect("parse");
        let info = check(&program, &|_| "<t>".to_string());
        info.findings
            .errors
            .iter()
            .map(|d| match &d.hint {
                Some(h) => format!("{}\n  {h}", d.message),
                None => d.message.clone(),
            })
            .collect()
    }

    fn clean(src: &str) {
        let found = errors(src);
        assert!(found.is_empty(), "{found:#?}");
    }

    fn has(src: &str, needle: &str) {
        let found = errors(src);
        assert!(
            found.iter().any(|e| e.contains(needle)),
            "no error containing {needle:?} in {found:#?}"
        );
    }

    fn type_of(src: &str, name: &str) -> String {
        let program = parse_v2(src, "<t>").expect("parse");
        let info = check(&program, &|_| "<t>".to_string());
        info.bindings
            .iter()
            .rev()
            .find(|b| b.name == name)
            .map(|b| b.ty.to_string())
            .unwrap_or_else(|| panic!("{name} has no type; bindings: {:?}", info.bindings))
    }

    const TODOS: &str = "enum Tone { calm, loud }\ntype Todo { id: String, title: String, done: Bool = false, tone: Tone = .calm, note: String? = null }\n";

    #[test]
    fn declared_and_inferred_types_of_names() {
        let src = format!(
            "{TODOS}store Todos {{\n  state items: [Todo] = []\n  state draft = \"\"\n  state n = 0\n  state sel = null\n  derived remaining = items.filter(t => !t.done).length\n  derived titles = items.map(t => t.title)\n  derived first = items.find(t => t.done)\n  action add(title: String) {{ items = items.concat([Todo(id: \"1\", title: title)]) }}\n  action count() {{ return items.length }}\n}}\npage P(path: \"/\", id: String) {{\n  use Todos\n  resource rows: [Todo] = fetch(\"/x\")\n  state t = Todo(id: \"a\", title: \"b\")\n  for todo in Todos.items by todo.id {{ Text(todo.title) }}\n  match rows {{ ready(list) {{ for r in list {{ Text(r.title) }} }} else {{ }} }}\n}}"
        );
        assert_eq!(type_of(&src, "items"), "[Todo]");
        assert_eq!(type_of(&src, "draft"), "String");
        assert_eq!(type_of(&src, "n"), "Number");
        assert_eq!(type_of(&src, "sel"), "Any?");
        assert_eq!(type_of(&src, "remaining"), "Number");
        assert_eq!(type_of(&src, "titles"), "[String]");
        assert_eq!(type_of(&src, "first"), "Todo?");
        assert_eq!(type_of(&src, "add"), "action(String)");
        assert_eq!(type_of(&src, "count"), "action() -> Number");
        assert_eq!(type_of(&src, "id"), "String");
        assert_eq!(type_of(&src, "rows"), "resource<[Todo]>");
        assert_eq!(type_of(&src, "t"), "Todo");
        assert_eq!(type_of(&src, "todo"), "Todo");
        assert_eq!(type_of(&src, "list"), "[Todo]");
        assert_eq!(type_of(&src, "r"), "Todo");
        clean(&src);
    }

    #[test]
    fn a_value_of_the_wrong_type_is_reported_where_it_is_given() {
        has(
            &format!("{TODOS}page P(path: \"/\") {{ state n: Number = \"x\" }}"),
            "[T01] `n` is `String`, but `Number` is wanted",
        );
        has(
            &format!(
                "{TODOS}component C(count: Number) {{ Text(count) }}\npage P(path: \"/\") {{ C(count: \"3\") }}"
            ),
            "`count` of `C` is `String`, but `Number` is wanted",
        );
        has(
            &format!(
                "{TODOS}component C(_ label: String) {{ Text(label) }}\npage P(path: \"/\") {{ C(3) }}"
            ),
            "`label` of `C` is `Number`, but `String` is wanted",
        );
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state x = Todo(id: \"a\", title: \"b\")\n Text(x.title) }}\ncomponent D(todo: Todo) {{ Text(todo.title) }}\npage Q(path: \"/q\") {{ state s = \"\"\n D(todo: s) }}"
            ),
            "`todo` of `D` is `String`, but `Todo` is wanted",
        );
        has(
            &format!("{TODOS}type Row {{ n: Number = \"no\" }}"),
            "the default of `n` is `String`, but `Number` is wanted",
        );
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state n = 1\n Button(\"x\") {{ on click {{ n = \"two\" }} }} }}"
            ),
            "`n` is `String`, but `Number` is wanted",
        );
    }

    #[test]
    fn enum_cases_are_checked_against_their_enum() {
        // A case an element argument gets wrong is the resolver's to report;
        // everywhere else it is the checker's.
        clean(&format!(
            "{TODOS}component C(tone: Tone) {{ Text(\"x\") }}\npage P(path: \"/\") {{ C(tone: .angry) }}"
        ));
        has(
            &format!("{TODOS}page P(path: \"/\") {{ state t: Tone = .angry }}"),
            "[T02] `Tone` has no case `.angry`",
        );
        clean(&format!(
            "{TODOS}component C(tone: Tone = .calm) {{ Text(\"x\") }}\npage P(path: \"/\") {{ C(tone: .loud)\n state t: Tone = .calm\n match t {{ .calm {{ Text(\"a\") }} else {{ Text(\"b\") }} }} }}"
        ));
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state t: Tone = .calm\n match t {{ .angry {{ Text(\"a\") }} else {{ }} }} }}"
            ),
            "[T02] `Tone` has no case `.angry`",
        );
    }

    #[test]
    fn fields_members_and_methods_that_do_not_exist() {
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state t = Todo(id: \"a\", title: \"b\")\n Text(t.name) }}"
            ),
            "[T05] `Todo` has no field `name`",
        );
        has(
            "store S { state n = 0 }\npage P(path: \"/\") { use S\n Text(S.count) }",
            "[T06] `S` has no member `count`",
        );
        has(
            "store S { state n = 0 action bump() { n = n + 1 } }\npage P(path: \"/\") { use S\n Button(\"x\") { on click { S.reset() } } }",
            "[T06] `S` has no action `reset`",
        );
        clean(
            "store S { state n = 0 action bump() { n = n + 1 } }\npage P(path: \"/\") { use S\n Text(\"{S.n}\")\n Button(\"x\") { on click { S.bump() } } }",
        );
    }

    #[test]
    fn a_store_is_checked_without_use_and_from_another_store() {
        // `use` says a body depends on a store; the name is reachable either
        // way, so the members are held to what the store has either way.
        has(
            "store S { state n = 0 }\npage P(path: \"/\") { Text(S.count) }",
            "[T06] `S` has no member `count`",
        );
        has(
            "store A { state n = 0 }\nstore B { action go() { A.bmup() } }\npage P(path: \"/\") { Button(\"x\") { on click { B.go() } } }",
            "[T06] `A` has no action `bmup`",
        );
        clean(
            "store A { state n = 0 action bump() { n = n + 1 } }\nstore B { action go() { A.bump() } }\npage P(path: \"/\") { Text(\"{A.n}\")\n Button(\"x\") { on click { B.go() } } }",
        );
    }

    #[test]
    fn a_services_headers_and_hooks_are_checked() {
        has(
            "store M { action record(s: Number) { log(s) } }\napi B(base: \"/api\") { on response(r) { M.recrd(r.status) }\n get me() -> Map }\npage P(path: \"/\") { resource m = B.me()\n match m { ready(v) { Text(\"{v}\") } else { Text(\"…\") } } }",
            "[T06] `M` has no action `recrd`",
        );
        has(
            "store S { state token = \"\" }\napi B(base: \"/api\") { headers { Authorization: \"Bearer {S.tokn}\" }\n get me() -> Map }\npage P(path: \"/\") { resource m = B.me()\n match m { ready(v) { Text(\"{v}\") } else { Text(\"…\") } } }",
            "[T06] `S` has no member `tokn`",
        );
        clean(
            "store S { state token = \"\"\n action refresh() { token = \"x\" } }\napi B(base: \"/api\") { headers { Authorization: \"Bearer {S.token}\" }\n on request(r) { r.headers[\"X-Id\"] = \"1\" }\n on error(e) { await S.refresh()  return \"retry\" }\n get me() -> Map }\npage P(path: \"/\") { resource m = B.me()\n match m { ready(v) { Text(\"{v}\") } else { Text(\"…\") } } }",
        );
    }

    #[test]
    fn a_value_that_may_be_null_must_be_unwrapped() {
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state t = Todo(id: \"a\", title: \"b\")\n Text(t.note.length) }}"
            ),
            "[T04] `t.note` may be null",
        );
        clean(&format!(
            "{TODOS}page P(path: \"/\") {{ state t = Todo(id: \"a\", title: \"b\")\n state sel: Todo? = null\n if let n = t.note {{ Text(n.length) }}\n Text(t.note ?? \"none\")\n if sel != null {{ Text(sel.title) }}\n if sel {{ Text(sel.title) }}\n if sel && t.done {{ Text(sel.title) }} }}"
        ));
        has(
            &format!("{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n Text(sel.title) }}"),
            "[T04] `sel` may be null, so `.title` may fail",
        );
        // The condition of an `if` expression narrows its then-arm too.
        clean(&format!(
            "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived label = if sel != null {{ sel.title }} else {{ \"none\" }}\n Text(label) }}"
        ));
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived label = if true {{ \"x\" }} else {{ sel.title }}\n Text(label) }}"
            ),
            "[T04] `sel` may be null, so `.title` may fail",
        );
    }

    #[test]
    fn calls_and_emits_take_the_declared_arguments() {
        has(
            "page P(path: \"/\") { action add(title: String, n: Number) { }\n Button(\"x\") { on click { add(\"a\") } } }",
            "[T10] `add` takes 2 arguments, but 1 is given",
        );
        has(
            "page P(path: \"/\") { action add(n: Number) { }\n Button(\"x\") { on click { add(\"a\") } } }",
            "argument 1 of `add` is `String`, but `Number` is wanted",
        );
        has(
            "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(1, 2) } } }",
            "[T09] `emit pick` passes 2 arguments, but the event takes 1",
        );
        has(
            "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(1) } } }",
            "`id` of `emit pick` is `Number`, but `String` is wanted",
        );
        clean(
            "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(\"a\") } } }\npage P(path: \"/\") { C { on pick(id) { log(id.length) } } }",
        );
    }

    #[test]
    fn controls_bind_state_of_their_own_kind() {
        has(
            "page P(path: \"/\") { state name = \"\"\n Checkbox(bind: name, label: \"x\") }",
            "`bind:` on `Checkbox` is `String`, but `Bool` is wanted",
        );
        has(
            "page P(path: \"/\") { state on = true\n Slider(bind: on) }",
            "`bind:` on `Slider` is `Bool`, but `Number` is wanted",
        );
        clean(
            "page P(path: \"/\") { state on = true\n state n = 0\n state s = \"\"\n Checkbox(bind: on, label: \"x\")\n Slider(bind: n)\n Input(bind: s, label: \"y\")\n Switch(bind: on, label: \"z\") }",
        );
    }

    #[test]
    fn loops_conditions_and_matches_need_the_right_shapes() {
        has(
            "page P(path: \"/\") { state name = \"x\"\n for c in name { Text(c) } }",
            "[T08] `for` loops over a list, but `name` is `String`",
        );
        has(
            "page P(path: \"/\") { state items = [1]\n if items { Text(\"some\") } }",
            "[T07] `if` reads `items` as a condition, but a list is always true",
        );
        has(
            "page P(path: \"/\") { state n = 1\n match n { else { Text(\"x\") } } }",
            "[T11] `match` needs a resource or an enum, but `n` is `Number`",
        );
        has(
            "page P(path: \"/\") { resource r = fetch(\"/x\")\n match r { .calm { } else { } } }",
            "`r` is a resource; `.calm` is not one of its states",
        );
        clean(
            "page P(path: \"/\") { state items = [1]\n state n = 0\n state s = \"\"\n if items.length > 0 { Text(\"some\") }\n if n { Text(\"n\") }\n if s { Text(\"s\") }\n show n > 1 { Text(\"x\") } }",
        );
    }

    #[test]
    fn the_layout_and_the_universal_props_are_typed_too() {
        has(
            "component Shell(crumb: String) { slot  children }\npage P(path: \"/\", layout: Shell(crumb: 3)) { Text(\"x\") }",
            "`crumb` of `Shell` is `Number`, but `String` is wanted",
        );
        clean(
            "component Shell(crumb: String) { slot  children }\npage P(path: \"/\", layout: Shell(crumb: \"Home\")) { Text(\"x\")\n Card(exit: .fadeOut, delay: \"100ms\").fadeIn { Text(\"y\") } }",
        );
    }

    #[test]
    fn untyped_programs_check_as_they_always_did() {
        clean(
            "store S { state items = [] state q = \"\" action add(item: Map) { items.push(item) } derived hits = items.filter(i => i.name.includes(q)) }\npage P(path: \"/\") { use S\n state user = null\n derived h = { a: 1 }\n Button(\"x\") { on click { user = { name: \"x\" }  S.add({ name: q })  h.a = 2  log(JSON.stringify(user))  localStorage.setItem(\"k\", \"v\") } }\n if user { Text(user.name) }\n for it in S.hits { Text(it.name) }\n Text(\"{S.q.length}\") }",
        );
    }

    #[test]
    fn a_loop_in_an_action_binds_the_item_and_the_index() {
        clean(&format!(
            "{TODOS}page P(path: \"/\") {{ state items: [Todo] = []\n state n = 0\n action all() {{ for t in items {{ n = n + t.title.length }}\n for t, i in items {{ n = n + i }} }} }}"
        ));
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state items: [Todo] = []\n action all() {{ for t in items {{ t.nam = 1 }} }} }}"
            ),
            "[T05] `Todo` has no field `nam`",
        );
        has(
            "page P(path: \"/\") { state s = \"x\"\n action all() { for c in s { log(c) } } }",
            "[T08] `for` loops over a list, but `s` is `String`",
        );
    }

    #[test]
    fn optional_chaining_reads_through_null_and_its_result_may_be_null() {
        clean(&format!(
            "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived t = sel?.title\n derived n = sel?.title?.length ?? 0\n Text(t ?? \"none\") }}"
        ));
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived t = sel?.nam\n Text(t ?? \"\") }}"
            ),
            "[T05] `Todo` has no field `nam`",
        );
        // The rest of the chain short-circuits with the `?.`: no fault, a
        // value that may be null.
        clean(&format!(
            "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived n = sel?.title.length\n Text(\"{{n}}\") }}"
        ));
        assert_eq!(
            type_of(
                &format!(
                    "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived n = sel?.title.length\n Text(\"x\") }}"
                ),
                "n"
            ),
            "Number?"
        );
        // Off the chain, the result is a value that may be null.
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived t = sel?.title\n derived n = t.length\n Text(\"{{n}}\") }}"
            ),
            "[T04] `t` may be null",
        );
        assert_eq!(
            type_of(
                &format!(
                    "{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n derived t = sel?.title\n Text(\"x\") }}"
                ),
                "t"
            ),
            "String?"
        );
    }

    #[test]
    fn a_map_literal_has_the_shape_it_was_written_with() {
        clean(
            "page P(path: \"/\") { state user = { name: \"\", age: 0 }\n derived n = user.name.length + user.age\n Text(\"{n}\") }",
        );
        has(
            "page P(path: \"/\") { state user = { name: \"\", age: 0 }\n Text(user.nam) }",
            "[T05] `user` has no field `nam`\n  It was written with `name`, `age`",
        );
        has(
            "page P(path: \"/\") { derived rows = [{ id: 1, label: \"a\" }]\n for r in rows { Text(r.lable) } }",
            "[T05] `r` has no field `lable`",
        );
        assert_eq!(
            type_of(
                "page P(path: \"/\") { state user = { name: \"\", age: 0 }\n Text(\"x\") }",
                "user"
            ),
            "{ name: String, age: Number }"
        );
        // A shape fits a record, a map or another shape that agrees on the
        // shared fields; an empty literal or a spread says nothing about the keys.
        clean(&format!(
            "{TODOS}page P(path: \"/\") {{ state items: [Todo] = []\n state form = {{}}\n state opts = {{ ...form, x: 1 }}\n action add() {{ items.push({{ id: \"1\", title: \"t\" }})  form.anything = 1  opts.other = 2 }} }}"
        ));
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state items: [Todo] = []\n action add() {{ items.push({{ id: 1, title: \"t\" }}) }} }}"
            ),
            "`id` is `Number`, but `String` is wanted",
        );
        // Two shapes join on the fields they share.
        assert_eq!(
            type_of(
                "page P(path: \"/\") { state on = true\n derived v = if on { { a: 1, b: \"\" } } else { { a: 2, c: true } }\n Text(\"x\") }",
                "v"
            ),
            "{ a: Number, b: String?, c: Bool? }"
        );
        clean(
            "page P(path: \"/\") { state rows = [{ id: 1 }, { id: 2, other: true }]\n derived n = rows.filter(r => r.other).length\n Text(\"{n}\") }",
        );
    }

    const STATUS: &str =
        "enum Status { idle, failed(reason: String), done(count: Number, label: String) }\n";

    #[test]
    fn a_case_carries_the_payload_it_was_declared_with() {
        clean(&format!(
            "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n action go() {{ s = .failed(\"boom\")  s = .done(1, \"one\") }}\n match s {{ .idle {{ Text(\"idle\") }} .failed(r) {{ Text(r.toUpperCase()) }} .done(n, l) {{ Text(\"{{n}} {{l}}\") }} }}\n Text(match s {{ .failed(r) {{ r }} .done {{ \"done\" }} else {{ \"-\" }} }}) }}"
        ));
        assert_eq!(
            type_of(
                &format!(
                    "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n derived r = match s {{ .failed(r) {{ r }} else {{ \"\" }} }}\n Text(r) }}"
                ),
                "r"
            ),
            "String"
        );
        has(
            &format!("{STATUS}page P(path: \"/\") {{ state s: Status = .failed  Text(\"x\") }}"),
            "[T02] `.failed` carries a payload; `.failed` alone is not a `Status`\n  Give it: `.failed(reason: String)`",
        );
        has(
            &format!("{STATUS}page P(path: \"/\") {{ state s: Status = .idle(1)  Text(\"x\") }}"),
            "[T02] `.idle` carries nothing; `.idle(…)` is not a `Status`",
        );
        has(
            &format!("{STATUS}page P(path: \"/\") {{ state s: Status = .failed(1)  Text(\"x\") }}"),
            "[T01] argument 1 of `.failed` is `Number`, but `String` is wanted",
        );
        has(
            &format!("{STATUS}page P(path: \"/\") {{ state s: Status = .done(1)  Text(\"x\") }}"),
            "[T10] `.done` takes 2 arguments, but 1 is given",
        );
        has(
            &format!(
                "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n match s {{ .done(n) {{ Text(\"{{n}}\") }} else {{ }} }} }}"
            ),
            "[T10] `.done` carries 2 values, but 1 is bound\n  Bind every part: `.done(count: Number, label: String)`",
        );
        has(
            &format!(
                "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n match s {{ .idle(x) {{ Text(x) }} else {{ }} }} }}"
            ),
            "[T10] `.idle` carries 0 values, but 1 is bound",
        );
        has(
            &format!(
                "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n match s {{ .done(n, l) {{ for x in n {{ Text(\"{{x}}\") }} }} else {{ }} }} }}"
            ),
            "[T08] `for` loops over a list, but `n` is `Number`",
        );
        has(
            &format!(
                "{STATUS}page P(path: \"/\") {{ state s: Status = .idle\n derived r = match s {{ .gone {{ 1 }} else {{ 0 }} }}\n Text(\"{{r}}\") }}"
            ),
            "[T02] `Status` has no case `.gone`",
        );
    }

    #[test]
    fn format_and_ago_are_strings_with_a_known_style() {
        clean(
            "page P(path: \"/\") { state n = 1234.5\n state d = \"2024-03-05\"\n derived a = format(n, .currency) + format(n, .currency, \"EUR\") + format(n) + format(d, \"yyyy-MM-dd\") + format(d, .date, \"long\") + ago(d)\n Text(a.toUpperCase()) }",
        );
        has(
            "page P(path: \"/\") { state n = 1\n Text(format(n, .money)) }",
            "[T02] `format` has no style `.money`",
        );
        has(
            "page P(path: \"/\") { state n = 1\n Text(format(n, 2)) }",
            "[T01] the style of `format` is `Number`, but a `.style` or a date pattern is wanted",
        );
        has(
            "page P(path: \"/\") { state n = 1\n Text(format()) }",
            "[T10] `format` takes 1 to 3 arguments, but 0 are given",
        );
        // A page's own `format` action is its own.
        clean(
            "page P(path: \"/\") { state n = 1\n action format(x: Number) { return \"{x}!\" }\n Text(format(n)) }",
        );
    }

    #[test]
    fn a_form_handle_and_an_actions_pending_are_typed() {
        clean(
            "page P(path: \"/\") { state ok = false\n action save() { let r = await fetch(\"/x\")  ok = true }\n Form(bind: f) { Input(name: \"a\")  Button(\"s\", disabled: !f.valid || save.pending) { on click { f.reset()  log(f.values.a) } } } }",
        );
        has(
            "page P(path: \"/\") { action save() { log(1) }\n Text(\"{save.nope}\") }",
            "[T05] `save` is an action; it has no field `nope`",
        );
        has(
            "page P(path: \"/\") { Form(bind: f) { Input(name: \"a\") }\n Text(f.vald) }",
            "[T05] `f` has no field `vald`",
        );
    }

    #[test]
    fn refs_and_timers_are_typed() {
        clean(
            "page P(path: \"/\") { state n = 0\n Input(bind: q, ref: box)\n state q = \"\"\n Button(\"x\") { on click { box.focus()  box.value = \"\" } }\n every(1000) { n = n + 1 }\n effect { log(n)  cleanup { log(n) } } }",
        );
        has(
            "page P(path: \"/\") { state n = 0\n every(\"soon\") { n = n + 1 } }",
            "[T01] `every` takes milliseconds, but `\"soon\"` is `String`",
        );
        has(
            "page P(path: \"/\") { state q = \"\"\n Input(bind: q, ref: \"box\") }",
            "[T01] `ref:` on `Input` names the handle, but `\"box\"` is given",
        );
    }

    #[test]
    fn a_scoped_slot_types_its_values_and_a_fills_names() {
        clean(&format!(
            "{TODOS}component Rows(items: [Todo]) {{ slot row(item: Todo, index: Number)  for it, i in items {{ row(item: it, index: i) }} }}\npage P(path: \"/\") {{ state todos: [Todo] = []\n  Rows(items: todos) {{ row(t, i) {{ Text(\"{{i}}: {{t.title}}\") }} }} }}"
        ));
        has(
            &format!(
                "{TODOS}component Rows(items: [Todo]) {{ slot row(item: Todo, index: Number)  for it, i in items {{ row(item: it, index: \"x\") }} }}\npage P(path: \"/\") {{ Text(\"x\") }}"
            ),
            "[T01] `index` of `row` is `String`, but `Number` is wanted",
        );
        has(
            &format!(
                "{TODOS}component Rows(items: [Todo]) {{ slot row(item: Todo)  for it in items {{ row(item: it) }} }}\npage P(path: \"/\") {{ state todos: [Todo] = []\n  Rows(items: todos) {{ row(t) {{ Text(t.nam) }} }} }}"
            ),
            "[T05] `Todo` has no field `nam`",
        );
    }

    #[test]
    fn an_error_is_placed_at_the_expression_it_names_when_the_source_is_at_hand() {
        let src = format!(
            "{TODOS}page P(path: \"/\") {{\n    state sel: Todo? = null\n    derived n = 1 + sel.title.length\n    Text(\"x\")\n}}"
        );
        let program = crate::syntax::parse_source(&src, "t.wf").unwrap();
        let info = check_in(&program, &|_| "t.wf".to_string(), &|_| Some(src.clone()));
        let error = info
            .findings
            .errors
            .iter()
            .find(|e| e.message.contains("T04"))
            .unwrap();
        // `sel` sits on the derived line, after `derived n = 1 + `.
        let line = src.lines().position(|l| l.contains("derived n")).unwrap() + 1;
        assert_eq!(error.line, line, "{error:?}");
        assert_eq!(error.column, "    derived n = 1 + ".len() + 1, "{error:?}");
    }
}
