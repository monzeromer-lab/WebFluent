//! The types the checker reasons about: `Type`, the language's scalars, and what makes two types agree.

use super::*;

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

    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any)
    }

    /// Whether a value of `self` may be given where `to` is expected.
    pub(super) fn assignable_to(&self, to: &Type) -> bool {
        match (self, to) {
            (Type::Any | Type::Never, _) | (_, Type::Any) => true,
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
    pub(super) fn join(a: Type, b: Type) -> Type {
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
                let params: Vec<String> = params
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect();
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

/// Whether a value of `ty` takes part in `-`, `*`, `/` and `%`: a number,
/// a duration (milliseconds), or something the checker cannot see.
pub(super) fn counts(ty: &Type) -> bool {
    matches!(
        ty.unwrapped(),
        Type::Number | Type::Any | Type::Scalar(Scalar::Duration) | Type::Null
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
            Type::Record(_) | Type::Shape(_) | Type::Map => 4,
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
pub const PRIMITIVE_TYPES: &[&str] = &["String", "Number", "Bool", "Map", "Any"];
