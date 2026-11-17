//! A service read from an OpenAPI specification, in JSON or YAML.
//!
//! One reader serves `api Backend from "openapi.json"`, which builds the
//! contract inside the compiler, and `wf api sync`, which writes the same
//! contract as files — so a spec means the same thing either way.
//!
//! What is read, in the file's order: every operation under `paths`, in
//! the group its first tag names (`default` for one with no tag), named by
//! its `operationId` (made a name only where it must be); its parameters,
//! each where the spec puts it — the path, the query, a header, a cookie —
//! under the name the server knows; its request body by media type; its
//! first success, by media type; and every error status it declares, as a
//! case of an error enum named for the operation (`CreateUserError`), with
//! the `http(error: HttpError)` case every error enum has.
//!
//! Every named schema becomes a declaration the program can name: an object
//! a `type`, a string with cases an `enum` (or `String`, when a case cannot
//! be written as a name), and anything else the type it reads as, wherever
//! it is named. An object or a string with cases written in place is a
//! record or an enum named by its place (`CreateUserBody`, `PetStatus`). An
//! `allOf` is a record that extends the first record it names. What says
//! nothing of its shape is `Json`, and a dictionary `[String: T]`. A value
//! that may be null — `"type": ["string", "null"]`, a `oneOf` with a null
//! beside a `$ref`, 3.0's `nullable: true` — is its type made optional, and
//! a string's `format` is the language's type for it (`date-time` a
//! `DateTime`) in what the server sends.
//!
//! Nothing is dropped silently: an operation the language cannot call (an
//! XML body) stops the read with its name, unless the caller skips it.

pub mod doc;
pub mod files;
mod yaml;

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};

use doc::{Object, Value};

use crate::diagnostics::error::{Diagnostic, Result, WebFluentError};
use crate::syntax::parser::ast::{
    ApiDecl, Declaration, Endpoint, EnumCase, EnumDecl, Expr, FieldDecl, ParamLocation, PropDecl,
    Span, TypeDecl, TypeRef,
};

/// What a specification describes, as declarations of the program.
#[derive(Debug, Default)]
pub struct Imported {
    /// The service's groups, one a tag, in the order the spec first names
    /// them.
    pub groups: Vec<Group>,
    /// The types and enums its schemas, inline objects and error statuses
    /// make, in the order they are first met.
    pub declarations: Vec<Declaration>,
    /// What was read but is worth saying: an operation with no
    /// `operationId`, one skipped.
    pub warnings: Vec<String>,
    /// What the service's own file is written from.
    pub facts: Facts,
}

/// One group of a service: the operations one tag names.
#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    /// The tag's description.
    pub doc: Option<String>,
    pub endpoints: Vec<Endpoint>,
    /// Each endpoint's example of a success, as the spec gives one.
    pub examples: Vec<(String, Value)>,
}

/// What a spec says of the service as a whole.
#[derive(Debug, Clone, Default)]
pub struct Facts {
    pub title: Option<String>,
    /// `servers[0].url`, its variables at their defaults.
    pub server: Option<String>,
    /// Whether a security scheme is a bearer token (`type: http, scheme:
    /// bearer`, or OAuth 2, which sends one).
    pub bearer: bool,
    /// The other schemes, by name and kind, for a comment.
    pub schemes: Vec<String>,
}

impl Imported {
    /// The group an endpoint is in, by its name.
    pub fn group(&self, name: &str) -> Option<&Group> {
        self.groups.iter().find(|g| g.name == name)
    }
}

/// Read `text` — JSON or YAML — as an OpenAPI document, for the service
/// `service`. An operation whose name or `METHOD /path` is in `skip` is
/// left out, with a warning; any other the language cannot call stops the
/// read.
///
/// # Errors
///
/// [`WebFluentError::Diagnostics`] (`E111`) when `text` is neither JSON nor
/// YAML, when two operations share a name, or when an operation cannot be
/// read.
pub fn read(text: &str, file: &str, skip: &[String]) -> Result<Imported> {
    let doc = doc::read(text).map_err(|e| {
        refused(
            file,
            e.line,
            e.col,
            format!("`{file}` cannot be read: {}", e.message),
        )
    })?;
    if doc.get("openapi").is_none() && doc.get("swagger").is_some() {
        return Err(refused(
            file,
            1,
            1,
            format!(
                "`{file}` is Swagger 2.0; the reader takes OpenAPI 3 — convert it with `npx swagger2openapi {file} -o openapi.json`"
            ),
        ));
    }
    let empty = Object::new();
    let schemas = doc
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let paths = doc.get("paths").and_then(Value::as_object);
    let spec = Spec::new(&doc, schemas, paths);
    let mut imported = Imported {
        facts: facts_of(&doc),
        ..Imported::default()
    };

    // Every named schema first, in the file's order: each record's inline
    // parts follow it.
    for (name, schema) in schemas {
        match &spec.shapes[name.as_str()] {
            Shape::Record => spec.declare_record(&type_name(name), schema, spec.way_of(name)),
            Shape::Enum(cases) => {
                spec.declare(Declaration::Enum(enum_of(&type_name(name), schema, cases)));
            }
            Shape::Value => {}
        }
    }

    // The operations, in the file's order, each in its first tag's group.
    let tag_docs: HashMap<&str, &str> = doc
        .get("tags")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|t| Some((t.get("name")?.as_str()?, t.get("description")?.as_str()?)))
        .collect();
    let mut problems: Vec<String> = Vec::new();
    let mut names: HashMap<String, String> = HashMap::new();
    let mut groups: Vec<Group> = Vec::new();
    for (path, item, method, operation) in operations(paths) {
        let label = format!("{} {path}", method.to_uppercase());
        let id = operation.get("operationId").and_then(Value::as_str);
        let name = if let Some(id) = id {
            op_name(id)
        } else {
            let made = name_of(method, path);
            imported.warnings.push(format!(
                "`{label}` has no operationId: it is called `{made}` — give it one, so a change of path does not rename it"
            ));
            made
        };
        if skip
            .iter()
            .any(|s| *s == name || *s == label || Some(s.as_str()) == id)
        {
            imported
                .warnings
                .push(format!("`{label}` (`{name}`) is skipped, as asked"));
            continue;
        }
        if let Some(first) = names.insert(name.clone(), label.clone()) {
            problems.push(format!(
                "`{first}` and `{label}` are both called `{name}`; give one another operationId"
            ));
            continue;
        }
        let tag = operation
            .get("tags")
            .and_then(Value::as_array)
            .and_then(|t| t.first())
            .and_then(Value::as_str);
        let group = tag.map_or_else(|| "default".to_string(), group_name);
        match spec.endpoint_of(&name, method, path, operation, item) {
            Ok((endpoint, example)) => {
                let at = if let Some(at) = groups.iter().position(|g| g.name == group) {
                    at
                } else {
                    groups.push(Group {
                        name: group,
                        doc: tag.and_then(|t| tag_docs.get(t)).map(|d| (*d).to_string()),
                        endpoints: Vec::new(),
                        examples: Vec::new(),
                    });
                    groups.len() - 1
                };
                if let Some(example) = example {
                    groups[at].examples.push((endpoint.name.clone(), example));
                }
                groups[at].endpoints.push(endpoint);
            }
            Err(why) => problems.push(format!("`{label}` ({name}): {why}")),
        }
    }
    if !problems.is_empty() {
        let mut message = format!("`{file}` has what the reader cannot call:");
        for p in &problems {
            message.push_str("\n  - ");
            message.push_str(p);
        }
        message.push_str("\nSkip an operation with `wf api sync --skip NAME`, or change the spec");
        return Err(refused(file, 1, 1, message));
    }
    // The extensions a team that owns its spec writes: what a write makes
    // stale, and how a list pages.
    for group in &mut groups {
        Spec::extensions(group, paths);
    }
    imported.groups = groups;
    imported.declarations = spec.declarations.into_inner();
    Ok(imported)
}

fn refused(file: &str, line: usize, col: usize, message: String) -> WebFluentError {
    WebFluentError::Diagnostics(vec![Diagnostic::coded("E111", message, file, line, col)])
}

/// Every operation under `paths`: its path, the path's item, its method and
/// the operation itself, in the file's order.
fn operations(paths: Option<&Object>) -> impl Iterator<Item = (&str, &Value, &str, &Value)> {
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

/// The title, the first server and the security schemes.
fn facts_of(doc: &Value) -> Facts {
    let server = doc.pointer("/servers/0").and_then(|s| {
        let mut url = s.get("url")?.as_str()?.to_string();
        for (name, var) in s
            .get("variables")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            if let Some(default) = var.get("default").and_then(Value::as_str) {
                url = url.replace(&format!("{{{name}}}"), default);
            }
        }
        Some(url)
    });
    let mut bearer = false;
    let mut schemes = Vec::new();
    for (name, scheme) in doc
        .pointer("/components/securitySchemes")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let kind = scheme.get("type").and_then(Value::as_str).unwrap_or("");
        match kind {
            "http"
                if scheme
                    .get("scheme")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.eq_ignore_ascii_case("bearer")) =>
            {
                bearer = true;
            }
            "oauth2" | "openIdConnect" => bearer = true,
            "apiKey" => schemes.push(format!(
                "`{name}`: a key in {} `{}`",
                scheme.get("in").and_then(Value::as_str).unwrap_or("header"),
                scheme.get("name").and_then(Value::as_str).unwrap_or(name)
            )),
            other => schemes.push(format!("`{name}`: {other}")),
        }
    }
    Facts {
        title: doc
            .pointer("/info/title")
            .and_then(Value::as_str)
            .map(str::to_string),
        server,
        bearer,
        schemes,
    }
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

/// A specification's named schemas, read once, and the declarations the
/// read makes.
struct Spec<'a> {
    /// The whole document, which a `$ref` to a shared parameter, request
    /// body or response points into.
    doc: &'a Value,
    schemas: &'a Object,
    /// What each named schema is, by its name in the document.
    shapes: HashMap<&'a str, Shape>,
    /// The names the program declares from it, as the program spells them:
    /// a format's type whose name one of them has taken is a string.
    declared: HashSet<String>,
    /// The named schemas a request reaches, through every `$ref` inside a
    /// parameter's schema or a body's.
    sent: HashSet<&'a str>,
    /// What the read declares, in the order it is met.
    declarations: RefCell<Vec<Declaration>>,
    /// Every name a declaration has taken, so one made from a place is never
    /// a second of a name.
    taken: RefCell<HashSet<String>>,
}

impl<'a> Spec<'a> {
    fn new(doc: &'a Value, schemas: &'a Object, paths: Option<&'a Object>) -> Self {
        let shapes: HashMap<&str, Shape> = schemas
            .iter()
            .map(|(name, schema)| (name.as_str(), shape_of(schema, schemas, 0)))
            .collect();
        let declared: HashSet<String> = shapes
            .iter()
            .filter(|(_, shape)| !matches!(shape, Shape::Value))
            .map(|(name, _)| type_name(name))
            .collect();
        let mut spec = Spec {
            doc,
            schemas,
            shapes,
            taken: RefCell::new(declared.clone()),
            declared,
            sent: HashSet::new(),
            declarations: RefCell::new(Vec::new()),
        };
        let mut sent = HashSet::new();
        for (_, item, _, operation) in operations(paths) {
            for source in [item.get("parameters"), operation.get("parameters")] {
                for parameter in source.and_then(Value::as_array).into_iter().flatten() {
                    let parameter = spec.resolve(parameter);
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

    fn way_of(&self, name: &str) -> Way {
        if self.sent.contains(name) {
            Way::Sent
        } else {
            Way::Received
        }
    }

    fn declare(&self, decl: Declaration) {
        self.declarations.borrow_mut().push(decl);
    }

    /// A name for something written in place, from where it is: the name
    /// itself when nothing has it, else with a number after it.
    fn claim(&self, place: &str) -> String {
        let mut taken = self.taken.borrow_mut();
        let mut name = place.to_string();
        let mut n = 2;
        while taken.contains(&name) {
            name = format!("{place}{n}");
            n += 1;
        }
        taken.insert(name.clone());
        name
    }

    /// What a local `$ref` points at — a shared parameter, a request body,
    /// a response — or `value` itself when it is not one.
    fn resolve(&self, value: &'a Value) -> &'a Value {
        let mut value = value;
        // A `$ref` to a `$ref` is followed; a loop of them is left where it
        // stands.
        for _ in 0..8 {
            let Some(target) = value
                .get("$ref")
                .and_then(Value::as_str)
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
    fn reach(&self, value: &'a Value, into: &mut HashSet<&'a str>) {
        match value {
            Value::Object(map) => {
                if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
                    let schema = reference
                        .strip_prefix("#/components/schemas/")
                        .and_then(|name| self.schemas.get_key_value(name));
                    if let Some((name, target)) = schema {
                        if into.insert(name.as_str()) {
                            self.reach(target, into);
                        }
                    } else {
                        let target = self.resolve(value);
                        if !std::ptr::eq(target, value) {
                            self.reach(target, into);
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
            Value::Array(list) => {
                for inner in list {
                    self.reach(inner, into);
                }
            }
            _ => {}
        }
    }

    /// One operation as an endpoint, and its example of a success.
    fn endpoint_of(
        &self,
        name: &str,
        method: &str,
        path: &str,
        operation: &'a Value,
        item: &'a Value,
    ) -> std::result::Result<(Endpoint, Option<Value>), String> {
        let span = Span::dummy();
        let place = pascal(name);
        let mut params: Vec<PropDecl> = Vec::new();
        let mut locations = Vec::new();
        let mut path_names: HashMap<String, String> = HashMap::new();
        // A path's own parameters count for every operation under it; the
        // operation's own replace one of the same name and place.
        let mut written: Vec<&Value> = Vec::new();
        for source in [item.get("parameters"), operation.get("parameters")] {
            for parameter in source.and_then(Value::as_array).into_iter().flatten() {
                let parameter = self.resolve(parameter);
                let key = |p: &Value| {
                    (
                        p.get("name").and_then(Value::as_str).map(str::to_string),
                        p.get("in").and_then(Value::as_str).map(str::to_string),
                    )
                };
                match written.iter().position(|w| key(w) == key(parameter)) {
                    Some(at) => written[at] = parameter,
                    None => written.push(parameter),
                }
            }
        }
        for parameter in written {
            let Some(raw) = parameter.get("name").and_then(Value::as_str) else {
                continue;
            };
            let place_in = parameter
                .get("in")
                .and_then(Value::as_str)
                .unwrap_or("query");
            let required = place_in == "path"
                || parameter
                    .get("required")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
            let schema = parameter.get("schema").unwrap_or(&Value::Null);
            let ty = self.type_of(schema, Way::Sent, &format!("{place}{}", pascal(raw)));
            let mut pname = param_name(raw);
            while params.iter().any(|p| p.name == pname) {
                pname.push('_');
            }
            // A default of an enum is its case: `order: Order = .asc`.
            let default = match (schema.get("default"), &ty) {
                (Some(Value::String(case)), TypeRef::Named(_)) if self.names_cases(schema) => {
                    Some(Expr::EnumCase(case.clone()))
                }
                (Some(d), _) => literal(d),
                _ => None,
            };
            let ty = if required || default.is_some() {
                ty
            } else {
                optional(ty)
            };
            match place_in {
                "path" => {
                    path_names.insert(raw.to_string(), pname.clone());
                }
                "header" => locations.push((pname.clone(), ParamLocation::Header(raw.to_string()))),
                "cookie" => locations.push((
                    pname.clone(),
                    ParamLocation::Cookie((pname != raw).then(|| raw.to_string())),
                )),
                _ if pname != raw => {
                    locations.push((pname.clone(), ParamLocation::Query(raw.to_string())));
                }
                _ => {}
            }
            params.push(PropDecl {
                name: pname,
                prop_type: ty,
                default,
                positional: false,
                optional: !required,
                doc: parameter
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                span,
            });
        }
        // A `{name}` the path has and no parameter declares is a string the
        // call must give.
        for raw in path_parts(path) {
            if let std::collections::hash_map::Entry::Vacant(slot) = path_names.entry(raw) {
                let pname = param_name(slot.key());
                params.push(prop(&pname, TypeRef::String, false, span));
                slot.insert(pname);
            }
        }

        let mut settings: Vec<(String, Expr)> = Vec::new();
        let request = operation.get("requestBody").map(|b| self.resolve(b));
        if let Some(content) = request
            .and_then(|b| b.get("content"))
            .and_then(Value::as_object)
        {
            let required = request
                .and_then(|b| b.get("required"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let body_place = format!("{place}Body");
            match pick(content) {
                Some((Media::Json, media)) => {
                    let ty = media
                        .get("schema")
                        .map_or_else(json, |s| self.type_of(s, Way::Sent, &body_place));
                    params.push(body_prop(ty, required, span));
                }
                Some((Media::Text, _)) => params.push(body_prop(TypeRef::String, required, span)),
                Some((Media::Form, media)) => {
                    let schema = media
                        .get("schema")
                        .map_or(&Value::Null, |s| self.resolve_schema(s));
                    let files: Vec<&String> = schema
                        .get("properties")
                        .and_then(Value::as_object)
                        .into_iter()
                        .flatten()
                        .filter(|(_, p)| is_binary(p))
                        .map(|(k, _)| k)
                        .collect();
                    let [file] = files.as_slice() else {
                        return Err(format!(
                            "a `multipart/form-data` body is sent with one file; this one has {}",
                            files.len()
                        ));
                    };
                    let rest = Value::Object({
                        let mut o = Object::new();
                        let mut props = Object::new();
                        for (k, v) in schema
                            .get("properties")
                            .and_then(Value::as_object)
                            .into_iter()
                            .flatten()
                        {
                            if k != *file {
                                props.insert(k.clone(), v.clone());
                            }
                        }
                        let any = !props.is_empty();
                        o.insert("type".into(), Value::String("object".into()));
                        o.insert("properties".into(), Value::Object(props));
                        if let Some(r) = schema.get("required") {
                            o.insert("required".into(), r.clone());
                        }
                        if !any {
                            o = Object::new();
                        }
                        o
                    });
                    params.push(prop(
                        &param_name(file),
                        TypeRef::Named("File".into()),
                        false,
                        span,
                    ));
                    settings.push(("fileField".into(), Expr::StringLiteral((*file).clone())));
                    if rest.as_object().is_some_and(|o| !o.is_empty()) {
                        let ty = self.type_of(&rest, Way::Sent, &body_place);
                        params.push(body_prop(ty, required, span));
                    }
                }
                Some((Media::Binary | Media::Other, _)) | None => {
                    let kinds: Vec<&String> = content.keys().collect();
                    return Err(format!(
                        "its request body is {}, which the language does not send",
                        kinds
                            .iter()
                            .map(|k| format!("`{k}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
        }

        // The first success, and every error status.
        let mut returns: Option<TypeRef> = None;
        let mut example = None;
        let mut cases: Vec<EnumCase> = Vec::new();
        for (code, response) in operation
            .get("responses")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let response = self.resolve(response);
            let content = response.get("content").and_then(Value::as_object);
            let status = Status::of(code);
            match status {
                Status::Success if returns.is_none() => {
                    let Some(content) = content.filter(|c| !c.is_empty()) else {
                        returns = Some(TypeRef::Unit);
                        continue;
                    };
                    match pick(content) {
                        Some((Media::Json, media)) => {
                            returns = Some(media.get("schema").map_or_else(json, |s| {
                                self.type_of(s, Way::Received, &format!("{place}Response"))
                            }));
                            example = self.example_of(media);
                        }
                        Some((Media::Text, _)) => {
                            returns = Some(TypeRef::String);
                            settings.push(("as".into(), Expr::EnumCase("text".into())));
                        }
                        Some((Media::Binary, _)) => {
                            returns = Some(TypeRef::Named("File".into()));
                            settings.push(("as".into(), Expr::EnumCase("blob".into())));
                        }
                        _ => {
                            let kinds: Vec<String> =
                                content.keys().map(|k| format!("`{k}`")).collect();
                            return Err(format!(
                                "it answers {}, which the language does not read",
                                kinds.join(", ")
                            ));
                        }
                    }
                }
                Status::Error(n) | Status::Range(n) => {
                    let case = reason(n);
                    if cases.iter().any(|c| c.name == case) {
                        continue;
                    }
                    let payload = match content.and_then(pick) {
                        Some((Media::Json, media)) => media.get("schema").map_or_else(json, |s| {
                            self.type_of(s, Way::Received, &format!("{place}{}", pascal(&case)))
                        }),
                        Some((Media::Text, _)) => TypeRef::String,
                        _ => optional(TypeRef::Named("Problem".into())),
                    };
                    cases.push(EnumCase {
                        name: case,
                        fields: vec![field("problem", payload, None)],
                        status: Some(n),
                    });
                }
                _ => {}
            }
        }
        let value = returns.unwrap_or(TypeRef::Unit);
        let error = if cases.is_empty() {
            None
        } else {
            cases.push(EnumCase {
                name: "http".into(),
                fields: vec![field("error", TypeRef::Named("HttpError".into()), None)],
                status: None,
            });
            let error_name = self.claim(&format!("{place}Error"));
            self.declare(Declaration::Enum(EnumDecl {
                name: error_name.clone(),
                cases,
                doc: None,
                span,
                header_span: span,
            }));
            Some(Box::new(TypeRef::Named(error_name)))
        };
        let mut doc = operation
            .get("summary")
            .or_else(|| operation.get("description"))
            .and_then(Value::as_str)
            .map(|d| d.trim().to_string());
        if operation.get("deprecated").and_then(Value::as_bool) == Some(true) {
            doc = Some(match doc {
                Some(d) => format!("{d}\n\nDeprecated."),
                None => "Deprecated.".to_string(),
            });
        }
        Ok((
            Endpoint {
                method: method.to_uppercase(),
                name: name.to_string(),
                path: to_path(path, &path_names),
                params,
                returns: Some(TypeRef::Result(Box::new(value), error)),
                errors: Vec::new(),
                settings,
                doc,
                span,
                locations,
                path_written: true,
            },
            example,
        ))
    }

    /// Whether a schema is a string with cases the program declares as an
    /// enum, written in place or named.
    fn names_cases(&self, schema: &Value) -> bool {
        let target = match schema.get("$ref").and_then(Value::as_str) {
            Some(r) => match self.schemas.get(ref_name(r)) {
                Some(t) => t,
                None => return false,
            },
            None => schema,
        };
        matches!(shape_of(target, self.schemas, 0), Shape::Enum(_))
    }

    /// A media type's example: `example`, the first of `examples`, or the
    /// schema's own.
    fn example_of(&self, media: &'a Value) -> Option<Value> {
        if let Some(e) = media.get("example") {
            return Some(e.clone());
        }
        if let Some((_, first)) = media
            .get("examples")
            .and_then(Value::as_object)
            .and_then(|o| o.iter().next())
            && let Some(v) = self.resolve(first).get("value")
        {
            return Some(v.clone());
        }
        let schema = self.resolve_schema(media.get("schema")?);
        schema.get("example").cloned()
    }

    /// A schema through its `$ref`s.
    fn resolve_schema(&self, schema: &'a Value) -> &'a Value {
        self.resolve(schema)
    }

    /// The `x-invalidates` and `x-pagination` an operation carries, onto its
    /// endpoint.
    fn extensions(group: &mut Group, paths: Option<&Object>) {
        // The program's name of an operation, by its `operationId`.
        let by_id: HashMap<&str, String> = operations(paths)
            .filter_map(|(_, _, _, op)| {
                let id = op.get("operationId")?.as_str()?;
                Some((id, op_name(id)))
            })
            .collect();
        for (_, _, _, operation) in operations(paths) {
            let Some(id) = operation.get("operationId").and_then(Value::as_str) else {
                continue;
            };
            let name = op_name(id);
            let Some(endpoint) = group.endpoints.iter_mut().find(|e| e.name == name) else {
                continue;
            };
            if let Some(list) = operation.get("x-invalidates").and_then(Value::as_array) {
                let items: Vec<Expr> = list
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|text| {
                        let (target, args) = match text.split_once('(') {
                            Some((t, rest)) => (t.trim(), rest.trim_end_matches(')')),
                            None => (text.trim(), ""),
                        };
                        let target = by_id
                            .get(target)
                            .cloned()
                            .unwrap_or_else(|| op_name(target));
                        let args: Vec<Expr> = args
                            .split(',')
                            .map(str::trim)
                            .filter(|a| !a.is_empty())
                            .map(|a| Expr::Identifier(param_name(a)))
                            .collect();
                        if args.is_empty() {
                            Expr::Identifier(target)
                        } else {
                            Expr::FunctionCall(target, args)
                        }
                    })
                    .collect();
                endpoint
                    .settings
                    .push(("invalidates".into(), Expr::ListLiteral(items)));
            }
            if let Some(paging) = operation.get("x-pagination")
                && let Some(expr) = paging_of(paging)
            {
                endpoint.settings.push(("pages".into(), expr));
            }
        }
    }

    /// A schema as a type the checker knows. `place` names what an object
    /// or a list of cases written in place is declared as.
    fn type_of(&self, schema: &Value, way: Way, place: &str) -> TypeRef {
        self.type_at(schema, way, place, 0)
    }

    fn type_at(&self, schema: &Value, way: Way, place: &str, depth: usize) -> TypeRef {
        // Names that only rename one another end somewhere in a sound
        // specification; a loop of them says nothing of its shape.
        if depth > 32 {
            return json();
        }
        let (kinds, _) = kinds_of(schema);
        let nullable = may_be_null(schema);
        let ty = if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let name = ref_name(reference);
            match (self.schemas.get(name), self.shapes.get(name)) {
                // A name for a string, a number, a list, a `oneOf` declares
                // nothing, so it is what it stands for.
                (Some(target), Some(Shape::Value)) => {
                    self.type_at(target, way, &type_name(name), depth + 1)
                }
                // A record or an enum that says it may be null is `T?`
                // wherever it is named.
                (Some(target), _) if may_be_null(target) => {
                    optional(TypeRef::Named(type_name(name)))
                }
                (Some(_), _) => TypeRef::Named(type_name(name)),
                // A `$ref` into another file, or to nothing.
                (None, _) => json(),
            }
        } else if self.inline_record(schema) {
            let name = self.claim(place);
            self.declare_record(&name, schema, way);
            TypeRef::Named(name)
        } else if let Shape::Enum(cases) = shape_of(schema, self.schemas, 0) {
            let name = self.claim(place);
            self.declare(Declaration::Enum(enum_of(&name, schema, &cases)));
            TypeRef::Named(name)
        } else if let Some(options) = composition(schema).filter(|_| kinds.is_empty()) {
            // `oneOf: [{ "type": "null" }, { "$ref": … }]` is how a 3.1
            // document says a named value may be null, and `allOf: [{ "$ref":
            // … }]` how a field with a description of its own names a type.
            let (nulls, rest): (Vec<&Value>, Vec<&Value>) =
                options.iter().partition(|o| self.is_null(o, depth));
            let ty = match rest.as_slice() {
                [one] => self.type_at(one, way, place, depth + 1),
                // Several shapes, and nothing that says which: JSON, matched
                // or decoded where it is read.
                _ => json(),
            };
            if nulls.is_empty() { ty } else { optional(ty) }
        } else {
            match kinds.as_slice() {
                ["string"] => self.format_of(schema, way).unwrap_or(TypeRef::String),
                // Cases with no `type` are strings all the same.
                [] if string_cases(schema) => TypeRef::String,
                ["integer" | "number"] => TypeRef::Number,
                ["boolean"] => TypeRef::Bool,
                ["array"] => {
                    TypeRef::List(Box::new(schema.get("items").map_or_else(json, |items| {
                        self.type_at(items, way, &format!("{place}Item"), depth + 1)
                    })))
                }
                ["object"] | [] => match schema.get("additionalProperties") {
                    // `{ "additionalProperties": { "type": "integer" } }`: a
                    // dictionary.
                    Some(values @ Value::Object(_))
                        if !values.as_object().is_some_and(Object::is_empty) =>
                    {
                        TypeRef::Dict(
                            Box::new(TypeRef::String),
                            Box::new(self.type_at(
                                values,
                                way,
                                &format!("{place}Value"),
                                depth + 1,
                            )),
                        )
                    }
                    _ => json(),
                },
                _ => json(),
            }
        };
        if nullable { optional(ty) } else { ty }
    }

    /// Whether a schema written in place is an object with fields: a record
    /// of its own, named by its place. An `allOf` of one name is that name.
    fn inline_record(&self, schema: &Value) -> bool {
        if !matches!(shape_of(schema, self.schemas, 0), Shape::Record) {
            return false;
        }
        if schema.get("properties").is_some() {
            return true;
        }
        let parts = schema
            .get("allOf")
            .and_then(Value::as_array)
            .map_or(0, |a| a.iter().filter(|p| !self.is_null(p, 0)).count());
        parts > 1
    }

    /// The language's own type for a string's `format`, where the server's
    /// values are what that type carries: `date-time`, `date`, `uuid`,
    /// `email`, `uri`. A JSON Schema `time` has an offset (`09:30:00Z`) that
    /// `Time` does not read, so it stays a string — as does a format whose
    /// type a schema of the same name has taken, and every format the
    /// program sends.
    fn format_of(&self, schema: &Value, way: Way) -> Option<TypeRef> {
        if way == Way::Sent {
            return None;
        }
        let name = match schema.get("format").and_then(Value::as_str)? {
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
    fn is_null(&self, schema: &Value, depth: usize) -> bool {
        if depth > 32 {
            return false;
        }
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            return self
                .schemas
                .get(ref_name(reference))
                .is_some_and(|target| self.is_null(target, depth + 1));
        }
        let (kinds, nullable) = kinds_of(schema);
        let only_null_cases = schema
            .get("enum")
            .and_then(Value::as_array)
            .is_some_and(|cases| !cases.is_empty() && cases.iter().all(Value::is_null));
        (kinds.is_empty() && nullable && schema.get("type").is_some()) || only_null_cases
    }

    /// A schema as a record the program can name, declared where it is met:
    /// its inline parts follow it.
    fn declare_record(&self, name: &str, schema: &Value, way: Way) {
        let at = self.declarations.borrow().len();
        self.declare(Declaration::Type(TypeDecl {
            alias: None,
            name: name.to_string(),
            extends: None,
            fields: Vec::new(),
            doc: None,
            span: Span::dummy(),
            header_span: Span::dummy(),
        }));
        let mut record = Record {
            spec: self,
            way,
            owner: name.to_string(),
            extends: None,
            fields: Vec::new(),
            read: vec![name.to_string()],
        };
        record.collect(schema, &[], 0);
        let decl = TypeDecl {
            alias: None,
            name: name.to_string(),
            extends: record.extends,
            fields: record.fields,
            doc: schema
                .get("description")
                .and_then(Value::as_str)
                .map(|d| d.trim().to_string()),
            span: Span::dummy(),
            header_span: Span::dummy(),
        };
        self.declarations.borrow_mut()[at] = Declaration::Type(decl);
    }
}

/// `x-pagination: { style: cursor, param: after, next: nextCursor, items: items }`
/// as the language's `.cursor(param: after, next: p => p.nextCursor, items: p => p.items)`.
fn paging_of(paging: &Value) -> Option<Expr> {
    let style = paging.get("style").and_then(Value::as_str)?;
    let lambda = |path: &str| {
        let mut body = Expr::Identifier("p".into());
        for part in path.split('.') {
            body = Expr::PropertyAccess(Box::new(body), part.to_string());
        }
        Expr::Lambda("p".into(), Box::new(body))
    };
    let mut args: Vec<(String, Expr)> = Vec::new();
    if let Some(param) = paging.get("param").and_then(Value::as_str) {
        args.push(("param".into(), Expr::Identifier(param_name(param))));
    }
    for key in ["next", "items"] {
        if let Some(path) = paging.get(key).and_then(Value::as_str) {
            args.push((key.into(), lambda(path)));
        }
    }
    Some(if args.is_empty() {
        Expr::EnumCase(style.to_string())
    } else {
        Expr::CaseValue(style.to_string(), vec![Expr::MapLiteral(args)])
    })
}

/// What a body or an answer is sent as.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Media {
    Json,
    Text,
    Form,
    Binary,
    Other,
}

/// The media type the language reads first: JSON, then text, then a form,
/// then bytes — the first of each kind in the file's order.
fn pick(content: &Object) -> Option<(Media, &Value)> {
    let kind = |m: &str| {
        let m = m.split(';').next().unwrap_or(m).trim().to_ascii_lowercase();
        if m == "application/json" || m.ends_with("+json") || m == "*/*" || m == "application/*" {
            Media::Json
        } else if m.starts_with("text/") && !m.contains("xml") && !m.contains("html") {
            Media::Text
        } else if m == "multipart/form-data" {
            Media::Form
        } else if m == "application/octet-stream"
            || m.starts_with("image/")
            || m.starts_with("audio/")
            || m.starts_with("video/")
            || m == "application/pdf"
            || m == "application/zip"
        {
            Media::Binary
        } else {
            Media::Other
        }
    };
    for want in [Media::Json, Media::Text, Media::Form, Media::Binary] {
        if let Some((_, v)) = content.iter().find(|(m, _)| kind(m) == want) {
            return Some((want, v));
        }
    }
    content.iter().next().map(|(_, v)| (Media::Other, v))
}

/// A binary property of a form: `type: string, format: binary`.
fn is_binary(schema: &Value) -> bool {
    matches!(
        schema.get("format").and_then(Value::as_str),
        Some("binary" | "byte" | "base64")
    ) || schema.get("contentMediaType").is_some()
}

/// A response's status, as the language takes it.
enum Status {
    Success,
    /// `404`: a case of the error enum.
    Error(u16),
    /// `4XX`, `5XX`: stored as 4 and 5.
    Range(u16),
    Other,
}

impl Status {
    fn of(code: &str) -> Status {
        match code.to_ascii_uppercase().as_str() {
            "2XX" => Status::Success,
            "4XX" => Status::Range(4),
            "5XX" => Status::Range(5),
            c => match c.parse::<u16>() {
                Ok(200..=299) => Status::Success,
                Ok(n @ 400..=599) => Status::Error(n),
                _ => Status::Other,
            },
        }
    }
}

/// A status's case: its reason phrase, camelCased (RFC 9110).
pub fn reason(status: u16) -> String {
    match status {
        4 => "clientError",
        5 => "serverError",
        400 => "badRequest",
        401 => "unauthorized",
        402 => "paymentRequired",
        403 => "forbidden",
        404 => "notFound",
        405 => "methodNotAllowed",
        406 => "notAcceptable",
        407 => "proxyAuthenticationRequired",
        408 => "requestTimeout",
        409 => "conflict",
        410 => "gone",
        411 => "lengthRequired",
        412 => "preconditionFailed",
        413 => "contentTooLarge",
        414 => "uriTooLong",
        415 => "unsupportedMediaType",
        416 => "rangeNotSatisfiable",
        417 => "expectationFailed",
        421 => "misdirectedRequest",
        422 => "unprocessableContent",
        423 => "locked",
        424 => "failedDependency",
        425 => "tooEarly",
        426 => "upgradeRequired",
        428 => "preconditionRequired",
        429 => "tooManyRequests",
        431 => "requestHeaderFieldsTooLarge",
        451 => "unavailableForLegalReasons",
        500 => "internalServerError",
        501 => "notImplemented",
        502 => "badGateway",
        503 => "serviceUnavailable",
        504 => "gatewayTimeout",
        505 => "httpVersionNotSupported",
        506 => "variantAlsoNegotiates",
        507 => "insufficientStorage",
        508 => "loopDetected",
        510 => "notExtended",
        511 => "networkAuthenticationRequired",
        n => return format!("status{n}"),
    }
    .to_string()
}

/// A JSON default as the language writes it, when it is a plain value.
fn literal(value: &Value) -> Option<Expr> {
    match value {
        Value::String(s) => Some(Expr::StringLiteral(s.clone())),
        Value::Number(n) => Some(Expr::NumberLiteral(*n)),
        Value::Bool(b) => Some(Expr::BoolLiteral(*b)),
        _ => None,
    }
}

fn json() -> TypeRef {
    TypeRef::Named("Json".into())
}

/// `#/components/schemas/User` is `User`.
fn ref_name(reference: &str) -> &str {
    reference.rsplit('/').next().unwrap_or(reference)
}

/// The types a schema allows besides null, and whether it allows null:
/// `"string"`, 3.1's `["string", "null"]`, or 3.0's `nullable: true`.
fn kinds_of(schema: &Value) -> (Vec<&str>, bool) {
    let mut kinds: Vec<&str> = match schema.get("type") {
        Some(Value::String(kind)) => vec![kind.as_str()],
        Some(Value::Array(list)) => list.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    let nullable =
        kinds.contains(&"null") || schema.get("nullable").and_then(Value::as_bool) == Some(true);
    kinds.retain(|kind| *kind != "null");
    (kinds, nullable)
}

/// Whether a named schema says its values may be null: by its `type`, by
/// `nullable`, or by a `null` among its cases.
fn may_be_null(schema: &Value) -> bool {
    kinds_of(schema).1
        || schema
            .get("enum")
            .and_then(Value::as_array)
            .is_some_and(|cases| cases.iter().any(Value::is_null))
}

/// The options of a `oneOf`, an `anyOf` or an `allOf`.
fn composition(schema: &Value) -> Option<&Vec<Value>> {
    ["oneOf", "anyOf", "allOf"]
        .iter()
        .find_map(|key| schema.get(key).and_then(Value::as_array))
}

/// Whether a schema lists cases, and every one but a `null` is a string.
fn string_cases(schema: &Value) -> bool {
    schema
        .get("enum")
        .and_then(Value::as_array)
        .is_some_and(|cases| {
            cases.iter().any(Value::is_string) && cases.iter().all(|c| c.is_string() || c.is_null())
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

fn shape_of(schema: &Value, schemas: &Object, depth: usize) -> Shape {
    let (kinds, _) = kinds_of(schema);
    if let Some(values) = schema.get("enum").and_then(Value::as_array)
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
        ["object"] | [] if has_fields(schema, schemas, depth) => Shape::Record,
        _ => Shape::Value,
    }
}

/// Whether a schema is an object with fields: it has properties, or an
/// `allOf` part that is a record.
fn has_fields(schema: &Value, schemas: &Object, depth: usize) -> bool {
    if schema
        .get("properties")
        .and_then(Value::as_object)
        .is_some_and(|p| !p.is_empty())
    {
        return true;
    }
    // A loop of names gives no fields.
    if depth > 16 {
        return false;
    }
    schema
        .get("allOf")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|part| {
            let part = match part.get("$ref").and_then(Value::as_str) {
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
    is_name(case) && case != "__proto__"
}

/// Whether `name` is a name as the lexer reads one, in ASCII, and not one of
/// the literal words.
pub fn is_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !matches!(name, "true" | "false" | "null")
}

/// A string schema with cases as an enum: each case is the string the
/// server sends, which is how an enum case is carried at run time.
fn enum_of(name: &str, schema: &Value, cases: &[String]) -> EnumDecl {
    EnumDecl {
        name: name.to_string(),
        cases: cases
            .iter()
            .map(|name| EnumCase {
                name: name.clone(),
                fields: Vec::new(),
                status: None,
            })
            .collect(),
        doc: schema
            .get("description")
            .and_then(Value::as_str)
            .map(|d| d.trim().to_string()),
        span: Span::dummy(),
        header_span: Span::dummy(),
    }
}

/// A record's fields, gathered from a schema and its `allOf` parts.
struct Record<'s, 'a> {
    spec: &'s Spec<'a>,
    way: Way,
    /// The record's name, which a field's inline type is named after.
    owner: String,
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
    fn collect(&mut self, schema: &Value, required: &[String], depth: usize) {
        if depth > 32 {
            return;
        }
        let mut required = required.to_vec();
        required.extend(
            schema
                .get("required")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string),
        );
        for part in schema
            .get("allOf")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(reference) = part.get("$ref").and_then(Value::as_str) else {
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
                self.extends = Some(type_name(base));
            } else {
                self.collect(target, &required, depth + 1);
            }
        }
        for (name, property) in schema
            .get("properties")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            // A `readOnly` field is one only the server sends, whichever way
            // the record travels.
            let way = if property.get("readOnly").and_then(Value::as_bool) == Some(true) {
                Way::Received
            } else {
                self.way
            };
            let ty = self
                .spec
                .type_of(property, way, &format!("{}{}", self.owner, pascal(name)));
            let declared = field(
                name,
                if required.contains(name) {
                    ty
                } else {
                    optional(ty)
                },
                property
                    .get("description")
                    .and_then(Value::as_str)
                    .map(|d| d.trim().to_string()),
            );
            // A field declared again replaces the one before, as one a
            // record declares replaces the one it extends.
            match self.fields.iter().position(|f| f.name == declared.name) {
                Some(at) => self.fields[at] = declared,
                None => self.fields.push(declared),
            }
        }
    }
}

fn field(name: &str, ty: TypeRef, doc: Option<String>) -> FieldDecl {
    FieldDecl {
        name: name.to_string(),
        ty,
        default: None,
        doc,
        span: Span::dummy(),
    }
}

/// A parameter: `optional` when a call may leave it out, which is not the
/// same as a value that may be null.
fn prop(name: &str, ty: TypeRef, optional: bool, span: Span) -> PropDecl {
    PropDecl {
        name: name.to_string(),
        prop_type: ty,
        default: None,
        positional: false,
        optional,
        doc: None,
        span,
    }
}

/// The request body: one a call may leave out is `T?`, unless the spec
/// requires it (OpenAPI's default is that it does not).
fn body_prop(ty: TypeRef, required: bool, span: Span) -> PropDecl {
    prop(
        "body",
        if required { ty } else { optional(ty) },
        !required,
        span,
    )
}

/// The `{name}`s a spec's path names, as written.
fn path_parts(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = path;
    while let Some(at) = rest.find('{') {
        let Some(end) = rest[at..].find('}') else {
            break;
        };
        out.push(rest[at + 1..at + end].to_string());
        rest = &rest[at + end + 1..];
    }
    out
}

/// `/users/{user-id}` as the language spells a path: `users/{userId}`, each
/// part named as its parameter is, and the leading slash gone, since the
/// base carries it.
fn to_path(path: &str, names: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let mut rest = path.trim_start_matches('/');
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        let Some(end) = rest[at..].find('}') else {
            break;
        };
        let raw = &rest[at + 1..at + end];
        out.push('{');
        out.push_str(names.get(raw).map_or(raw, String::as_str));
        out.push('}');
        rest = &rest[at + end + 1..];
    }
    out.push_str(rest);
    out
}

/// `GET /users/{id}` with no operation id is `getUsersId`.
fn name_of(method: &str, path: &str) -> String {
    let mut out = method.to_string();
    for part in path.split(['/', '-', '_', '.', '{', '}']) {
        if part.is_empty() {
            continue;
        }
        out.push_str(&pascal(part));
    }
    out
}

/// An `operationId` as a name: as written when it is one, otherwise its
/// words joined, camelCased (`list-users` → `listUsers`, `getV1Users.json`
/// → `getV1UsersJson`).
pub fn op_name(id: &str) -> String {
    if is_name(id) {
        return id.to_string();
    }
    let made = words_joined(id, false);
    if made.chars().next().is_some_and(|c| c.is_ascii_digit()) || made.is_empty() {
        format!("op{}", pascal(&made))
    } else {
        made
    }
}

/// A tag as a group's name: `Pet Store` is `petStore`, `users` `users`.
pub fn group_name(tag: &str) -> String {
    let made = words_joined(tag, false);
    let mut chars = made.chars();
    let made = match chars.next() {
        Some(c) => format!("{}{}", c.to_ascii_lowercase(), chars.as_str()),
        None => "default".to_string(),
    };
    if made.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("g{made}")
    } else {
        made
    }
}

/// A parameter's name in the program: as the server names it when that is
/// a name, otherwise its words camelCased (`user-id` → `userId`), and the
/// server's name kept where it goes.
fn param_name(raw: &str) -> String {
    if is_name(raw) && raw != "body" {
        return raw.to_string();
    }
    let made = words_joined(raw, false);
    let made = match made.chars().next() {
        Some(c) => format!("{}{}", c.to_ascii_lowercase(), &made[c.len_utf8()..]),
        None => "value".to_string(),
    };
    if made.chars().next().is_some_and(|c| c.is_ascii_digit()) || made == "body" || !is_name(&made)
    {
        format!("p{}", pascal(&made))
    } else {
        made
    }
}

/// A schema's name as a type's: `user-profile` is `UserProfile`,
/// `Pet.Status` `PetStatus`.
pub fn type_name(raw: &str) -> String {
    let made = pascal(raw);
    if made.chars().next().is_some_and(|c| c.is_ascii_digit()) || made.is_empty() {
        format!("T{made}")
    } else {
        made
    }
}

/// The ASCII words of `text`, the first as written and each after it with a
/// capital: `list-users` is `listUsers`.
fn words_joined(text: &str, capital_first: bool) -> String {
    let mut out = String::new();
    for (i, word) in text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .enumerate()
    {
        let mut chars = word.chars();
        let first = chars.next().unwrap_or_default();
        if i == 0 && !capital_first {
            out.push(first);
        } else {
            out.push(first.to_ascii_uppercase());
        }
        out.push_str(chars.as_str());
    }
    out
}

fn pascal(name: &str) -> String {
    words_joined(name, true)
}

/// Fill in every service that names a specification, before anything else
/// reads the program: its groups — one a tag — beside it, and the types its
/// schemas make. Then every service's `invalidations { }` and `pagination
/// { }` onto the endpoints they name.
///
/// # Errors
///
/// [`WebFluentError::Diagnostics`] (`E111`) when a specification's path
/// leaves the project, when it is not found, and what [`read`] returns.
pub fn expand(
    program: &mut crate::syntax::parser::Program,
    read_file: &dyn Fn(&str) -> Option<String>,
) -> Result<()> {
    let mut added: Vec<Declaration> = Vec::new();
    let mut groups: Vec<Declaration> = Vec::new();
    let mut problems: Vec<Diagnostic> = Vec::new();
    for decl in &program.declarations {
        let Declaration::Api(api) = decl else {
            continue;
        };
        let Some(file) = api.from.clone() else {
            continue;
        };
        if !crate::base::vfs::stays_inside(&file) {
            return Err(crate::project::data::outside_project(
                &format!("`api {} from`", api.name),
                &file,
                &file,
                1,
                1,
            ));
        }
        let Some(text) = read_file(&file) else {
            problems.push(Diagnostic::coded(
                "E111",
                format!("`{file}` was not found"),
                &file,
                1,
                1,
            ));
            continue;
        };
        let imported = match read(&text, &file, &[]) {
            Ok(imported) => imported,
            Err(e) => {
                problems.extend(e.diagnostics());
                continue;
            }
        };
        for group in imported.groups {
            groups.push(Declaration::Api(group_decl(&api.name, &group, api.span)));
        }
        added.extend(imported.declarations);
    }
    if !problems.is_empty() {
        return Err(WebFluentError::Diagnostics(problems));
    }
    // A type, an enum or a group the program declares itself wins over one
    // from a spec.
    let own: HashSet<String> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Type(t) => Some(t.name.clone()),
            Declaration::Enum(e) => Some(e.name.clone()),
            Declaration::Api(a) => Some(a.name.clone()),
            _ => None,
        })
        .collect();
    let mut seen: HashSet<String> = HashSet::new();
    for decl in groups.into_iter().chain(added) {
        let name = match &decl {
            Declaration::Type(t) => t.name.clone(),
            Declaration::Enum(e) => e.name.clone(),
            Declaration::Api(a) => a.name.clone(),
            _ => continue,
        };
        if !own.contains(&name) && seen.insert(name) {
            program.declarations.push(decl);
        }
    }
    place_service_blocks(program);
    Ok(())
}

/// A group as the declaration `api Service.group { … }`.
pub fn group_decl(service: &str, group: &Group, span: Span) -> ApiDecl {
    ApiDecl {
        name: format!("{service}.{}", group.name),
        settings: Vec::new(),
        headers: Vec::new(),
        headers_span: Span::dummy(),
        hooks: Vec::new(),
        endpoints: group
            .endpoints
            .iter()
            .cloned()
            .map(|mut e| {
                e.span = span;
                e
            })
            .collect(),
        doc: group.doc.clone(),
        span,
        from: None,
        invalidations: Vec::new(),
        pagination: Vec::new(),
    }
}

/// A service's `invalidations { }` and `pagination { }` — what a spec cannot
/// say about a generated endpoint, written beside it — onto the endpoints
/// they name, as if each had been written there. One that names no endpoint
/// is the checker's to report (`N05`, `N14`).
pub fn place_service_blocks(program: &mut crate::syntax::parser::Program) {
    let mut placed: BTreeMap<(String, String), Vec<(String, Expr)>> = BTreeMap::new();
    for decl in &program.declarations {
        let Declaration::Api(api) = decl else {
            continue;
        };
        for inv in &api.invalidations {
            let Some((group, endpoint)) = inv.endpoint.split_once('.') else {
                continue;
            };
            // A read of another group is the checker's to report, once.
            let items: Vec<Expr> = inv
                .targets
                .iter()
                .filter(|t| t.name.split('.').next() == Some(group))
                .map(|t| {
                    let local = t
                        .name
                        .split_once('.')
                        .map_or(t.name.as_str(), |(_, n)| n)
                        .to_string();
                    if t.args.is_empty() {
                        Expr::Identifier(local)
                    } else {
                        Expr::FunctionCall(
                            local,
                            t.args.iter().map(|a| Expr::Identifier(a.clone())).collect(),
                        )
                    }
                })
                .collect();
            placed
                .entry((format!("{}.{group}", api.name), endpoint.to_string()))
                .or_default()
                .push(("invalidates".into(), Expr::ListLiteral(items)));
        }
        for (endpoint, how, _) in &api.pagination {
            let Some((group, endpoint)) = endpoint.split_once('.') else {
                continue;
            };
            placed
                .entry((format!("{}.{group}", api.name), endpoint.to_string()))
                .or_default()
                .push(("pages".into(), how.clone()));
        }
    }
    if placed.is_empty() {
        return;
    }
    for decl in &mut program.declarations {
        let Declaration::Api(api) = decl else {
            continue;
        };
        for e in &mut api.endpoints {
            if let Some(settings) = placed.get(&(api.name.clone(), e.name.clone())) {
                for (key, value) in settings {
                    e.settings.retain(|(k, _)| k != key);
                    e.settings.push((key.clone(), value.clone()));
                }
            }
        }
    }
}

/// The spec each service is read from, and what it skips: `api B from
/// "openapi.json"`, or the config's `apis`.
pub fn specs_of(
    declarations: &[Declaration],
    apis: &std::collections::BTreeMap<String, crate::project::config::project::ApiSource>,
) -> HashMap<String, (String, Vec<String>)> {
    let mut out: HashMap<String, (String, Vec<String>)> = apis
        .iter()
        .map(|(name, s)| (name.clone(), (s.spec.clone(), s.skip.clone())))
        .collect();
    for d in declarations {
        if let Declaration::Api(a) = d
            && let Some(from) = &a.from
        {
            out.insert(a.name.clone(), (from.clone(), Vec::new()));
        }
    }
    out
}

/// `fake Examples for Backend.users from examples`: every endpoint of the
/// group the fake does not answer itself answers `.ok(example)`, with the
/// example of a success the spec gives — held to the endpoint's result as an
/// answer written by hand is (`N06`).
pub fn answer_from_examples(
    declarations: &mut [Declaration],
    specs: &HashMap<String, (String, Vec<String>)>,
    read_file: &dyn Fn(&str) -> Option<String>,
) {
    let mut seen: HashMap<String, Option<Imported>> = HashMap::new();
    for d in declarations.iter_mut() {
        let Declaration::Fake(f) = d else {
            continue;
        };
        if !f.from_examples {
            continue;
        }
        let Some((service, group)) = f.target.split_once('.') else {
            continue;
        };
        let Some((spec, skip)) = specs.get(service) else {
            continue;
        };
        let imported = seen
            .entry(service.to_string())
            .or_insert_with(|| read_file(spec).and_then(|text| read(&text, spec, skip).ok()));
        let Some(group) = imported.as_ref().and_then(|i| i.group(group)) else {
            continue;
        };
        let declarations = &imported
            .as_ref()
            .map_or(&[][..], |i| i.declarations.as_slice());
        for (endpoint, example) in &group.examples {
            if f.answers.iter().any(|a| a.endpoint == *endpoint) {
                continue;
            }
            let value = group
                .endpoints
                .iter()
                .find(|e| e.name == *endpoint)
                .and_then(|e| match &e.returns {
                    Some(TypeRef::Result(value, _)) => Some(&**value),
                    _ => None,
                });
            f.answers.push(crate::syntax::parser::ast::FakeAnswer {
                endpoint: endpoint.clone(),
                params: Vec::new(),
                body: Expr::CaseValue("ok".into(), vec![expr_of(example, value, declarations)]),
                span: f.header_span,
            });
        }
    }
}

/// A JSON example as the expression that writes it, read as `ty`: a string
/// where an enum is wanted is its case (`"engineer"` is `.engineer`).
fn expr_of(value: &Value, ty: Option<&TypeRef>, declarations: &[Declaration]) -> Expr {
    let ty = match ty {
        Some(TypeRef::Optional(inner) | TypeRef::Refined(inner, _)) => Some(&**inner),
        other => other,
    };
    match value {
        Value::Null => Expr::Null,
        Value::Bool(b) => Expr::BoolLiteral(*b),
        Value::Number(n) => Expr::NumberLiteral(*n),
        Value::String(s) => match ty {
            Some(TypeRef::Named(name))
                if declarations
                    .iter()
                    .any(|d| matches!(d, Declaration::Enum(e) if e.name == *name)) =>
            {
                Expr::EnumCase(s.clone())
            }
            _ => Expr::StringLiteral(s.clone()),
        },
        Value::Array(list) => {
            let item = match ty {
                Some(TypeRef::List(item)) => Some(&**item),
                _ => None,
            };
            Expr::ListLiteral(
                list.iter()
                    .map(|v| expr_of(v, item, declarations))
                    .collect(),
            )
        }
        Value::Object(o) => Expr::MapLiteral(
            o.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        expr_of(v, field_type(ty, k, declarations), declarations),
                    )
                })
                .collect(),
        ),
    }
}

/// The type of `field` in the record `ty` names (its parents' too), or a
/// dictionary's values.
fn field_type<'d>(
    ty: Option<&'d TypeRef>,
    field: &str,
    declarations: &'d [Declaration],
) -> Option<&'d TypeRef> {
    match ty? {
        TypeRef::Dict(_, values) => Some(values),
        TypeRef::Named(name) => {
            let record = declarations.iter().find_map(|d| match d {
                Declaration::Type(t) if t.name == *name => Some(t),
                _ => None,
            })?;
            if let Some(f) = record.fields.iter().find(|f| f.name == field) {
                return Some(&f.ty);
            }
            let parent = record.extends.as_ref()?;
            declarations.iter().find_map(|d| match d {
                Declaration::Type(t) if t.name == *parent => {
                    t.fields.iter().find(|f| f.name == field).map(|f| &f.ty)
                }
                _ => None,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;
