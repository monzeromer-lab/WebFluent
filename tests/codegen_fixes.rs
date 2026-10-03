//! The programs the compiler accepted and then compiled wrong
//! (`spec/DIAGNOSTICS_PLAN.md`, Part B): each is held to the JavaScript the
//! browser gets, which must also be a script the browser takes.

mod common;

use common::spa_js;
use webfluent::codegen::jscheck::check_js;
use webfluent::parse_source;

fn page(body: &str) -> String {
    format!(
        "page P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{body}\n}}\n"
    )
}

/// The bundle, read back as the build reads it.
fn js(src: &str) -> String {
    let out = spa_js(src);
    if let Err(fault) = check_js(&out) {
        panic!(
            "the bundle is not a script a browser takes: {} at line {}\n{}",
            fault.message,
            fault.line,
            webfluent::codegen::jscheck::line_of(&out, fault.line)
        );
    }
    out
}

fn semantic_errors(src: &str) -> Vec<String> {
    let program = parse_source(src, "t.wf").unwrap();
    webfluent::linter::semantic::validate_semantics(&program, "t.wf")
        .into_iter()
        .map(|d| d.message)
        .collect()
}

// ─── B1, B2: a store action's `let` and `if let` ─────────────────────

#[test]
fn a_store_actions_let_is_a_local_and_its_if_let_binds() {
    let out = js(&format!(
        "store S {{\n    state total = 0\n    state names: [String] = [\"a\"]\n    state out = \"\"\n    \
         action add(x: Number) {{ let n = x  n = n * 2  total = total + n }}\n    \
         action pick() {{ if let u = names.first() {{ out = u }}  if let u = names.last() {{ out = out + u }} }}\n}}\n{}",
        page(
            "    use S\n    Text(\"{S.total} {S.out}\")\n    Button(\"Add\") { on click { S.add(2) } }\n    Button(\"Pick\") { on click { S.pick() } }"
        )
    ));
    assert!(out.contains("let n = x;"), "a local, not a store member");
    assert!(out.contains("const u = "), "the binding, declared");
}

// ─── B3: a number field and a select write back typed values ─────────

#[test]
fn a_bound_control_writes_back_through_the_runtime() {
    let out = js(&page(
        "    state qty = 3\n    state size = 1\n    Input(bind: qty, label: \"Qty\").number\n    \
         Select(bind: size, label: \"Size\") { Select.Option(\"One\", value: 1) Select.Option(\"Two\", value: 2) }\n    \
         Text(\"{qty + size}\")",
    ));
    assert!(
        out.contains("_qty.set(WF.bound(e.target))"),
        "a number, not \"3\""
    );
    assert!(
        out.contains("_size.set(WF.bound(e.target))"),
        "the option's value as written"
    );
}

// ─── B4: a button in a form submits it only when it says so ──────────

#[test]
fn a_button_is_a_button_unless_it_submits() {
    let out = js(&page(
        "    Form {\n        Button(\"Add a row\")\n        Button(\"Send\", type: .submit)\n        IconButton(icon: \"plus\", label: \"Add\")\n    }",
    ));
    assert!(
        out.contains(r#"WF.el("button", { className: "wf-btn", type: "button" }, "Add a row")"#),
        "{out}"
    );
    assert!(out.contains(r#"type: "submit""#));
    assert!(!out.contains(r#"type: "submit", type: "button""#));
    let html = common::ssg_html(&page(
        "    Form { Button(\"Add a row\")  Button(\"Send\", type: .submit) }",
    ));
    assert!(
        html.contains(r#"type="button""#) && html.contains(r#"type="submit""#),
        "{html}"
    );
}

// ─── B5, B6: a state or derived that holds a function ────────────────

#[test]
fn a_held_function_is_called_through_its_signal() {
    let out = js(&page(
        "    state factor = 2\n    state pick = (x) => x + 1\n    derived scale = (x) => x * factor\n    \
         Text(\"{scale(3)} {pick(3)}\")\n    Button(\"Swap\") { on click { pick = (x) => x * 10 } }",
    ));
    assert!(
        out.contains("_scale()(3)") && out.contains("_pick()(3)"),
        "{out}"
    );
    let runtime = webfluent::runtime::full();
    assert!(
        !runtime.contains("if (typeof v === \"function\") v = v(value);"),
        "`set(fn)` stores the function rather than calling it"
    );
}

// ─── B7: `bind:` on a loop item's field ───────────────────────────────

#[test]
fn a_bound_loop_field_writes_the_item_and_tells_the_list() {
    let keyed = js(&page(
        "    state todos = [{ id: \"1\", title: \"Milk\", done: false }]\n    \
         for t in todos by t.id {\n        Input(bind: t.title, label: \"Title\")\n        Checkbox(bind: t.done, label: \"Done\")\n    }",
    ));
    assert!(
        keyed.contains("t.title = WF.bound(e.target); _todos.set(_todos().slice())"),
        "{keyed}"
    );
    assert!(
        keyed.contains("t.done = !t.done; _todos.set(_todos().slice())"),
        "{keyed}"
    );

    // With no key every item is redrawn when the list changes, so the
    // list hears of it when the field commits, not on every keystroke.
    let unkeyed = js(&page(
        "    state rows = [{ name: \"a\" }]\n    for r in rows { Input(bind: r.name, label: \"Name\") }",
    ));
    assert!(
        unkeyed.contains("\"on:input\": (e) => { r.name = WF.bound(e.target); }"),
        "{unkeyed}"
    );
    assert!(
        unkeyed.contains("\"on:change\": () => { _rows.set(_rows().slice()); }"),
        "{unkeyed}"
    );
}

#[test]
fn every_control_binds_a_stores_member() {
    let out = js(&format!(
        "store Prefs {{ state dark = false  state volume = 3  state day = null  state plan = \"a\" }}\n{}",
        page(
            "    use Prefs\n    Switch(bind: Prefs.dark, label: \"Dark\")\n    Checkbox(bind: Prefs.dark, label: \"Dark too\")\n    \
             Slider(bind: Prefs.volume, min: 0, max: 10, label: \"Volume\")\n    DatePicker(bind: Prefs.day, label: \"Day\")\n    \
             Radio(bind: Prefs.plan, value: \"a\", label: \"A\")",
        )
    ));
    assert!(
        out.contains("Prefs.dark = !Prefs.dark"),
        "the switch has its input"
    );
    assert!(out.contains("Prefs.volume = Number(event.target.value)"));
    assert!(out.contains("Prefs.day = e.target.value || null"));
    assert!(out.contains("Prefs.plan = \"a\""));
}

// ─── B8: one name, one meaning ────────────────────────────────────────

#[test]
fn a_name_declared_twice_is_refused_where_it_is_written() {
    let twice = semantic_errors(&page(
        "    state a = 1\n    derived a = 2\n    Text(\"{a}\")",
    ));
    assert_eq!(twice.len(), 1, "{twice:?}");
    assert!(twice[0].contains("`a` is declared twice"), "{twice:?}");
    let stores = semantic_errors("store A { state x = 1 }\nstore A { state y = 2 }\n");
    assert!(stores[0].contains("duplicate store `A`"), "{stores:?}");
}

// ─── B9: the build reads back what it wrote ───────────────────────────

#[test]
fn the_build_refuses_a_script_the_browser_would() {
    let dir = std::env::temp_dir().join(format!("wf-codegen-fixes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), r#"{"name":"t"}"#).unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        format!(
            "app {{ Router }}\n{}",
            page("    state n = 0\n    Text(\"{n}\")")
        ),
    )
    .unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d", dir.to_str().unwrap()])
        .output()
        .expect("wf runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let app = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    assert_eq!(check_js(&app), Ok(()), "and what it wrote reads back");
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── D2: a change inside a state's value repaints ─────────────────────

#[test]
fn a_change_inside_a_states_value_is_an_update_of_the_state() {
    let out = js(&format!(
        "store Cart {{\n    state items = [{{ done: false }}]\n    action tick() {{ items[0].done = true  items.sort((a, b) => 0) }}\n}}\n{}",
        page(
            "    use Cart\n    state form = { name: \"\" }\n    state rows = [3, 1]\n    derived sorted = rows.sort((a, b) => a - b)\n    Text(\"{sorted}\")\n    Button(\"x\") { on click { form.name = \"Ada\"  rows.reverse()  Cart.items[0].done = false } }"
        )
    ));
    assert!(
        out.contains(r#"_form.set(WF.setIn(_form(), ["name"], "Ada"));"#),
        "{out}"
    );
    assert!(
        out.contains(r#"_rows.set(WF.mutated(_rows(), "reverse", []));"#),
        "{out}"
    );
    assert!(
        out.contains(r#"Cart.items = WF.setIn(Cart.items, [(0), "done"], false);"#),
        "{out}"
    );
    assert!(
        out.contains(r#"store.items = WF.setIn(store.items, [(0), "done"], true);"#),
        "{out}"
    );
    assert!(
        out.contains(r#"store.items = WF.mutated(store.items, "sort", ["#),
        "{out}"
    );
    // Reading a sorted state does not sort the state.
    assert!(out.contains("[..._rows()].sort("), "{out}");
}

// ─── D4: a spliced value in an address is encoded ────────────────────

#[test]
fn a_spliced_value_in_a_route_or_a_fetch_is_encoded() {
    let out = js(&page(
        "    state team = \"R&D\"\n    resource rows = fetch(\"/api/teams/{team}?q={team}\")\n    Link(\"Team\", to: \"/team/{team}\")\n    Text(\"{rows.state}\")",
    ));
    assert!(
        out.contains("`/team/${encodeURIComponent(_team())}`"),
        "{out}"
    );
    assert!(
        out.contains(
            "`/api/teams/${encodeURIComponent(_team())}?q=${encodeURIComponent(_team())}`"
        ),
        "{out}"
    );
    // The static paint links where the live page does.
    let html = common::ssg_html(&format!(
        "const NAME = \"R&D/Ops\"\n{}",
        page("    Link(\"Team\", to: \"/team/{NAME}\")")
    ));
    assert!(html.contains("href=\"/team/R%26D%2FOps\""), "{html}");
}

/// E3: a build for `wf serve` carries each declared response type's shape,
/// which the runtime holds the response to; a build for a host carries none.
#[test]
fn a_dev_build_carries_the_shapes_of_declared_responses() {
    let src = "type User { id: String, name: String, tag: String = \"\" }\nenum Tone { calm, loud }\napi Backend(base: \"/api\") {\n    get user(id: String) at \"users/:id\" -> User\n    get tones() -> [Tone]\n}\npage P(path: \"/\", title: \"T\", description: \"D\") {\n    Heading(\"T\").h1\n    resource users: [User] = fetch(\"/api/users\")\n    match users { ready(list) { Text(\"{list.length}\") } else { Text(\"…\") } }\n}\n";
    let program = common::parse_program_as(src, "<test>").expect("parses");
    let mut dev = webfluent::codegen::JsCodegen::new();
    dev.set_dev(true);
    let out = dev.generate(&program);
    assert!(
        out.contains(
            r#"shape: ["l", ["r", "User", { "id": "s", "name": "s", "tag": ["?", "s"] }]]"#
        ),
        "the resource's shape:\n{out}"
    );
    assert!(
        out.contains(r#"shape: ["r", "User""#),
        "the endpoint's shape:\n{out}"
    );
    assert!(
        out.contains(r#"shape: ["l", ["e", ["calm", "loud"]]]"#),
        "an enum's shape:\n{out}"
    );
    check_js(&out).expect("a dev bundle is a script");
    let host = webfluent::codegen::JsCodegen::new().generate(&program);
    assert!(
        !host.contains("shape: ["),
        "a build for a host carries no shapes"
    );
}

/// D03's way out: `persist … { key: k }` stores one value per instance,
/// under the owner, the name and the key.
#[test]
fn a_persist_key_names_each_instance_s_storage() {
    let out = js(
        "component Panel(_ title: String) {\n    persist open = false { key: title }\n    Button(title) { on click { open = !open } }\n}\npage P(path: \"/\", title: \"T\", description: \"D\") {\n    Heading(\"T\").h1\n    Panel(\"North\")\n    Panel(\"South\")\n}\n",
    );
    assert!(
        out.contains("WF.persist(\"Panel.open:\" + String("),
        "the key joins the storage name:\n{out}"
    );
    let plain = js(&page("    persist open = false\n    Text(\"{open}\")"));
    assert!(plain.contains("WF.persist(\"P.open\""), "{plain}");
}
