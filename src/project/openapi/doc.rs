//! A specification as it was written: JSON or YAML read into one value
//! that keeps every object's keys in the file's order, so "the first
//! content type", "the first success" and the order the generated files
//! list things in are the spec's, never the alphabet's.

use std::fmt;

/// A JSON value whose objects keep their keys in the order they were read.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Object),
}

/// An object's keys and values, in the order they were written. A key
/// written twice keeps its last value, in its first place.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Object(Vec<(String, Value)>);

impl Object {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add `key`, or give it a new value where it already is.
    pub fn insert(&mut self, key: String, value: Value) {
        match self.0.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = value,
            None => self.0.push((key, value)),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_key_value(&self, key: &str) -> Option<(&String, &Value)> {
        self.0.iter().find(|(k, _)| k == key).map(|(k, v)| (k, v))
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.0.iter().map(|(k, v)| (k, v))
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.0.iter().map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (&'a String, &'a Value);
    type IntoIter = std::iter::Map<
        std::slice::Iter<'a, (String, Value)>,
        fn(&'a (String, Value)) -> (&'a String, &'a Value),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(|(k, v)| (k, v))
    }
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(o) => o.get(key),
            _ => None,
        }
    }

    /// RFC 6901: `/components/schemas/User`, with `~1` a `/` and `~0` a `~`.
    pub fn pointer(&self, pointer: &str) -> Option<&Value> {
        if pointer.is_empty() {
            return Some(self);
        }
        let mut at = self;
        for part in pointer.strip_prefix('/')?.split('/') {
            let part = part.replace("~1", "/").replace("~0", "~");
            at = match at {
                Value::Object(o) => o.get(&part)?,
                Value::Array(list) => list.get(part.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(at)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// The value as `serde_json` holds one, for what writes JSON out.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Number(n) =>
            {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "a whole number in range is written as one"
                )]
                if n.fract() == 0.0 && n.abs() < 9.0e15 {
                    serde_json::Value::from(*n as i64)
                } else {
                    serde_json::Number::from_f64(*n)
                        .map_or(serde_json::Value::Null, serde_json::Value::Number)
                }
            }
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Array(list) => {
                serde_json::Value::Array(list.iter().map(Value::to_json).collect())
            }
            Value::Object(o) => {
                serde_json::Value::Object(o.iter().map(|(k, v)| (k.clone(), v.to_json())).collect())
            }
        }
    }
}

/// What reading a document found wrong, and on which line.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (line {}, column {})",
            self.message, self.line, self.col
        )
    }
}

/// Read `text` as JSON, or as YAML when it is not JSON: a document that
/// opens with `{` is JSON, anything else YAML (of which JSON is a part).
///
/// # Errors
///
/// What is wrong with the text, and where.
pub fn read(text: &str) -> Result<Value, ReadError> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if text.trim_start().starts_with('{') {
        read_json(text)
    } else {
        super::yaml::read(text)
    }
}

/// Read `text` as JSON, keeping every object's order.
///
/// # Errors
///
/// `serde_json`'s message, and where.
pub fn read_json(text: &str) -> Result<Value, ReadError> {
    let mut de = serde_json::Deserializer::from_str(text);
    let value = <Value as serde::Deserialize>::deserialize(&mut de).map_err(|e| ReadError {
        message: e
            .to_string()
            .split(" at line")
            .next()
            .unwrap_or_default()
            .to_string(),
        line: e.line(),
        col: e.column(),
    })?;
    de.end().map_err(|e| ReadError {
        message: "there is more after the document".to_string(),
        line: e.line(),
        col: e.column(),
    })?;
    Ok(value)
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visit;
        impl<'de> serde::de::Visitor<'de> for Visit {
            type Value = Value;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON value")
            }

            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
                Ok(Value::Bool(v))
            }

            #[expect(clippy::cast_precision_loss, reason = "JSON numbers are doubles")]
            fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
                Ok(Value::Number(v as f64))
            }

            #[expect(clippy::cast_precision_loss, reason = "JSON numbers are doubles")]
            fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
                Ok(Value::Number(v as f64))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
                Ok(Value::Number(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Value, E> {
                Ok(Value::String(v.to_string()))
            }

            fn visit_string<E>(self, v: String) -> Result<Value, E> {
                Ok(Value::String(v))
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Value, A::Error> {
                let mut list = Vec::new();
                while let Some(v) = seq.next_element()? {
                    list.push(v);
                }
                Ok(Value::Array(list))
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Value, A::Error> {
                let mut object = Object::new();
                while let Some((k, v)) = map.next_entry::<String, Value>()? {
                    object.insert(k, v);
                }
                Ok(Value::Object(object))
            }
        }
        deserializer.deserialize_any(Visit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_keeps_the_order_it_was_written_in() {
        let v = read(r#"{ "z": 1, "a": { "y": true, "b": null }, "m": [1, "two"] }"#).unwrap();
        let keys: Vec<&String> = v.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["z", "a", "m"]);
        let inner: Vec<&String> = v.get("a").unwrap().as_object().unwrap().keys().collect();
        assert_eq!(inner, ["y", "b"]);
        assert_eq!(v.pointer("/m/1").and_then(Value::as_str), Some("two"));
    }

    #[test]
    fn a_pointer_reads_escaped_names() {
        let v = read(r#"{ "paths": { "/users/{id}": { "get": 1 } } }"#).unwrap();
        assert_eq!(
            v.pointer("/paths/~1users~1{id}/get")
                .and_then(Value::as_f64),
            Some(1.0)
        );
    }

    #[test]
    fn json_that_does_not_parse_says_where() {
        let e = read("{ \"a\": 1,\n  \"b\" }").unwrap_err();
        assert_eq!(e.line, 2);
    }
}
