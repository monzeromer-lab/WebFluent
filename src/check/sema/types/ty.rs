//! The types the checker reasons about: `Type`, the language's scalars, and what makes two types agree.

use super::TypeRef;

/// A type the checker reasons about.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Unknown, or anything: agrees with every type.
    Any,
    /// What a value is after an error about it has been reported: agrees
    /// with every type, as `Any` does, so one mistake is reported once
    /// rather than at every place the value goes. Never written, and never
    /// a gap in what the program says ([`super::Hole`]).
    Error,
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
    /// A `resource`: what it holds, and the body of a failure the server
    /// declared (`errors { 422 -> T }`), `Any` when it declared none.
    Resource(Box<Type>, Box<Type>),
    /// A store brought into scope with `use`.
    Store(String),
    /// A type the language knows and the program did not declare: a date,
    /// a duration, money, a URL. Each is a plain JSON value at run time.
    Scalar(Scalar),
    /// A service declared with `api`.
    Api(String),
    /// What a request can fail with: `.offline`, `.timeout`, `.aborted`,
    /// `.parse`, `.network`, `.status(code, body)` — matched like an enum,
    /// and read like a record. It holds the body the server's failure is
    /// declared to carry, `Any` when it is not.
    NetError(Box<Type>),
    /// What an async action hands back before it is awaited: a promise of
    /// its result.
    Promise(Box<Type>),
    /// What no value is: the arm of a `match` that covers every case.
    Never,
    /// `[K: V]`: a dictionary.
    Dict(Box<Type>, Box<Type>),
    /// `T ! E`: `.ok(value)` or `.err(error)`. A result with no value
    /// holds `Null`.
    Result(Box<Type>, Box<Type>),
    /// An object the browser hands a program — an element, an event,
    /// `localStorage` — by its name in the browser table
    /// (`registry::browser::BROWSER_TYPES`).
    Host(String),
    /// A connection the page holds open — a socket, a stream, a channel, a
    /// peer, a worker — with what it sends and what it receives (`Json`
    /// where it does not say).
    Connection(
        crate::syntax::parser::ast::ConnectionKind,
        Box<Type>,
        Box<Type>,
    ),
}

/// The name of the language's open JSON: an enum whose cases are JSON's
/// kinds, a plain JSON value at run time.
pub(super) const JSON: &str = "Json";

/// The name of what goes wrong between the page and the server.
pub(super) const HTTP_ERROR: &str = "HttpError";

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

    /// Whether it is a string with nothing to choose about its spelling — a
    /// URL, an address, a colour, an id — so it fits where a `String` is
    /// wanted. A date or a time has many spellings, and is shown through a
    /// splice that picks one; a secret is never shown.
    pub fn is_string(self) -> bool {
        matches!(
            self,
            Scalar::Url | Scalar::Email | Scalar::Color | Scalar::Uuid
        )
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
            TypeRef::Dict(k, v) => {
                Type::Dict(Box::new(Type::from_ref(k)), Box::new(Type::from_ref(v)))
            }
            TypeRef::Func(params, ret) => Type::Func(
                params.iter().map(Type::from_ref).collect(),
                Box::new(Type::from_ref(ret)),
            ),
            // Nothing is what a function that returns no value gives.
            TypeRef::Unit => Type::Null,
            // `T!` names no error of its own: what goes wrong between the
            // page and the server.
            TypeRef::Result(value, error) => Type::Result(
                Box::new(Type::from_ref(value)),
                Box::new(error.as_ref().map_or_else(
                    || Type::Record(HTTP_ERROR.to_string()),
                    |e| Type::from_ref(e),
                )),
            ),
        }
    }

    /// Whether a value of this type is plain data — what JSON can carry —
    /// and so fits where `Json` is wanted.
    fn is_data(&self) -> bool {
        match self {
            Type::String
            | Type::Number
            | Type::Bool
            | Type::Null
            | Type::Map
            | Type::Record(_)
            | Type::Enum(_)
            | Type::Case(_) => true,
            Type::Scalar(s) => *s != Scalar::File && *s != Scalar::Secret,
            Type::List(t) | Type::Optional(t) => t.is_data(),
            Type::Dict(_, v) => v.is_data(),
            Type::Shape(fields) => fields.iter().all(|(_, t)| t.is_data()),
            _ => false,
        }
    }

    pub(super) fn list(inner: Type) -> Type {
        Type::List(Box::new(inner))
    }

    pub(super) fn optional(inner: Type) -> Type {
        match inner {
            Type::Optional(_) | Type::Null => inner,
            other => Type::Optional(Box::new(other)),
        }
    }

    /// The type without its `null`.
    pub(super) fn unwrapped(&self) -> Type {
        match self {
            Type::Optional(inner) => (**inner).clone(),
            other => other.clone(),
        }
    }

    /// Whether the checker cannot say what the value is: `Any`, or a value
    /// an error has already been reported about.
    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any | Type::Error)
    }

    /// Whether a value of `self` may be given where `to` is expected.
    pub(super) fn assignable_to(&self, to: &Type) -> bool {
        match (self, to) {
            (Type::Any | Type::Error | Type::Never, _) | (_, Type::Any | Type::Error) => true,
            (Type::Case(_), Type::Enum(_)) => true,
            (Type::Null, Type::Optional(_)) => true,
            (Type::Null, Type::Null) => true,
            // Plain data is JSON; `Json` is read only by matching or decoding.
            (from, Type::Enum(name)) if name == JSON && from.is_data() => true,
            (Type::Optional(a), Type::Optional(b)) => a.assignable_to(b),
            (from, Type::Optional(b)) => from.assignable_to(b),
            (Type::List(a), Type::List(b)) => a.assignable_to(b),
            (Type::Map, Type::Record(_)) | (Type::Record(_), Type::Map) => true,
            // `Map`, which 7.0 retires, is open JSON already.
            (Type::Enum(name), Type::Map) if name == JSON => true,
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
            // What it returns fits what is wanted of it, unless nothing is.
            (Type::Func(from, from_ret), Type::Func(to, to_ret)) => {
                from.len() <= to.len()
                    && from.iter().zip(to.iter()).all(|(f, t)| t.assignable_to(f))
                    && (**to_ret == Type::Null || from_ret.assignable_to(to_ret))
            }
            (Type::Dict(k, v), Type::Dict(k2, v2)) => k.assignable_to(k2) && v.assignable_to(v2),
            // A map literal is a dictionary when every value fits.
            (Type::Shape(fields), Type::Dict(_, v)) => {
                fields.iter().all(|(_, t)| t.assignable_to(v))
            }
            (Type::Map, Type::Dict(..)) | (Type::Dict(..), Type::Map) => true,
            (Type::Result(a, e), Type::Result(b, f)) => a.assignable_to(b) && e.assignable_to(f),
            // A scalar whose text is its one spelling — an address, a
            // colour, an id — is a string. One that has many — a date, a
            // time — is shown through a splice that says which (`T34`), and
            // a `Duration` is not a `Number` (`3.seconds + 500.ms`).
            (Type::Scalar(s), Type::String) => s.is_string(),
            (Type::Resource(a, _), Type::Resource(b, _)) => a.assignable_to(b),
            (Type::NetError(_), Type::NetError(_)) => true,
            (Type::Promise(a), Type::Promise(b)) => a.assignable_to(b),
            // A `KeyboardEvent` is an `Event`.
            (Type::Host(a), Type::Host(b)) => crate::registry::browser::browser().extends(a, b),
            (a, b) => a == b,
        }
    }

    /// The type both `a` and `b` fit: their common type, or `Any`.
    pub(super) fn join(a: Type, b: Type) -> Type {
        match (a, b) {
            (a, b) if a == b => a,
            (Type::Never, other) | (other, Type::Never) => other,
            (Type::Error, _) | (_, Type::Error) => Type::Error,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Null, other) | (other, Type::Null) => Type::optional(other),
            (Type::Optional(a), b) | (b, Type::Optional(a)) => Type::optional(Type::join(*a, b)),
            (Type::List(a), Type::List(b)) => Type::list(Type::join(*a, *b)),
            (Type::Dict(k, a), Type::Dict(_, b)) => Type::Dict(k, Box::new(Type::join(*a, *b))),
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
            // An address, a colour or an id is a string.
            (Type::Scalar(s), Type::String) | (Type::String, Type::Scalar(s)) if s.is_string() => {
                Type::String
            }
            (Type::Case(_), Type::Case(_)) => Type::Any,
            _ => Type::Any,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // `Any` and `Map` are not types a program can write (7.0), so a
            // message never names them: what an empty list holds is not
            // known yet (`[]`), and an open map is `{…}`.
            Type::Any => write!(f, "unknown"),
            Type::Error => write!(f, "unknown"),
            Type::String => write!(f, "String"),
            Type::Number => write!(f, "Number"),
            Type::Bool => write!(f, "Bool"),
            Type::Null => write!(f, "null"),
            Type::Token => write!(f, "$token"),
            Type::Regex => write!(f, "Regex"),
            Type::List(inner) if **inner == Type::Any => write!(f, "[]"),
            Type::List(inner) => write!(f, "[{inner}]"),
            Type::Map => write!(f, "{{…}}"),
            Type::Shape(fields) => {
                let inner: Vec<String> = fields.iter().map(|(n, t)| format!("{n}: {t}")).collect();
                write!(f, "{{ {} }}", inner.join(", "))
            }
            Type::Record(name) | Type::Enum(name) | Type::Store(name) | Type::Host(name) => {
                write!(f, "{name}")
            }
            Type::Case(name) => write!(f, ".{name}"),
            Type::Optional(inner) => write!(f, "{inner}?"),
            Type::Func(params, ret) => {
                let params: Vec<String> = params
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
                if **ret == Type::Null {
                    write!(f, "({}) -> ()", params.join(", "))
                } else {
                    write!(f, "({}) -> {ret}", params.join(", "))
                }
            }
            Type::Dict(k, v) => write!(f, "[{k}: {v}]"),
            Type::Result(value, error) => {
                let error = match &**error {
                    Type::Enum(n) if n == HTTP_ERROR => String::new(),
                    other => format!(" {other}"),
                };
                match &**value {
                    Type::Null => write!(f, "!{error}"),
                    v if error.is_empty() => write!(f, "{v}!"),
                    v => write!(f, "{v} !{error}"),
                }
            }
            Type::Resource(inner, _) => write!(f, "resource<{inner}>"),
            Type::Promise(inner) => write!(f, "Promise<{inner}>"),
            Type::Never => write!(f, "never"),
            Type::Scalar(s) => write!(f, "{}", s.name()),
            Type::Api(name) => write!(f, "{name}"),
            Type::Connection(kind, _, receive) => {
                use crate::syntax::parser::ast::ConnectionKind as K;
                let word = match kind {
                    K::Socket => "socket",
                    K::Stream => "stream",
                    K::Channel => "channel",
                    K::Peer => "peer",
                    _ => "worker",
                };
                write!(f, "{word} of {receive}")
            }
            Type::NetError(_) => write!(f, "NetworkError"),
        }
    }
}

/// Whether a value of `ty` takes part in `-`, `*`, `/` and `%`: a number,
/// a duration (milliseconds), or something the checker cannot see.
pub(super) fn counts(ty: &Type) -> bool {
    matches!(
        ty.unwrapped(),
        Type::Number | Type::Any | Type::Error | Type::Scalar(Scalar::Duration) | Type::Null
    )
}

/// Whether a value of `ty` takes part in a `+` that is not a string's: as
/// `counts`, or a string, which `+` joins.
pub(super) fn adds(ty: &Type) -> bool {
    counts(ty)
        || ty.unwrapped() == Type::String
        || matches!(ty.unwrapped(), Type::Scalar(s) if s.is_text())
}

/// Whether no value of `a` can equal a value of `b`.
pub(super) fn disjoint(a: &Type, b: &Type) -> bool {
    fn family(t: &Type) -> Option<u8> {
        Some(match t {
            Type::String | Type::Scalar(_) => 0,
            Type::Number => 1,
            Type::Bool => 2,
            Type::List(_) => 3,
            Type::Record(_) | Type::Shape(_) | Type::Map | Type::Dict(..) => 4,
            Type::Enum(_) | Type::Case(_) => 0,
            _ => return None,
        })
    }
    // A duration is a number, a text scalar a string, an enum's case a
    // string at run time.
    let a_fam = match a {
        Type::Scalar(Scalar::Duration) => Some(1),
        Type::Scalar(Scalar::Money | Scalar::File) => Some(4),
        other => family(other),
    };
    let b_fam = match b {
        Type::Scalar(Scalar::Duration) => Some(1),
        Type::Scalar(Scalar::Money | Scalar::File) => Some(4),
        other => family(other),
    };
    match (a_fam, b_fam) {
        (Some(x), Some(y)) => x != y,
        _ => false,
    }
}

/// The plain types a program writes, beside its own and [`Scalar::ALL`].
pub const PRIMITIVE_TYPES: &[&str] = &["String", "Number", "Bool", "Json"];

/// Why `given` does not fit `wanted` when the language would once have
/// converted it: a date or a time where text is wanted — it has many
/// spellings, and a splice says which — or a `Duration` and a `Number`
/// taken for each other. `T34`'s message and hint.
pub(super) fn conversion(given: &Type, wanted: &Type) -> Option<(String, &'static str)> {
    let (from, to) = match (given, wanted) {
        (Type::Optional(a), Type::Optional(b)) => (&**a, &**b),
        (from, Type::Optional(b)) => (from, &**b),
        (from, to) => (from, to),
    };
    match (from, to) {
        (Type::Scalar(s @ (Scalar::Date | Scalar::Time | Scalar::DateTime)), Type::String) => {
            Some((
                format!(
                    "and a `{}` is not a `String`: it has more than one spelling",
                    s.name()
                ),
                "Show it through a splice, which says how: `\"{due}\"`, or `\"{due:.date(long)}\"`",
            ))
        }
        (Type::Scalar(Scalar::Duration), Type::Number) => Some((
            "and a `Duration` is not a `Number`".to_string(),
            "Say what it counts: `wait.ms()`, `wait.seconds()`",
        )),
        (Type::Number, Type::Scalar(Scalar::Duration)) => Some((
            "and a `Number` is not a `Duration`".to_string(),
            "Give it its unit: `500.ms`, `3.seconds`, or `n.ms` for a number held in a value",
        )),
        _ => None,
    }
}
