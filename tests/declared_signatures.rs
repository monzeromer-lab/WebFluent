//! What another declaration, another file or a caller sees is declared;
//! what only the body sees is inferred. One test per rule: store members
//! worked out in the order they read each other (and `T35` where they read
//! each other back), a state whose first value says nothing (`T25`), a
//! parameter written with no type (`T32`), a project script's JSDoc as its
//! signature (`T31`, `@typedef`, classes), and an action's declared result.
//! What a script's `@returns` is held to as the page runs is
//! `tests/browser/returns.mjs`.

use webfluent::diagnostics::fixes::apply;
use webfluent::syntax::parser::ast::{Declaration, ScriptDecl};

/// The program `src`, with each `(path, js)` beside it as a project script.
fn program(src: &str, scripts: &[(&str, &str)]) -> webfluent::syntax::parser::ast::Program {
    let (mut program, out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    for (path, js) in scripts {
        program.declarations.push(Declaration::Script(ScriptDecl {
            path: (*path).to_string(),
            names: webfluent::project_js::scan::scan(js).names,
        }));
    }
    program
}

fn diagnostics(src: &str, scripts: &[(&str, &str)]) -> Vec<webfluent::diagnostics::Diagnostic> {
    let program = program(src, scripts);
    let source = src.to_string();
    let js: Vec<(String, String)> = scripts
        .iter()
        .map(|(p, t)| ((*p).to_string(), (*t).to_string()))
        .collect();
    let mut found =
        webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
            program: &program,
            file_of: &|_| "t.wf".to_string(),
            source_of: &|_| Some(source.clone()),
            dir: None,
            config: None,
            declaration_files: &[],
            scripts: &[],
            stylesheets: "",
            incomplete: false,
        })
        .diagnostics;
    // The fixes are written against the files they edit.
    for d in &mut found {
        webfluent::diagnostics::fixes::realise_with(d, &|f| {
            if f == "t.wf" {
                Some(source.clone())
            } else {
                js.iter().find(|(p, _)| p == f).map(|(_, t)| t.clone())
            }
        });
    }
    found
}

fn errors(src: &str, scripts: &[(&str, &str)]) -> Vec<String> {
    diagnostics(src, scripts)
        .into_iter()
        .filter(webfluent::diagnostics::Diagnostic::is_error)
        .map(|d| format!("[{}] {}", d.code, d.message))
        .collect()
}

fn clean(src: &str) {
    let found = errors(src, &[]);
    assert!(found.is_empty(), "{found:#?}\n---\n{src}");
}

fn finds(src: &str, scripts: &[(&str, &str)], want: &str) {
    let found = errors(src, scripts);
    assert!(
        found.iter().any(|e| e.contains(want)),
        "wanted `{want}` in {found:#?}\n---\n{src}"
    );
}

/// What the first fix of the first `code` finding makes of `file`.
fn fixed(src: &str, scripts: &[(&str, &str)], code: &str, file: &str) -> String {
    let d = diagnostics(src, scripts)
        .into_iter()
        .find(|d| d.code == code)
        .unwrap_or_else(|| panic!("no {code}"));
    let fix = d
        .fixes
        .first()
        .unwrap_or_else(|| panic!("{code} has no fix"));
    let text = if file == "t.wf" {
        src.to_string()
    } else {
        scripts
            .iter()
            .find(|(p, _)| *p == file)
            .unwrap()
            .1
            .to_string()
    };
    apply(&text, fix)
}

/// The type the checker gives the name `name`.
fn type_of(src: &str, name: &str) -> String {
    let program = program(src, &[]);
    let info = webfluent::sema::types::check(&program, &|_| "t.wf".to_string());
    info.bindings
        .iter()
        .find(|b| b.name == name)
        .map_or_else(|| panic!("no binding `{name}`"), |b| b.ty.to_string())
}

fn page(body: &str) -> String {
    format!(
        "app {{ Router }}\npage P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"P\").h1\n{body}\n}}\n"
    )
}

// ─── Store members, in the order they read each other ─────────────────

#[test]
fn a_store_reads_another_whichever_is_declared_first() {
    let a = "store A { derived n = B.count + 1  derived label = \"{n} items\" }";
    let b = "store B { state count = 0  derived twice = count * 2 }";
    let reads = page("    use A\n    use B\n    Text(A.label)\n    Text(\"{B.twice}\")");
    let first = format!("{a}\n{b}\n{reads}");
    let second = format!("{b}\n{a}\n{reads}");
    clean(&first);
    clean(&second);
    for name in ["n", "label", "twice"] {
        assert_eq!(type_of(&first, name), type_of(&second, name), "{name}");
    }
    assert_eq!(type_of(&first, "n"), "Number");
}

#[test]
fn a_derived_value_may_read_one_below_it() {
    let src = format!(
        "store C {{ derived x = y + 1  derived y = 2 }}\n{}",
        page("    use C\n    Text(\"{C.x}\")")
    );
    clean(&src);
    assert_eq!(type_of(&src, "x"), "Number");
}

#[test]
fn values_worked_out_from_each_other_are_t35() {
    finds(
        &format!(
            "store D {{ derived p = q + 1  derived q = p + 1 }}\n{}",
            page("    use D\n    Text(\"{D.p}\")")
        ),
        &[],
        "[T35] `p` and `q` are each worked out from the other",
    );
    finds(
        &format!(
            "store E {{ derived u = F.v }}\nstore F {{ derived v = E.u }}\n{}",
            page("    use E\n    Text(\"{E.u}\")")
        ),
        &[],
        "[T35] `E.u` and `F.v` are each worked out from the other",
    );
    // One that says what it is breaks the loop.
    clean(&format!(
        "store G {{ derived p: Number = q + 1  derived q = if p > 3 {{ 0 }} else {{ 1 }} }}\n{}",
        page("    use G\n    Text(\"{G.p}\")")
    ));
    // A name a lambda inside binds is the lambda's, not the value's own.
    clean(&format!(
        "enum Status {{ idle, failed(reason: String) }}\n{}",
        page(
            "    state s: Status = .idle\n    derived r = match s { .failed(r) { r } else { \"\" } }\n    Text(r)"
        )
    ));
}

// ─── A state whose first value says nothing ───────────────────────────

#[test]
fn a_state_is_inferred_only_from_a_value_that_says_everything() {
    clean(&format!(
        "type Todo {{ id: String }}\nenum Tone {{ calm, loud }}\n{}",
        page(
            "    state n = 0\n    state s = \"x\"\n    state tone: Tone = .calm\n    state todo = Todo(id: \"1\")\n    state xs = [1, 2]\n    state form = { name: \"\", age: 0 }\n    Text(\"{n} {s} {todo.id} {xs.length} {form.name}\")\n    Button(\"x\") { on click { tone = .loud } }",
        )
    ));
    for start in ["[]", "null", "{}", "JSON.parse(\"1\")"] {
        finds(
            &page(&format!("    state v = {start}\n    Text(\"{{v}}\")")),
            &[],
            "[T25] `state v`'s first value does not say what it holds",
        );
    }
    finds(
        &page("    persist seen = []\n    Text(\"{seen.length}\")"),
        &[],
        "[T25] `persist seen`",
    );
}

#[test]
fn t25_writes_the_type_the_body_assigns() {
    let src = format!(
        "type Todo {{ id: String, title: String }}\ntype User {{ name: String }}\n{}",
        page(
            "    state items = []\n    state picked = null\n    for todo in items by todo.id { Text(todo.title) }\n    Button(\"a\") { on click { items = items.concat([Todo(id: \"1\", title: \"x\")])  picked = User(name: \"Ada\") } }\n    if let u = picked { Text(u.name) }",
        )
    );
    let once = fixed(&src, &[], "T25", "t.wf");
    assert!(once.contains("state items: [Todo] = []"), "{once}");
    let twice = fixed(&once, &[], "T25", "t.wf");
    assert!(twice.contains("state picked: User? = null"), "{twice}");
    clean(&twice);
}

// ─── A parameter written with no type ─────────────────────────────────

#[test]
fn a_parameter_says_its_type() {
    let src = format!(
        "{}{}",
        "store S {\n    state titles: [String] = []\n    action add(title, done) { titles = titles.concat([title]) }\n}\n",
        page(
            "    use S\n    Button(\"x\") { on click { S.add(\"Milk\", true) } }\n    Text(\"{S.titles.length}\")"
        )
    );
    finds(&src, &[], "[T32] `title` of `add` says nothing of its type");
    let once = fixed(&src, &[], "T32", "t.wf");
    assert!(once.contains("action add(title: String, done)"), "{once}");
    let twice = fixed(&once, &[], "T32", "t.wf");
    assert!(
        twice.contains("action add(title: String, done: Bool)"),
        "{twice}"
    );
    clean(&twice);
    // Written `Any` is a type; written nothing is not.
    finds(
        "component Row2 {\n    event pick(id)\n    Button(\"x\") { on click { emit pick(\"a\") } }\n}\n",
        &[],
        "[T32] `id` of the event `pick`",
    );
    finds(
        "component Rows2(items: [String]) {\n    slot row(item)\n    for it in items { row(item: it) }\n}\n",
        &[],
        "[T32] `item` of the slot `row`",
    );
}

#[test]
fn a_lambdas_parameter_is_typed_by_what_it_is_passed_to() {
    clean(&page(
        "    state xs = [3, 1, 2]\n    derived big = xs.filter(x => x > 1)\n    derived byValue = (a: Number, b: Number) => a - b\n    Text(\"{big.length} {xs.slice().sort(byValue).length}\")",
    ));
    finds(
        &page("    derived byValue = (a, b) => a - b\n    Text(\"{byValue(1, 2)}\")"),
        &[],
        "[T32] `a` is a lambda's parameter nothing gives a type to",
    );
}

#[test]
fn an_actions_declared_result_is_what_callers_see() {
    let src = format!(
        "{}{}",
        "store S {\n    state n = 1\n    action twice() -> Number { return n * 2 }\n}\n",
        page("    use S\n    Text(\"{S.twice()}\")\n    Text(S.twice().toUpperCase())")
    );
    finds(&src, &[], "[T16] a `Number` has no method `toUpperCase`");
}

// ─── A script's JSDoc is its signature ─────────────────────────────────

const TOOLS: &str = "function shout(text, times) {\n  return text.repeat(times);\n}\n\n/**\n * @param {string|number} v\n * @returns {Array}\n */\nfunction either(v) { return [v]; }\n\n/**\n * @param {HTMLElement} node\n * @param {Object<string, number>} sizes\n * @param {*} raw\n */\nfunction poke(node, sizes, raw) {}\n\nclass Counter {\n  /** @param {number} start */\n  constructor(start) { this.n = start; }\n  /** @returns {number} */\n  next() { return ++this.n; }\n}\n";

#[test]
fn a_function_with_no_jsdoc_is_t31_and_its_fix_writes_one() {
    let src = page("    Text(shout(\"a\", 3))");
    finds(
        &src,
        &[("src/tools.js", TOOLS)],
        "[T31] `shout` has no JSDoc",
    );
    // The fix edits the script, so it is applied to a project on disk.
    let dir = std::env::temp_dir().join(format!("wf-t31-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "t" }"#).unwrap();
    std::fs::write(
        dir.join("src/tools.js"),
        "function shout(text, times) {\n  return text.repeat(times);\n}\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/App.wf"), &src).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["fix", "-d", dir.to_str().unwrap(), "--code", "T31"])
        .output()
        .unwrap();
    let js = std::fs::read_to_string(dir.join("src/tools.js")).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        js.starts_with("/**\n * @param {string} text\n * @param {number} times\n * @returns {*}\n */\nfunction shout("),
        "{js}"
    );
}

#[test]
fn a_type_the_language_cannot_say_is_t31() {
    let src = page("    Text(\"{either(1).length}\")");
    finds(
        &src,
        &[("src/tools.js", TOOLS)],
        "`v` of `either`: `string|number` is one of several types",
    );
    finds(
        &src,
        &[("src/tools.js", TOOLS)],
        "`@returns` of `either`: `Array` does not say what it holds",
    );
}

#[test]
fn jsdoc_names_the_languages_types() {
    // An element, a dictionary and any JSON are what they are: an element
    // is not a map, a map of numbers takes no text.
    let tools = [("src/tools.js", TOOLS)];
    let src = page("    Host(tag: \"div\", mount: (n) => poke(n, { a: 1 }, 2))");
    assert!(
        errors(&src, &tools).is_empty(),
        "{:#?}",
        errors(&src, &tools)
    );
    finds(
        &page("    Host(tag: \"div\", mount: (n) => poke(n, { a: \"x\" }, 2))"),
        &tools,
        "`a` is `String`, but `Number` is wanted",
    );
    // A class is a type, its members its JSDoc'd methods.
    let src =
        page("    state c = Counter(1)\n    Text(\"{c.next() + 1}\")\n    Text(\"{c.prev()}\")");
    let found = errors(&src, &tools);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("`c` is a `Counter`; it has no member `prev`"),
        "{found:#?}"
    );
}

#[test]
fn a_typedef_is_a_record_the_program_names() {
    let dir = std::env::temp_dir().join(format!("wf-typedef-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "t" }"#).unwrap();
    std::fs::write(
        dir.join("src/rows.js"),
        "/** @typedef {{ id: string, title: string, done?: boolean }} Row */\n\n/**\n * @param {string} q\n * @returns {Row[]}\n */\nfunction search(q) { return [{ id: \"1\", title: q }]; }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        page("    state rows: [Row] = []\n    Button(\"go\") { on click { rows = search(\"x\") } }\n    for r in rows by r.id { Text(r.titel) }"),
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["check", "-d", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        said.contains("error[T05]: `Row` has no field `titel`"),
        "{said}"
    );
    assert!(said.contains("Did you mean `title`?"), "{said}");
}

#[test]
fn a_documented_return_is_checked_where_it_is_called() {
    let tools = "/**\n * @param {string} q\n * @returns {string[]}\n */\nfunction tags(q) { return [q]; }\n\n/** @returns {*} */\nfunction anything() { return 1; }\n";
    let src = page(
        "    state shown: [String] = []\n    Button(\"go\") { on click { shown = tags(\"x\")  log(anything()) } }\n    Text(\"{shown.length}\")",
    );
    let program = program(&src, &[("src/tags.js", tools)]);
    let js = webfluent::codegen::JsCodegen::new().generate(&program);
    assert!(
        js.contains("WF.returned(\"tags\", [\"l\", \"s\"], tags(\"x\"))"),
        "{js}"
    );
    assert!(
        js.contains("log(anything())"),
        "any JSON is the script's to vouch for: {js}"
    );
}
