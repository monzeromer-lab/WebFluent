//! The two grammars, one program: every pair here is the same site written
//! in the original grammar and in WebFluent 3, and every backend — the
//! bundle, the static paint, the template renderer — must produce the same
//! output for both. The pairs double as a corpus of what the new grammar
//! spells and how.

mod common;

use common::{Backend, raw_output};

/// Assert that `v1` and `v2` compile to identical output on every backend.
fn same(v1: &str, v2: &str) {
    for backend in [Backend::Spa, Backend::Ssg, Backend::Template] {
        let a = raw_output(backend, v1);
        let b = raw_output(backend, v2);
        assert_eq!(
            a, b,
            "{backend:?} output differs between the grammars\n--- v1 ---\n{v1}\n--- v2 ---\n{v2}"
        );
    }
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
