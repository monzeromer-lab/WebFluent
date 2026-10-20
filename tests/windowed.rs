//! W3: `VirtualList` draws its one keyed `for` a window at a time. What a
//! real browser does with it is `tests/browser/windowed.mjs`.

mod common;

use common::{page, spa_generated, ssg_html};

fn warnings(src: &str, code: &str) -> Vec<String> {
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
    .filter(|d| d.code == code)
    .map(|d| d.message)
    .collect()
}

#[test]
fn a_virtual_list_s_for_is_drawn_a_window_at_a_time() {
    let src = page(
        "state rows = [\"a\", \"b\"]\n    state current = \"a\"\n    VirtualList(rowHeight: 32, reveal: current, label: \"Files\") {\n        for r in rows by r { Text(r) }\n    }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("className: \"wf-virtual-list\", \"aria-label\": \"Files\""),
        "{js}"
    );
    assert!(js.contains("WF.windowed(_e"), "{js}");
    assert!(
        js.contains("{ key: (r) => r, live: true, rowHeight: 32, reveal: () => _current() }"),
        "{js}"
    );
    assert!(!js.contains("WF.each("), "{js}");
    assert!(warnings(&src, "V12").is_empty());
}

#[test]
fn a_virtual_list_that_is_not_one_keyed_for_draws_everything_and_says_so() {
    for body in [
        "for r in rows { Text(r) }",
        "Text(\"Head\")\n        for r in rows by r { Text(r) }",
    ] {
        let src = page(&format!(
            "state rows = [\"a\"]\n    VirtualList {{\n        {body}\n    }}"
        ));
        assert_eq!(warnings(&src, "V12").len(), 1, "{body}");
        let js = spa_generated(&src);
        assert!(!js.contains("WF.windowed("), "{js}");
    }
}

#[test]
fn the_paint_carries_a_screenful_of_a_long_list() {
    let src = page(
        "state rows = (0..5000).map((i) => \"row {i}\")\n    VirtualList(rowHeight: 32) {\n        for r in rows by r { Text(r) }\n    }\n    for r in [\"x\", \"y\"] { Text(r) }",
    );
    let html = ssg_html(&src);
    let rows = html.matches(">row ").count();
    assert!((1..=60).contains(&rows), "{rows} rows painted");
    assert!(
        html.contains(">x<") && html.contains(">y<"),
        "the next list is whole: {html}"
    );
}
