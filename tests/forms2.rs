//! The second group of form controls: the ones the runtime draws whole are
//! handed their binding and their options; the ones drawn as others
//! (`CheckboxGroup`) are painted; a flag on a control that exists
//! (`.autosize`, `.reveal`, `.range`, `.dropzone`) changes what it is. What
//! a real browser does with them is `tests/browser/forms2.mjs`.

use crate::common::{page, spa_generated, ssg_html};

#[test]
fn the_runtime_drawn_controls_are_handed_their_binding_and_options() {
    let js = spa_generated(&page(
        "state code = \"\"\n    state tags: [String] = []\n    state stars = 2\n    state price = [1, 9]\n    state stay = { start: null, end: null }\n    InputOTP(bind: code, length: 4, label: \"Code\")\n    TagInput(bind: tags, label: \"Tags\")\n    Rating(bind: stars, label: \"R\").half\n    Slider(bind: price, min: 0, max: 10, label: \"P\").range\n    DateRangePicker(bind: stay, label: \"Stay\", presets: [.last7])",
    ));
    for call in [
        "WF.otp(",
        "length: () => 4",
        "WF.tagInput(",
        "WF.rating(",
        "half: () => true",
        "WF.rangeSlider(",
        "WF.dateRange(",
        "presets: () => [\"last7\"]",
    ] {
        assert!(js.contains(call), "{call}: {js}");
    }
}

#[test]
fn a_checkbox_group_ticks_what_its_list_holds() {
    let src = page(
        "state picked: [String] = [\"a\"]\n    CheckboxGroup(bind: picked, label: \"Pick\", all: \"All\") {\n        CheckboxGroup.Option(\"A\", value: \"a\")\n        CheckboxGroup.Option(\"B\", value: \"b\")\n    }",
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<legend class=\"wf-checkbox-group__legend\">Pick</legend>"),
        "{html}"
    );
    assert!(html.contains("checked>A</label>"), "{html}");
    let js = spa_generated(&src);
    assert!(js.contains("WF.checkboxGroup(n)"), "{js}");
}

#[test]
fn a_flag_changes_what_a_control_is() {
    let js = spa_generated(&page(
        "state note = \"\"\n    state pw = \"\"\n    state files: [File] = []\n    Textarea(bind: note, label: \"N\").autosize\n    Input(bind: pw, label: \"P\").password.reveal\n    FileUpload(bind: files, label: \"F\").dropzone.multiple",
    ));
    assert!(
        js.contains("WF.autosize(n)") && js.contains("\"data-autosize\": \"true\""),
        "{js}"
    );
    assert!(js.contains("WF.reveal(n)"), "{js}");
    assert!(js.contains("WF.dropzone("), "{js}");
}
