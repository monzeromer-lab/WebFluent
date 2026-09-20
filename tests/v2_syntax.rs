//! Two spellings, one program: every pair here is the same site written
//! in the grammar of WebFluent 2 and, by hand, in WebFluent 3. `wf migrate`
//! must turn the first into a program that every backend — the bundle, the
//! static paint, the template renderer — compiles to exactly what the
//! second does. The pairs double as a corpus of what the new grammar
//! spells and how, and hold the migrator to the idiomatic spelling.

mod common;

use common::{Backend, migrated, raw_output, raw_output_as};

/// Assert that `v1`, migrated, and `v2` compile to identical output on
/// every backend.
fn same(v1: &str, v2: &str) {
    let from_v1 = migrated(v1);
    // The indented layout of the hand-written spelling, as `wf fmt --to
    // wfx` writes it; the converter itself holds it to the same tokens.
    let wfx = webfluent::layout::to_offside(&dedent(v2), "t.wf")
        .unwrap_or_else(|e| panic!("to_offside failed for:\n{v2}\n{e}"));
    for backend in [Backend::Spa, Backend::Ssg, Backend::Template] {
        let a = raw_output(backend, &from_v1);
        let b = raw_output(backend, v2);
        assert_eq!(
            a, b,
            "{backend:?} output differs between the migrated and the hand-written spelling\n--- migrated ---\n{from_v1}\n--- v2 ---\n{v2}"
        );
        let c = raw_output_as(backend, &wfx, "t.wfx");
        assert_eq!(
            b, c,
            "{backend:?} output differs between the braced and the indented layout\n--- wf ---\n{v2}\n--- wfx ---\n{wfx}"
        );
    }
}

/// The sources here are indented as Rust string literals inside a
/// function; the layout converter wants the file's own indentation.
fn dedent(src: &str) -> String {
    let indent = src
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut out = String::new();
    for (i, line) in src.lines().enumerate() {
        if i == 0 {
            out.push_str(line.trim_start());
        } else if line.len() >= indent {
            out.push_str(&line[indent..]);
        } else {
            out.push_str(line.trim_start());
        }
        out.push('\n');
    }
    out
}

#[test]
fn a_page_of_typography() {
    same(
        r#"Page P (path: "/", title: "T", description: "d") {
            Container {
                Heading("Hello", h1)
                Text("Welcome", bold, muted, center)
                Text("Small print", small, secondary)
            }
        }"#,
        r#"page P(path: "/", title: "T", description: "d") {
            Container {
                Heading("Hello").h1
                Text("Welcome").bold.muted.center
                Text("Small print").sm.secondary
            }
        }"#,
    );
}

#[test]
fn flags_and_named_cases_are_the_same_props() {
    same(
        r#"Page P (path: "/", title: "T") {
            Button("Save", primary, large, pill, outlined, full)
            Badge("New", success, pill)
            Card(elevated) { Text("x") }
            Spacer(lg)
            Alert("Careful", warning)
            Icon("home", large, muted)
            Row(gap: md, align: center, justify: between) { Text("a") }
        }"#,
        r#"page P(path: "/", title: "T") {
            Button("Save").primary.lg.pill.outlined.full
            Badge("New", tone: .success).pill
            Card.elevated { Text("x") }
            Spacer.lg
            Alert("Careful").warning
            Icon("home").lg.muted
            Row(gap: .md, align: .center, justify: .between) { Text("a") }
        }"#,
    );
}

#[test]
fn a_form_with_a_bound_input_and_a_handler() {
    same(
        r#"Page P (path: "/", title: "T") {
            state name = ""
            state agreed = false
            Form {
                on:submit { save() }
                Input(email, bind: name, label: "Email", hint: "h", placeholder: "you@x", required: true)
                Checkbox(bind: agreed, label: "I agree")
                Button("Send", submit, primary, disabled: true) {
                    on:click { count = count + 1 }
                }
            }
        }"#,
        r#"page P(path: "/", title: "T") {
            state name = ""
            state agreed = false
            Form {
                on submit { save() }
                Input(bind: name, label: "Email", hint: "h", placeholder: "you@x").email.required
                Checkbox(bind: agreed, label: "I agree")
                Button("Send", type: .submit).primary.disabled {
                    on click { count = count + 1 }
                }
            }
        }"#,
    );
}

#[test]
fn style_blocks_in_css_are_the_same_rules() {
    same(
        r#"Page P (path: "/", title: "T") {
            state pct = 50
            Card {
                style {
                    padding: "6px 0"
                    border: "1px solid var(--line)"
                    width: "{pct}%"
                    hover { background: "var(--surface-hover)" }
                    @media (max-width: 768px) { padding: "0" }
                }
                Text("x")
            }
        }"#,
        r#"page P(path: "/", title: "T") {
            state pct = 50
            Card {
                style {
                    padding: 6px 0
                    border: 1px solid $line
                    width: {pct}%
                    &:hover { background: $surface-hover }
                    @media (max-width: 768px) { padding: 0 }
                }
                Text("x")
            }
        }"#,
    );
}

#[test]
fn control_flow_and_lists() {
    same(
        r#"Page P (path: "/", title: "T") {
            state items = ["a", "b"]
            state open = true
            if open { Text("open") } else if items.length > 0 { Text("some") } else { Text("none") }
            for item, i in items { Text("{i}: {item}") }
            show open { Text("shown") }
        }"#,
        r#"page P(path: "/", title: "T") {
            state items = ["a", "b"]
            state open = true
            if open { Text("open") } else if items.length > 0 { Text("some") } else { Text("none") }
            for item, i in items { Text("{i}: {item}") }
            show open { Text("shown") }
        }"#,
    );
}

#[test]
fn table_parts_and_select_options() {
    same(
        r#"Page P (path: "/", title: "T") {
            state role = "a"
            Table(caption: "Rows") {
                Thead { Trow { Tcell("Name") Tcell("Role") } }
                Tbody { Trow { Tcell("Ada") Tcell("Admin", header) } }
            }
            Select(bind: role, label: "Role") {
                Option("a", "Admin")
                Option("u", "User")
            }
        }"#,
        r#"page P(path: "/", title: "T") {
            state role = "a"
            Table(caption: "Rows") {
                Table.Head { Table.Row { Table.Cell("Name") Table.Cell("Role") } }
                Table.Body { Table.Row { Table.Cell("Ada") Table.Cell("Admin").header } }
            }
            Select(bind: role, label: "Role") {
                Select.Option("Admin", value: "a")
                Select.Option("User", value: "u")
            }
        }"#,
    );
}

#[test]
fn components_props_children_and_stores() {
    same(
        r#"Store Counter {
            state n = 0
            derived double = n * 2
            action bump(by: Number) { n = n + by }
        }
        Component Panel (title: String, tone: String = "info") {
            Card { Heading(title, h3) children }
        }
        Page P (path: "/", title: "T") {
            use Counter
            Panel(title: "Keys") { Text("count {Counter.n}") Button("More") { on:click { Counter.bump(1) } } }
            Panel(title: "Empty")
        }
        App { Container { Router { Route(path: "/", page: P) } } }"#,
        r#"store Counter {
            state n = 0
            derived double = n * 2
            action bump(by: Number) { n = n + by }
        }
        component Panel(title: String, tone: String = "info") {
            Card { Heading(title).h3 children }
        }
        page P(path: "/", title: "T") {
            use Counter
            Panel(title: "Keys") { Text("count {Counter.n}") Button("More") { on click { Counter.bump(1) } } }
            Panel(title: "Empty")
        }
        app { Container { Router { Route(path: "/", page: P) } } }"#,
    );
}

#[test]
fn a_short_token_resolves_through_the_propertys_group() {
    same(
        r#"Page P (path: "/", title: "T") {
            Card { style { padding: xl  color: primary  border-radius: md  font-size: "var(--font-size-lg)" } Text("x") }
        }"#,
        r#"page P(path: "/", title: "T") {
            Card { style { padding: $xl; color: $primary; border-radius: $md; font-size: $lg } Text("x") }
        }"#,
    );
    // A name the theme declares is itself, whatever the property.
    same(
        r##"Theme T { token md: "4px" }
        Page P (path: "/", title: "T") {
            Card { style { padding: "var(--md)" } Text("x") }
        }"##,
        r##"theme T { md: 4px }
        page P(path: "/", title: "T") {
            Card { style { padding: $md } Text("x") }
        }"##,
    );
}

#[test]
fn a_transition_may_take_its_timing_from_the_theme() {
    same(
        r#"Theme T { token d-fast: "120ms" }
        Page P (path: "/", title: "T") {
            Card { transition { background "var(--d-fast)" easeOut } Text("x") }
        }"#,
        r#"theme T { d-fast: 120ms }
        page P(path: "/", title: "T") {
            Card { transition { background: $d-fast easeOut } Text("x") }
        }"#,
    );
}

#[test]
fn a_theme_and_tokens() {
    same(
        r##"Theme Brand {
            token color-primary: "#8B5CF6"
            token radius-md: "10px"
        }
        Page P (path: "/", title: "T") {
            Button("x", primary) { style { border-radius: "var(--radius-md)" } }
        }"##,
        r##"theme Brand {
            color-primary: #8B5CF6
            radius-md: 10px
        }
        page P(path: "/", title: "T") {
            Button("x").primary { style { border-radius: $radius-md } }
        }"##,
    );
}
