//! What reading every built-in's props against what it draws found, each
//! kept from coming back: a prop the reference said one thing of and the
//! page did another. The words themselves are `src/registry/prop_docs.rs`.

use crate::common::{built_in_css, spa_generated, ssg_html};

fn page(body: &str) -> String {
    format!(
        "app {{ Router }}\npage P(path: \"/\", title: \"T\", description: \"D\") {{\n    Heading(\"T\").h1\n{body}\n}}\npage Other(path: \"/other\", title: \"O\", description: \"d\") {{ Heading(\"O\").h1 }}\n"
    )
}

/// The rule of `selector` in the engine's sheet, as written.
fn rule(css: &str, selector: &str) -> String {
    css.split('}')
        .find(|r| {
            r.split('{')
                .next()
                .and_then(|s| s.rsplit("*/").next())
                .is_some_and(|s| s.split(',').any(|s| s.trim() == selector))
        })
        .unwrap_or_default()
        .to_string()
}

#[test]
fn a_button_with_to_goes_there_when_pressed_and_carries_no_href() {
    let src = page("    Button(\"Go\", to: \"/other\", id: \"go\")");
    let html = ssg_html(&src);
    let button = html
        .split("id=\"go\"")
        .next()
        .unwrap()
        .rsplit('<')
        .next()
        .unwrap();
    assert!(!button.contains("href"), "the static button: {button}");
    let js = spa_generated(&src);
    let line = js.lines().find(|l| l.contains("id: \"go\"")).unwrap();
    assert!(
        line.contains("WF.navigate(\"/other\")") && !line.contains("href"),
        "{line}"
    );
}

#[test]
fn a_dropdown_s_first_argument_is_its_label() {
    let html = ssg_html(&page(
        "    Dropdown(\"Actions\") { Dropdown.Item { Text(\"One\") } }",
    ));
    assert!(html.contains(">Actions<"), "{html}");
}

#[test]
fn flags_a_part_takes_are_read() {
    let html = ssg_html(&page(
        "    state step = 0\n    Tabs { Tabs.Page(\"One\") { Text(\"1\") }  Tabs.Page(\"Two\").disabled { Text(\"2\") } }\n    Stepper(bind: step) { Stepper.Step(\"A\") { Text(\"a\") }  Stepper.Step(\"B\").done { Text(\"b\") } }",
    ));
    let two = html
        .split(">Two</button>")
        .next()
        .unwrap()
        .rsplit('<')
        .next()
        .unwrap();
    assert!(two.contains("disabled"), "the second tab: {two}");
    let js = spa_generated(&page(
        "    state step = 0\n    Stepper(bind: step) { Stepper.Step(\"A\") { Text(\"a\") }  Stepper.Step(\"B\").done { Text(\"b\") } }",
    ));
    assert!(
        js.contains("(true ? \"done\""),
        "the second step is done: {js}"
    );
}

#[test]
fn a_sticky_named_bottom_sticks_to_the_bottom() {
    let html = ssg_html(&page("    Sticky(bottom: true) { Text(\"x\") }"));
    assert!(html.contains("wf-sticky--bottom"), "{html}");
}

#[test]
fn a_direction_as_a_case_turns_a_row() {
    let html = ssg_html(&page(
        "    Row(direction: .column) { Text(\"a\")  Text(\"b\") }",
    ));
    assert!(
        html.contains("wf-direction--column") || html.contains("wf-row--column"),
        "{html}"
    );
    let css = built_in_css();
    assert!(
        css.contains(".wf-direction--column"),
        "the sheet defines it"
    );
}

#[test]
fn a_hand_written_callout_title_is_the_only_one() {
    let html = ssg_html(&page(
        "    Callout.warning { Callout.Title { Text(\"Mine\") }  Text(\"Body\") }",
    ));
    assert_eq!(html.matches("wf-callout__title").count(), 1, "{html}");
    assert!(
        html.contains("Mine") && !html.contains(">Warning<"),
        "{html}"
    );
}

#[test]
fn a_stat_with_no_format_is_written_as_format_writes_it() {
    let html = ssg_html(&page("    Stat(\"Revenue\", value: €1299.50)"));
    assert!(html.contains("€1,299.50"), "{html}");
}

#[test]
fn an_image_keeps_the_loading_it_was_given() {
    let html = ssg_html(&page(
        "    Image(src: \"https://example.com/a.png\", alt: \"a\")\n    Image(src: \"https://example.com/b.png\", alt: \"b\", loading: .eager)",
    ));
    let b = html
        .split("b.png")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(b.contains("loading=\"eager\""), "{b}");
}

#[test]
fn a_number_input_is_held_to_its_ends_when_it_changes() {
    let js = spa_generated(&page(
        "    state qty: Number? = 5\n    NumberInput(bind: qty, min: 1, max: 10, label: \"Qty\")",
    ));
    let change = js
        .lines()
        .find(|l| l.contains("addEventListener(\"change\""))
        .unwrap_or_default();
    assert!(change.contains("Math.min(10, Math.max(1,"), "{change}");
}

#[test]
fn a_carousel_s_autoplay_may_follow_state() {
    let js = spa_generated(&page(
        "    state playing = true\n    Carousel(autoplay: playing, interval: 4000) { Carousel.Slide { Text(\"1\") } }",
    ));
    assert!(
        js.contains("autoplay: !!(_playing())") && js.contains("interval: 4000"),
        "{js}"
    );
}

#[test]
fn a_breadcrumb_step_goes_through_the_router() {
    let js = spa_generated(&page(
        "    Breadcrumb { Breadcrumb.Item(to: \"/other\") { Text(\"Other\") }  Breadcrumb.Item { Text(\"Here\") } }",
    ));
    assert!(js.contains("WF.navigate(\"/other\")"), "{js}");
}

#[test]
fn the_sheet_draws_what_the_flags_say() {
    let css = built_in_css();
    assert!(
        rule(&css, ".wf-btn--rounded").contains("radius-lg"),
        "rounded"
    );
    for tone in ["secondary", "success", "warning", "info"] {
        let r = css
            .split('}')
            .find(|r| r.contains(&format!(".wf-btn--outlined.wf-btn--{tone}")))
            .unwrap_or_default();
        assert!(
            r.contains(&format!("--color-{tone}")),
            "an outlined {tone} button: {r}"
        );
    }
}

#[test]
fn a_bare_closable_dialog_draws_its_close_button() {
    let js = spa_generated(&page(
        "    state open = false\n    Sheet(visible: open, label: \"Filters\").closable { Text(\"x\") }",
    ));
    assert!(js.contains("header--bare"), "{js}");
}

/// The errors of `src`, as `code: message`, through the build's pipeline.
fn errors(src: &str) -> Vec<String> {
    let (program, out) = webfluent::syntax::parse_source_recovering(src, "t.wf");
    assert!(out.is_empty(), "does not parse: {out:?}");
    let source = src.to_string();
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
    .diagnostics
    .into_iter()
    .filter(webfluent::Diagnostic::is_error)
    .map(|d| format!("{}: {}", d.code, d.message))
    .collect()
}

#[test]
fn a_grid_of_more_than_twelve_columns_is_refused() {
    let found = errors(&page("    Grid(columns: 13) { Text(\"x\") }"));
    assert!(
        found
            .iter()
            .any(|e| e.starts_with("T01") && e.contains("13")),
        "{found:?}"
    );
    assert_eq!(
        errors(&page("    Grid(columns: 12) { Text(\"x\") }")),
        Vec::<String>::new()
    );
    assert_eq!(
        errors(&page(
            "    Grid(columns: { base: 1, md: 16 }) { Text(\"x\") }"
        )),
        Vec::<String>::new()
    );
}

#[test]
fn a_value_per_breakpoint_is_only_for_a_layout() {
    let found = errors(&page(
        "    state n = 1\n    Pagination(bind: n, pages: 10, siblings: { base: 1, md: 2 })",
    ));
    assert!(
        found
            .iter()
            .any(|e| e.starts_with("T01") && e.contains("siblings")),
        "{found:?}"
    );
    assert_eq!(
        errors(&page(
            "    Row(gap: { base: .sm, lg: .lg }) { Text(\"x\") }"
        )),
        Vec::<String>::new()
    );
}
