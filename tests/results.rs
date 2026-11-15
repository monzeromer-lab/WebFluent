//! What a call that may fail gives, and what a program does with it: error
//! enums with the statuses they name, `HttpError` and `Problem`, results
//! `T ! E`, `try` and the ways it passes an error up, patterns — `_`,
//! literals, a case inside a case, `if let` with a case — `match` inside an
//! action, `.result` on an action, and `T26`. Each is read back as it was
//! written, and checked: what it accepts, and what it refuses with a code.

use webfluent::syntax::parse_source;
use webfluent::syntax::print::file_text;

/// A service and its error enum, beside every program here.
const SERVICE: &str = r#"type User { id: String, name: String }
type NewUser { name: String }
type FieldError { field: String, message: String }
type ValidationProblem = Problem { errors: [FieldError] }

enum CreateUserError {
    unprocessableContent(problem: ValidationProblem) on 422
    conflict(problem: Problem) on 409
    http(error: HttpError)
}

enum LookupError {
    notFound(problem: Problem) on 404
    http(error: HttpError)
}

api Backend(base: "/api") {
    post create(body: NewUser) at "users" -> User ! CreateUserError
    get byId(id: String) at "users/:id" -> User ! LookupError
    get ping() at "ping" -> !
    get list() at "users" -> [User]!
}
"#;

fn with_service(rest: &str) -> String {
    format!("{SERVICE}\n{rest}")
}

/// Every finding the type checker reports for `src`, as `CODE message`.
fn findings(src: &str) -> Vec<String> {
    let program = parse_source(src, "t.wf").unwrap_or_else(|e| panic!("{e}"));
    let info = webfluent::sema::types::check(&program, &|_| "t.wf".into());
    info.findings
        .errors
        .iter()
        .chain(info.findings.warnings.iter())
        .map(|d| format!("{} {}", d.code, d.message))
        .collect()
}

fn clean(src: &str) {
    let found: Vec<String> = findings(src)
        .into_iter()
        .filter(|f| f.starts_with('T') || f.starts_with('N') || f.starts_with('C'))
        .collect();
    assert!(found.is_empty(), "{found:#?}\n{src}");
}

fn reports(src: &str, code: &str, words: &str) {
    let found = findings(src);
    assert!(
        found
            .iter()
            .any(|f| f.starts_with(code) && f.contains(words)),
        "expected {code} saying `{words}`, got {found:#?}"
    );
}

fn store(body: &str) -> String {
    with_service(&format!("store S {{\n    state said = \"\"\n{body}\n}}\n"))
}

#[test]
fn a_service_s_failures_print_as_they_were_written() {
    let src = with_service(
        r#"store S {
    state said = ""
    action submit(name: String) {
        match await Backend.create(body: NewUser(name: name)) {
            .ok(u) { said = u.name }
            .err(.unprocessableContent(p)) { said = p.title ?? "" }
            .err(.http(.status(503, _))) { said = "down" }
            .err(_) { said = "failed" }
        }
    }
    action invite(name: String) -> User ! CreateUserError {
        let user = try await Backend.create(body: NewUser(name: name))
        return user
    }
}
"#,
    );
    let program = parse_source(&src, "t.wf").expect("parses");
    let printed = file_text(&program);
    for written in [
        "unprocessableContent(problem: ValidationProblem) on 422",
        "post create(body: NewUser) at \"users\" -> User ! CreateUserError",
        "get ping -> !",
        "get list at \"users\" -> [User]!",
        ".err(.unprocessableContent(p))",
        ".err(.http(.status(503, _)))",
        ".err(_)",
        "let user = try await Backend.create(",
        "action invite(name: String) -> User ! CreateUserError",
    ] {
        assert!(printed.contains(written), "`{written}` is lost:\n{printed}");
    }
    let again = parse_source(&printed, "t.wf").expect("the printed text parses");
    assert_eq!(file_text(&again), printed);
}

#[test]
fn an_error_enum_keeps_one_http_case_and_names_each_status_once() {
    clean(SERVICE);
    reports(
        "enum E { conflict(problem: Problem) on 409 }\n",
        "N12",
        "keeps no `http` case",
    );
    reports(
        "enum E { conflict(problem: Problem) on 409  gone(problem: Problem) on 409  http(error: HttpError) }\n",
        "N12",
        "both stand for 409",
    );
    reports(
        "enum E { conflict on 409  http(error: HttpError) }\n",
        "N12",
        "carries 0 values",
    );
    reports(
        "enum E { conflict(problem: Problem)  missing(problem: Problem) on 404  http(error: HttpError) }\n",
        "N12",
        "`E.conflict` names no status",
    );
    // An endpoint fails with an error enum or an `HttpError`, nothing else.
    reports(
        "type User { id: String }\napi B(base: \"/\") { get a() at \"a\" -> User ! String }\n",
        "N12",
        "which is no error enum",
    );
}

#[test]
fn a_call_is_its_result_and_match_takes_each_case() {
    clean(&store(
        r#"    action submit(name: String) {
        match await Backend.create(body: NewUser(name: name)) {
            .ok(u) { said = u.name }
            .err(.unprocessableContent(p)) { said = "{p.errors.length}" }
            .err(.conflict(p)) { said = p.detail ?? "" }
            .err(.http(e)) { said = e.message }
        }
    }"#,
    ));
    // The cases inside `.err` cover it only together.
    reports(
        &store(
            r#"    action submit(name: String) {
        match await Backend.create(body: NewUser(name: name)) {
            .ok(u) { said = u.name }
            .err(.unprocessableContent(p)) { said = "invalid" }
            .err(.http(e)) { said = e.message }
        }
    }"#,
        ),
        "T15",
        "`.err`",
    );
    // A literal and `_` inside a case.
    clean(&store(
        r#"    action ping() {
        match await Backend.ping() {
            .ok { said = "up" }
            .err(.status(503, _)) { said = "busy" }
            .err(e) { said = e.message }
        }
    }"#,
    ));
    reports(
        &store(
            r#"    action ping() {
        match await Backend.ping() {
            .ok { said = "up" }
            .err(.status("503", _)) { said = "busy" }
            else { said = "?" }
        }
    }"#,
        ),
        "T14",
        "never matches",
    );
    reports(
        &store(
            r#"    action ping() {
        match await Backend.ping() {
            .ok { said = "up" }
            .err(.gone) { said = "?" }
            else { said = "?" }
        }
    }"#,
        ),
        "T02",
        "has no case `.gone`",
    );
    // What a case carries, bound with its type.
    reports(
        &store(
            r#"    action submit(name: String) {
        match await Backend.create(body: NewUser(name: name)) {
            .ok(u) { said = u.nmae }
            else { said = "?" }
        }
    }"#,
        ),
        "T05",
        "has no field `nmae`",
    );
}

#[test]
fn try_passes_an_error_up_where_the_action_can_hold_it() {
    // The same enum.
    clean(&store(
        "    action invite(n: String) -> User ! CreateUserError {\n        return try await Backend.create(body: NewUser(name: n))\n    }",
    ));
    // An `HttpError` into an error enum's `http` case.
    clean(&store(
        "    action all() -> [User] ! CreateUserError {\n        return try await Backend.list()\n    }",
    ));
    // An error enum's `http` case into an `HttpError`, when it has no other
    // case the action cannot hold.
    reports(
        &store(
            "    action one(id: String) -> User! {\n        return try await Backend.byId(id: id)\n    }",
        ),
        "N03",
        "`.notFound`",
    );
    // Another enum's cases the action's enum does not have.
    reports(
        &store(
            "    action one(id: String) -> User ! CreateUserError {\n        return try await Backend.byId(id: id)\n    }",
        ),
        "N03",
        "`.notFound`, which `CreateUserError` has no case for",
    );
    // An action that declares nothing, or nothing that can fail.
    reports(
        &store(
            "    action one(id: String) {\n        let u = try await Backend.byId(id: id)\n        said = u.name\n    }",
        ),
        "N03",
        "declares nothing it returns",
    );
    reports(
        &store(
            "    action one(id: String) -> String {\n        let u = try await Backend.byId(id: id)\n        return u.name\n    }",
        ),
        "N03",
        "which cannot fail",
    );
    // `try` takes a result, after it is awaited.
    reports(
        &store("    action n() -> ! {\n        let x = try 3\n        said = \"{x}\"\n    }"),
        "N03",
        "`try` takes a result",
    );
    reports(
        &store(
            "    action n() -> ! {\n        let x = try Backend.list()\n        said = \"{x.length}\"\n    }",
        ),
        "T17",
        "until it is awaited",
    );
    // In a handler, it is reported rather than returned.
    clean(&with_service(
        "page P(path: \"/\") {\n    state n = 0\n    Button(\"Go\") { on click { let all = try await Backend.list()\n n = all.length } }\n}\n",
    ));
}

#[test]
fn an_action_that_may_fail_returns_its_value() {
    clean(&store(
        "    action first() -> User ! LookupError {\n        let u = try await Backend.byId(id: \"1\")\n        return u\n    }",
    ));
    clean(&store(
        "    action none() -> User ! LookupError {\n        return .err(.http(.offline))\n    }",
    ));
    reports(
        &store(
            "    action first() -> User ! LookupError {\n        return await Backend.byId(id: \"1\")\n    }",
        ),
        "T01",
        "is a result; this action returns its value",
    );
    reports(
        &store("    action first() -> User ! LookupError {\n        return \"ada\"\n    }"),
        "T01",
        "`String`, but `User` is wanted",
    );
}

#[test]
fn if_let_takes_a_case_and_result_is_the_last_outcome() {
    clean(&with_service(
        r#"store S {
    action invite(n: String) -> User ! CreateUserError {
        return try await Backend.create(body: NewUser(name: n))
    }
}
page P(path: "/") {
    use S
    if let .err(.unprocessableContent(p)) = S.invite.result {
        Text("{p.errors.length}")
    }
    if let .ok(u) = S.invite.result {
        Text(u.name)
    }
}
"#,
    ));
    reports(
        &with_service(
            "store S {\n    action n() {\n        log(\"x\")\n    }\n}\npage P(path: \"/\") {\n    use S\n    Text(\"{S.n.result}\")\n}\n",
        ),
        "T05",
        "declares no result",
    );
}

#[test]
fn an_action_that_returns_a_value_says_what() {
    reports(
        "store S {\n    action n() {\n        return 3\n    }\n}\n",
        "T26",
        "`n` returns a value and does not say what",
    );
    clean("store S {\n    action n() -> Number {\n        return 3\n    }\n}\n");
    clean("store S {\n    action n() {\n        return\n    }\n}\n");
}
