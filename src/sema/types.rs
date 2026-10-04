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

use std::collections::{HashMap, HashSet};

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
    /// What an async action hands back before it is awaited: a promise of
    /// its result.
    Promise(Box<Type>),
    /// What no value is: the arm of a `match` that covers every case.
    Never,
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
    /// Every one, in the order the guide lists them.
    pub const ALL: [Scalar; 11] = [
        Scalar::Date,
        Scalar::Time,
        Scalar::DateTime,
        Scalar::Duration,
        Scalar::Money,
        Scalar::Url,
        Scalar::Email,
        Scalar::Color,
        Scalar::Uuid,
        Scalar::File,
        Scalar::Secret,
    ];

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
            (Type::Any, _) | (_, Type::Any) | (Type::Never, _) => true,
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
            // A function fits where one is wanted when it takes no more
            // arguments than it will be given — JavaScript passes a
            // function every argument, and one it does not name is
            // dropped — and can take the ones it is given.
            (Type::Func(from, _), Type::Func(to, _)) => {
                from.len() <= to.len()
                    && from.iter().zip(to.iter()).all(|(f, t)| t.assignable_to(f))
            }
            // A scalar is a plain value at run time, so it shows wherever
            // text shows — except a `Secret`, which must not be shown at
            // all, and `Duration`, which is a number of milliseconds.
            (Type::Scalar(s), Type::String) => s.is_text() && *s != Scalar::Secret,
            (Type::Scalar(Scalar::Duration), Type::Number) => true,
            (Type::Number, Type::Scalar(Scalar::Duration)) => true,
            (Type::Resource(a), Type::Resource(b)) => a.assignable_to(b),
            (Type::Promise(a), Type::Promise(b)) => a.assignable_to(b),
            (a, b) => a == b,
        }
    }

    /// The type both `a` and `b` fit: their common type, or `Any`.
    fn join(a: Type, b: Type) -> Type {
        match (a, b) {
            (a, b) if a == b => a,
            (Type::Never, other) | (other, Type::Never) => other,
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
            Type::Promise(inner) => write!(f, "Promise<{inner}>"),
            Type::Never => write!(f, "never"),
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
    /// What the project's scripts make global, by name, with the file each
    /// is in.
    scripts: HashMap<&'p str, (&'p str, &'p crate::project_js::scan::Name)>,
    /// The top-level names nothing may assign to, and what each is: a
    /// `const`, a `data` constant, an `image`.
    fixed: HashMap<String, &'static str>,
    /// Each store's members nothing may assign to: its derived values and
    /// its actions.
    store_fixed: HashMap<&'p str, HashMap<String, &'static str>>,
    /// Each store's actions, by name, for what they write.
    store_actions: HashMap<&'p str, HashMap<String, &'p [Statement]>>,
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
    /// Names read or called that nothing declares — not the program, not
    /// the language, not the browser. The build refuses them (`T13`); a
    /// template rendered with data reads its data's keys by name, so the
    /// template engine does not.
    pub unresolved: Vec<Diagnostic>,
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
        scripts: HashMap::new(),
        fixed: HashMap::new(),
        store_fixed: HashMap::new(),
        store_actions: HashMap::new(),
    };
    for decl in &program.declarations {
        match decl {
            Declaration::Script(script) => {
                for n in &script.names {
                    world
                        .scripts
                        .entry(n.name.as_str())
                        .or_insert((script.path.as_str(), n));
                }
            }
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
            Declaration::Store(store) => {
                let mut fixed = HashMap::new();
                for stmt in &store.body {
                    match &stmt.kind {
                        StatementKind::Derived(d) => {
                            fixed.insert(d.name.clone(), "a `derived` value");
                        }
                        StatementKind::Action(a) => {
                            fixed.insert(a.name.clone(), "an action");
                        }
                        _ => {}
                    }
                }
                world.store_fixed.insert(store.name.as_str(), fixed);
                let actions: HashMap<String, &[Statement]> = store
                    .body
                    .iter()
                    .filter_map(|s| match &s.kind {
                        StatementKind::Action(a) => Some((a.name.clone(), a.body.as_slice())),
                        _ => None,
                    })
                    .collect();
                world.store_actions.insert(store.name.as_str(), actions);
            }
            Declaration::Const(c) => {
                world.fixed.insert(c.name.clone(), "a `const`");
            }
            Declaration::Data(d) => {
                world.fixed.insert(
                    d.name.clone(),
                    if d.is_image {
                        "an `image`"
                    } else {
                        "a `data` constant"
                    },
                );
            }
            _ => {}
        }
    }
    let mut info = TypeInfo::default();

    // Constants first: plain values, read everywhere. A `data` declaration
    // the build has not resolved yet is what it declares, or `Any`.
    for (index, decl) in program.declarations.iter().enumerate() {
        if let Declaration::Data(d) = decl {
            // An `image` is the asset the build made of it; `data` what it
            // declares, or `Any`.
            let ty = if d.is_image {
                image_type()
            } else {
                d.ty.as_ref()
                    .map(|t| world.resolve(Type::from_ref(t)))
                    .unwrap_or(Type::Any)
            };
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
                    cx.bind_fixed(&param.name, ty, param.span, "a route parameter");
                }
                if let Some(layout) = &p.layout {
                    cx.current_span = layout.span;
                    cx.layout(layout);
                }
                cx.bound = bound_names(&p.body);
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
                    cx.bind_fixed(&prop.name, ty, prop.span, "a prop");
                }
                cx.component = Some(c);
                cx.bound = bound_names(&c.body);
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
                    // Every `:name` in the path is filled by the parameter of
                    // that name; one that is not stays `:name` in the
                    // address, and the value it was meant to carry goes as a
                    // query instead (`/users/:id?userId=42`).
                    let params: Vec<&str> =
                        endpoint.params.iter().map(|p| p.name.as_str()).collect();
                    for segment in endpoint.path.split('/') {
                        let Some(name) = segment.strip_prefix(':') else {
                            continue;
                        };
                        if !params.contains(&name) {
                            let hint = if params.is_empty() {
                                format!(
                                    "Give it the parameter: `{}({name}: String)`",
                                    endpoint.name
                                )
                            } else {
                                format!(
                                    "Name a parameter `{name}`; it takes {}",
                                    params
                                        .iter()
                                        .map(|p| format!("`{p}`"))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            };
                            cx.error(
                                endpoint.span,
                                "T10",
                                format!(
                                    "`{}.{}` is at `{}`, and no parameter fills `:{name}`",
                                    a.name, endpoint.name, endpoint.path
                                ),
                                &hint,
                            );
                        }
                    }
                }
                for hook in &a.hooks {
                    cx.push_scope();
                    if let Some(param) = &hook.param {
                        cx.bind(param, Type::Any, hook.span);
                    }
                    cx.returns.push(Vec::new());
                    cx.async_ok = true;
                    cx.in_handler += 1;
                    cx.statements(&hook.body, Body::Imperative);
                    cx.in_handler -= 1;
                    cx.async_ok = false;
                    cx.returns.pop();
                    cx.pop_scope();
                }
            }
            Declaration::Store(_)
            | Declaration::Theme(_)
            | Declaration::Enum(_)
            | Declaration::Script(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => {}
        }
    }
    info
}

/// What a `match` arm's binding is, for a write to it.
const ARM: &str = "a value a `match` arm binds";

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
    /// The names in each scope nothing may assign to, and what each is: a
    /// derived value, a prop, a loop variable. Parallel to `scopes`.
    fixed: Vec<HashMap<String, &'static str>>,
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
    /// Whether the body being checked compiles to an async function — an
    /// action, a handler, a timer, a hook — where `await` may be written.
    async_ok: bool,
    /// Whether a `derived` value is being checked, for what `await` there
    /// should be instead.
    in_derived: bool,
    /// How many handlers enclose what is being checked: `event` (`e`),
    /// and `value` and `key`, are names only there.
    in_handler: usize,
    /// Inside a `Header` or `Footer`: `page` and `pages` are names there.
    in_running: usize,
    /// The states whose declared type says what their values must be —
    /// `Number(1..=30)` — held to it at every assignment, not only the
    /// first.
    refined: HashMap<String, TypeRef>,
    /// The lists an enclosing condition proves are not empty — `if
    /// items.length > 0 { items[0] }` — by their text.
    non_empty: Vec<String>,
    /// Whether the expression being checked is a later arm of a `match`
    /// expression whose first arm has already been checked as a whole.
    in_chain: bool,
    /// The names some control in the body binds, which a `validate` block
    /// guards.
    bound: HashSet<String>,
    /// The body's own actions, by name, for what they write.
    own_actions: HashMap<String, Vec<Statement>>,
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
            fixed: Vec::new(),
            component: None,
            returns: Vec::new(),
            current_span: Span::dummy(),
            in_args: false,
            async_ok: false,
            in_derived: false,
            in_handler: 0,
            in_running: 0,
            refined: HashMap::new(),
            non_empty: Vec::new(),
            in_chain: false,
            bound: HashSet::new(),
            own_actions: HashMap::new(),
            source,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.fixed.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        self.fixed.pop();
    }

    /// Bind a name nothing may assign to, saying what it is.
    fn bind_fixed(&mut self, name: &str, ty: Type, span: Span, what: &'static str) {
        self.bind(name, ty, span);
        if let Some(fixed) = self.fixed.last_mut() {
            fixed.insert(name.to_string(), what);
        }
    }

    /// What `name` is, when it is something nothing may assign to: the
    /// innermost binding of it decides.
    fn fixed_kind(&self, name: &str) -> Option<&'static str> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return self.fixed.get(i).and_then(|f| f.get(name).copied());
            }
        }
        self.world.fixed.get(name).copied()
    }

    fn bind(&mut self, name: &str, ty: Type, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty.clone());
        }
        if let Some(fixed) = self.fixed.last_mut() {
            fixed.remove(name);
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
        // A narrowed name is still what it was: a derived value checked
        // for null is still not assignable.
        let what = self.fixed_kind(name);
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
        if let (Some(what), Some(fixed)) = (what, self.fixed.last_mut()) {
            fixed.insert(name.to_string(), what);
        }
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }

    fn warn(&mut self, code: &'static str, message: String, hint: &str) {
        let span = self.located(self.current_span, &message);
        let d = Diagnostic::coded(
            code,
            message,
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source.as_deref())
        .with_hint(hint);
        self.info.findings.warnings.push(d);
    }

    fn error(&mut self, span: Span, code: &'static str, message: String, hint: &str) {
        let d = Diagnostic::coded(
            code,
            message,
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source.as_deref())
        .with_hint(hint);
        self.info.findings.errors.push(d);
    }

    /// A `class:` value: a string, a map whose values are conditions, a list
    /// of strings and maps, or a value holding one of those.
    fn check_class_value(&mut self, value: &Expr, at: Span, element: &str) {
        match value {
            Expr::MapLiteral(pairs) => {
                for (key, on) in pairs {
                    if key == "..." {
                        self.infer(on, None);
                        continue;
                    }
                    let given = self.infer(on, Some(&Type::Bool));
                    if !given.unwrapped().assignable_to(&Type::Bool) {
                        self.error(
                            at,
                            "T01",
                            format!(
                                "`{}` in `class:` on `{element}` is `{given}`, but `Bool` is wanted",
                                key.trim_matches('"')
                            ),
                            "Each key of a `class:` map is a class, on while its value is true: `{ \"is-done\": todo.done }`",
                        );
                    }
                }
            }
            Expr::ListLiteral(items) => {
                for item in items {
                    self.check_class_value(item, at, element);
                }
            }
            _ => {
                let given = self.infer(value, Some(&Type::String));
                let fits = |t: &Type| {
                    matches!(
                        t,
                        Type::Any
                            | Type::String
                            | Type::Null
                            | Type::Map
                            | Type::Shape(_)
                            | Type::List(_)
                    )
                };
                // A bare `Bool` would paint the class `true`: a condition
                // belongs in a map, `{ "is-on": on }`.
                let ok = match &given {
                    Type::Optional(inner) => fits(inner),
                    other => fits(other),
                };
                if !ok {
                    self.error(
                        at,
                        "T01",
                        format!("`class:` on `{element}` is `{given}`, but a class name is wanted"),
                        "Give a string, a map of class to condition (`{ \"is-on\": on }`) or a list of those",
                    );
                }
            }
        }
    }

    /// Report `given` where `wanted` was expected, at `span`, naming `what`.
    fn expect(&mut self, given: &Type, wanted: &Type, span: Span, what: &str) {
        // A bare case fits an enum only when the enum has it: `.quiet`
        // given for a `Tone` that has `.calm` and `.loud` is never equal
        // to anything a `Tone` holds.
        if let (Type::Case(case), Type::Enum(e)) = (given, wanted.unwrapped())
            && let Some(decl) = self.world.enums.get(e.as_str())
            && !decl.case_names().contains(case)
        {
            self.error(
                span,
                "T02",
                format!("`{e}` has no case `.{case}`"),
                &format!(
                    "`{e}` takes {}",
                    decl.case_names()
                        .iter()
                        .map(|c| format!(".{c}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
            return;
        }
        if given.assignable_to(wanted) {
            return;
        }
        if let Type::Promise(inner) = given {
            self.error(
                span,
                "T17",
                format!("{what} is an async action's result, which is a promise of `{inner}` until it is awaited"),
                "`await` it inside an action or a handler: `let value = await load()`",
            );
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
                    if let Some(t) = &s.ty
                        && !t.refinement().is_empty()
                    {
                        self.refined.insert(s.name.clone(), t.clone());
                    }
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
                    self.bind_fixed(
                        &r.name,
                        Type::Resource(Box::new(inner)),
                        stmt.span,
                        "a `resource`",
                    );
                }
                StatementKind::Use(u) => {
                    self.bind_fixed(
                        &u.store_name,
                        Type::Store(u.store_name.clone()),
                        stmt.span,
                        "a store",
                    );
                }
                // `socket chat = ws(…)`, `peer link = rtc(…)`: the handle,
                // bound before the body so a handler may name another
                // connection declared after it.
                StatementKind::Connection(c) => {
                    self.bind_fixed(&c.name, Type::Any, stmt.span, "a connection");
                }
                _ => {}
            }
            // `ref: name` anywhere in the body declares `name`: a handle on
            // the element, read like the element itself.
            for name in ref_names(std::slice::from_ref(stmt)) {
                self.bind_fixed(&name, Type::Any, stmt.span, "an element's handle");
            }
            // `Form(bind: form)` declares `form`: its validity and values.
            for name in form_names(std::slice::from_ref(stmt)) {
                self.bind_fixed(&name, form_type(), stmt.span, "a form's handle");
            }
        }
        // Actions before derived values, and all of them before any is
        // inferred: a store's members are mutually visible, and the runtime
        // binds its actions first. Inferring a derived value in source order
        // reported an action declared below it as undeclared.
        for stmt in stmts {
            self.current_span = stmt.span;
            if let StatementKind::Action(a) = &stmt.kind {
                self.own_actions.insert(a.name.clone(), a.body.clone());
                let params: Vec<Type> = a
                    .params
                    .iter()
                    .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                    .collect();
                // The return type is found when the body is checked; an
                // action that awaits hands back a promise of it.
                let ret = if awaits(&a.body) {
                    Type::Promise(Box::new(Type::Any))
                } else {
                    Type::Any
                };
                self.bind_fixed(
                    &a.name,
                    Type::Func(params, Box::new(ret)),
                    stmt.span,
                    "an action",
                );
            }
        }
        for stmt in stmts {
            self.current_span = stmt.span;
            if let StatementKind::Derived(d) = &stmt.kind {
                let declared = d.ty.as_ref().map(|t| self.world.resolve(Type::from_ref(t)));
                let given = self.infer_quiet(&d.value);
                let ty = declared.unwrap_or(given);
                self.bind_fixed(&d.name, ty, stmt.span, "a `derived` value");
            }
        }
    }

    // ─── Statements ──────────────────────────────────────

    fn statements(&mut self, stmts: &[Statement], body: Body) {
        // What follows a `return` in the same block never runs.
        if let Some(at) = stmts
            .iter()
            .position(|s| matches!(s.kind, StatementKind::Return(_)))
            && let Some(next) = stmts.get(at + 1)
        {
            self.current_span = next.span;
            self.warn(
                "U06",
                "this never runs: the `return` above leaves first".to_string(),
                "Remove it, or move it above the `return`",
            );
        }
        for stmt in stmts {
            self.statement(stmt, body);
            // `if x == null { return }`: what follows runs only when `x`
            // is not null.
            if let StatementKind::If(i) = &stmt.kind
                && i.binding.is_none()
                && i.else_if_branches.is_empty()
                && i.else_body.is_none()
                && i.then_body
                    .last()
                    .is_some_and(|s| matches!(s.kind, StatementKind::Return(_)))
            {
                let names = null_names(&i.condition);
                self.narrow_all(&names);
            }
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
                // What `persist` keeps is written as JSON: a function, a
                // file or a promise does not survive it.
                if s.persist {
                    let kept = declared.clone().unwrap_or_else(|| given.clone());
                    if let Some(what) = not_serialisable(&kept) {
                        self.error(
                            span,
                            "D02",
                            format!("`persist {}` keeps {what}, which cannot be written to the browser's storage", s.name),
                            "Keep what it is made from — a name, an id, the data — and build it again when it is read",
                        );
                    }
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
                self.derived_assigns(&d.name, &d.value, span);
                let declared = d.ty.as_ref().map(|t| self.world.resolve(Type::from_ref(t)));
                self.in_derived = true;
                let given = self.infer(&d.value, declared.as_ref());
                self.in_derived = false;
                if let Some(declared) = &declared {
                    // A derived value says what it is; what it works out to
                    // must be that.
                    self.expect(&given, declared, span, &format!("`{}`", d.name));
                }
                self.narrow(&d.name, declared.unwrap_or(given));
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
                // The rules are shown on the control bound to the state, and
                // a form is valid only when they pass: with no control, they
                // never can, and the submit stays disabled.
                if self.fixed_kind(&v.name) == Some("a `derived` value") {
                    self.error(
                        span,
                        "F03",
                        format!(
                            "`validate {}` guards a `derived` value, which nobody types into",
                            v.name
                        ),
                        "Validate the state the reader edits; a derived value follows it",
                    );
                } else if self.lookup(&v.name).is_some() && !self.bound.contains(&v.name) {
                    self.error(
                        span,
                        "F03",
                        format!("`validate {}` guards a state no control binds, so its rules can never be met", v.name),
                        &format!("Bind it to the control the reader fills in: `Input(bind: {}, …)`", v.name),
                    );
                }
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
                        // An `async` rule's check is an async function: it
                        // may `await` the server.
                        let was = std::mem::replace(&mut self.async_ok, rule.name == "async");
                        let checked = self.infer(body, Some(&Type::Bool));
                        self.async_ok = was;
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
                if let Some(url) = &c.url {
                    let url = self.infer(url, Some(&Type::String));
                    self.expect(&url, &Type::String, span, "a connection's address");
                }
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
                    let was = std::mem::replace(&mut self.async_ok, true);
                    self.in_handler += 1;
                    self.statements(&handler.body, Body::Imperative);
                    self.in_handler -= 1;
                    self.async_ok = was;
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
                let was = std::mem::replace(&mut self.async_ok, true);
                self.statements(&a.body, Body::Imperative);
                self.async_ok = was;
                let returned = self.returns.pop().unwrap_or_default();
                self.pop_scope();
                let ret = returned
                    .into_iter()
                    .reduce(Type::join)
                    .unwrap_or(Type::Null);
                let ret = if awaits(&a.body) {
                    Type::Promise(Box::new(ret))
                } else {
                    ret
                };
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
                self.effect_feeds_itself(&e.body, span);
                self.push_scope();
                let was = std::mem::replace(&mut self.async_ok, false);
                self.statements(&e.body, Body::Imperative);
                self.statements(&e.cleanup, Body::Imperative);
                self.async_ok = was;
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
                let was = std::mem::replace(&mut self.async_ok, true);
                self.statements(&t.body, Body::Imperative);
                self.async_ok = was;
                self.pop_scope();
            }
            StatementKind::EventHandler(h) => {
                self.push_scope();
                if let Some(param) = &h.param {
                    self.bind(param, Type::Any, h.span);
                }
                let was = std::mem::replace(&mut self.async_ok, true);
                self.in_handler += 1;
                self.statements(&h.body, Body::Imperative);
                self.in_handler -= 1;
                self.async_ok = was;
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
                self.bind_fixed(&f.item, item, span, "a loop variable");
                if let Some(index) = &f.index {
                    self.bind_fixed(index, Type::Number, span, "a loop variable");
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
                self.constant_condition(&s.condition);
                self.statements(&s.body, body);
            }
            StatementKind::Match(m) => self.match_statement(m, span, body),
            StatementKind::Assignment(a) => {
                self.check_writable(&a.target, span);
                let target = match &a.target {
                    // A name nothing declares compiles to a signal's
                    // `.set`, `_name.set(…)`, which throws: `T13`.
                    Expr::Identifier(name) => match self.lookup(name) {
                        Some(ty) => ty,
                        None => {
                            if !self.is_known_name(name) {
                                self.unresolved(name, false);
                            }
                            Type::Any
                        }
                    },
                    other => self.infer(other, None),
                };
                let value = self.infer(&a.value, Some(&target));
                let what = format!("`{}`", expr_text(&a.target));
                self.expect(&value, &target, span, &what);
                // A value the checker can read is held to what the state's
                // type says its values must be, here as at its declaration.
                if let Expr::Identifier(name) = &a.target
                    && let Some(ty) = self.refined.get(name).cloned()
                    && let Some(fault) = refinement_fault(&ty, &a.value)
                {
                    self.error(
                        span,
                        "T01",
                        format!("`{name}` {fault}"),
                        "The type says what its values may be; this one is outside it",
                    );
                }
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
        let facts = self.non_empty.len();
        if i.binding.is_none() {
            self.non_empty.extend(non_empty_lists(&i.condition));
        }
        match &i.binding {
            Some(name) => {
                let ty = self.infer(&i.condition, None);
                self.bind_fixed(name, ty.unwrapped(), span, "a value `if let` binds");
            }
            None => {
                let ty = self.infer(&i.condition, Some(&Type::Bool));
                self.condition(&ty, &i.condition, span, "if");
                self.constant_condition(&i.condition);
                // `if x != null { }`, `if x { }`: `x` is not null inside.
                for name in narrowed_names(&i.condition) {
                    if let Some(Type::Optional(inner)) = self.lookup(&name) {
                        self.narrow(&name, *inner);
                    }
                }
            }
        }
        self.statements(&i.then_body, body);
        self.non_empty.truncate(facts);
        self.pop_scope();
        // A branch after `if x == null { … }` runs only when `x` is not
        // null: `else { x.title }` needs no unwrapping.
        let mut ruled_out: Vec<String> = if i.binding.is_none() {
            null_names(&i.condition)
        } else {
            Vec::new()
        };
        for (cond, branch) in &i.else_if_branches {
            self.push_scope();
            self.narrow_all(&ruled_out);
            let ty = self.infer(cond, Some(&Type::Bool));
            self.condition(&ty, cond, span, "else if");
            for name in narrowed_names(cond) {
                if let Some(Type::Optional(inner)) = self.lookup(&name) {
                    self.narrow(&name, *inner);
                }
            }
            self.statements(branch, body);
            self.pop_scope();
            ruled_out.extend(null_names(cond));
        }
        if let Some(b) = &i.else_body {
            self.push_scope();
            self.narrow_all(&ruled_out);
            self.statements(b, body);
            self.pop_scope();
        }
    }

    /// Narrow each of `names` out of its `null`, in the current scope.
    fn narrow_all(&mut self, names: &[String]) {
        for name in names {
            if let Some(Type::Optional(inner)) = self.lookup(name) {
                self.narrow(name, *inner);
            }
        }
    }

    fn match_statement(&mut self, m: &MatchStmt, span: Span, body: Body) {
        let scrutinee = self.infer(&m.scrutinee, None);
        let has_else = m.arms.iter().any(|a| matches!(a.pattern, ArmPattern::Else));
        // What the arms cover, and what they cover twice.
        let mut cases: Vec<String> = Vec::new();
        let mut dupes: Vec<String> = Vec::new();
        for arm in &m.arms {
            let key = match &arm.pattern {
                ArmPattern::Case(c) => c.clone(),
                ArmPattern::Loading => "loading".into(),
                ArmPattern::Error => "error".into(),
                ArmPattern::Ready => "ready".into(),
                ArmPattern::State(s) => s.clone(),
                ArmPattern::Else => continue,
            };
            if cases.contains(&key) {
                dupes.push(key);
            } else {
                cases.push(key);
            }
        }
        self.current_span = span;
        match scrutinee.unwrapped() {
            Type::Enum(_) => {
                self.match_coverage(
                    &expr_text(&m.scrutinee),
                    &scrutinee.unwrapped(),
                    &cases,
                    &dupes,
                    has_else,
                    true,
                );
            }
            Type::Resource(_) => {
                for state in &dupes {
                    self.error_at_current(
                        "T15",
                        format!(
                            "`{state}` has two arms in this `match`; the second is never reached"
                        ),
                        "Keep one of them",
                    );
                }
                if !has_else && !cases.iter().any(|c| c == "error") {
                    self.warn(
                        "T21",
                        format!(
                            "this `match` on `{}` has no `error` arm, so a failed request shows nothing",
                            expr_text(&m.scrutinee)
                        ),
                        "Add `error(e) { Alert(e.message).danger }`, or `else { … }`",
                    );
                }
            }
            _ => {}
        }
        match &scrutinee {
            Type::Resource(inner) => {
                for arm in &m.arms {
                    self.push_scope();
                    match (&arm.pattern, &arm.binding) {
                        (ArmPattern::Ready, Some(name)) => {
                            self.bind_fixed(name, (**inner).clone(), arm.span, ARM)
                        }
                        (ArmPattern::Error, Some(name)) => {
                            self.bind_fixed(name, Type::NetError, arm.span, ARM)
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
                        self.bind_fixed(name, Type::Any, arm.span, ARM);
                    }
                    if let ArmPattern::Case(case) = &arm.pattern {
                        let fields = self.payload_of(name, case).unwrap_or_default();
                        for (i, bound) in arm.bindings.iter().enumerate() {
                            let ty = fields.get(i).map(|(_, t)| t.clone()).unwrap_or(Type::Any);
                            self.bind_fixed(bound, ty, arm.span, ARM);
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
                        self.bind_fixed(name, Type::Any, arm.span, ARM);
                    }
                    for bound in &arm.bindings {
                        self.bind_fixed(bound, Type::Any, arm.span, ARM);
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
                        self.bind_fixed(name, bound, arm.span, ARM);
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
                        self.bind_fixed(name, Type::Any, arm.span, ARM);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
        }
    }

    /// A record built by name, `Todo(id: "1", title: "x")`: each value
    /// against its field's type, a field the record lacks, and a field it
    /// needs that is not given.
    fn record_fields_given(&mut self, record: &str, given: &[(&str, &Expr)], what: &str) {
        let Some(all) = self.world.record_fields(record) else {
            return;
        };
        let mut spread = false;
        for (key, value) in given {
            if *key == "..." {
                self.infer(value, None);
                spread = true;
                continue;
            }
            match all.iter().find(|f| f.name == *key) {
                Some(field) => {
                    let wanted = self.world.resolve(Type::from_ref(&field.ty));
                    let ty = self.infer(value, Some(&wanted));
                    if !ty.assignable_to(&wanted) {
                        self.error_at_current(
                            "T01",
                            format!(
                                "`{key}` of `{record}` is `{wanted}`, but `{}` is `{ty}`",
                                expr_text(value)
                            ),
                            "",
                        );
                    }
                }
                None => {
                    self.infer(value, None);
                }
            }
        }
        if !spread {
            let keys: Vec<String> = given.iter().map(|(k, _)| k.to_string()).collect();
            self.record_keys_given(record, &keys, what);
        }
    }

    /// The keys given for a record: one it does not have is `T05`, one it
    /// needs and does not get is `C01`.
    fn record_keys_given(&mut self, record: &str, keys: &[String], what: &str) {
        let Some(all) = self.world.record_fields(record) else {
            return;
        };
        let names: Vec<String> = all.iter().map(|f| format!("`{}`", f.name)).collect();
        for key in keys {
            if !all.iter().any(|f| &f.name == key) {
                let known: Vec<String> = all.iter().map(|f| f.name.clone()).collect();
                let known: Vec<&str> = known.iter().map(String::as_str).collect();
                self.error_naming(
                    "T05",
                    format!("`{record}` has no field `{key}`"),
                    &format!("Its fields are {}", names.join(", ")),
                    key,
                    &known,
                    false,
                );
            }
        }
        let missing: Vec<String> = all
            .iter()
            .filter(|f| {
                f.default.is_none()
                    && !matches!(f.ty, TypeRef::Optional(_))
                    && !keys.iter().any(|k| k == &f.name)
            })
            .map(|f| format!("`{}`", f.name))
            .collect();
        if !missing.is_empty() {
            self.error_at_current(
                "C01",
                format!(
                    "{what} leaves out {} of `{record}`, which {} no default",
                    missing.join(", "),
                    if missing.len() == 1 { "has" } else { "have" }
                ),
                "Give it, or declare a default in the type: `field: Type = value`",
            );
        }
    }

    /// An assignment's target must be something that can change: a state,
    /// a `persist`, a `let`, a store's state — not a constant, a derived
    /// value, a prop, a route parameter or a loop variable. At run time an
    /// assignment to one throws, or writes a copy nothing reads.
    fn check_writable(&mut self, target: &Expr, span: Span) {
        let (name, what) = match target {
            Expr::Identifier(name) => match self.fixed_kind(name) {
                Some(what) => (name.clone(), what),
                None => return,
            },
            Expr::PropertyAccess(base, member) => match base.as_ref() {
                Expr::Identifier(store)
                    if self
                        .lookup(store)
                        .is_none_or(|t| matches!(t, Type::Store(_))) =>
                {
                    match self
                        .world
                        .store_fixed
                        .get(store.as_str())
                        .and_then(|m| m.get(member))
                    {
                        Some(what) => (format!("{store}.{member}"), *what),
                        None => return,
                    }
                }
                _ => return,
            },
            _ => return,
        };
        let hint = match what {
            "a `const`" | "a `data` constant" | "an `image`" => {
                "Declare it `state` if it changes; a constant is the same on every page"
            }
            "a `derived` value" => {
                "It is worked out from what it reads: assign to that, or make it a `state`"
            }
            "a prop" => {
                "The caller owns it: keep a `state` of your own seeded from it, or `emit` an event so the caller changes it"
            }
            "a route parameter" => "The address owns it: `navigate` to the new address",
            "a loop variable" => {
                "Each pass gets its own copy: change the item through the list it came from"
            }
            "an action" => "Call it: `name()`",
            _ => "Hold what changes in a `state`",
        };
        let located = self.located(span, &format!("`{name}`"));
        self.error(
            located,
            "X01",
            format!("`{name}` is {what}, and nothing may assign to it"),
            hint,
        );
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
        self.component_args(&layout.args, &[], &[], component, layout.span);
    }

    // ─── Elements ────────────────────────────────────────

    fn element(&mut self, el: &UIElement, span: Span) {
        match &el.component {
            ComponentRef::UserDefined(name) => {
                if let Some(component) = self.world.components.get(name.as_str()) {
                    self.component_args(&el.args, &el.arg_spans, &el.modifiers, component, span);
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
                    self.component_args(&el.args, &el.arg_spans, &el.modifiers, component, span);
                } else {
                    let sig = registry::part(owner, part);
                    self.builtin_args(el, sig, span);
                }
            }
        }
        if let Some(style) = &el.style_block {
            for prop in &style.properties {
                self.current_span = prop.span;
                let ty = self.infer(&prop.value, None);
                // V07: a number spliced alone where CSS wants a length is
                // a length with no unit, which the browser drops.
                let bare = !matches!(
                    prop.value,
                    Expr::StringLiteral(_)
                        | Expr::InterpolatedString(_)
                        | Expr::NumberLiteral(_)
                        | Expr::Token(_)
                );
                if bare && ty.unwrapped() == Type::Number && takes_length(&prop.name) {
                    self.warn(
                        "V07",
                        format!(
                            "`{}: {{{}}}` is a number with no unit, which the browser drops",
                            prop.name,
                            expr_text(&prop.value)
                        ),
                        &format!(
                            "Give it one: `{}: {{{}}}px` (or `%`, `rem`)",
                            prop.name,
                            expr_text(&prop.value)
                        ),
                    );
                }
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
            let was = std::mem::replace(&mut self.async_ok, true);
            self.in_handler += 1;
            self.statements(&handler.body, Body::Imperative);
            self.in_handler -= 1;
            self.async_ok = was;
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
                self.bind_fixed(name, ty, fill.span, "a value a slot hands over");
            }
            self.statements(&fill.body, Body::Page);
            self.pop_scope();
        }
        // Inside a paged document's running element, `page` and `pages`
        // are the number of the page it is drawn on and the count.
        let running =
            matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Header" || n == "Footer");
        self.in_running += running as usize;
        self.statements(&el.children, Body::Page);
        self.in_running -= running as usize;
    }

    /// The arguments of a call to a user component, against its props.
    fn component_args(
        &mut self,
        args: &[Arg],
        spans: &[Span],
        modifiers: &[String],
        component: &ComponentDecl,
        span: Span,
    ) {
        self.required_props(args, modifiers, component, span);
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

    /// The names an action writes — its own body's assignments, and the
    /// store's state written as `Store.member` when it is a store's.
    fn action_writes(&self, call: &Expr) -> Option<Vec<String>> {
        // What the action assigns that is state, not its own locals — a
        // `let`, or a name a store's action assigns without one, which
        // the store's code declares as a local.
        let locals = |body: &[Statement]| -> HashSet<String> {
            fn walk(stmts: &[Statement], out: &mut HashSet<String>) {
                for s in stmts {
                    if let StatementKind::State(st) = &s.kind {
                        out.insert(st.name.clone());
                    }
                    for b in s.kind.bodies() {
                        walk(b, out);
                    }
                }
            }
            let mut out = HashSet::new();
            walk(body, &mut out);
            out
        };
        match call {
            Expr::FunctionCall(name, _) if self.fixed_kind(name) == Some("an action") => {
                let body = self.own_actions.get(name)?;
                let own = locals(body);
                Some(
                    assigned(body, false)
                        .into_iter()
                        .filter(|n| !own.contains(n) && self.lookup(n).is_some())
                        .collect(),
                )
            }
            Expr::MethodCall(obj, method, _) => match obj.as_ref() {
                Expr::Identifier(store) => {
                    let body = self.world.store_actions.get(store.as_str())?.get(method)?;
                    let own = locals(body);
                    let members = self.world.stores.get(store.as_str());
                    let fixed = self.world.store_fixed.get(store.as_str());
                    Some(
                        assigned(body, false)
                            .into_iter()
                            .filter(|n| {
                                !own.contains(n)
                                    && members.is_some_and(|m| m.contains_key(n))
                                    && !fixed.is_some_and(|f| f.contains_key(n))
                            })
                            .map(|n| format!("{store}.{n}"))
                            .collect(),
                    )
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// X02: an effect that writes, every time it runs, a value it reads.
    /// Writing it runs the effect again, which writes it again: the page
    /// never settles, and the browser overflows its stack.
    fn effect_feeds_itself(&mut self, body: &[Statement], span: Span) {
        let locals: HashSet<String> = body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::State(st) => Some(st.name.clone()),
                _ => None,
            })
            .collect();
        let mut writes: Vec<String> = assigned(body, true)
            .into_iter()
            .filter(|n| !locals.contains(n) && self.lookup(n).is_some())
            .collect();
        // What the actions it calls, unconditionally, write.
        for stmt in body {
            let call = match &stmt.kind {
                StatementKind::ExprStatement(e) => Some(e.clone()),
                StatementKind::MethodCall(mc) => Some(Expr::MethodCall(
                    Box::new(mc.object.clone()),
                    mc.method.clone(),
                    mc.args.clone(),
                )),
                _ => None,
            };
            if let Some(call) = call
                && let Some(w) = self.action_writes(&call)
            {
                writes.extend(w);
            }
        }
        let reads = read_names(body);
        if let Some(name) = writes.iter().find(|w| reads.contains(*w)) {
            self.error(
                span,
                "X02",
                format!("this effect writes `{name}`, which it also reads, so each run starts the next and the page never settles"),
                "Work the value out with `derived` instead, or write it from the action that changes what it reads",
            );
        }
    }

    /// X04: a `derived` value that calls an action that assigns. Working a
    /// value out must not change anything — it runs whenever what it reads
    /// changes, as often as the page needs it.
    fn derived_assigns(&mut self, name: &str, value: &Expr, span: Span) {
        fn calls(e: &Expr, out: &mut Vec<Expr>) {
            if matches!(e, Expr::FunctionCall(..) | Expr::MethodCall(..)) {
                out.push(e.clone());
            }
            for child in e.children() {
                calls(child, out);
            }
        }
        let mut found = Vec::new();
        calls(value, &mut found);
        for call in found {
            if let Some(writes) = self.action_writes(&call)
                && let Some(w) = writes.first()
            {
                self.error(
                    span,
                    "X04",
                    format!("`{name}` is worked out by calling `{}`, which assigns `{w}`", expr_text(&call)),
                    "A derived value only reads; call the action from a handler, and derive from what it sets",
                );
                return;
            }
        }
    }

    /// F01: `bind:` writes what the control holds back to what it names,
    /// so it names something that can change: a state, a store's state, a
    /// field of a loop's item — not a constant, a derived value, a prop or
    /// a value worked out on the spot.
    fn check_bindable(&mut self, value: &Expr, at: Span, control: &str) -> bool {
        let refused = match value {
            Expr::Identifier(n) => self.fixed_kind(n).map(|what| format!("`{n}` is {what}")),
            Expr::PropertyAccess(base, member) => match base.as_ref() {
                Expr::Identifier(store) => self
                    .world
                    .store_fixed
                    .get(store.as_str())
                    .and_then(|m| m.get(member))
                    .map(|what| format!("`{store}.{member}` is {what}")),
                _ => None,
            },
            Expr::OptionalProperty(..) | Expr::IndexAccess(..) | Expr::OptionalIndex(..) => None,
            other => Some(format!(
                "`{}` is a value, not something to write to",
                expr_text(other)
            )),
        };
        let Some(why) = refused else {
            return true;
        };
        self.error(
            at,
            "F01",
            format!("`bind:` on `{control}` writes back to what it names, and {why}"),
            "Bind a `state` (or a store's state, or a field of a loop's item); show a value with `value:`",
        );
        false
    }

    /// C01: a prop with no default that a call does not give — by name,
    /// positionally, or as a flag (`.active` for a `Bool`, `.loud` for a
    /// case of an enum-typed prop). In the component it would be
    /// `undefined` where its declaration promises a value.
    fn required_props(
        &mut self,
        args: &[Arg],
        modifiers: &[String],
        component: &ComponentDecl,
        span: Span,
    ) {
        let positional = component
            .props
            .iter()
            .find(|p| p.positional)
            .or(component.props.first())
            .map(|p| p.name.as_str());
        let given: Vec<&str> = args
            .iter()
            .filter_map(|a| match a {
                Arg::Named(k, _) => Some(k.as_str()),
                Arg::Positional(_) => positional,
            })
            .collect();
        let by_flag = |prop: &PropDecl| {
            modifiers.iter().any(|m| {
                m == &prop.name
                    || matches!(&prop.prop_type, TypeRef::Named(e)
                        if self.world.enums.get(e.as_str()).is_some_and(|d| d.case_names().contains(m)))
            })
        };
        let missing: Vec<String> = component
            .props
            .iter()
            .filter(|p| {
                p.default.is_none()
                    && !p.optional
                    && !matches!(
                        p.prop_type,
                        TypeRef::Bool | TypeRef::Optional(_) | TypeRef::Any
                    )
                    && !given.contains(&p.name.as_str())
                    && !by_flag(p)
            })
            .map(|p| format!("`{}`", p.name))
            .collect();
        if missing.is_empty() {
            return;
        }
        // The fix: each missing prop, with a value of its type to replace.
        let filled: Vec<String> = component
            .props
            .iter()
            .filter(|p| missing.contains(&format!("`{}`", p.name)))
            .map(|p| format!("{}: {}", p.name, self.placeholder(&p.prop_type)))
            .collect();
        self.error(
            span,
            "C01",
            format!(
                "`{}` is placed without {}, which {} no default",
                component.name,
                missing.join(", "),
                if missing.len() == 1 { "has" } else { "have" }
            ),
            &format!(
                "Pass {}, or give the prop a default in `component {}(…)`",
                missing.join(", "),
                component.name
            ),
        );
        self.plan_last(
            format!("Pass {}", missing.join(", ")),
            crate::diagnostics::fixes::Plan::AddArgument {
                text: filled.join(", "),
            },
        );
    }

    /// A value of `ty` for a fix to write where one is missing: the empty
    /// one where there is one, an enum's first case.
    fn placeholder(&self, ty: &TypeRef) -> String {
        match ty {
            TypeRef::String => "\"\"".to_string(),
            TypeRef::Number => "0".to_string(),
            TypeRef::Bool => "false".to_string(),
            TypeRef::Map => "{}".to_string(),
            TypeRef::List(_) => "[]".to_string(),
            TypeRef::Optional(_) | TypeRef::Any => "null".to_string(),
            TypeRef::Refined(inner, _) => self.placeholder(inner),
            TypeRef::Named(name) => match self.world.enums.get(name.as_str()) {
                Some(e) => e
                    .case_names()
                    .first()
                    .map(|c| format!(".{c}"))
                    .unwrap_or_else(|| "null".to_string()),
                None => format!("{name}()"),
            },
        }
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
                && !matches!(arg, Arg::Named(k, _) if k == "class")
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
                && self.infer_quiet(value).unwrapped() == Type::Scalar(Scalar::Secret)
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
            match (prop.ty, prop.name) {
                // `class:` — a string, `{ "is-on": cond }`, or a list of either.
                (_, "class") => self.check_class_value(value, at, &name),
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
                    let refused = name != "Form" && !self.check_bindable(value, at, &name);
                    let wanted = if refused {
                        None
                    } else {
                        bound_type(&name, &el.modifiers)
                    };
                    let given = self.infer(value, wanted.as_ref());
                    if let Some(wanted) = wanted {
                        // A state that starts as `null` may hold the value later.
                        let given = match given {
                            Type::Optional(inner) => *inner,
                            other => other,
                        };
                        if !given.assignable_to(&wanted) {
                            let hint = match (&wanted, &given) {
                                (Type::String, Type::Number) if name == "Input" => {
                                    "A number field is `Input(…).number`, which holds a number"
                                        .to_string()
                                }
                                (Type::Number, Type::String) => {
                                    "Hold text in a `String` state, or drop the `.number`"
                                        .to_string()
                                }
                                _ => format!("`{name}` holds a `{wanted}`"),
                            };
                            self.error(
                                at,
                                "F02",
                                format!(
                                    "`bind:` on `{name}` holds a `{wanted}`, but `{}` is `{given}`",
                                    expr_text(value)
                                ),
                                &hint,
                            );
                        }
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
                        // A list, a map or a record in text shows as
                        // `[object Object]` or its items run together.
                        if matches!(
                            ty.unwrapped(),
                            Type::List(_)
                                | Type::Map
                                | Type::Shape(_)
                                | Type::Record(_)
                                | Type::Func(..)
                        ) {
                            let shown = if matches!(ty.unwrapped(), Type::List(_)) {
                                "its items run together with commas"
                            } else {
                                "`[object Object]`"
                            };
                            self.warn(
                                "T20",
                                format!(
                                    "`{}` is `{}`, and in text it shows as {shown}",
                                    expr_text(e),
                                    ty.unwrapped()
                                ),
                                "Show a field of it, or `.join(\", \")` a list of text",
                            );
                        }
                        // A value that may be null shows as "null" (or
                        // "undefined") in the text.
                        if let Type::Optional(inner) = &ty
                            && !inner.is_any()
                        {
                            self.warn(
                                "T19",
                                format!(
                                    "`{}` may be null, and in text it would show as `null`",
                                    expr_text(e)
                                ),
                                &format!(
                                    "Say what to show instead: `{{{} ?? \"\"}}`",
                                    expr_text(e)
                                ),
                            );
                        }
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
                if let (Some(scalar), Expr::StringLiteral(text)) =
                    (Scalar::of_name(name), carrier.as_ref())
                    && let Some(shape) = ill_formed(scalar, text)
                {
                    self.error_at_current("T01", format!("`@{text}` is not a `{name}`"), &shape);
                }
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
                .unwrap_or_else(|| {
                    if !self.is_known_name(name) {
                        self.unresolved(name, false);
                    }
                    global_type(name)
                }),
            // A plain access after a `?.` in the same chain is short-circuited
            // with it: `a?.b.c` is null when `a` is, never a fault.
            Expr::PropertyAccess(base, field) if in_optional_chain(base) => {
                // `a?.b.c`: the `?.` covers `a` being null, not `b`. When
                // `b` may itself be null, `.c` may fail.
                let (base_ty, _) = self.chain(base);
                if let Type::Optional(_) = base_ty {
                    self.error_at_current(
                        "T04",
                        format!(
                            "`{}` may be null even when the `?.` before it is not, so `.{field}` may fail",
                            expr_text(base)
                        ),
                        &format!("Read it through null too: `{}?.{field}`", expr_text(base)),
                    );
                }
                Type::optional(self.property(&base_ty.unwrapped(), base, field))
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
                // `todos[0]` is nothing when the list is empty: the item
                // may be null, unless a condition around it says the list
                // has items.
                let fixed_index = matches!(**index, Expr::NumberLiteral(_));
                let ty = match base_ty.unwrapped() {
                    Type::List(inner)
                        if fixed_index && !self.non_empty.contains(&expr_text(base)) =>
                    {
                        Type::optional(*inner)
                    }
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
                // A case on one side is read against the other side's enum:
                // `tone == .quiet` asks the `Tone` enum about `.quiet`.
                let lt = self.infer(l, None);
                let rt = match (op, lt.unwrapped()) {
                    (BinOp::Eq | BinOp::Neq, Type::Enum(_)) if matches!(**r, Expr::EnumCase(_)) => {
                        Type::Case(match &**r {
                            Expr::EnumCase(c) => c.clone(),
                            _ => unreachable!(),
                        })
                    }
                    _ => self.infer(r, None),
                };
                match op {
                    BinOp::Add => {
                        for (side, ty) in [(l, &lt), (r, &rt)] {
                            if ty.unwrapped() == Type::Scalar(Scalar::Secret) {
                                self.error_at_current(
                                    "T12",
                                    format!("`{}` is a `Secret`, and `+` would put it in text", expr_text(side)),
                                    "A secret must not be shown, logged or put in a URL; send it as a value",
                                );
                            }
                        }
                        if lt.unwrapped() == Type::String || rt.unwrapped() == Type::String {
                            Type::String
                        } else if lt == Type::Number && rt == Type::Number {
                            Type::Number
                        } else {
                            for (side, ty) in [(l, &lt), (r, &rt)] {
                                if !adds(ty) {
                                    self.arithmetic_fault("+", side, ty);
                                }
                            }
                            Type::Any
                        }
                    }
                    BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        let sign = match op {
                            BinOp::Sub => "-",
                            BinOp::Mul => "*",
                            BinOp::Div => "/",
                            _ => "%",
                        };
                        for (side, ty) in [(l, &lt), (r, &rt)] {
                            if !counts(ty) {
                                self.arithmetic_fault(sign, side, ty);
                            }
                        }
                        Type::Number
                    }
                    BinOp::Eq | BinOp::Neq => {
                        self.comparison(l, &lt, r, &rt, matches!(op, BinOp::Eq));
                        Type::Bool
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => Type::Bool,
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
                // A map written where a record is wanted is that record: a
                // key it does not have, or a field it needs and lacks, is
                // a fault here, where it is written.
                if !spread
                    && let Some(Type::Record(record)) = expected.map(|t| t.unwrapped())
                    && self.world.record_fields(&record).is_some()
                {
                    let keys: Vec<String> = shape.iter().map(|(k, _)| k.clone()).collect();
                    self.record_keys_given(&record, &keys, "the map");
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
                if self.world.record_fields(name).is_some() {
                    let given: Vec<(&str, &Expr)> =
                        fields.iter().map(|(k, v)| (k.as_str(), v)).collect();
                    self.record_fields_given(name, &given, &format!("`{name}(…)`"));
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
                if !self.async_ok {
                    let (message, hint) = if self.in_derived {
                        (
                            "a `derived` value cannot `await`: it is worked out at once, whenever what it reads changes",
                            "Fetch it with a `resource`, which is loading until the answer arrives and follows what its address reads",
                        )
                    } else {
                        (
                            "`await` is written in an action, a handler, a timer or a service's hook, and this is none of them",
                            "Move the call into an `action` and call that; to show data as it arrives, use a `resource`",
                        )
                    };
                    self.error_at_current("T17", message.to_string(), hint);
                }
                let ty = self.infer(e, None);
                match ty {
                    Type::Resource(inner) | Type::Promise(inner) => *inner,
                    other => other,
                }
            }
        }
    }

    fn property(&mut self, base_ty: &Type, base: &Expr, field: &str) -> Type {
        if let Type::Promise(inner) = base_ty {
            self.error_at_current(
                "T17",
                format!(
                    "`{}` is an async action's result, a promise of `{inner}`; it has no `{field}` until it is awaited",
                    expr_text(base)
                ),
                "`await` it inside an action or a handler: `(await load()).field`",
            );
            return Type::Any;
        }
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
                    let known: Vec<String> = self
                        .world
                        .apis
                        .get(name.as_str())
                        .map(|a| a.endpoints.iter().map(|e| e.name.clone()).collect())
                        .unwrap_or_default();
                    let known: Vec<&str> = known.iter().map(String::as_str).collect();
                    self.error_naming(
                        "T06",
                        format!("`{name}` has no endpoint `{field}`"),
                        &format!("Its endpoints are {}", names.join(", ")),
                        field,
                        &known,
                        true,
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
                // A secret is not a string: reading it apart is how it
                // leaks a character at a time.
                (Scalar::Secret, _) => {
                    self.error_at_current(
                        "T12",
                        format!("`{}` is a `Secret`, which has no `{field}`: it is not read apart", expr_text(base)),
                        "Hand it on whole — to `api` headers, or a request body — and read what it unlocks",
                    );
                    Type::Any
                }
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
                    let known: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                    self.error_naming(
                        "T05",
                        format!("`{}` has no field `{field}`", expr_text(base)),
                        &format!("It was written with {}", names.join(", ")),
                        field,
                        &known,
                        true,
                    );
                    Type::Any
                }
            },
            Type::Record(name) => match self.world.record_field(name, field) {
                Some(ty) => self.world.resolve(ty),
                None => {
                    let known: Vec<String> = self
                        .world
                        .record_fields(name)
                        .map(|all| all.iter().map(|f| f.name.clone()).collect())
                        .unwrap_or_default();
                    let fields: Vec<String> = known.iter().map(|f| format!("`{f}`")).collect();
                    let known: Vec<&str> = known.iter().map(String::as_str).collect();
                    self.error_naming(
                        "T05",
                        format!("`{name}` has no field `{field}`"),
                        &format!("Its fields are {}", fields.join(", ")),
                        field,
                        &known,
                        true,
                    );
                    Type::Any
                }
            },
            Type::Store(name) => match self.world.stores.get(name.as_str()) {
                Some(members) => match members.get(field) {
                    Some(ty) => ty.clone(),
                    None => {
                        let mut names: Vec<String> = members.keys().cloned().collect();
                        names.sort();
                        let known: Vec<&str> = names.iter().map(String::as_str).collect();
                        let listed: Vec<String> = names.iter().map(|n| format!("`{n}`")).collect();
                        self.error_naming(
                            "T06",
                            format!("`{name}` has no member `{field}`"),
                            &format!("Its members are {}", listed.join(", ")),
                            field,
                            &known,
                            true,
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
                self.plan_last(
                    format!("Read it through null: `?.{field}`"),
                    crate::diagnostics::fixes::Plan::Rename {
                        from: format!(".{field}"),
                        to: format!("?.{field}"),
                    },
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
            // `save.pending`: whether a call of the action is under way —
            // which is only ever for an action that awaits something.
            Type::Func(_, ret) => match field {
                "pending" => {
                    let is_action = match base {
                        Expr::Identifier(n) => self.fixed_kind(n) == Some("an action"),
                        Expr::PropertyAccess(store, member) => matches!(store.as_ref(),
                            Expr::Identifier(s) if self.world.store_fixed.get(s.as_str())
                                .and_then(|m| m.get(member)).copied() == Some("an action")),
                        _ => false,
                    };
                    if is_action && !matches!(**ret, Type::Promise(_)) {
                        self.error_at_current(
                            "T17",
                            format!(
                                "`{}` awaits nothing, so `.pending` is always false",
                                expr_text(base)
                            ),
                            "`.pending` is true while a call of an async action runs; an action that awaits nothing finishes before the page repaints",
                        );
                    }
                    Type::Bool
                }
                // `Backend.avatar.progress`: how far an upload has got, 0 to 1
                // — a service's endpoint's, which an action does not have.
                "progress"
                    if matches!(base, Expr::PropertyAccess(owner, _)
                        if matches!(owner.as_ref(), Expr::Identifier(n) if self.world.apis.contains_key(n.as_str()))) =>
                {
                    Type::Number
                }
                _ => {
                    self.error_at_current(
                        "T05",
                        format!(
                            "`{}` is an action; it has no field `{field}`",
                            expr_text(base)
                        ),
                        "An action has `pending`, true while a call of it runs; a service's endpoint also has `progress`",
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
        // The head of a `match` expression's arms is checked as a whole:
        // every case covered, none twice.
        if (method == "__if" || method == "__iflet") && !self.in_chain {
            self.check_match_chain(obj, method, args);
        }
        if method == "__exhaustive" {
            return Type::Never;
        }
        if method == "__iflet"
            && args.len() == 2
            && let Expr::Lambda(name, then_expr) = &args[0]
        {
            let value_ty = self.infer(obj, None);
            self.push_scope();
            self.narrow(name, value_ty.unwrapped());
            let was = std::mem::replace(&mut self.in_chain, false);
            let then_ty = self.infer(then_expr, None);
            self.pop_scope();
            self.in_chain = match_arm(&args[1]).is_some();
            let else_ty = self.infer(&args[1], None);
            self.in_chain = was;
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
            let was = std::mem::replace(&mut self.in_chain, false);
            let then_ty = self.infer(&args[0], None);
            self.pop_scope();
            self.in_chain = match_arm(&args[1]).is_some();
            let else_ty = self.infer(&args[1], None);
            self.in_chain = was;
            return Type::join(then_ty, else_ty);
        }
        // A secret handed to the browser itself — logged, stored, written
        // into the document — is a secret anyone at the keyboard can read.
        if let Expr::Identifier(global) = obj
            && SECRET_SINKS.contains(&global.as_str())
            && self.lookup(global).is_none()
        {
            self.secret_args(&format!("{global}.{method}"), args);
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
                                    // `cache`, `on`, `timeout`, `signal` and
                                    // `paginate` are the call's own, not the
                                    // service's.
                                    None if !matches!(
                                        key,
                                        "cache"
                                            | "on"
                                            | "timeout"
                                            | "signal"
                                            | "headers"
                                            | "as"
                                            | "retry"
                                            | "paginate"
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
                    let known: Vec<String> = self
                        .world
                        .apis
                        .get(api.as_str())
                        .map(|a| a.endpoints.iter().map(|e| e.name.clone()).collect())
                        .unwrap_or_default();
                    let known: Vec<&str> = known.iter().map(String::as_str).collect();
                    self.error_naming(
                        "T06",
                        format!("`{api}` has no endpoint `{method}`"),
                        &format!("Its endpoints are {}", names.join(", ")),
                        method,
                        &known,
                        true,
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
        if let Type::Promise(inner) = &obj_ty
            && !matches!(method, "then" | "catch" | "finally")
        {
            for a in args {
                self.infer(a, None);
            }
            self.error_at_current(
                "T17",
                format!(
                    "`{}` is an async action's result, a promise of `{inner}`; it has no `{method}` until it is awaited",
                    expr_text(obj)
                ),
                "`await` it inside an action or a handler",
            );
            return Type::Any;
        }
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
                        if let Some(members) = self.world.stores.get(name.as_str()) {
                            let mut known: Vec<String> = members.keys().cloned().collect();
                            known.sort();
                            let known: Vec<&str> = known.iter().map(String::as_str).collect();
                            self.error_naming(
                                "T06",
                                format!("`{name}` has no action `{method}`"),
                                "",
                                method,
                                &known,
                                true,
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
                // JavaScript hands each item, its index and the list itself.
                let lambda = |ret: Type| {
                    Type::Func(
                        vec![item.clone(), Type::Number, Type::list(item.clone())],
                        Box::new(ret),
                    )
                };
                match method {
                    "map" => {
                        let f = self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        let ret = match f {
                            Type::Func(_, ret) => *ret,
                            _ => Type::Any,
                        };
                        Type::list(ret)
                    }
                    "filter" | "slice" | "concat" | "reverse" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, Some(&lambda(Type::Bool)));
                        }
                        Type::list(item)
                    }
                    // A comparator is handed two elements, not an element and
                    // an index: `(a, b) => a.age - b.age`.
                    "sort" => {
                        self.infer_arg(
                            args,
                            0,
                            Some(&Type::Func(
                                vec![item.clone(), item.clone()],
                                Box::new(Type::Number),
                            )),
                        );
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
                                vec![init.clone(), item.clone(), Type::Number, Type::list(item)],
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
                        if !LIST_METHODS.contains(&method) {
                            self.unknown_method(&obj_ty, method, LIST_METHODS);
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
                    m if STRING_METHODS.contains(&m) => Type::Any,
                    _ => {
                        self.unknown_method(&obj_ty, method, STRING_METHODS);
                        Type::Any
                    }
                }
            }
            // The language's own types: a date's arithmetic, money's, a
            // URL's parts, a colour's mixing.
            Type::Scalar(Scalar::Secret) => {
                for a in args {
                    self.infer(a, None);
                }
                self.error_at_current(
                    "T12",
                    format!(
                        "`{}` is a `Secret`, which has no `{method}()`: it is not read or changed",
                        expr_text(obj)
                    ),
                    "Hand it on whole — to `api` headers, or a request body",
                );
                Type::Any
            }
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
                    "toFixed" | "toString" | "toPrecision" | "toExponential" | "toLocaleString" => {
                        Type::String
                    }
                    "valueOf" => Type::Number,
                    _ => {
                        self.unknown_method(&obj_ty, method, NUMBER_METHODS);
                        Type::Any
                    }
                }
            }
            Type::Bool => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "toString" => Type::String,
                    "valueOf" => Type::Bool,
                    _ => {
                        self.unknown_method(&obj_ty, method, &["toString", "valueOf"]);
                        Type::Any
                    }
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

    /// The type of a step of a `?.` chain as if the chain had not been
    /// cut short — what the step itself may be — and whether a `?.` in it
    /// may cut it short.
    fn chain(&mut self, expr: &Expr) -> (Type, bool) {
        match expr {
            Expr::OptionalProperty(base, field) => {
                let (base_ty, _) = self.chain(base);
                (self.property(&base_ty.unwrapped(), base, field), true)
            }
            Expr::PropertyAccess(base, field) if in_optional_chain(base) => {
                let (base_ty, short) = self.chain(base);
                if let Type::Optional(_) = base_ty {
                    self.error_at_current(
                        "T04",
                        format!(
                            "`{}` may be null even when the `?.` before it is not, so `.{field}` may fail",
                            expr_text(base)
                        ),
                        &format!("Read it through null too: `{}?.{field}`", expr_text(base)),
                    );
                }
                (self.property(&base_ty.unwrapped(), base, field), short)
            }
            other => {
                let ty = self.infer(other, None);
                let short = in_optional_chain(other);
                if short {
                    (ty.unwrapped(), true)
                } else {
                    (ty, false)
                }
            }
        }
    }

    /// A `match` expression's arms, read back from what it lowers to: every
    /// case of the enum covered when there is no `else`, no case twice, and
    /// no `else` that no value reaches.
    fn check_match_chain(&mut self, obj: &Expr, method: &str, args: &[Expr]) {
        let head = Expr::MethodCall(Box::new(obj.clone()), method.to_string(), args.to_vec());
        let Some((subject, _, _)) = match_arm(&head) else {
            return;
        };
        let subject_text = expr_text(subject);
        let mut cases: Vec<String> = Vec::new();
        let mut dupes: Vec<String> = Vec::new();
        let mut at = &head;
        let fallback = loop {
            match match_arm(at) {
                Some((s, case, rest)) if expr_text(s) == subject_text => {
                    if cases.contains(&case) {
                        dupes.push(case);
                    } else {
                        cases.push(case);
                    }
                    at = rest;
                }
                _ => break at,
            }
        };
        let has_else = !matches!(fallback, Expr::MethodCall(_, m, _) if m == "__exhaustive");
        let subject_ty = self.infer_quiet(subject).unwrapped();
        self.match_coverage(&subject_text, &subject_ty, &cases, &dupes, has_else, false);
    }

    /// What a `match` — statement or expression — covers of an enum.
    fn match_coverage(
        &mut self,
        subject: &str,
        ty: &Type,
        cases: &[String],
        dupes: &[String],
        has_else: bool,
        statement: bool,
    ) {
        for case in dupes {
            self.error_at_current(
                "T15",
                format!("`.{case}` has two arms in this `match`; the second is never reached"),
                "Keep one of them",
            );
        }
        let Type::Enum(name) = ty else {
            if !has_else {
                self.error_at_current(
                    "T15",
                    format!("this `match` has no `else`, and nothing says `{subject}` is an enum whose every case it covers"),
                    "Add `else { … }`, or declare its type so the cases can be counted",
                );
            }
            return;
        };
        let all = self
            .world
            .enums
            .get(name.as_str())
            .map(|e| e.case_names())
            .unwrap_or_default();
        let missing: Vec<String> = all
            .iter()
            .filter(|c| !cases.contains(c))
            .map(|c| format!("`.{c}`"))
            .collect();
        if !has_else && !missing.is_empty() {
            self.error_at_current(
                "T15",
                format!(
                    "this `match` on `{subject}` has no arm for {}, and no `else`",
                    missing.join(", ")
                ),
                "Add an arm for each, or `else { … }` for the rest",
            );
            // An arm for each, empty: a statement's draws nothing, an
            // expression's is `null`; a payload is named by its fields.
            let arms: Vec<String> = all
                .iter()
                .filter(|c| !cases.contains(c))
                .map(|c| {
                    let names = self
                        .payload_of(name, c)
                        .filter(|fields| !fields.is_empty())
                        .map(|fields| {
                            let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                            format!("({})", names.join(", "))
                        })
                        .unwrap_or_default();
                    let body = if statement { "{ }" } else { "{ null }" };
                    format!(".{c}{names} {body}")
                })
                .collect();
            self.plan_last(
                format!("Add an arm for {}", missing.join(", ")),
                crate::diagnostics::fixes::Plan::AddArms { arms },
            );
        } else if has_else && missing.is_empty() && !all.is_empty() {
            self.warn(
                "U10",
                format!("every case of `{name}` has its own arm, so this `match`'s `else` is never reached"),
                "Remove the `else`",
            );
        }
    }

    /// U08: a condition that is always the same — a literal, or a value
    /// compared with itself — so one branch never runs.
    fn constant_condition(&mut self, cond: &Expr) {
        let always = match cond {
            Expr::BoolLiteral(b) => Some(if *b { "true" } else { "false" }),
            Expr::NumberLiteral(n) => Some(if *n != 0.0 { "true" } else { "false" }),
            Expr::StringLiteral(t) => Some(if t.is_empty() { "false" } else { "true" }),
            Expr::Null => Some("false"),
            Expr::BinaryOp(l, op, r)
                if matches!(
                    op,
                    BinOp::Eq | BinOp::Neq | BinOp::Lte | BinOp::Gte | BinOp::Lt | BinOp::Gt
                ) && expr_text(l) == expr_text(r)
                    && !matches!(**l, Expr::FunctionCall(..) | Expr::MethodCall(..)) =>
            {
                Some(if matches!(op, BinOp::Eq | BinOp::Lte | BinOp::Gte) {
                    "true"
                } else {
                    "false"
                })
            }
            _ => None,
        };
        if let Some(always) = always {
            self.warn(
                "U08",
                format!(
                    "`{}` is always {always}, so one branch never runs",
                    expr_text(cond)
                ),
                "Use the value that decides, or remove the branch that cannot run",
            );
        }
    }

    /// `==` or `!=` between values that can never be equal: a string and a
    /// number (`"1" == 1` is `false` in the compiled code), a record and a
    /// string, or an enum and a case it does not have. The comparison is
    /// always the same, so it is almost always a mistake.
    fn comparison(&mut self, l: &Expr, lt: &Type, r: &Expr, rt: &Type, eq: bool) {
        let (a, b) = (lt.unwrapped(), rt.unwrapped());
        // An enum against a case it does not have.
        for (e, c) in [(&a, &b), (&b, &a)] {
            if let (Type::Enum(name), Type::Case(case)) = (e, c)
                && let Some(decl) = self.world.enums.get(name.as_str())
                && !decl.case_names().contains(case)
            {
                self.error_at_current(
                    "T14",
                    format!(
                        "`{name}` has no case `.{case}`, so this is always {}",
                        if eq { "false" } else { "true" }
                    ),
                    &format!(
                        "`{name}` takes {}",
                        decl.case_names()
                            .iter()
                            .map(|c| format!(".{c}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
                return;
            }
        }
        if disjoint(&a, &b) {
            let hint = match (&a, &b) {
                (Type::String, Type::Number) | (Type::Number, Type::String) => {
                    "Convert one side: `Number(text) == n`, or `\"{n}\" == text`"
                }
                _ => "Compare values of one type",
            };
            self.error_at_current(
                "T14",
                format!(
                    "`{}` is `{a}` and `{}` is `{b}`, which are never equal, so this is always {}",
                    expr_text(l),
                    expr_text(r),
                    if eq { "false" } else { "true" }
                ),
                hint,
            );
        }
    }

    /// Arithmetic on something that is not a number: `"a" - 1` is `NaN`,
    /// `items * 2` is `NaN`, and `list + 1` is a string nobody wanted.
    fn arithmetic_fault(&mut self, sign: &str, side: &Expr, ty: &Type) {
        self.error_at_current(
            "T18",
            format!(
                "`{sign}` takes numbers, but `{}` is `{ty}`",
                expr_text(side)
            ),
            match ty.unwrapped() {
                Type::String => {
                    "Convert it: `Number(value)`; `+` joins text when one side is a string"
                }
                Type::List(_) => "Use its length, or `sum()` its numbers",
                _ => "Use a number field of it",
            },
        );
    }

    /// A method a number, a string or a list does not have: in the browser,
    /// `x.method is not a function`, the first time the line runs.
    fn unknown_method(&mut self, on: &Type, method: &str, known: &[&str]) {
        let hint = if crate::diagnostics::fixes::nearest(method, known.iter().copied()).is_some() {
            String::new()
        } else {
            let shown: Vec<String> = known.iter().take(12).map(|k| format!("`{k}`")).collect();
            format!("It has {}, …", shown.join(", "))
        };
        self.error_naming(
            "T16",
            format!("a `{on}` has no method `{method}`"),
            &hint,
            method,
            known,
            true,
        );
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
            Some(e) => {
                let ty = self.infer(e, expected);
                // A function handed to a list's method takes no more
                // arguments than the method gives it: `sort` hands two.
                if let (Type::Func(given, _), Some(Type::Func(offered, _))) = (&ty, expected)
                    && given.len() > offered.len()
                {
                    self.error_at_current(
                        "T10",
                        format!(
                            "`{}` takes {} argument{}, but it is given {}",
                            expr_text(e),
                            given.len(),
                            if given.len() == 1 { "" } else { "s" },
                            offered.len()
                        ),
                        "Name only the arguments the method passes",
                    );
                }
                ty
            }
            None => Type::Any,
        }
    }

    /// T12 for each argument that is a `Secret`, handed to `to`.
    fn secret_args(&mut self, to: &str, args: &[Expr]) {
        for arg in args {
            if self.infer_quiet(arg).unwrapped() == Type::Scalar(Scalar::Secret) {
                self.error_at_current(
                    "T12",
                    format!("`{}` is a `Secret`, and `{to}` would give it to anyone who opens the page", expr_text(arg)),
                    "A secret goes to the server only — an `api`'s headers, a request body — never into the browser's own hands",
                );
            }
        }
    }

    fn function_call(&mut self, name: &str, args: &[Expr]) -> Type {
        if self.lookup(name).is_none() {
            match name {
                "alert" | "confirm" | "prompt" => self.secret_args(name, args),
                // In a request's address it is in every log the request
                // passes; as a header or body it is where it belongs.
                "fetch" => self.secret_args("a fetch's address", &args[..args.len().min(1)]),
                _ => {}
            }
        }
        if self.lookup(name).is_none()
            && let Some((file, script)) = self.world.scripts.get(name).copied()
        {
            return self.script_call(file, script, args);
        }
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
                if !self.is_known_name(name) && !BUILT_IN_FUNCTIONS.contains(&name) {
                    self.unresolved(name, true);
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

    /// A call to a function or class a project script declares: as many
    /// arguments as it takes — fewer where the last have defaults, any
    /// number past a `...rest`.
    fn script_call(
        &mut self,
        file: &str,
        script: &crate::project_js::scan::Name,
        args: &[Expr],
    ) -> Type {
        use crate::project_js::scan::NameKind;
        let name = script.name.as_str();
        let params = match &script.kind {
            NameKind::Function { params, .. } | NameKind::Class { params, .. } => params,
            NameKind::Value => {
                // A value a script declares may hold a function the scanner
                // could not see; the browser decides.
                for a in args {
                    self.infer(a, None);
                }
                return Type::Any;
            }
        };
        let documented = script
            .doc
            .as_deref()
            .map(crate::project_js::jsdoc::parse)
            .unwrap_or_default();
        // A default in the code or `[name]` in the comment: either says a
        // call may leave it out.
        let optional = |p: &crate::project_js::scan::Param| {
            p.optional || documented.param(&p.name).is_some_and(|d| d.optional)
        };
        let rest = params.last().is_some_and(|p| p.rest);
        let fixed = params.iter().filter(|p| !p.rest).count();
        let required = params
            .iter()
            .rposition(|p| !optional(p) && !p.rest)
            .map_or(0, |i| i + 1);
        if args.len() < required || (!rest && args.len() > fixed) {
            let takes = match (required == fixed, rest) {
                (true, false) => format!("{fixed}"),
                (false, false) => format!("{required} to {fixed}"),
                (_, true) => format!("at least {required}"),
            };
            let shape = params
                .iter()
                .map(|p| {
                    if p.rest {
                        format!("...{}", p.name)
                    } else if optional(p) {
                        format!("{}?", p.name)
                    } else {
                        p.name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            self.error_at_current(
                "T10",
                format!(
                    "`{name}` takes {takes} argument{}, but {} {} given",
                    if takes == "1" { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "is" } else { "are" }
                ),
                &format!(
                    "It is `{name}({shape})`, at {file}:{}:{}",
                    script.line, script.col
                ),
            );
        }
        // What its doc comment says each parameter is, and what it gives
        // back; without one, everything is `Any`.
        let doc = script
            .doc
            .as_deref()
            .map(crate::project_js::jsdoc::parse)
            .unwrap_or_default();
        for (i, arg) in args.iter().enumerate() {
            let param = params.get(i).or_else(|| params.last().filter(|p| p.rest));
            let wanted = param
                .and_then(|p| doc.param(&p.name))
                .and_then(|d| d.ty.as_deref())
                .map(jsdoc_type)
                .unwrap_or(Type::Any);
            let given = self.infer(arg, Some(&wanted));
            if !given.assignable_to(&wanted) {
                self.error_at_current(
                    "T01",
                    format!(
                        "`{}` of `{name}` is `{wanted}`, but `{}` is `{given}`",
                        param.map(|p| p.name.as_str()).unwrap_or("an argument"),
                        expr_text(arg)
                    ),
                    &format!(
                        "As its doc comment says, at {file}:{}:{}",
                        script.line, script.col
                    ),
                );
            }
        }
        match &script.kind {
            // A class's call is the instance, which the checker takes as
            // given.
            NameKind::Class { .. } => Type::Any,
            _ => doc.returns.as_deref().map(jsdoc_type).unwrap_or(Type::Any),
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
    /// Whether `name` means something without the program declaring it:
    /// the browser's globals and values, the names a handler, a loop or a
    /// route brings with it, a service, a store or an imported module.
    fn is_known_name(&self, name: &str) -> bool {
        crate::codegen::js::BROWSER_GLOBALS.contains(&name)
            || crate::codegen::js::BROWSER_VALUES.contains(&name)
            || (self.in_handler > 0 && matches!(name, "event" | "e" | "value" | "key"))
            || (self.in_running > 0 && matches!(name, "page" | "pages"))
            || matches!(
                name,
                "params"
                    | "env"
                    | "locale"
                    | "dir"
                    | "undefined"
                    | "NaN"
                    | "Infinity"
                    | "globalThis"
                    | "this"
            )
            || self.world.scripts.contains_key(name)
            || self.world.apis.contains_key(name)
            || self.world.stores.contains_key(name)
            || self.world.consts.contains_key(name)
            || self.world.components.contains_key(name)
    }

    /// `name` resolves to nothing: recorded, for the build to refuse.
    fn unresolved(&mut self, name: &str, called: bool) {
        let message = if called {
            format!("`{name}(…)` calls a function nothing declares")
        } else {
            format!("nothing declares `{name}`")
        };
        let span = self.located(self.current_span, &message);
        // What the name may have meant: anything in scope here, or declared
        // at the top of the program.
        let mut known: Vec<&str> = self
            .scopes
            .iter()
            .flat_map(|scope| scope.keys().map(String::as_str))
            .collect();
        known.extend(self.world.consts.keys().map(|k| &**k));
        known.extend(self.world.stores.keys().map(|k| &**k));
        known.extend(self.world.apis.keys().map(|k| &**k));
        known.extend(self.world.scripts.keys().map(|k| &**k));
        let near =
            crate::diagnostics::fixes::nearest(name, known.iter().copied()).map(str::to_string);
        let declare = "Declare it — a `state`, a `const`, an `action`, a prop — or check the spelling. In the browser it would be a ReferenceError";
        let mut d = Diagnostic::coded(
            "T13",
            message,
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source.as_deref())
        .with_hint(match &near {
            Some(n) => format!("Did you mean `{n}`? {declare}"),
            None => declare.to_string(),
        });
        if let Some(n) = near {
            d = d.with_plan(
                format!("Change to `{n}`"),
                crate::diagnostics::fixes::Plan::Rename {
                    from: name.to_string(),
                    to: n,
                },
            );
        }
        if !self
            .info
            .unresolved
            .iter()
            .any(|u| u.message == d.message && u.line == d.line)
        {
            self.info.unresolved.push(d);
        }
    }

    /// The type of `e`, reporting nothing: a probe of a value the caller
    /// goes on to check properly, which used to report each finding in it
    /// once per probe.
    fn infer_quiet(&mut self, e: &Expr) -> Type {
        let errors = self.info.findings.errors.len();
        let warnings = self.info.findings.warnings.len();
        let unresolved = self.info.unresolved.len();
        let ty = self.infer(e, None);
        self.info.findings.errors.truncate(errors);
        self.info.findings.warnings.truncate(warnings);
        self.info.unresolved.truncate(unresolved);
        ty
    }

    fn error_at_current(&mut self, code: &'static str, message: String, hint: &str) {
        let span = self.located(self.current_span, &message);
        self.error(span, code, message, hint);
    }

    /// A fix for the error just reported.
    fn plan_last(&mut self, title: String, plan: crate::diagnostics::fixes::Plan) {
        if let Some(d) = self.info.findings.errors.pop() {
            self.info.findings.errors.push(d.with_plan(title, plan));
        }
    }

    /// An error about a name that is wrong — `.nmae`, `Cart.totl`, `cuont`:
    /// the nearest of `known` is suggested, and offered as a fix. `dotted`
    /// names a member, matched with the `.` before it.
    fn error_naming(
        &mut self,
        code: &'static str,
        message: String,
        hint: &str,
        wrong: &str,
        known: &[&str],
        dotted: bool,
    ) {
        let near = crate::diagnostics::fixes::nearest(wrong, known.iter().copied());
        let hint = match near {
            Some(n) if hint.is_empty() => format!("Did you mean `{n}`?"),
            Some(n) => format!("Did you mean `{n}`? {hint}"),
            None => hint.to_string(),
        };
        let span = self.located(self.current_span, &message);
        let mut d = Diagnostic::coded(
            code,
            message,
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source.as_deref())
        .with_hint(hint);
        if let Some(n) = near {
            let (from, to) = if dotted {
                (format!(".{wrong}"), format!(".{n}"))
            } else {
                (wrong.to_string(), n.to_string())
            };
            d = d.with_plan(
                format!("Change to `{n}`"),
                crate::diagnostics::fixes::Plan::Rename { from, to },
            );
        }
        self.info.findings.errors.push(d);
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

/// Whether a value of `ty` takes part in `-`, `*`, `/` and `%`: a number,
/// a duration (milliseconds), or something the checker cannot see.
fn counts(ty: &Type) -> bool {
    matches!(
        ty.unwrapped(),
        Type::Number | Type::Any | Type::Scalar(Scalar::Duration) | Type::Null
    )
}

/// Whether a value of `ty` takes part in a `+` that is not a string's: as
/// `counts`, or a string, which `+` joins.
fn adds(ty: &Type) -> bool {
    counts(ty)
        || ty.unwrapped() == Type::String
        || matches!(ty.unwrapped(), Type::Scalar(s) if s.is_text())
}

/// Whether no value of `a` can equal a value of `b`.
fn disjoint(a: &Type, b: &Type) -> bool {
    fn family(t: &Type) -> Option<u8> {
        Some(match t {
            Type::String | Type::Scalar(_) => 0,
            Type::Number => 1,
            Type::Bool => 2,
            Type::List(_) => 3,
            Type::Record(_) | Type::Shape(_) | Type::Map => 4,
            Type::Enum(_) | Type::Case(_) => 0,
            _ => return None,
        })
    }
    // A duration is a number, a text scalar a string, an enum's case a
    // string at run time.
    let a_fam = match a {
        Type::Scalar(Scalar::Duration) => Some(1),
        Type::Scalar(Scalar::Money) | Type::Scalar(Scalar::File) => Some(4),
        other => family(other),
    };
    let b_fam = match b {
        Type::Scalar(Scalar::Duration) => Some(1),
        Type::Scalar(Scalar::Money) | Type::Scalar(Scalar::File) => Some(4),
        other => family(other),
    };
    match (a_fam, b_fam) {
        (Some(x), Some(y)) => x != y,
        _ => false,
    }
}

/// Whether a CSS property takes a length, which a bare number is not.
fn takes_length(name: &str) -> bool {
    matches!(
        name,
        "width"
            | "height"
            | "min-width"
            | "max-width"
            | "min-height"
            | "max-height"
            | "top"
            | "left"
            | "right"
            | "bottom"
            | "inset"
            | "gap"
            | "row-gap"
            | "column-gap"
            | "font-size"
            | "border-radius"
            | "radius"
            | "border-width"
            | "outline-width"
            | "outline-offset"
            | "letter-spacing"
            | "word-spacing"
            | "text-indent"
            | "flex-basis"
            | "inline-size"
            | "block-size"
    ) || name.starts_with("margin")
        || name.starts_with("padding")
}

/// The browser's own objects a secret must not be handed to.
const SECRET_SINKS: &[&str] = &[
    "console",
    "localStorage",
    "sessionStorage",
    "document",
    "window",
    "navigator",
    "history",
];

/// Every method a list has: the browser's own, and the helpers the
/// runtime adds (`sortBy`, `groupBy`, `unique`, `take`, `first`, `last`,
/// `sum`, `remove`, `contains`).
const LIST_METHODS: &[&str] = &[
    "map",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "some",
    "every",
    "includes",
    "indexOf",
    "lastIndexOf",
    "join",
    "slice",
    "concat",
    "reverse",
    "sort",
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "reduce",
    "reduceRight",
    "forEach",
    "flat",
    "flatMap",
    "fill",
    "copyWithin",
    "entries",
    "keys",
    "values",
    "at",
    "with",
    "toReversed",
    "toSorted",
    "toSpliced",
    "toString",
    "toLocaleString",
    "sortBy",
    "groupBy",
    "unique",
    "take",
    "first",
    "last",
    "sum",
    "remove",
    "contains",
];

/// Every method a string has: the browser's own and the runtime's
/// (`capitalize`, `truncate`, `dedent`, `lines`, `words`).
const STRING_METHODS: &[&str] = &[
    "toLowerCase",
    "toUpperCase",
    "toLocaleLowerCase",
    "toLocaleUpperCase",
    "trim",
    "trimStart",
    "trimEnd",
    "replace",
    "replaceAll",
    "slice",
    "substring",
    "substr",
    "charAt",
    "charCodeAt",
    "codePointAt",
    "at",
    "toString",
    "valueOf",
    "padStart",
    "padEnd",
    "repeat",
    "normalize",
    "concat",
    "indexOf",
    "lastIndexOf",
    "search",
    "localeCompare",
    "includes",
    "startsWith",
    "endsWith",
    "split",
    "match",
    "matchAll",
    "isWellFormed",
    "toWellFormed",
    "capitalize",
    "truncate",
    "dedent",
    "lines",
    "words",
    "contains",
    "toUpper",
    "toLower",
];

/// Every method a number has.
const NUMBER_METHODS: &[&str] = &[
    "toFixed",
    "toString",
    "toPrecision",
    "toExponential",
    "toLocaleString",
    "valueOf",
];

/// The type a control's `bind:` state must hold: an `Input` by its type
/// flag — a number field a number, any other its text.
fn bound_type(component: &str, modifiers: &[String]) -> Option<Type> {
    match component {
        "Checkbox" | "Switch" => Some(Type::Bool),
        "Slider" => Some(Type::Number),
        // A date picker picks a date, which is a `Date` — not a string
        // that happens to look like one. Empty, it holds nothing.
        "DatePicker" => Some(Type::optional(Type::Scalar(Scalar::Date))),
        "Input" if modifiers.iter().any(|m| m == "number") => Some(Type::Number),
        "Input" | "Textarea" => Some(Type::String),
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

/// The names `stmts` assign to: `x = …`. With `top`, only the assignments
/// every run makes — not those under an `if`, a loop or a `match`.
fn assigned(stmts: &[Statement], top: bool) -> Vec<String> {
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
fn read_names(stmts: &[Statement]) -> HashSet<String> {
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
fn bound_names(stmts: &[Statement]) -> HashSet<String> {
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

/// One arm of a lowered `match` expression: its subject, its case, and
/// the rest of the arms after it.
fn match_arm(expr: &Expr) -> Option<(&Expr, String, &Expr)> {
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

/// What in `ty` the browser's storage cannot keep as JSON, said for a
/// finding.
fn not_serialisable(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Func(..) => Some("a function"),
        Type::Scalar(Scalar::File) => Some("a `File`"),
        Type::Promise(_) => Some("a promise"),
        Type::Resource(_) => Some("a resource"),
        Type::Optional(inner) | Type::List(inner) => not_serialisable(inner),
        Type::Shape(fields) => fields.iter().find_map(|(_, t)| not_serialisable(t)),
        _ => None,
    }
}

/// The names a condition is true for only when they are null: `x ==
/// null`, `!x`, either side of an `||`.
fn null_names(cond: &Expr) -> Vec<String> {
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
fn non_empty_lists(cond: &Expr) -> Vec<String> {
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
/// The functions the language gives a program, which it calls by name.
pub(crate) const BUILT_IN_FUNCTIONS: &[&str] = &[
    "log",
    "navigate",
    "format",
    "ago",
    "t",
    "setLocale",
    "setTheme",
    "uuid",
    "sanitize",
    "fetch",
    "optimistic",
    "beacon",
    "animate",
    "replayAnimation",
    "every",
    "after",
    "now",
    "ws",
    "sse",
    "broadcast",
    "rtc",
    "String",
    "Number",
    "Boolean",
    "Bool",
    "Money",
];

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
            // The writes kept for when the connection returns (`offline.sync`).
            ("queued".to_string(), Type::Number),
        ]),
        // A new version of the site, installed and waiting (`offline`).
        "update" => Type::Shape(vec![
            ("available".to_string(), Type::Bool),
            (
                "apply".to_string(),
                Type::Func(Vec::new(), Box::new(Type::Null)),
            ),
        ]),
        _ => Type::Any,
    }
}

/// What `Form(bind: form)` binds: whether every control is valid, the
/// values by field name, and `reset()`.
/// What a JSDoc type expression means here. What the checker has no word
/// for — `HTMLElement`, a class of the page's own, a generic it does not
/// know — is `Any`, which agrees with everything, so a comment can only
/// ever narrow a call, never break a correct one.
pub fn jsdoc_type(text: &str) -> Type {
    /// Split `text` at `sep` where no bracket is open.
    fn split_top(text: &str, sep: char) -> Vec<&str> {
        let mut parts = Vec::new();
        let (mut depth, mut start) = (0i32, 0);
        let mut prev = '\0';
        for (i, c) in text.char_indices() {
            match c {
                '(' | '[' | '{' | '<' => depth += 1,
                // `=>` closes nothing.
                '>' if prev == '=' => {}
                ')' | ']' | '}' | '>' => depth -= 1,
                _ if c == sep && depth == 0 => {
                    parts.push(&text[start..i]);
                    start = i + c.len_utf8();
                }
                _ => {}
            }
            prev = c;
        }
        parts.push(&text[start..]);
        parts
    }
    /// The index just past the bracket closing the one `text` opens with.
    fn close_of(text: &str) -> Option<usize> {
        let mut depth = 0;
        for (i, c) in text.char_indices() {
            match c {
                '(' | '[' | '{' | '<' => depth += 1,
                ')' | ']' | '}' | '>' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                }
                _ => {}
            }
        }
        None
    }
    /// `name: T` / `name?: T` / `T`, as a parameter or a field.
    fn field(text: &str) -> (String, bool, Type) {
        let text = text.trim();
        match split_top(text, ':').as_slice() {
            [name, ty] => {
                let optional = name.trim().ends_with('?');
                let name = name.trim().trim_end_matches('?').trim().to_string();
                (name, optional, jsdoc_type(ty))
            }
            _ => (String::new(), false, jsdoc_type(text)),
        }
    }

    let t = text.trim();
    let t = t.strip_prefix("...").unwrap_or(t).trim();
    if t.is_empty() {
        return Type::Any;
    }
    if let Some(inner) = t.strip_prefix('?') {
        return Type::optional(jsdoc_type(inner));
    }
    let t = t.strip_prefix('!').unwrap_or(t);
    let alternatives = split_top(t, '|');
    if alternatives.len() > 1 {
        let (nulls, others): (Vec<&str>, Vec<&str>) = alternatives
            .iter()
            .map(|a| a.trim())
            .partition(|a| matches!(*a, "null" | "undefined"));
        let joined = match others.as_slice() {
            [one] => jsdoc_type(one),
            // `string | number`: the language has no union to say it with.
            _ => Type::Any,
        };
        return if nulls.is_empty() {
            joined
        } else {
            Type::optional(joined)
        };
    }
    // `(a: T) => R`, or `(T)` for grouping.
    if t.starts_with('(')
        && let Some(end) = close_of(t)
    {
        let rest = t[end..].trim();
        if let Some(ret) = rest.strip_prefix("=>") {
            let inside = &t[1..end - 1];
            let params = if inside.trim().is_empty() {
                Vec::new()
            } else {
                split_top(inside, ',')
                    .into_iter()
                    .map(|p| field(p).2)
                    .collect()
            };
            return Type::Func(params, Box::new(jsdoc_type(ret)));
        }
        if rest.is_empty() {
            return jsdoc_type(&t[1..end - 1]);
        }
    }
    // `function(T, U): R`
    if let Some(after) = t.strip_prefix("function")
        && after.trim_start().starts_with('(')
    {
        let after = after.trim_start();
        if let Some(end) = close_of(after) {
            let inside = &after[1..end - 1];
            let params = if inside.trim().is_empty() {
                Vec::new()
            } else {
                split_top(inside, ',')
                    .into_iter()
                    .map(|p| field(p).2)
                    .collect()
            };
            let ret = after[end..]
                .trim()
                .strip_prefix(':')
                .map_or(Type::Any, jsdoc_type);
            return Type::Func(params, Box::new(ret));
        }
    }
    // `{ a: T, b?: U }`: a shape of the fields written; a method in it
    // (`destroy(): void`) is a field the checker takes as given.
    if t.starts_with('{') && t.ends_with('}') {
        let fields = split_top(&t[1..t.len() - 1], ',')
            .into_iter()
            .filter(|f| !f.trim().is_empty())
            .map(|f| {
                let f = f.trim();
                if let Some(paren) = f.find('(')
                    && f[..paren]
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                {
                    return (f[..paren].to_string(), Type::Any);
                }
                let (name, optional, ty) = field(f);
                (name, if optional { Type::optional(ty) } else { ty })
            })
            .filter(|(name, _)| !name.is_empty())
            .collect::<Vec<_>>();
        return Type::Shape(fields);
    }
    if let Some(element) = t.strip_suffix("[]") {
        return Type::list(jsdoc_type(element));
    }
    // `Name<T>` and Closure's `Name.<T>`.
    if let Some(open) = t.find('<')
        && t.ends_with('>')
    {
        let name = t[..open].trim_end_matches('.');
        let args = split_top(&t[open + 1..t.len() - 1], ',');
        return match (name, args.as_slice()) {
            ("Array", [element]) => Type::list(jsdoc_type(element)),
            // What a call awaits is the value; an action awaits it.
            ("Promise", [value]) => jsdoc_type(value),
            ("Object" | "Record" | "Map", _) => Type::Map,
            _ => Type::Any,
        };
    }
    match t {
        "string" | "String" => Type::String,
        "number" | "Number" => Type::Number,
        "boolean" | "Boolean" | "bool" => Type::Bool,
        "object" | "Object" => Type::Map,
        "Array" | "array" => Type::list(Type::Any),
        "void" | "undefined" | "null" => Type::Null,
        _ => Type::Any,
    }
}

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
/// The rules a `validate` block may hold, for a tool to offer;
/// `rule_wants` reads each.
pub const VALIDATE_RULES: &[&str] = &[
    "required",
    "email",
    "url",
    "minLength",
    "maxLength",
    "min",
    "max",
    "pattern",
    "matches",
    "oneOf",
    "custom",
    "async",
];

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
            let shaped = p.len() == 3
                && p[0].len() == 4
                && p[1].len() == 2
                && p[2].len() == 2
                && p.iter().all(|s| s.chars().all(|c| c.is_ascii_digit()));
            if shaped {
                // A day the calendar has: `2026-02-30` is not one.
                let (y, m, d): (u32, u32, u32) = (
                    p[0].parse().unwrap_or(0),
                    p[1].parse().unwrap_or(0),
                    p[2].parse().unwrap_or(0),
                );
                let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
                let days = match m {
                    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                    4 | 6 | 9 | 11 => 30,
                    2 if leap => 29,
                    2 => 28,
                    _ => 0,
                };
                if days == 0 || d == 0 || d > days {
                    return Some(if days == 0 {
                        format!("There is no month `{}`: months are `01` to `12`", p[1])
                    } else {
                        format!("{} has {days} days", month_name(m))
                    });
                }
            }
            shaped
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
fn month_name(m: u32) -> &'static str {
    [
        "",
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ][m as usize]
}

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

/// The plain types a program writes, beside its own and [`Scalar::ALL`].
pub const PRIMITIVE_TYPES: &[&str] = &["String", "Number", "Bool", "Map", "Any"];

/// One thing a value of some type has, as a tool offers it after `value.`:
/// a field (`price.amount`) or a method (`due.plus(…)`).
#[derive(Debug, Clone, PartialEq)]
pub struct Member {
    pub name: String,
    pub method: bool,
    /// What it gives back, where the checker knows.
    pub ty: Option<Type>,
}

/// Every name a scalar's methods may take, held to [`scalar_method`] for
/// each scalar by [`members`]; a test keeps it in step with the methods the
/// code generator routes (`codegen::js::SCALAR_METHODS`).
const SCALAR_MEMBER_NAMES: &[&str] = &[
    "year",
    "month",
    "day",
    "weekday",
    "hour",
    "minute",
    "second",
    "native",
    "date",
    "time",
    "inZone",
    "startOfDay",
    "startOfWeek",
    "startOfMonth",
    "endOfDay",
    "isBefore",
    "isAfter",
    "isSame",
    "until",
    "plus",
    "minus",
    "days",
    "hours",
    "minutes",
    "seconds",
    "ms",
    "times",
    "convert",
    "host",
    "path",
    "query",
    "with",
    "domain",
    "mix",
    "lighten",
    "darken",
    "alpha",
    "contrast",
    "preview",
];

/// What a value of type `ty` has after a dot — the same tables and rules
/// the checker holds a program to, so an editor offers exactly what the
/// build accepts. A record's fields, a store's members and a service's
/// endpoints need the program, and are the caller's to add.
pub fn members(ty: &Type) -> Vec<Member> {
    let field = |name: &str, ty: Type| Member {
        name: name.to_string(),
        method: false,
        ty: Some(ty),
    };
    let method = |name: &str, ty: Option<Type>| Member {
        name: name.to_string(),
        method: true,
        ty,
    };
    let text = |out: &mut Vec<Member>| {
        out.push(field("length", Type::Number));
        out.extend(STRING_METHODS.iter().map(|m| method(m, string_method(m))));
    };
    let ty = match ty {
        Type::Optional(inner) => inner.as_ref(),
        other => other,
    };
    let mut out = Vec::new();
    match ty {
        Type::List(_) => {
            out.push(field("length", Type::Number));
            out.extend(LIST_METHODS.iter().map(|m| method(m, None)));
        }
        Type::String => text(&mut out),
        Type::Number => out.extend(NUMBER_METHODS.iter().map(|m| method(m, Some(Type::String)))),
        Type::Scalar(scalar) => {
            match scalar {
                Scalar::Money => {
                    out.push(field("amount", Type::Number));
                    out.push(field("currency", Type::String));
                }
                Scalar::File => {
                    out.push(field("name", Type::String));
                    out.push(field("size", Type::Number));
                    out.push(field("type", Type::String));
                }
                _ => {}
            }
            // Exactly what `scalar_method` types: its own methods, and — for
            // one carried by a string — the string methods it allows.
            if *scalar != Scalar::Secret {
                let strings: &[&'static str] = if scalar.is_text() {
                    STRING_METHODS
                } else {
                    &[]
                };
                let mut seen: Vec<&str> = Vec::new();
                for m in SCALAR_MEMBER_NAMES
                    .iter()
                    .chain(strings)
                    .chain(["length"].iter())
                {
                    if seen.contains(m) {
                        continue;
                    }
                    seen.push(m);
                    if let Some(ty) = scalar_method(*scalar, m) {
                        out.push(if *m == "length" {
                            field("length", ty)
                        } else {
                            method(m, Some(ty))
                        });
                    }
                }
            }
        }
        Type::Shape(fields) => {
            for (name, ty) in fields {
                out.push(Member {
                    name: name.clone(),
                    method: matches!(ty, Type::Func(..)),
                    ty: Some(ty.clone()),
                });
            }
        }
        Type::Resource(inner) => {
            out.push(field("state", Type::String));
            out.push(field("data", Type::optional((**inner).clone())));
            out.push(field("error", Type::optional(Type::NetError)));
            out.push(field("items", Type::list(Type::Any)));
            out.push(field("hasMore", Type::Bool));
            out.push(method("reload", None));
            out.push(method("loadMore", None));
            out.push(method("invalidate", None));
            out.push(method("cancel", None));
        }
        Type::NetError => {
            out.push(field("message", Type::String));
            out.push(field("status", Type::Number));
            out.push(field("kind", Type::String));
            out.push(field("body", Type::Any));
            out.push(field("headers", Type::Map));
        }
        _ => {}
    }
    out
}

/// What an `image` name is: the asset the build made of the picture —
/// `media::Asset::as_json`, which a test holds to this.
pub fn image_type() -> Type {
    Type::Shape(vec![
        ("src".to_string(), Type::String),
        ("width".to_string(), Type::Number),
        ("height".to_string(), Type::Number),
        ("color".to_string(), Type::String),
        ("placeholder".to_string(), Type::String),
        ("srcset".to_string(), Type::String),
        ("sources".to_string(), Type::list(Type::Map)),
    ])
}

/// The type of a name every program can read — `viewport`, `network`,
/// `now` — for a tool to describe and complete.
pub fn browser_value_type(name: &str) -> Option<Type> {
    crate::codegen::js::BROWSER_VALUES
        .contains(&name)
        .then(|| global_type(name))
}

/// The functions the language gives a program, which it calls by name.
pub fn built_in_functions() -> &'static [&'static str] {
    BUILT_IN_FUNCTIONS
}

#[cfg(test)]
mod tests {

    #[test]
    fn every_type_name_a_tool_offers_is_one_the_checker_reads() {
        for s in Scalar::ALL {
            assert_eq!(Scalar::of_name(s.name()), Some(s));
        }
        for name in PRIMITIVE_TYPES {
            let src = format!("page P(path: \"/\") {{ state x: {name}? = null\n Text(\"a\") }}");
            let program = crate::syntax::parse_source(&src, "t.wf").unwrap();
            assert!(
                check(&program, &|_| String::new())
                    .findings
                    .errors
                    .is_empty(),
                "{name}"
            );
        }
    }

    #[test]
    fn every_rule_a_tool_offers_is_one_the_checker_reads() {
        for rule in VALIDATE_RULES {
            assert!(
                rule_wants(rule).is_some(),
                "`{rule}` is offered and not a rule"
            );
        }
    }

    #[test]
    fn members_are_the_checker_s_own_tables() {
        let names = |ty: Type| -> Vec<&'static str> {
            members(&ty)
                .into_iter()
                .map(|m| &*Box::leak(m.name.into_boxed_str()))
                .collect()
        };
        let list = names(Type::list(Type::Number));
        for m in [
            "length", "map", "sortBy", "groupBy", "unique", "take", "first", "sum",
        ] {
            assert!(list.contains(&m), "a list's `{m}`: {list:?}");
        }
        let text = names(Type::String);
        for m in [
            "capitalize",
            "truncate",
            "dedent",
            "lines",
            "words",
            "toLowerCase",
        ] {
            assert!(text.contains(&m), "a string's `{m}`: {text:?}");
        }
        let date = names(Type::Scalar(Scalar::Date));
        for m in ["plus", "year", "isBefore", "startOfMonth", "until"] {
            assert!(date.contains(&m), "a date's `{m}`: {date:?}");
        }
        assert!(!date.contains(&"amount") && !date.contains(&"host"));
        let money = names(Type::Scalar(Scalar::Money));
        for m in ["amount", "currency", "plus", "times", "convert"] {
            assert!(money.contains(&m), "money's `{m}`: {money:?}");
        }
        let url = names(Type::Scalar(Scalar::Url));
        assert!(url.contains(&"host") && url.contains(&"with") && url.contains(&"toLowerCase"));
        assert_eq!(url.iter().filter(|m| **m == "length").count(), 1);
        assert!(
            names(Type::Scalar(Scalar::Secret)).is_empty(),
            "a secret is not read apart"
        );
        let viewport = names(browser_value_type("viewport").unwrap());
        assert!(
            viewport.contains(&"md") && viewport.contains(&"width"),
            "{viewport:?}"
        );
        // Every value `members` offers for a scalar is one the checker types.
        for scalar in [
            Scalar::Date,
            Scalar::Time,
            Scalar::DateTime,
            Scalar::Duration,
            Scalar::Money,
            Scalar::Url,
            Scalar::Email,
            Scalar::Color,
            Scalar::File,
            Scalar::Uuid,
        ] {
            for m in members(&Type::Scalar(scalar)) {
                if m.method {
                    assert!(
                        scalar_method(scalar, &m.name).is_some(),
                        "{scalar:?}.{}",
                        m.name
                    );
                }
            }
        }
        // …and every scalar method the code generator routes is offered.
        for (m, _) in crate::codegen::js::SCALAR_METHODS {
            assert!(
                SCALAR_MEMBER_NAMES.contains(m),
                "`{m}` is routed but not offered"
            );
        }
    }

    use super::*;
    use crate::parser::v2::parse_v2;

    fn errors(src: &str) -> Vec<String> {
        let program = parse_v2(src, "<t>").expect("parse");
        let info = check(&program, &|_| "<t>".to_string());
        info.findings
            .errors
            .iter()
            .map(|d| match &d.hint {
                Some(h) => format!("[{}] {}\n  {h}", d.code, d.message),
                None => format!("[{}] {}", d.code, d.message),
            })
            .collect()
    }

    /// `sort` hands its comparator two elements. Typed like `map`'s
    /// `(element, index)`, the second parameter read as a `Number`.
    #[test]
    fn a_sort_comparator_takes_two_elements() {
        let src = r#"
            type Row { age: Number }
            store S {
                state rows: [Row] = []
                derived oldest = rows.slice().sort((a, b) => b.age - a.age)
            }
            page P(path: "/") { use S  Text("{S.oldest.length}") }
        "#;
        assert_eq!(errors(src), Vec::<String>::new());
    }

    /// A store's members are mutually visible. Inferring each derived value
    /// as it was met meant an action declared below it read as undeclared,
    /// though the runtime binds actions first and an action may call one
    /// declared after it.
    #[test]
    fn a_derived_value_may_call_an_action_declared_below_it() {
        let src = r#"
            store S {
                state n = 2
                derived after = twice(n)
                action twice(v: Number) { return v * 2 }
            }
            page P(path: "/") { use S  Text("{S.after}") }
        "#;
        assert_eq!(errors(src), Vec::<String>::new());
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
            "[F02] `bind:` on `Checkbox` holds a `Bool`, but `name` is `String`",
        );
        has(
            "page P(path: \"/\") { state on = true\n Slider(bind: on) }",
            "[F02] `bind:` on `Slider` holds a `Number`, but `on` is `Bool`",
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
            "[T05] `user` has no field `nam`\n  Did you mean `name`? It was written with `name`, `age`",
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
            .find(|e| e.code == "T04")
            .unwrap();
        // `sel` sits on the derived line, after `derived n = 1 + `.
        let line = src.lines().position(|l| l.contains("derived n")).unwrap() + 1;
        assert_eq!(error.line, line, "{error:?}");
        assert_eq!(error.column, "    derived n = 1 + ".len() + 1, "{error:?}");
    }
}
#[cfg(test)]
mod once {
    /// Each mistake is reported once: an argument used to be inferred once
    /// to look for a secret, again by a copy of that check, and a third
    /// time for its type, and said everything it found three times.
    #[test]
    fn a_mistake_in_an_argument_is_reported_once() {
        let src = "type User { name: String }\npage P(path: \"/\") {\n    state user = User(name: \"Ada\")\n    Heading(user.nmae).h1\n    Text(user.nmae).bold\n    Button(user.nmae) { on click { log(1) } }\n}\n";
        let program = crate::parser::v2::parse_v2(src, "t").unwrap();
        let info = super::check(&program, &|_| "t".into());
        let lines: Vec<usize> = info.findings.errors.iter().map(|e| e.line).collect();
        assert_eq!(lines, vec![4, 5, 6], "{:?}", info.findings.errors);
    }
}
