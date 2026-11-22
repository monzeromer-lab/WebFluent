//! A record's declared defaults, given to data that leaves the field out.
//!
//! `Quote(number: "7")` carries `currency: "EUR"` when the type says
//! `currency: String = "EUR"`, because constructing a record fills what it
//! leaves out. Data read as the type — a `data` file, a template's data, a
//! test's `data:` — is held to the same promise: a field left out that has
//! a default is that default, and one that may be null is `null`, so the
//! page reads the value the type says it has rather than nothing.

use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::syntax::parser::ast::{
    Declaration, Expr, Statement, StatementKind, TypeDecl, TypeRef, UnaryOp,
};

/// The records a program declares, read for their defaults.
#[derive(Debug)]
pub(crate) struct Defaults<'p> {
    types: HashMap<&'p str, &'p TypeDecl>,
}

impl<'p> Defaults<'p> {
    /// The records and aliases of `declarations`.
    pub(crate) fn of(declarations: &'p [Declaration]) -> Self {
        let types = declarations
            .iter()
            .filter_map(|d| match d {
                Declaration::Type(t) => Some((t.name.as_str(), t)),
                _ => None,
            })
            .collect();
        Self { types }
    }

    /// The records of `types`, by their names.
    pub(crate) fn of_types(types: impl IntoIterator<Item = &'p TypeDecl>) -> Self {
        Self {
            types: types.into_iter().map(|t| (t.name.as_str(), t)).collect(),
        }
    }

    /// A field's default as the JSON it is at run time, when the type can
    /// say it without running anything; `None` otherwise.
    pub(crate) fn default_json(&self, default: &Expr) -> Option<Value> {
        self.constant(default, 0)
    }

    /// `value`, read as `ty`, with every record field it leaves out (or
    /// gives as `null`, when the field may not be null) that declares a
    /// default given that default, and every optional one it leaves out
    /// given `null` — at any depth. What is not the type is left as it is,
    /// for the check that follows to report.
    pub(crate) fn fill(&self, value: &mut Value, ty: &TypeRef) {
        self.fill_at(value, ty, 0);
    }

    fn fill_at(&self, value: &mut Value, ty: &TypeRef, depth: usize) {
        // A type that holds itself is filled as far as any data goes.
        if depth > 32 {
            return;
        }
        match ty {
            TypeRef::List(item) => {
                if let Value::Array(items) = value {
                    for v in items {
                        self.fill_at(v, item, depth + 1);
                    }
                }
            }
            TypeRef::Dict(_, item) => {
                if let Value::Object(map) = value {
                    for v in map.values_mut() {
                        self.fill_at(v, item, depth + 1);
                    }
                }
            }
            TypeRef::Optional(inner) | TypeRef::Refined(inner, _) => {
                self.fill_at(value, inner, depth + 1);
            }
            TypeRef::Named(name) => {
                let Some(decl) = self.types.get(name.as_str()) else {
                    return;
                };
                if let Some(alias) = &decl.alias {
                    self.fill_at(value, alias, depth + 1);
                    return;
                }
                let Value::Object(map) = value else {
                    return;
                };
                for field in decl.all_fields(&|n| self.types.get(n).copied()) {
                    let optional = matches!(field.ty, TypeRef::Optional(_));
                    let absent = match map.get(&field.name) {
                        None => true,
                        Some(Value::Null) => !optional,
                        Some(_) => false,
                    };
                    if absent {
                        let given = match &field.default {
                            Some(default) => self.constant(default, depth + 1),
                            None if optional => Some(Value::Null),
                            None => None,
                        };
                        if let Some(given) = given {
                            map.insert(field.name.clone(), given);
                        }
                    }
                    if let Some(v) = map.get_mut(&field.name) {
                        self.fill_at(v, &field.ty, depth + 1);
                    }
                }
            }
            _ => {}
        }
    }

    /// [`fill`](Defaults::fill) for data written in the language — a test's
    /// `data: { quote: { number: "7" } }` — rather than JSON: a map literal
    /// read as a record is given each field it leaves out that has a
    /// default, as that default's expression, and `null` for an optional
    /// one. A map with a spread may hold any field, and is left as it is.
    pub(crate) fn fill_expr(&self, expr: &mut Expr, ty: &TypeRef) {
        self.fill_expr_at(expr, ty, 0);
    }

    fn fill_expr_at(&self, expr: &mut Expr, ty: &TypeRef, depth: usize) {
        if depth > 32 {
            return;
        }
        match (ty, expr) {
            (TypeRef::List(item), Expr::ListLiteral(items)) => {
                for e in items {
                    self.fill_expr_at(e, item, depth + 1);
                }
            }
            (TypeRef::Dict(_, item), Expr::MapLiteral(pairs)) => {
                for (_, e) in pairs {
                    self.fill_expr_at(e, item, depth + 1);
                }
            }
            (TypeRef::Optional(inner) | TypeRef::Refined(inner, _), e) => {
                self.fill_expr_at(e, inner, depth + 1);
            }
            (TypeRef::Named(name), e) => {
                let Some(decl) = self.types.get(name.as_str()) else {
                    return;
                };
                if let Some(alias) = &decl.alias {
                    self.fill_expr_at(e, alias, depth + 1);
                    return;
                }
                let Expr::MapLiteral(pairs) = e else {
                    return;
                };
                if pairs.iter().any(|(k, _)| k == "...") {
                    return;
                }
                for field in decl.all_fields(&|n| self.types.get(n).copied()) {
                    let at = pairs
                        .iter()
                        .position(|(k, _)| k.trim_matches('"') == field.name);
                    let optional = matches!(field.ty, TypeRef::Optional(_));
                    match at {
                        Some(i) if !matches!(pairs[i].1, Expr::Null) || optional => {
                            self.fill_expr_at(&mut pairs[i].1, &field.ty, depth + 1);
                        }
                        _ => {
                            let given = match &field.default {
                                Some(default) => default.clone(),
                                None if optional => Expr::Null,
                                None => continue,
                            };
                            match at {
                                Some(i) => pairs[i].1 = given,
                                None => pairs.push((field.name.clone(), given)),
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// A test's `data: { … }`, each seed read as what it seeds: a state of
    /// `body` (the test's own), a state of a page or component of
    /// `declarations` of that name, or — under a store's name — that store's
    /// states. A seed whose state declares no type is left as it is.
    pub(crate) fn fill_seeds(
        &self,
        data: &mut Expr,
        body: &[Statement],
        declarations: &[Declaration],
    ) {
        let Expr::MapLiteral(pairs) = data else {
            return;
        };
        let bodies = || {
            declarations.iter().filter_map(|d| match d {
                Declaration::Page(p) => Some(p.body.as_slice()),
                Declaration::Component(c) => Some(c.body.as_slice()),
                _ => None,
            })
        };
        for (key, given) in pairs {
            let key = key.trim_matches('"');
            let ty = body
                .iter()
                .find_map(|s| match &s.kind {
                    StatementKind::State(st) if st.name == key => st.ty.clone(),
                    _ => None,
                })
                .or_else(|| {
                    bodies().find_map(|b| {
                        b.iter().find_map(|s| match &s.kind {
                            StatementKind::State(st) if st.name == key => st.ty.clone(),
                            _ => None,
                        })
                    })
                });
            if let Some(ty) = ty {
                self.fill_expr(given, &ty);
                continue;
            }
            let store = declarations.iter().find_map(|d| match d {
                Declaration::Store(s) if s.name == key => Some(s),
                _ => None,
            });
            if let (Some(store), Expr::MapLiteral(members)) = (store, given) {
                for (member, value) in members {
                    let found = store.body.iter().find_map(|s| match &s.kind {
                        StatementKind::State(st) if st.name == member.trim_matches('"') => {
                            st.ty.as_ref()
                        }
                        _ => None,
                    });
                    if let Some(ty) = found {
                        self.fill_expr(value, ty);
                    }
                }
            }
        }
    }

    /// A default as the JSON it is at run time, when it is a constant the
    /// type can say without running anything: a literal, a list or map of
    /// them, a case, a scalar's literal (`€12.99`), a record built of them
    /// (`Offers()`).
    fn constant(&self, expr: &Expr, depth: usize) -> Option<Value> {
        if depth > 32 {
            return None;
        }
        Some(match expr {
            Expr::StringLiteral(s) => Value::String(s.clone()),
            Expr::NumberLiteral(n) => number(*n)?,
            Expr::BoolLiteral(b) => Value::Bool(*b),
            Expr::Null => Value::Null,
            Expr::Typed(_, inner) => self.constant(inner, depth + 1)?,
            Expr::UnaryOp(UnaryOp::Neg, inner) => match self.constant(inner, depth + 1)? {
                Value::Number(n) => number(-n.as_f64()?)?,
                _ => return None,
            },
            Expr::ListLiteral(items) => Value::Array(
                items
                    .iter()
                    .map(|e| self.constant(e, depth + 1))
                    .collect::<Option<_>>()?,
            ),
            Expr::MapLiteral(pairs) => Value::Object(
                pairs
                    .iter()
                    .map(|(k, e)| {
                        Some((
                            k.trim_matches('"').to_string(),
                            self.constant(e, depth + 1)?,
                        ))
                    })
                    .collect::<Option<Map<_, _>>>()?,
            ),
            Expr::EnumCase(case) => Value::String(case.clone()),
            Expr::CaseValue(case, args) => Value::Array(
                std::iter::once(Some(Value::String(case.clone())))
                    .chain(args.iter().map(|e| self.constant(e, depth + 1)))
                    .collect::<Option<_>>()?,
            ),
            // A record built here is the record, its own defaults filled.
            Expr::Record(name, _) | Expr::FunctionCall(name, _)
                if self.types.contains_key(name.as_str()) =>
            {
                let given: &[(String, Expr)] = match expr {
                    Expr::Record(_, fields) => fields,
                    Expr::FunctionCall(_, args) if args.is_empty() => &[],
                    _ => return None,
                };
                let mut value = Value::Object(
                    given
                        .iter()
                        .map(|(k, e)| Some((k.clone(), self.constant(e, depth + 1)?)))
                        .collect::<Option<Map<_, _>>>()?,
                );
                self.fill_at(&mut value, &TypeRef::Named(name.clone()), depth + 1);
                value
            }
            _ => return None,
        })
    }
}

/// A number as JSON writes it: whole numbers without a fraction.
fn number(n: f64) -> Option<Value> {
    if n.fract() == 0.0 && n.abs() < 9e15 {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a whole number well inside i64"
        )]
        Some(Value::from(n as i64))
    } else {
        serde_json::Number::from_f64(n).map(Value::Number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn filled(src: &str, ty: &str, mut value: Value) -> Value {
        let program = crate::syntax::parse_source(src, "t.wf").unwrap();
        let ty = TypeRef::Named(ty.to_string());
        Defaults::of(&program.declarations).fill(&mut value, &ty);
        value
    }

    #[test]
    fn a_field_left_out_is_its_default_at_any_depth() {
        let src = "enum Status { draft, sent }\ntype Line { qty: Number = 1, note: String? }\ntype Offers { free: Bool = false }\ntype Quote { number: String, currency: String = \"EUR\", status: Status = .sent, vat: Number = -0.5, lines: [Line] = [], offers: Offers = Offers(), price: Money = €12.99 }\n";
        let got = filled(
            src,
            "Quote",
            json!({ "number": "7", "currency": null, "lines": [{ "note": "x" }, {}] }),
        );
        assert_eq!(
            got,
            json!({
                "number": "7", "currency": "EUR", "status": "sent", "vat": -0.5,
                "lines": [{ "qty": 1, "note": "x" }, { "qty": 1, "note": null }],
                "offers": { "free": false },
                "price": { "amount": 1299, "currency": "EUR" },
            })
        );
        // What is given is kept, and what is not the type is left for the
        // check to report.
        let kept = filled(src, "Quote", json!({ "currency": "GBP", "lines": 3 }));
        assert_eq!(kept["currency"], "GBP");
        assert_eq!(kept["lines"], 3);
        assert!(kept.get("number").is_none());
    }

    #[test]
    fn a_map_written_in_the_language_is_given_its_defaults_as_expressions() {
        let src = "enum Status { draft, sent }\ntype Quote { number: String, currency: String = \"EUR\", status: Status = .sent, note: String? }\nconst Q = { number: \"7\" }\nconst S = { ...Q }\n";
        let program = crate::syntax::parse_source(src, "t.wf").unwrap();
        let defaults = Defaults::of(&program.declarations);
        let ty = TypeRef::List(Box::new(TypeRef::Named("Quote".to_string())));
        let mut value = Expr::ListLiteral(vec![
            program
                .declarations
                .iter()
                .find_map(|d| match d {
                    Declaration::Const(c) if c.name == "Q" => Some(c.value.clone()),
                    _ => None,
                })
                .unwrap(),
        ]);
        defaults.fill_expr(&mut value, &ty);
        let Expr::ListLiteral(items) = &value else {
            panic!()
        };
        let Expr::MapLiteral(pairs) = &items[0] else {
            panic!()
        };
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["number", "currency", "status", "note"]);
        assert!(matches!(&pairs[1].1, Expr::StringLiteral(s) if s == "EUR"));
        assert!(matches!(&pairs[2].1, Expr::EnumCase(c) if c == "sent"));
        assert!(matches!(pairs[3].1, Expr::Null));
    }
}
