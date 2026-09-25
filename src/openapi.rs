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
//! becomes a `type` declaration the program can name.

use std::collections::BTreeMap;

use crate::error::{Diagnostic, Result, WebFluentError};
use crate::parser::ast::{
    ApiDecl, Declaration, Endpoint, FieldDecl, PropDecl, Span, TypeDecl, TypeRef,
};

/// The endpoints and the record types a specification describes.
pub struct Imported {
    pub endpoints: Vec<Endpoint>,
    pub types: Vec<TypeDecl>,
}

/// Read `text` as an OpenAPI document, for the service `api`.
pub fn read(text: &str, api: &ApiDecl, file: &str) -> Result<Imported> {
    let spec: serde_json::Value = serde_json::from_str(text).map_err(|e| {
        WebFluentError::ParseError(Diagnostic::new(
            format!("`{file}` is not JSON: {e}"),
            file,
            1,
            1,
        ))
    })?;
    let mut types = Vec::new();
    if let Some(schemas) = spec
        .pointer("/components/schemas")
        .and_then(|v| v.as_object())
    {
        for (name, schema) in schemas {
            types.push(record_of(name, schema, api.span));
        }
    }
    let mut endpoints = Vec::new();
    let paths = spec.get("paths").and_then(|v| v.as_object());
    for (path, item) in paths.into_iter().flatten() {
        let Some(operations) = item.as_object() else {
            continue;
        };
        for (method, operation) in operations {
            if !matches!(
                method.as_str(),
                "get" | "post" | "put" | "patch" | "delete" | "head" | "options"
            ) {
                continue;
            }
            endpoints.push(endpoint_of(method, path, operation, item, api.span));
        }
    }
    endpoints.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Imported { endpoints, types })
}

/// One operation as an endpoint.
fn endpoint_of(
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
            let Some(pname) = parameter.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let required = parameter
                .get("required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let ty = type_of(parameter.get("schema").unwrap_or(&serde_json::Value::Null));
            params.push(prop(
                pname,
                if required {
                    ty
                } else {
                    TypeRef::Optional(Box::new(ty))
                },
                span,
            ));
        }
    }
    if let Some(body) = operation.pointer("/requestBody/content") {
        let schema = body
            .as_object()
            .and_then(|m| m.values().next())
            .and_then(|v| v.get("schema"));
        params.push(prop(
            "body",
            schema.map(type_of).unwrap_or(TypeRef::Map),
            span,
        ));
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
            200..=299 if returns.is_none() => returns = schema.map(type_of),
            400..=599 => {
                if let Some(schema) = schema {
                    errors.push((status, type_of(schema)));
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
fn type_of(schema: &serde_json::Value) -> TypeRef {
    if let Some(reference) = schema.get("$ref").and_then(|v| v.as_str()) {
        let name = reference.rsplit('/').next().unwrap_or(reference);
        return TypeRef::Named(pascal(name));
    }
    match schema.get("type").and_then(|v| v.as_str()) {
        Some("string") => TypeRef::String,
        Some("integer") | Some("number") => TypeRef::Number,
        Some("boolean") => TypeRef::Bool,
        Some("array") => TypeRef::List(Box::new(
            schema.get("items").map(type_of).unwrap_or(TypeRef::Any),
        )),
        Some("object") => TypeRef::Map,
        _ => TypeRef::Any,
    }
}

/// A named schema as a record the program can name.
fn record_of(name: &str, schema: &serde_json::Value, span: Span) -> TypeDecl {
    let required: Vec<&str> = schema
        .get("required")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let mut fields = Vec::new();
    for (field, property) in schema
        .get("properties")
        .and_then(|v| v.as_object())
        .into_iter()
        .flatten()
    {
        let ty = type_of(property);
        fields.push(FieldDecl {
            name: field.clone(),
            ty: if required.contains(&field.as_str()) {
                ty
            } else {
                TypeRef::Optional(Box::new(ty))
            },
            default: None,
            doc: property
                .get("description")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            span,
        });
    }
    TypeDecl {
        name: pascal(name),
        extends: None,
        fields,
        doc: schema
            .get("description")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        span,
        header_span: span,
    }
}

fn prop(name: &str, ty: TypeRef, span: Span) -> PropDecl {
    let optional = matches!(ty, TypeRef::Optional(_));
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
    let mut added: BTreeMap<String, TypeDecl> = BTreeMap::new();
    for decl in &mut program.declarations {
        let Declaration::Api(api) = decl else {
            continue;
        };
        let Some(file) = api.from.clone() else {
            continue;
        };
        let Some(text) = read_file(&file) else {
            return Err(WebFluentError::ParseError(Diagnostic::new(
                format!("`{file}` was not found"),
                &file,
                1,
                1,
            )));
        };
        let imported = read(&text, api, &file)?;
        api.endpoints.extend(imported.endpoints);
        for ty in imported.types {
            added.entry(ty.name.clone()).or_insert(ty);
        }
    }
    // A type the program declares itself wins over one from a spec.
    let own: Vec<String> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Type(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect();
    for (name, ty) in added {
        if !own.contains(&name) {
            program.declarations.push(Declaration::Type(ty));
        }
    }
    Ok(())
}
