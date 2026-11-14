//! The choosing built-ins: a toggle, a group of them, radio cards, pages,
//! steps and a number with − and +, drawn as other built-ins (the static
//! paint and the template engine alike), and the calendar the runtime
//! draws. What a real browser does with them — the keys, right to left —
//! is `tests/browser/choosing.mjs`.

use crate::common::{page, spa_generated, ssg_html, template_html};

fn painted(body: &str) -> [String; 2] {
    let src = page(body);
    [ssg_html(&src), template_html(&src)]
}

#[test]
fn a_toggle_is_pressed_while_its_state_holds() {
    for html in painted("state bold = true\n    Toggle(\"Bold\", bind: bold)") {
        assert!(html.contains("<button class=\"wf-btn wf-toggle\" type=\"button\" aria-pressed=\"true\">Bold</button>"), "{html}");
    }
    let js = spa_generated(&page(
        "state bold = false\n    Toggle(\"Bold\", bind: bold)",
    ));
    assert!(js.contains("_bold.set(!_bold())"), "{js}");
}

#[test]
fn a_toggle_group_presses_its_chosen_one_and_roves() {
    let body = "state view = \"list\"\n    ToggleGroup(bind: view, label: \"View\") {\n        ToggleGroup.Item(\"Grid\", value: \"grid\")\n        ToggleGroup.Item(\"List\", value: \"list\")\n    }";
    for html in painted(body) {
        assert!(
            html.contains("aria-label=\"View\" role=\"group\""),
            "{html}"
        );
        assert!(
            html.contains("aria-pressed=\"false\">Grid</button>"),
            "{html}"
        );
        assert!(
            html.contains("aria-pressed=\"true\">List</button>"),
            "{html}"
        );
    }
    let js = spa_generated(&page(body));
    assert!(js.contains("WF.roving("), "{js}");
}

#[test]
fn pagination_draws_the_ends_the_neighbours_and_the_gaps() {
    for html in painted("state page = 5\n    Pagination(bind: page, pages: 12)") {
        // Each page by its name, a gap as `…`, in the order drawn.
        let mut words: Vec<String> = Vec::new();
        let mut rest = html.as_str();
        while let Some(at) = rest.find("wf-pagination__") {
            rest = &rest[at + "wf-pagination__".len()..];
            if rest.starts_with("gap") {
                words.push("…".to_string());
            } else if rest.starts_with("page")
                && let Some(n) = rest.split("aria-label=\"Page ").nth(1)
            {
                words.push(n.split('"').next().unwrap_or("").to_string());
            }
        }
        assert_eq!(words, ["1", "…", "4", "5", "6", "…", "12"], "{html}");
        assert!(
            html.contains("aria-label=\"Page 5\" aria-current=\"page\">5</button>"),
            "{html}"
        );
    }
}

#[test]
fn a_stepper_shows_the_current_steps_block() {
    for html in painted(
        "state step = 1\n    Stepper(bind: step) {\n        Stepper.Step(\"A\") { Text(\"first\") }\n        Stepper.Step(\"B\") { Text(\"second\") }\n    }",
    ) {
        assert!(html.contains("<ol class=\"wf-stepper__list\">"), "{html}");
        assert!(html.contains("<li class=\"wf-stepper__item\">"), "{html}");
        assert!(
            html.contains(
                "data-state=\"current\" aria-current=\"step\" type=\"button\">B</button>"
            ),
            "{html}"
        );
        assert!(
            html.contains("<div class=\"wf-stepper__panel\" hidden aria-label=\"A\">"),
            "the others are hidden: {html}"
        );
        assert!(
            html.contains("<div class=\"wf-stepper__panel\" aria-label=\"B\">"),
            "{html}"
        );
    }
}

#[test]
fn radio_cards_and_a_number_input() {
    for html in painted(
        "state plan = \"a\"\n    state qty = 2\n    RadioGroup(bind: plan, label: \"Plan\").cards { RadioGroup.Option(\"A\", value: \"a\", hint: \"The first\") }\n    NumberInput(bind: qty, min: 1, max: 9, label: \"Qty\")",
    ) {
        assert!(
            html.contains("<fieldset class=\"wf-radio-group\" data-look=\"cards\">"),
            "{html}"
        );
        assert!(html.contains("aria-description=\"The first\""), "{html}");
        assert!(html.contains("aria-label=\"Decrease\""), "{html}");
        assert!(html.contains("type=\"number\""), "{html}");
    }
}

#[test]
fn a_calendar_is_the_runtimes_to_draw() {
    let js = spa_generated(&page(
        "state day: Date? = null\n    Calendar(bind: day, mode: .range, min: @2026-01-01, disabled: d => d.weekday() >= 6)",
    ));
    assert!(js.contains("WF.calendar("), "{js}");
    assert!(js.contains("mode: \"range\""), "{js}");
    assert!(
        js.contains("WF.weekday("),
        "a day's methods work on the ISO date it is handed: {js}"
    );
}
