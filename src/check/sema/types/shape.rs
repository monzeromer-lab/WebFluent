//! What a declared type allows of a JSON value, checked where data comes
//! in from outside the program: a `data` file at build time, a template's
//! data before it renders, a test's seeds.
//!
//! It is the runtime's `shapeFault` (`json.js`) in Rust, saying what is
//! wrong in the same words — `$[3].date is "March 5", not a Date` — and a
//! little more, since a build can afford to read a date or an address and
//! say it is not one.

use std::collections::HashMap;

use serde_json::Value;

use super::refine::ill_formed;
use super::ty::Scalar;
use crate::syntax::parser::ast::{Declaration, EnumDecl, TypeDecl, TypeRef};

/// What a type allows of a JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// A string.
    String,
    /// A number.
    Number,
    /// `true` or `false`.
    Bool,
    /// An object of any keys: `Map`, which 7.0 retires.
    Map,
    /// Anything JSON holds: `Json`.
    Json,
    /// Anything at all: `Any`, or what JSON does not carry (a function).
    Any,
    /// The language's own types, carried by a string or a number and read
    /// for what they are: a date, an address.
    Scalar(Scalar),
    /// `null`, or the shape.
    Optional(Box<Shape>),
    /// A list of the shape.
    List(Box<Shape>),
    /// An object whose every value is the shape.
    Dict(Box<Shape>),
    /// A record, by name: each field's shape; a field with a default, or
    /// one that may be null, may be left out.
    Record(String, Vec<(String, Shape)>),
    /// An enum: one of its cases, bare (`"calm"`) or with its payload
    /// (`["failed", "timed out"]`).
    Enum(String, Vec<String>),
}

/// Where a value is not what its type says, and what it is instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// The place, from `$`: `$[3].date`.
    pub at: String,
    /// What is wrong there: `is missing`, `is "3", not a Number`.
    pub message: String,
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.at, self.message)
    }
}

/// The types a program declares, read for their shapes.
#[derive(Debug)]
pub struct Shapes<'p> {
    types: HashMap<&'p str, &'p TypeDecl>,
    enums: HashMap<&'p str, &'p EnumDecl>,
}

impl<'p> Shapes<'p> {
    /// The records, aliases and enums of `declarations`.
    pub fn of(declarations: &'p [Declaration]) -> Self {
        let mut types = HashMap::new();
        let mut enums = HashMap::new();
        for d in declarations {
            match d {
                Declaration::Type(t) => {
                    types.insert(t.name.as_str(), t);
                }
                Declaration::Enum(e) => {
                    enums.insert(e.name.as_str(), e);
                }
                _ => {}
            }
        }
        Self { types, enums }
    }

    /// What `ty` allows.
    pub fn shape(&self, ty: &TypeRef) -> Shape {
        self.shape_at(ty, 0)
    }

    fn shape_at(&self, ty: &TypeRef, depth: usize) -> Shape {
        // A type that holds itself is checked as far as any data goes.
        if depth > 16 {
            return Shape::Any;
        }
        match ty {
            TypeRef::String => Shape::String,
            TypeRef::Number => Shape::Number,
            TypeRef::Bool => Shape::Bool,
            TypeRef::Map => Shape::Map,
            TypeRef::Any | TypeRef::Func(..) | TypeRef::Unit | TypeRef::Result(..) => Shape::Any,
            TypeRef::List(t) => Shape::List(Box::new(self.shape_at(t, depth + 1))),
            TypeRef::Optional(t) => Shape::Optional(Box::new(self.shape_at(t, depth + 1))),
            TypeRef::Refined(t, _) => self.shape_at(t, depth),
            TypeRef::Dict(_, v) => Shape::Dict(Box::new(self.shape_at(v, depth + 1))),
            TypeRef::Named(name) => {
                if let Some(decl) = self.types.get(name.as_str()) {
                    if let Some(alias) = &decl.alias {
                        return self.shape_at(alias, depth + 1);
                    }
                    let fields = decl
                        .all_fields(&|n| self.types.get(n).copied())
                        .into_iter()
                        .map(|f| {
                            let shape = self.shape_at(&f.ty, depth + 1);
                            let shape =
                                if f.default.is_some() && !matches!(shape, Shape::Optional(_)) {
                                    Shape::Optional(Box::new(shape))
                                } else {
                                    shape
                                };
                            (f.name.clone(), shape)
                        })
                        .collect();
                    return Shape::Record(name.clone(), fields);
                }
                if let Some(decl) = self.enums.get(name.as_str()) {
                    return Shape::Enum(name.clone(), decl.case_names());
                }
                if name == "Json" {
                    return Shape::Json;
                }
                match Scalar::of_name(name) {
                    Some(Scalar::Money) => Shape::Record(
                        "Money".to_string(),
                        vec![
                            ("amount".to_string(), Shape::Number),
                            ("currency".to_string(), Shape::String),
                        ],
                    ),
                    Some(Scalar::File) => Shape::Any,
                    Some(s) => Shape::Scalar(s),
                    None => Shape::Any,
                }
            }
        }
    }
}

/// What is wrong with `value` against `shape`, the first place it goes
/// wrong in the order the value is written, or nothing.
pub fn fault(value: &Value, shape: &Shape) -> Option<Fault> {
    fault_at(Some(value), shape, "$")
}

/// [`fault`], for a value that may be missing altogether — a field a map
/// does not have.
fn fault_at(value: Option<&Value>, shape: &Shape, at: &str) -> Option<Fault> {
    let wrong = |message: String| {
        Some(Fault {
            at: at.to_string(),
            message,
        })
    };
    if *shape == Shape::Any {
        return None;
    }
    if let Shape::Optional(inner) = shape {
        return match value {
            None | Some(Value::Null) => None,
            Some(v) => fault_at(Some(v), inner, at),
        };
    }
    let Some(value) = value else {
        return wrong("is missing".to_string());
    };
    if *shape == Shape::Json {
        return None;
    }
    if value.is_null() {
        return wrong("is null".to_string());
    }
    match shape {
        Shape::String => (!value.is_string()).then(|| not(value, "a String", at)),
        Shape::Number => (!value.is_number()).then(|| not(value, "a Number", at)),
        Shape::Bool => (!value.is_boolean()).then(|| not(value, "a Bool", at)),
        Shape::Map => (!value.is_object()).then(|| not(value, "a Map", at)),
        Shape::Scalar(s) => match (s, value) {
            (Scalar::Duration, Value::Number(_)) => None,
            (Scalar::Duration, other) => Some(not(other, "a Duration", at)),
            (s, Value::String(text)) => ill_formed(*s, text).map(|_| Fault {
                at: at.to_string(),
                message: format!("is {}, not a {}", shown(value), s.name()),
            }),
            (s, other) => Some(not(other, &format!("a {}", s.name()), at)),
        },
        Shape::List(item) => match value {
            Value::Array(items) => items
                .iter()
                .enumerate()
                .find_map(|(i, v)| fault_at(Some(v), item, &format!("{at}[{i}]"))),
            other => Some(not(other, "a list", at)),
        },
        Shape::Dict(item) => match value {
            Value::Object(map) => map.iter().find_map(|(k, v)| {
                fault_at(
                    Some(v),
                    item,
                    &format!("{at}[{}]", serde_json::to_string(k).unwrap_or_default()),
                )
            }),
            other => Some(not(other, "a dictionary", at)),
        },
        Shape::Record(name, fields) => match value {
            Value::Object(map) => fields
                .iter()
                .find_map(|(field, s)| fault_at(map.get(field), s, &format!("{at}.{field}"))),
            other => Some(not(other, &format!("a {name}"), at)),
        },
        Shape::Enum(_, cases) => {
            let case = match value {
                Value::Array(parts) => parts.first().and_then(Value::as_str),
                Value::String(s) => Some(s.as_str()),
                _ => None,
            };
            if case.is_some_and(|c| cases.iter().any(|k| k == c)) {
                None
            } else {
                let names: Vec<String> = cases.iter().map(|c| format!(".{c}")).collect();
                wrong(format!(
                    "is {}, which is none of {}",
                    shown(value),
                    names.join(", ")
                ))
            }
        }
        Shape::Any | Shape::Json | Shape::Optional(_) => None,
    }
}

/// `is "3", not a Number`, as the runtime words it.
fn not(value: &Value, wanted: &str, at: &str) -> Fault {
    Fault {
        at: at.to_string(),
        message: format!("is {}, not {wanted}", shown(value)),
    }
}

/// A value as a message shows it: a list, a map, or the JSON of a scalar.
fn shown(value: &Value) -> String {
    match value {
        Value::Array(_) => "a list".to_string(),
        Value::Object(_) => "a map".to_string(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn shapes_of(src: &str) -> Vec<Declaration> {
        crate::syntax::parse_source(src, "t.wf")
            .expect("parses")
            .declarations
    }

    fn fault_of(src: &str, ty: &str, value: &Value) -> Option<String> {
        let decls = shapes_of(src);
        let shapes = Shapes::of(&decls);
        let ty = TypeRef::Named(ty.to_string());
        fault(value, &shapes.shape(&ty)).map(|f| f.to_string())
    }

    const POST: &str = "type Post { slug: String, title: String, date: Date, tags: [String] = [], note: String? }\nenum Tone { calm, loud }\ntype Posts = [Post]\n";

    #[test]
    fn a_value_is_held_to_its_record_field_by_field_and_says_where_it_is_not() {
        let good = json!([{ "slug": "a", "title": "A", "date": "2026-03-01" }]);
        assert_eq!(fault_of(POST, "Posts", &good), None);
        let bad = json!([
            { "slug": "a", "title": "A", "date": "2026-03-01" },
            { "slug": "b", "title": "B", "date": "March 5" }
        ]);
        assert_eq!(
            fault_of(POST, "Posts", &bad).as_deref(),
            Some("$[1].date is \"March 5\", not a Date")
        );
        let missing = json!([{ "slug": "a", "date": "2026-03-01" }]);
        assert_eq!(
            fault_of(POST, "Posts", &missing).as_deref(),
            Some("$[0].title is missing")
        );
        let wrong = json!([{ "slug": "a", "title": 3, "date": "2026-03-01", "tags": ["x", 1] }]);
        assert_eq!(
            fault_of(POST, "Posts", &wrong).as_deref(),
            Some("$[0].title is 3, not a String")
        );
    }

    #[test]
    fn an_enum_takes_its_cases_and_a_dictionary_its_values() {
        assert_eq!(fault_of(POST, "Tone", &json!("calm")), None);
        assert_eq!(
            fault_of(POST, "Tone", &json!("quiet")).as_deref(),
            Some("$ is \"quiet\", which is none of .calm, .loud")
        );
        let decls = shapes_of("type Scores = [String: Number]\n");
        let shapes = Shapes::of(&decls);
        let shape = shapes.shape(&TypeRef::Named("Scores".to_string()));
        assert_eq!(
            fault(&json!({ "ada": 3, "bo": "five" }), &shape).map(|f| f.to_string()),
            Some("$[\"bo\"] is \"five\", not a Number".to_string())
        );
    }
}
