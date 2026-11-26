//! One program, two layouts: every site here is written with braces, as
//! a `.wf` file has them, and must compile — on every backend: the
//! bundle, the static paint, the template renderer — to exactly what its
//! indented spelling (`.wfx`, as `wf fmt --to wfx` writes it) compiles to.
//! The sites double as a corpus of how the grammar spells what it says.

use crate::common::{Backend, raw_output, raw_output_as};

/// Assert that `src` and its indented layout compile to identical output
/// on every backend.
fn same(src: &str) {
    // The indented layout, as `wf fmt --to wfx` writes it; the converter
    // itself holds it to the same tokens.
    let wfx = webfluent::syntax::layout::to_offside(&dedent(src), "t.wf")
        .unwrap_or_else(|e| panic!("to_offside failed for:\n{src}\n{e}"));
    for backend in [Backend::Spa, Backend::Ssg, Backend::Template] {
        let b = raw_output(backend, src);
        let c = raw_output_as(backend, &wfx, "t.wfx");
        assert_eq!(
            b, c,
            "{backend:?} output differs between the braced and the indented layout\n--- wf ---\n{src}\n--- wfx ---\n{wfx}"
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
        r#"page P(path: "/", title: "T") {
            Button("Save").primary.lg.pill.outlined.full
            Badge("New", tone: .success).pill
            Card.elevated { Text("x") }
            Spacer.lg
            Alert("Careful").warning
            Icon("home").lg.muted
            Row(gap: .md, align: .center, justify: .between).stacks { Text("a") }
        }"#,
    );
}

#[test]
fn a_form_with_a_bound_input_and_a_handler() {
    same(
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
        r#"store Counter {
            state n = 0
            derived double = n * 2
            action bump(step: Number) { n = n + step }
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
        r#"page P(path: "/", title: "T") {
            Card { style { padding: $xl; color: $primary; border-radius: $md; font-size: $lg } Text("x") }
        }"#,
    );
    // A name the theme declares is itself, whatever the property.
    same(
        r#"theme T { md: 4px }
        page P(path: "/", title: "T") {
            Card { style { padding: $md } Text("x") }
        }"#,
    );
}

#[test]
fn a_transition_may_take_its_timing_from_the_theme() {
    same(
        r#"theme T { d-fast: 120ms }
        page P(path: "/", title: "T") {
            Card { transition { background: $d-fast easeOut } Text("x") }
        }"#,
    );
}

#[test]
fn a_theme_and_tokens() {
    same(
        r#"theme Brand {
            color-primary: #8B5CF6
            radius-md: 10px
        }
        page P(path: "/", title: "T") {
            Button("x").primary { style { border-radius: $radius-md } }
        }"#,
    );
}
