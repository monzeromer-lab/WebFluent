//! What a value of a type has: its fields and methods, as a completion list reads them.

use super::*;

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
pub(super) const SCALAR_MEMBER_NAMES: &[&str] = &[
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
            out.push(field("refreshing", Type::Bool));
            out.push(field("pending", Type::Bool));
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
