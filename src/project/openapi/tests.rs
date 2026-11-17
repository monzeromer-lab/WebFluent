use super::*;
use crate::syntax::print;

/// The spec read, every declaration printed as `wf api sync` writes it:
/// each group, then the types.
fn printed(spec: &str) -> String {
    let imported = read(spec, "spec.json", &[]).unwrap_or_else(|e| panic!("{e}"));
    let mut program = crate::syntax::parser::Program {
        declarations: Vec::new(),
    };
    for g in &imported.groups {
        program
            .declarations
            .push(Declaration::Api(group_decl("Backend", g, Span::dummy())));
    }
    program.declarations.extend(imported.declarations);
    print::file_text(&program)
}

const PEOPLE: &str = r##"{
  "openapi": "3.1.0",
  "info": { "title": "People", "version": "1" },
  "servers": [{ "url": "https://{region}.example.com/v1", "variables": { "region": { "default": "eu" } } }],
  "tags": [{ "name": "people", "description": "Who is on the team." }],
  "components": {
    "securitySchemes": { "bearerAuth": { "type": "http", "scheme": "bearer" } },
    "schemas": {
      "Person": { "type": "object", "required": ["id", "name", "role"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "name": { "type": "string" },
          "role": { "type": "string", "enum": ["engineer", "designer"] },
          "address": { "type": "object", "properties": { "city": { "type": "string" } } },
          "first-name": { "type": "string" },
          "labels": { "type": "object", "additionalProperties": { "type": "string" } },
          "extra": { "type": "object" }
        } },
      "PersonPage": { "type": "object", "required": ["items"],
        "properties": { "items": { "type": "array", "items": { "$ref": "#/components/schemas/Person" } },
                        "nextCursor": { "type": ["string", "null"] } } },
      "Problem": { "type": "object", "properties": { "title": { "type": "string" } } }
    },
    "responses": {
      "NotFound": { "description": "gone", "content": { "application/problem+json":
        { "schema": { "$ref": "#/components/schemas/Problem" } } } }
    }
  },
  "paths": {
    "/people": {
      "get": { "operationId": "list-people", "tags": ["people"], "summary": "Everyone.",
        "parameters": [
          { "name": "q", "in": "query", "schema": { "type": "string" } },
          { "name": "page-size", "in": "query", "schema": { "type": "integer", "default": 20 } },
          { "name": "X-Tenant", "in": "header", "required": true, "schema": { "type": "string" } }
        ],
        "x-pagination": { "style": "cursor", "param": "after", "next": "nextCursor", "items": "items" },
        "responses": { "200": { "content": { "application/json": {
          "schema": { "$ref": "#/components/schemas/PersonPage" },
          "example": { "items": [], "nextCursor": null } } } } } },
      "post": { "operationId": "createPerson", "tags": ["people"],
        "x-invalidates": ["list-people"],
        "requestBody": { "required": true, "content": { "application/json": {
          "schema": { "type": "object", "required": ["name"], "properties": { "name": { "type": "string" } } } } } },
        "responses": {
          "201": { "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Person" } } } },
          "422": { "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Problem" } } } },
          "4XX": { "description": "anything else of the client's" },
          "default": { "description": "who knows" } } }
    },
    "/people/{person_id}": {
      "parameters": [{ "name": "person_id", "in": "path", "required": true, "schema": { "type": "string" } }],
      "delete": { "operationId": "deletePerson", "tags": ["people"], "deprecated": true,
        "responses": { "204": { "description": "gone" }, "404": { "$ref": "#/components/responses/NotFound" } } }
    },
    "/health": { "get": { "responses": { "200": { "content": { "text/plain": { "schema": { "type": "string" } } } } } } },
    "/avatar": { "post": { "operationId": "upload", "requestBody": { "content": { "multipart/form-data": {
      "schema": { "type": "object", "properties": { "image": { "type": "string", "format": "binary" }, "caption": { "type": "string" } } } } } },
      "responses": { "200": { "content": { "image/png": {} } } } } }
  }
}"##;

#[test]
fn a_spec_reads_as_groups_records_and_error_enums_in_its_own_order() {
    let text = printed(PEOPLE);
    let expected = r#"/// Who is on the team.
api Backend.people {
    /// Everyone.
    get listPeople(q: String?, pageSize: Number = 20 in query "page-size", xTenant: String in header "X-Tenant") at "people" -> PersonPage! pages .cursor(param: after, next: p => p.nextCursor, items: p => p.items)
    post createPerson(body: CreatePersonBody) at "people" -> Person ! CreatePersonError invalidates [listPeople]
    /// Deprecated.
    delete deletePerson(person_id: String) at "people/{person_id}" -> ! DeletePersonError
}

api Backend.default {
    get getHealth at "health" -> String! as: .text
    post upload(image: File, body: UploadBody?) at "avatar" -> File! fileField: "image" as: .blob
}
"#;
    assert!(text.starts_with(expected), "{text}");
    for decl in [
        "type Person {\n    id: Uuid\n    name: String\n    role: PersonRole\n    address: PersonAddress?\n    \"first-name\": String?\n    labels: [String: String]?\n    extra: Json?\n}",
        "enum PersonRole {\n    engineer\n    designer\n}",
        "type PersonAddress {\n    city: String?\n}",
        "type PersonPage {\n    items: [Person]\n    nextCursor: String?\n}",
        "type CreatePersonBody {\n    name: String\n}",
        "enum CreatePersonError {\n    unprocessableContent(problem: Problem) on 422\n    clientError(problem: Problem?) on 4XX\n    http(error: HttpError)\n}",
        "enum DeletePersonError {\n    notFound(problem: Problem) on 404\n    http(error: HttpError)\n}",
        "type UploadBody {\n    caption: String?\n}",
    ] {
        assert!(text.contains(decl), "missing\n{decl}\nin\n{text}");
    }
    // Each record is followed by what it declares in place.
    let person = text.find("type Person {").unwrap();
    let role = text.find("enum PersonRole").unwrap();
    let page = text.find("type PersonPage").unwrap();
    assert!(person < role && role < page, "{text}");
}

#[test]
fn what_a_spec_says_of_the_service_and_its_examples() {
    let imported = read(PEOPLE, "spec.json", &[]).unwrap();
    assert_eq!(imported.facts.title.as_deref(), Some("People"));
    assert_eq!(
        imported.facts.server.as_deref(),
        Some("https://eu.example.com/v1")
    );
    assert!(imported.facts.bearer);
    let people = imported.group("people").unwrap();
    assert_eq!(people.examples.len(), 1);
    assert_eq!(people.examples[0].0, "listPeople");
    assert!(
        imported
            .warnings
            .iter()
            .any(|w| w.contains("`GET /health` has no operationId")),
        "{:?}",
        imported.warnings
    );
}

#[test]
fn yaml_reads_as_the_same_json() {
    let yaml = r#"
openapi: 3.0.3
info:
  title: Pets
  version: "1"
paths:
  /pets/{petId}:
    get:
      operationId: showPet
      tags: [pets]
      parameters:
        - name: petId
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: one pet
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Pet'
components:
  schemas:
    Pet:
      type: object
      required: [id]
      properties:
        id: { type: integer }
        tag:
          type: string
          description: >
            A label, folded
            over two lines.
"#;
    let json = r##"{ "openapi": "3.0.3", "info": { "title": "Pets", "version": "1" },
      "paths": { "/pets/{petId}": { "get": { "operationId": "showPet", "tags": ["pets"],
        "parameters": [{ "name": "petId", "in": "path", "required": true, "schema": { "type": "string" } }],
        "responses": { "200": { "description": "one pet", "content": { "application/json":
          { "schema": { "$ref": "#/components/schemas/Pet" } } } } } } } },
      "components": { "schemas": { "Pet": { "type": "object", "required": ["id"], "properties": {
        "id": { "type": "integer" }, "tag": { "type": "string", "description": "A label, folded over two lines.\n" } } } } } }"##;
    assert_eq!(printed(yaml), printed(json));
    assert!(
        printed(yaml).contains("/// A label, folded over two lines.\n    tag: String?"),
        "{}",
        printed(yaml)
    );
}

#[test]
fn an_operation_the_language_cannot_call_stops_the_read_unless_skipped() {
    let spec = r#"{ "openapi": "3.0.0", "paths": {
      "/a": { "post": { "operationId": "sendXml", "requestBody": { "content": { "application/xml": {} } },
        "responses": { "200": { "description": "ok" } } } },
      "/b": { "get": { "operationId": "getB", "responses": { "200": { "content": { "application/xml": {} } } } } } } }"#;
    let e = read(spec, "spec.json", &[]).unwrap_err().to_string();
    assert!(
        e.contains("`POST /a` (sendXml): its request body is `application/xml`"),
        "{e}"
    );
    assert!(
        e.contains("`GET /b` (getB): it answers `application/xml`"),
        "{e}"
    );
    assert!(e.contains("--skip"), "{e}");
    let imported = read(spec, "spec.json", &["sendXml".into(), "GET /b".into()]).unwrap();
    assert!(imported.groups.is_empty());
    assert_eq!(
        imported
            .warnings
            .iter()
            .filter(|w| w.contains("skipped"))
            .count(),
        2
    );
}

#[test]
fn two_operations_of_one_name_are_named_both() {
    let spec = r#"{ "openapi": "3.0.0", "paths": {
      "/a": { "get": { "operationId": "list", "responses": { "204": { "description": "" } } } },
      "/b": { "get": { "operationId": "list", "responses": { "204": { "description": "" } } } } } }"#;
    let e = read(spec, "spec.json", &[]).unwrap_err().to_string();
    assert!(
        e.contains("`GET /a` and `GET /b` are both called `list`"),
        "{e}"
    );
}

#[test]
fn names_are_the_specs_made_names_only_where_they_must_be() {
    assert_eq!(op_name("listUsers"), "listUsers");
    assert_eq!(op_name("ListUsers"), "ListUsers");
    assert_eq!(op_name("list-users"), "listUsers");
    assert_eq!(op_name("getV1Users.json"), "getV1UsersJson");
    assert_eq!(op_name("2fa-verify"), "op2faVerify");
    assert_eq!(group_name("Pet Store"), "petStore");
    assert_eq!(type_name("user-profile"), "UserProfile");
    assert_eq!(param_name("user_id"), "user_id");
    assert_eq!(param_name("user-id"), "userId");
    assert_eq!(param_name("body"), "pBody");
    assert_eq!(reason(422), "unprocessableContent");
    assert_eq!(reason(499), "status499");
}

#[test]
fn a_shape_the_spec_does_not_say_is_json_never_anything() {
    let spec = r#"{ "openapi": "3.1.0", "paths": { "/x": { "get": { "operationId": "x",
      "responses": { "200": { "content": { "application/json": { "schema": {
        "oneOf": [{ "type": "string" }, { "type": "integer" }] } } } } } } } } }"#;
    assert!(
        printed(spec).contains("get x at \"x\" -> Json!"),
        "{}",
        printed(spec)
    );
}

#[test]
fn a_service_block_puts_what_it_says_on_the_endpoints_it_names() {
    let src = r#"
api Backend(base: "/api") {
    invalidations {
        users.remove(id) invalidates [users.list, users.byId(id), posts.feed]
    }
    pagination {
        users.list pages .number(param: page)
    }
}
api Backend.users {
    get list(page: Number = 1) at "users" -> [String]!
    get byId(id: String) at "users/{id}" -> String!
    delete remove(id: String) at "users/{id}" -> !
}
"#;
    let mut program = crate::syntax::parse_source(src, "t.wf").unwrap();
    expand(&mut program, &|_| None).unwrap();
    let text = print::file_text(&program);
    assert!(
        text.contains(
            "delete remove(id: String) at \"users/{id}\" -> ! invalidates [list, byId(id)]"
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "get list(page: Number = 1) at \"users\" -> [String]! pages .number(param: page)"
        ),
        "{text}"
    );
    assert!(text.contains("invalidations {\n        users.remove(id) invalidates [users.list, users.byId(id), posts.feed]\n    }"), "{text}");
    // And it reads back as written.
    let again = crate::syntax::parse_source(&text, "t.wf").unwrap();
    assert_eq!(print::file_text(&again), text);
}
