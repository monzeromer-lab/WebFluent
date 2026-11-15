//! The surfaces a program reaches that it did not declare, typed: the
//! browser (its globals, its functions, the objects it hands over), DOM
//! events, an element's handle, what `catch` binds, `env`, `query`,
//! `params`, the form handle, connections, and the built-ins' props. Each
//! family: a correct use passes, a misspelt one is found where it is
//! written, with the nearest name. What the runtime does with a thrown
//! value is `tests/browser/thrown.mjs`.

use crate::common::spa_generated;

fn found(src: &str, env: &[(&str, &str)]) -> Vec<String> {
    let (program, out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
    let mut config = webfluent::config::ProjectConfig::default_config("t");
    for (name, value) in env {
        config.env.insert(
            (*name).to_string(),
            serde_json::Value::String((*value).to_string()),
        );
    }
    webfluent::diagnostics::check::check_project(&webfluent::diagnostics::check::Project {
        program: &program,
        file_of: &|_| "t.wf".to_string(),
        source_of: &|_| Some(source.clone()),
        dir: None,
        config: Some(&config),
        declaration_files: &[],
        scripts: &[],
        stylesheets: "",
        incomplete: false,
    })
    .diagnostics
    .into_iter()
    .filter(|d| d.severity == webfluent::diagnostics::Severity::Error)
    .map(|d| {
        format!(
            "[{}] {}{}",
            d.code,
            d.message,
            d.hint.map(|h| format!(" — {h}")).unwrap_or_default()
        )
    })
    .collect()
}

fn page(body: &str) -> String {
    format!(
        "app {{ Router }}\npage P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"P\").h1\n{body}\n}}\n"
    )
}

fn clean(src: &str) {
    let errors = found(src, &[]);
    assert!(errors.is_empty(), "{errors:#?}\n---\n{src}");
}

fn finds(src: &str, want: &str) {
    let errors = found(src, &[]);
    assert!(
        errors.iter().any(|e| e.contains(want)),
        "wanted `{want}` in {errors:#?}\n---\n{src}"
    );
}

#[test]
fn the_browser_s_globals_and_functions_are_typed() {
    clean(&page(
        "    state n = 0
    state s = \"\"
    Button(\"Go\") {
        on click {
            n = Math.max(1, Math.round(2.5), 3) + window.innerWidth + Date.now()
            s = JSON.stringify({ a: 1 })
            localStorage.setItem(\"k\", s)
            s = localStorage.getItem(\"k\") ?? \"\"
            console.log(\"saved\", n)
            window.open(\"/a\", \"_blank\", \"noopener\")
            document.title = location.pathname
            setTimeout(() => log(\"later\"), 100)
            n = parseInt(\"12\") + Object.keys({ a: 1 }).length
        }
    }",
    ));
    // Misspelt, with the nearest name.
    finds(
        &page("    Button(\"Go\") { on click { Math.florr(2) } }"),
        "[T30] the language does not type `Math.florr` — Did you mean `floor`?",
    );
    // A member of a real object the table leaves out, and a global.
    finds(
        &page("    Button(\"Go\") { on click { navigator.bluetooth.requestDevice() } }"),
        "[T30] the language does not type `navigator.bluetooth`",
    );
    finds(
        &page("    Button(\"Go\") { on click { crypto.randomUUID() } }"),
        "[T30] the language does not type `crypto`",
    );
    // Arguments are held to the table.
    finds(
        &page("    Button(\"Go\") { on click { localStorage.setItem(\"k\") } }"),
        "[T10] `localStorage.setItem` takes 2 arguments, but 1 is given",
    );
    finds(
        &page("    Button(\"Go\") { on click { localStorage.setItem(\"k\", [1]) } }"),
        "[T01] `value` of `localStorage.setItem` is a `String`",
    );
    // `JSON.parse` hands back `Json`, which a program matches or decodes.
    finds(
        &page("    state n = 0\n    Button(\"Go\") { on click { n = JSON.parse(\"1\") } }"),
        "`Json`",
    );
}

#[test]
fn a_dom_event_is_the_event_the_browser_sends() {
    clean(&page(
        "    state s = \"\"
    state x = 0
    Input(bind: s, label: \"S\") {
        on keydown(e) { if e.key == \"Enter\" && !e.shiftKey { e.preventDefault() } }
        on input { s = value }
    }
    Button(\"Go\") { on click(e) { x = e.clientX + e.clientY } }
    Card { on swipe(g) { s = g.direction }  on longpress(p) { x = p.x } }
    Form { on submit { event.preventDefault() } }",
    ));
    finds(
        &page("    Input(label: \"S\") { on keydown(e) { log(e.kye) } }"),
        "[T30] the language does not type `e.kye` — Did you mean `key`?",
    );
    // A click is a mouse's: it has no key.
    finds(
        &page("    Button(\"Go\") { on click(e) { log(e.key) } }"),
        "[T30] the language does not type `e.key`",
    );
    // `target` is an element.
    finds(
        &page("    Button(\"Go\") { on click(e) { e.target.focuss() } }"),
        "[T30] the language does not type `e.target.focuss` — Did you mean `focus`?",
    );
}

#[test]
fn a_ref_is_an_element() {
    clean(&page(
        "    state s = \"\"
    Input(bind: s, ref: box, label: \"S\")
    Button(\"Focus\") { on click { box.focus()  s = box.value  box.scrollIntoView({ block: \"nearest\" }) } }",
    ));
    finds(
        &page(
            "    Input(ref: box, label: \"S\")\n    Button(\"Focus\") { on click { box.focuss() } }",
        ),
        "[T30] the language does not type `box.focuss` — Did you mean `focus`?",
    );
}

#[test]
fn what_catch_binds_is_an_error() {
    clean(&page(
        "    state said = \"\"
    Button(\"Go\") { on click { try { log(1) } catch e { said = e.message } } }",
    ));
    finds(
        &page(
            "    state said = \"\"\n    Button(\"Go\") { on click { try { log(1) } catch e { said = e.mesage } } }",
        ),
        "[T30] the language does not type `e.mesage` — Did you mean `message`?",
    );
}

#[test]
fn catch_compiles_to_an_error_whatever_was_thrown() {
    let js = spa_generated(&page(
        "    state said = \"\"\n    Button(\"Go\") { on click { try { log(1) } catch e { said = e.message } } }",
    ));
    assert!(
        js.contains("catch ($thrown) { const e = WF.thrown($thrown);"),
        "{js}"
    );
}

#[test]
fn env_is_the_names_the_build_knows() {
    let src = page("    Text(env.PUBLIC_API)\n    Text(env.PUBLIC_APII)");
    let errors = found(&src, &[("PUBLIC_API", "/api")]);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .starts_with("[T05] `env` has no field `PUBLIC_APII` — Did you mean `PUBLIC_API`?"),
        "{errors:#?}"
    );
}

#[test]
fn query_and_params_are_typed() {
    // `query` is text by key, which may be missing.
    clean(&page("    Text(query.tab ?? \"all\")"));
    finds(&page("    Text(query.tab)"), "[T01]");
    // A page's `params` are its own route's.
    clean(
        "app { Router }\npage U(path: \"/u/:id\", id: String, title: \"U\", description: \"D\") {\n    Heading(params.id).h1\n}\n",
    );
    finds(
        "app { Router }\npage U(path: \"/u/:id\", id: String, title: \"U\", description: \"D\") {\n    Heading(params.idd).h1\n}\n",
        "[T05] `params` has no field `idd` — Did you mean `id`?",
    );
    clean(&page("    Text(\"{locale} {dir}\")"));
}

#[test]
fn the_form_handle_is_typed() {
    clean(&page(
        "    state email = \"\"
    Form(bind: form) {
        Input(bind: email, name: \"email\", label: \"Email\")
        Text(form.errors.email ?? \"\")
        Button(\"Send\", type: .submit, disabled: !form.valid).primary
        Button(\"Reset\") { on click { form.reset()  form.element.focus() } }
    }",
    ));
}

#[test]
fn a_connection_is_what_it_declares() {
    let src = |use_it: &str| {
        format!(
            "app {{ Router }}\ntype Msg {{ text: String }}\npage P(path: \"/\", title: \"T\", description: \"D\") {{
    state heard = \"\"
    socket chat = ws(\"wss://example.com/chat\") {{
        send Msg
        receive Msg
        on message(m) {{ heard = m.text }}
    }}
    Heading(\"Chat\").h1
    match chat {{
        connecting {{ Text(\"…\") }}
        open {{ for m in chat.messages {{ Text(m.text) }} }}
        closed(c) {{ Text(\"Closed {{c.code}}\") }}
        error(e) {{ Text(e.message) }}
    }}
    Button(\"Send\") {{ on click {{ {use_it} }} }}
}}
"
        )
    };
    clean(&src("chat.send(Msg(text: \"hi\"))"));
    finds(
        &src("chat.sned(Msg(text: \"hi\"))"),
        "[T05] `chat` has no member `sned` — Did you mean `send`?",
    );
    finds(&src("chat.send(3)"), "[T01]");
    // One that says nothing sends and receives `Json`.
    finds(
        "app { Router }\npage P(path: \"/\", title: \"T\", description: \"D\") {\n    state n = 0\n    stream ticks = sse(\"/events\") { on message(m) { n = m } }\n    Heading(\"T\").h1\n}\n",
        "`Json`",
    );
}

#[test]
fn the_built_ins_props_are_typed() {
    clean(
        &page(
            "    state picked: [String] = []
    state tab = \"a\"
    state rows: [Row] = []
    DataTable(rows: rows, key: r => r.id, bind: picked, label: \"Rows\") {
        DataTable.Column(\"Name\", value: r => r.name)
        DataTable.Column(\"Count\") { cell(r) { Text(r.count) } }
    }
    Tabs(bind: tab, label: \"T\") { Tabs.Page(\"A\", value: \"a\") { Text(\"a\") } }
    AspectRatio(ratio: 16 / 9) { Text(\"x\") }
    Sparkline([1, 2, 3], label: \"S\")
    Card(mount: (n) => n.focus(), cleanup: (h) => log(\"gone\")) { Text(\"c\") }",
        )
        .replace(
            "app { Router }",
            "app { Router }\ntype Row { id: String, name: String, count: Number }",
        ),
    );
    let with_rows = |body: &str| {
        page(body).replace(
            "app { Router }",
            "app { Router }\ntype Row { id: String, name: String, count: Number }",
        )
    };
    // A row's field, misspelt in a column and in a cell.
    finds(
        &with_rows(
            "    state rows: [Row] = []\n    DataTable(rows: rows, key: r => r.id, label: \"R\") { DataTable.Column(\"N\", value: r => r.nmae) }",
        ),
        "[T05] `Row` has no field `nmae`",
    );
    finds(
        &with_rows(
            "    state rows: [Row] = []\n    DataTable(rows: rows, key: r => r.id, label: \"R\") { DataTable.Column(\"N\") { cell(r) { Text(r.cuont) } } }",
        ),
        "[T05] `Row` has no field `cuont`",
    );
    finds(
        &page("    AspectRatio(ratio: \"wide\") { Text(\"x\") }"),
        "[T01]",
    );
    finds(
        &page("    state xs = [1, 2]\n    Text(xs)"),
        "[T01] `content:` on `Text` takes `String`, `Number` or `Bool`, but `xs` is `[Number]`",
    );
    // A moment may be written as an ISO string.
    clean(&page("    RelativeTime(\"2020-01-01T00:00:00Z\")"));
}

#[test]
fn no_built_in_prop_is_untyped() {
    for c in webfluent::registry::COMPONENTS {
        for p in c.positional.iter().chain(c.props.iter()) {
            if let webfluent::registry::PropType::Ty(spelling) = p.ty {
                for part in spelling.split(" or ") {
                    let part = part
                        .replace("Item", "String")
                        .replace("Handle", "String")
                        .replace("Picture", "String");
                    assert!(
                        webfluent::syntax::parser::v2::parse_type_text(&part).is_some(),
                        "`{}({}:)` is `{spelling}`, and `{part}` is not a type",
                        c.name,
                        p.name
                    );
                }
            }
        }
    }
}
