//! A service read from an OpenAPI specification.
//!
//! `api Backend from "openapi.json" (base: env.API)` names a file, and every
//! endpoint, parameter, response type and error shape in it becomes part of
//! the program — so the day the server changes its contract the build says
//! so, rather than the page.
//!
//! What is read: `paths`, each operation's `operationId` (or a name made
//! from the method and the path), its path and query parameters with their
//! types, the request body when it has one, the success response's schema,
//! and the failure responses, which become the endpoint's `errors`.
//! `$ref`s into `components/schemas` are followed, and each named schema
//! becomes a declaration the program can name: an object a `type`, a string
//! with cases an `enum` (or `String`, when a case cannot be written as a
//! name), and anything else the type it reads as, wherever it is named. An
//! `allOf` is a record that extends the first record it names, with every
//! other part as fields of its own. A value that may be null — `"type":
//! ["string", "null"]`, a `oneOf` with a null beside a `$ref`, 3.0's
//! `nullable: true` — is its type made optional, and a string's `format`
//! is the language's type for it (`date-time` a `DateTime`).

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::error::{Diagnostic, Result, WebFluentError};
use crate::parser::ast::{
    ApiDecl, Declaration, Endpoint, EnumCase, EnumDecl, FieldDecl, PropDecl, Span, TypeDecl,
    TypeRef,
};

/// The endpoints, the record types and the enums a specification describes.
pub struct Imported {
    pub endpoints: Vec<Endpoint>,
    pub types: Vec<TypeDecl>,
    pub enums: Vec<EnumDecl>,
}

/// The named schemas of a specification, which a `$ref` is read through.
type Schemas = serde_json::Map<String, serde_json::Value>;

/// Read `text` as an OpenAPI document, for the service `api`.
pub fn read(text: &str, api: &ApiDecl, file: &str) -> Result<Imported> {
    let doc: serde_json::Value = serde_json::from_str(text).map_err(|e| {
        WebFluentError::ParseError(Box::new(Diagnostic::coded(
            "E111",
            format!("`{file}` is not JSON: {e}"),
            file,
            1,
            1,
        )))
    })?;
    let none = Schemas::new();
    let schemas = doc
        .pointer("/components/schemas")
        .and_then(|v| v.as_object())
        .unwrap_or(&none);
    let paths = doc.get("paths").and_then(|v| v.as_object());
    let spec = Spec::new(&doc, schemas, paths);

    let mut types = Vec::new();
    let mut enums = Vec::new();
    for (name, schema) in schemas {
        match &spec.shapes[name.as_str()] {
            Shape::Record => types.push(spec.record_of(name, schema, api.span)),
            Shape::Enum(cases) => enums.push(enum_of(name, schema, cases, api.span)),
            // Nothing to declare: a `$ref` to it reads as what it is.
            Shape::Value => {}
        }
    }
    let mut endpoints = Vec::new();
    for (path, item, method, operation) in operations(paths) {
        endpoints.push(spec.endpoint_of(method, path, operation, item, api.span));
    }
    endpoints.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Imported {
        endpoints,
        types,
        enums,
    })
}

/// Every operation under `paths`: its path, the path's item, its method and
/// the operation itself.
fn operations(
    paths: Option<&Schemas>,
) -> impl Iterator<Item = (&str, &serde_json::Value, &str, &serde_json::Value)> {
    paths.into_iter().flatten().flat_map(|(path, item)| {
        item.as_object()
            .into_iter()
            .flatten()
            .filter(|(method, _)| {
                matches!(
                    method.as_str(),
                    "get" | "post" | "put" | "patch" | "delete" | "head" | "options"
                )
            })
            .map(move |(method, operation)| (path.as_str(), item, method.as_str(), operation))
    })
}

/// Which way a value travels, which decides what a string's `format` is.
#[derive(Clone, Copy, PartialEq)]
enum Way {
    /// What the server sends: a response, an error, a record only those
    /// reach. `date-time` is a `DateTime` here.
    Received,
    /// What the program sends: a parameter, a request body, and every record
    /// one of them reaches. A format stays a `String`, so a value the program
    /// holds as a `String` — a route parameter, an input's text — may be sent.
    Sent,
}

/// A specification's named schemas, read once.
struct Spec<'a> {
    /// The whole document, which a `$ref` to a shared parameter or request
    /// body points into.
    doc: &'a serde_json::Value,
    schemas: &'a Schemas,
    /// What each named schema is, by its name in the document.
    shapes: HashMap<&'a str, Shape>,
    /// The names the program declares from it, as the program spells them:
    /// a format's type whose name one of them has taken is a string.
    declared: HashSet<String>,
    /// The named schemas a request reaches, through every `$ref` inside a
    /// parameter's schema or a body's.
    sent: HashSet<&'a str>,
}

impl<'a> Spec<'a> {
    fn new(doc: &'a serde_json::Value, schemas: &'a Schemas, paths: Option<&'a Schemas>) -> Self {
        let shapes: HashMap<&str, Shape> = schemas
            .iter()
            .map(|(name, schema)| (name.as_str(), shape_of(schema, schemas, 0)))
            .collect();
        let declared = shapes
            .iter()
            .filter(|(_, shape)| !matches!(shape, Shape::Value))
            .map(|(name, _)| pascal(name))
            .collect();
        let mut spec = Spec {
            doc,
            schemas,
            shapes,
            declared,
            sent: HashSet::new(),
        };
        let mut sent = HashSet::new();
        for (_, item, _, operation) in operations(paths) {
            for source in [item.get("parameters"), operation.get("parameters")] {
                for parameter in source.and_then(|v| v.as_array()).into_iter().flatten() {
                    if let Some(schema) = parameter.get("schema") {
                        spec.reach(schema, &mut sent);
                    }
                }
            }
            if let Some(body) = operation.get("requestBody") {
                spec.reach(body, &mut sent);
            }
        }
        spec.sent = sent;
        spec
    }

    /// What a local `$ref` points at — a shared parameter, a request body
    /// — or `value` itself when it is not one.
    fn resolve(&self, value: &'a serde_json::Value) -> &'a serde_json::Value {
        let mut value = value;
        // A `$ref` to a `$ref` is followed; a loop of them is left where it
        // stands.
        for _ in 0..8 {
            let Some(target) = value
                .get("$ref")
                .and_then(|v| v.as_str())
                .and_then(|r| r.strip_prefix('#'))
                .and_then(|pointer| self.doc.pointer(pointer))
            else {
                break;
            };
            value = target;
        }
        value
    }

    /// Every named schema `value` reaches through a `$ref`, and every one
    /// those reach, into `into`. An example, a default or a list of cases is
    /// a value, not a schema, so what it holds reaches nothing.
    fn reach(&self, value: &'a serde_json::Value, into: &mut HashSet<&'a str>) {
        match value {
            serde_json::Value::Object(map) => {
                if let Some(reference) = map.get("$ref").and_then(|v| v.as_str()) {
                    let schema = reference
                        .strip_prefix("#/components/schemas/")
                        .and_then(|name| self.schemas.get_key_value(name));
                    match schema {
                        Some((name, target)) => {
                            if into.insert(name.as_str()) {
                                self.reach(target, into);
                            }
                        }
                        // A shared parameter or request body: what it holds.
                        None => {
                            let target = self.resolve(value);
                            if !std::ptr::eq(target, value) {
                                self.reach(target, into);
                            }
                        }
                    }
                }
                for (key, inner) in map {
                    if !matches!(
                        key.as_str(),
                        "example" | "examples" | "default" | "enum" | "const"
                    ) {
                        self.reach(inner, into);
                    }
                }
            }
            serde_json::Value::Array(list) => {
                for inner in list {
                    self.reach(inner, into);
                }
            }
            _ => {}
        }
    }

    /// One operation as an endpoint.
    fn endpoint_of(
        &self,
        method: &str,
        path: &str,
        operation: &serde_json::Value,
        item: &serde_json::Value,
        span: Span,
    ) -> Endpoint {
        let name = operation
            .get("operationId")
            .and_then(|v| v.as_str())
            .map(camel)
            .unwrap_or_else(|| name_of(method, path));
        let mut params = Vec::new();
        // A path's own parameters count for every operation under it.
        for source in [item.get("parameters"), operation.get("parameters")] {
            for parameter in source.and_then(|v| v.as_array()).into_iter().flatten() {
                // A parameter several operations share is a `$ref` to it.
                let parameter = self.resolve(parameter);
                let Some(pname) = parameter.get("name").and_then(|v| v.as_str()) else {
                    continue;
                };
                let required = parameter
                    .get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let ty = self.type_of(
                    parameter.get("schema").unwrap_or(&serde_json::Value::Null),
                    Way::Sent,
                );
                // Required and nullable is a value that must be passed, and
                // may be null.
                let ty = if required { ty } else { optional(ty) };
                params.push(prop(pname, ty, !required, span));
            }
        }
        let request = operation.get("requestBody").map(|b| self.resolve(b));
        if let Some(body) = request.and_then(|b| b.get("content")) {
            let schema = body
                .as_object()
                .and_then(|m| m.values().next())
                .and_then(|v| v.get("schema"));
            let ty = schema
                .map(|s| self.type_of(s, Way::Sent))
                .unwrap_or(TypeRef::Map);
            // OpenAPI's default: a body is not required unless it says so.
            let required = request
                .and_then(|b| b.get("required"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            params.push(prop("body", ty, !required, span));
        }

        let mut returns = None;
        let mut errors = Vec::new();
        for (code, response) in operation
            .get("responses")
            .and_then(|v| v.as_object())
            .into_iter()
            .flatten()
        {
            let schema = response
                .pointer("/content")
                .and_then(|c| c.as_object())
                .and_then(|m| m.values().next())
                .and_then(|v| v.get("schema"));
            let status: u16 = code.parse().unwrap_or(0);
            match status {
                200..=299 if returns.is_none() => {
                    returns = schema.map(|s| self.type_of(s, Way::Received))
                }
                400..=599 => {
                    if let Some(schema) = schema {
                        errors.push((status, self.type_of(schema, Way::Received)));
                    }
                }
                _ => {}
            }
        }
        Endpoint {
            method: method.to_uppercase(),
            name,
            // The path as the language writes one: `{id}` is `:id`, and the
            // leading slash goes, since the base carries it.
            path: to_path(path),
            params,
            returns,
            errors,
            settings: Vec::new(),
            doc: operation
                .get("summary")
                .or_else(|| operation.get("description"))
                .and_then(|v| v.as_str())
                .map(str::to_string),
            span,
        }
    }

    /// A schema as a type the checker knows.
    fn type_of(&self, schema: &serde_json::Value, way: Way) -> TypeRef {
        self.type_at(schema, way, 0)
    }

    fn type_at(&self, schema: &serde_json::Value, way: Way, depth: usize) -> TypeRef {
        // Names that only rename one another end somewhere in a sound
        // specification; a loop of them is anything.
        if depth > 32 {
            return TypeRef::Any;
        }
        // Whether it may be null is said beside everything else, so it is
        // read first: 3.0's `nullable: true` sits next to an `allOf` or a
        // `$ref` as often as next to a `type`, and a `null` among the cases
        // says it too.
        let (kinds, _) = kinds_of(schema);
        let nullable = may_be_null(schema);
        let ty = if let Some(reference) = schema.get("$ref").and_then(|v| v.as_str()) {
            let name = ref_name(reference);
            match (self.schemas.get(name), self.shapes.get(name)) {
                // A name for a string, a number, a list, a `oneOf` declares
                // nothing, so it is what it stands for.
                (Some(target), Some(Shape::Value)) => self.type_at(target, way, depth + 1),
                // A record or an enum that says it may be null is `T?`
                // wherever it is named.
                (Some(target), _) if may_be_null(target) => {
                    TypeRef::Optional(Box::new(TypeRef::Named(pascal(name))))
                }
                _ => TypeRef::Named(pascal(name)),
            }
        } else if let Some(options) = composition(schema).filter(|_| kinds.is_empty()) {
            // `oneOf: [{ "type": "null" }, { "$ref": … }]` is how a 3.1
            // document says a named value may be null, and `allOf: [{ "$ref":
            // … }]` how a field with a description of its own names a type.
            // Beside a `type`, a composition only narrows it.
            let (nulls, rest): (Vec<_>, Vec<_>) =
                options.iter().partition(|o| self.is_null(o, depth));
            let ty = match rest.as_slice() {
                [one] => self.type_at(one, way, depth + 1),
                _ => TypeRef::Any,
            };
            if nulls.is_empty() { ty } else { optional(ty) }
        } else {
            match kinds.as_slice() {
                ["string"] => self.format_of(schema, way).unwrap_or(TypeRef::String),
                // Cases with no `type` are strings all the same.
                [] if string_cases(schema) => TypeRef::String,
                ["integer"] | ["number"] => TypeRef::Number,
                ["boolean"] => TypeRef::Bool,
                ["array"] => TypeRef::List(Box::new(
                    schema
                        .get("items")
                        .map(|items| self.type_at(items, way, depth + 1))
                        .unwrap_or(TypeRef::Any),
                )),
                ["object"] => TypeRef::Map,
                _ => TypeRef::Any,
            }
        };
        if nullable { optional(ty) } else { ty }
    }

    /// The language's own type for a string's `format`, where the server's
    /// values are what that type carries: `date-time`, `date`, `uuid`,
    /// `email`, `uri`. A JSON Schema `time` has an offset (`09:30:00Z`) that
    /// `Time` does not read, so it stays a string — as does a format whose
    /// type a schema of the same name has taken, and every format the
    /// program sends.
    fn format_of(&self, schema: &serde_json::Value, way: Way) -> Option<TypeRef> {
        if way == Way::Sent {
            return None;
        }
        let name = match schema.get("format").and_then(|v| v.as_str())? {
            "date-time" => "DateTime",
            "date" => "Date",
            "uuid" => "Uuid",
            "email" => "Email",
            "uri" | "url" => "Url",
            _ => return None,
        };
        (!self.declared.contains(name)).then(|| TypeRef::Named(name.to_string()))
    }

    /// The half of a `oneOf` that says "or nothing": `{ "type": "null" }`,
    /// cases that are only `null`, or a `$ref` to either.
    fn is_null(&self, schema: &serde_json::Value, depth: usize) -> bool {
        if depth > 32 {
            return false;
        }
        if let Some(reference) = schema.get("$ref").and_then(|v| v.as_str()) {
            return self
                .schemas
                .get(ref_name(reference))
                .is_some_and(|target| self.is_null(target, depth + 1));
        }
        let (kinds, nullable) = kinds_of(schema);
        let only_null_cases = schema
            .get("enum")
            .and_then(|v| v.as_array())
            .is_some_and(|cases| !cases.is_empty() && cases.iter().all(|c| c.is_null()));
        (kinds.is_empty() && nullable && schema.get("type").is_some()) || only_null_cases
    }

    /// A named schema as a record the program can name.
    fn record_of(&self, name: &str, schema: &serde_json::Value, span: Span) -> TypeDecl {
        let way = if self.sent.contains(name) {
            Way::Sent
        } else {
            Way::Received
        };
        let mut record = Record {
            spec: self,
            way,
            span,
            extends: None,
            fields: Vec::new(),
            read: vec![name.to_string()],
        };
        record.collect(schema, &[], 0);
        TypeDecl {
            name: pascal(name),
            extends: record.extends,
            fields: record.fields,
            doc: schema
                .get("description")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            span,
            header_span: span,
        }
    }
}

/// `#/components/schemas/User` is `User`.
fn ref_name(reference: &str) -> &str {
    reference.rsplit('/').next().unwrap_or(reference)
}

/// The types a schema allows besides null, and whether it allows null:
/// `"string"`, 3.1's `["string", "null"]`, or 3.0's `nullable: true`.
fn kinds_of(schema: &serde_json::Value) -> (Vec<&str>, bool) {
    let mut kinds: Vec<&str> = match schema.get("type") {
        Some(serde_json::Value::String(kind)) => vec![kind.as_str()],
        Some(serde_json::Value::Array(list)) => list.iter().filter_map(|v| v.as_str()).collect(),
        _ => Vec::new(),
    };
    let nullable =
        kinds.contains(&"null") || schema.get("nullable").and_then(|v| v.as_bool()) == Some(true);
    kinds.retain(|kind| *kind != "null");
    (kinds, nullable)
}

/// Whether a named schema says its values may be null: by its `type`, by
/// `nullable`, or by a `null` among its cases.
fn may_be_null(schema: &serde_json::Value) -> bool {
    kinds_of(schema).1
        || schema
            .get("enum")
            .and_then(|v| v.as_array())
            .is_some_and(|cases| cases.iter().any(|c| c.is_null()))
}

/// The options of a `oneOf`, an `anyOf` or an `allOf`.
fn composition(schema: &serde_json::Value) -> Option<&Vec<serde_json::Value>> {
    ["oneOf", "anyOf", "allOf"]
        .iter()
        .find_map(|key| schema.get(*key).and_then(|v| v.as_array()))
}

/// Whether a schema lists cases, and every one but a `null` is a string.
fn string_cases(schema: &serde_json::Value) -> bool {
    schema
        .get("enum")
        .and_then(|v| v.as_array())
        .is_some_and(|cases| {
            cases.iter().any(|c| c.is_string())
                && cases.iter().all(|c| c.is_string() || c.is_null())
        })
}

/// `T?`, once: a value that may be null and is not required is still `T?`.
fn optional(ty: TypeRef) -> TypeRef {
    match ty {
        TypeRef::Optional(_) => ty,
        ty => TypeRef::Optional(Box::new(ty)),
    }
}

/// What a named schema is to the program.
enum Shape {
    /// An object with fields: a `type`.
    Record,
    /// A string whose every case can be written as a case name: an `enum`.
    Enum(Vec<String>),
    /// Anything else — a string, a number, a list, a `oneOf`, another name,
    /// a string with a case no name can spell (`"operate.flags"`) — is the
    /// type it reads as, wherever it is named.
    Value,
}

fn shape_of(schema: &serde_json::Value, schemas: &Schemas, depth: usize) -> Shape {
    let (kinds, _) = kinds_of(schema);
    if let Some(values) = schema.get("enum").and_then(|v| v.as_array())
        && matches!(kinds.as_slice(), [] | ["string"])
    {
        // A null among the cases says the value may be null; the cases are
        // the rest.
        let cases: Option<Vec<String>> = values
            .iter()
            .filter(|v| !v.is_null())
            .map(|v| v.as_str().filter(|c| is_case_name(c)).map(str::to_string))
            .collect();
        return match cases {
            Some(cases) if !cases.is_empty() => Shape::Enum(cases),
            _ => Shape::Value,
        };
    }
    match kinds.as_slice() {
        ["object"] => Shape::Record,
        [] if has_fields(schema, schemas, depth) => Shape::Record,
        _ => Shape::Value,
    }
}

/// Whether a schema with no `type` is an object all the same: it has
/// properties, or an `allOf` part that is a record.
fn has_fields(schema: &serde_json::Value, schemas: &Schemas, depth: usize) -> bool {
    if schema.get("properties").is_some() {
        return true;
    }
    // A loop of names gives no fields.
    if depth > 16 {
        return false;
    }
    schema
        .get("allOf")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .any(|part| {
            let part = match part.get("$ref").and_then(|v| v.as_str()) {
                Some(reference) => match schemas.get(ref_name(reference)) {
                    Some(target) => target,
                    None => return false,
                },
                None => part,
            };
            matches!(shape_of(part, schemas, depth + 1), Shape::Record)
        })
}

/// Whether `case` can be written `.case`, and compiled as a JavaScript
/// property name: a word as the lexer reads one (`prod`, `tech_lead`, `SA`)
/// in ASCII, not one of the literal words, and not `__proto__`, which an
/// object literal reads as its prototype.
fn is_case_name(case: &str) -> bool {
    let mut chars = case.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !matches!(case, "true" | "false" | "null" | "__proto__")
}

/// A named string schema with cases as an enum: each case is the string
/// the server sends, which is how an enum case is carried at run time.
fn enum_of(name: &str, schema: &serde_json::Value, cases: &[String], span: Span) -> EnumDecl {
    EnumDecl {
        name: pascal(name),
        cases: cases
            .iter()
            .map(|name| EnumCase {
                name: name.clone(),
                fields: Vec::new(),
            })
            .collect(),
        doc: schema
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        span,
        header_span: span,
    }
}

/// A record's fields, gathered from a schema and its `allOf` parts.
struct Record<'s, 'a> {
    spec: &'s Spec<'a>,
    way: Way,
    span: Span,
    /// `type ServerDetail = EnvSummary { … }`: the first record an `allOf`
    /// names. A record extends one, so every other part — a second record
    /// named, an object written in place — is fields of its own.
    extends: Option<String>,
    fields: Vec<FieldDecl>,
    /// The schemas whose fields are already here, so a part named twice —
    /// or a record that names itself — is read once.
    read: Vec<String>,
}

impl Record<'_, '_> {
    /// The fields `schema` declares, its parts' first. `required` is what an
    /// enclosing schema requires, which a part's properties answer to too.
    fn collect(&mut self, schema: &serde_json::Value, required: &[String], depth: usize) {
        if depth > 32 {
            return;
        }
        let mut required = required.to_vec();
        required.extend(
            schema
                .get("required")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str())
                .map(str::to_string),
        );
        for part in schema
            .get("allOf")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let Some(reference) = part.get("$ref").and_then(|v| v.as_str()) else {
                self.collect(part, &required, depth + 1);
                continue;
            };
            let base = ref_name(reference);
            // A part that names no record has no fields to give.
            let Some(target) = self
                .spec
                .schemas
                .get(base)
                .filter(|_| matches!(self.spec.shapes.get(base), Some(Shape::Record)))
            else {
                continue;
            };
            if self.read.iter().any(|r| r == base) {
                continue;
            }
            self.read.push(base.to_string());
            if self.extends.is_none() {
                self.extends = Some(pascal(base));
            } else {
                self.collect(target, &required, depth + 1);
            }
        }
        for (field, property) in schema
            .get("properties")
            .and_then(|v| v.as_object())
            .into_iter()
            .flatten()
        {
            // A `readOnly` field is one only the server sends, whichever way
            // the record travels.
            let way = if property.get("readOnly").and_then(|v| v.as_bool()) == Some(true) {
                Way::Received
            } else {
                self.way
            };
            let ty = self.spec.type_of(property, way);
            let declared = FieldDecl {
                name: field.clone(),
                ty: if required.contains(field) {
                    ty
                } else {
                    optional(ty)
                },
                default: None,
                doc: property
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                span: self.span,
            };
            // A field declared again replaces the one before, as one a
            // record declares replaces the one it extends.
            match self.fields.iter().position(|f| f.name == declared.name) {
                Some(at) => self.fields[at] = declared,
                None => self.fields.push(declared),
            }
        }
    }
}

/// A parameter: `optional` when a call may leave it out, which is not the
/// same as a value that may be null.
fn prop(name: &str, ty: TypeRef, optional: bool, span: Span) -> PropDecl {
    PropDecl {
        name: camel(name),
        prop_type: ty,
        default: None,
        positional: false,
        optional,
        doc: None,
        span,
    }
}

/// `/users/{id}` as the language spells a path: `users/:id`.
fn to_path(path: &str) -> String {
    let mut out = String::new();
    let mut rest = path.trim_start_matches('/');
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        let Some(end) = rest[at..].find('}') else {
            break;
        };
        out.push(':');
        out.push_str(&rest[at + 1..at + end]);
        rest = &rest[at + end + 1..];
    }
    out.push_str(rest);
    out
}

/// `GET /users/{id}` with no operation id is `getUsersId`.
fn name_of(method: &str, path: &str) -> String {
    let mut out = method.to_string();
    for part in path.split('/') {
        let part = part.trim_matches(['{', '}']);
        if part.is_empty() {
            continue;
        }
        out.push_str(&pascal(part));
    }
    out
}

/// `user_name` and `user-name` are `userName`; a name already camel is
/// left as it is.
fn camel(name: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in name.chars() {
        if c == '_' || c == '-' || c == ' ' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn pascal(name: &str) -> String {
    let camel = camel(name);
    let mut chars = camel.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => camel,
    }
}

/// Fill in every service that names a specification, before anything else
/// reads the program.
pub fn expand(
    program: &mut crate::parser::Program,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> Result<()> {
    let mut added: BTreeMap<String, Declaration> = BTreeMap::new();
    for decl in &mut program.declarations {
        let Declaration::Api(api) = decl else {
            continue;
        };
        let Some(file) = api.from.clone() else {
            continue;
        };
        let Some(text) = read_file(&file) else {
            return Err(WebFluentError::ParseError(Box::new(Diagnostic::coded(
                "E111",
                format!("`{file}` was not found"),
                &file,
                1,
                1,
            ))));
        };
        let imported = read(&text, api, &file)?;
        api.endpoints.extend(imported.endpoints);
        for ty in imported.types {
            added
                .entry(ty.name.clone())
                .or_insert(Declaration::Type(ty));
        }
        for en in imported.enums {
            added
                .entry(en.name.clone())
                .or_insert(Declaration::Enum(en));
        }
    }
    // A type or an enum the program declares itself wins over one from a
    // spec.
    let own: Vec<String> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Type(t) => Some(t.name.clone()),
            Declaration::Enum(e) => Some(e.name.clone()),
            _ => None,
        })
        .collect();
    for (name, decl) in added {
        if !own.contains(&name) {
            program.declarations.push(decl);
        }
    }
    Ok(())
}
