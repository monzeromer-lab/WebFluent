//! The second group of display and layout built-ins: the ones drawn as
//! other elements are painted in a static build (a description list, a
//! timeline, an item, a meter, a sparkline, a relative time), the ones that
//! need a running page are handed to the runtime (a ring, an avatar group,
//! a scroll area, a sticky box, a panel group), and a literal shape or size
//! is a stylesheet rule. What a real browser does with them is
//! `tests/browser/display.mjs`.

use crate::common::{page, spa_generated, ssg_html};

#[test]
fn pairs_events_and_items_are_painted() {
    let html = ssg_html(&page(
        "DescriptionList.grid {\n        DescriptionList.Item(\"Region\", value: \"eu-west\")\n    }\n    Timeline {\n        Timeline.Item(\"Released\", at: \"2026-11-14\") { Text(\"6.1\") }\n    }\n    Item(title: \"Alerts\", description: \"On your phone\") { Item.Actions { Button(\"Edit\") } }",
    ));
    for want in [
        "<dl class=\"wf-description-list wf-description-list--grid\">",
        "<dt class=\"wf-description-list__term\">Region</dt>",
        "<dd class=\"wf-description-list__details\">eu-west</dd>",
        "<time class=\"wf-timeline__time\" datetime=\"2026-11-14\">Nov 14, 2026</time>",
        "<div class=\"wf-item__content\">",
        "<div class=\"wf-item__actions\">",
    ] {
        assert!(html.contains(want), "{want}: {html}");
    }
}

#[test]
fn a_meter_a_sparkline_and_a_relative_time_are_painted() {
    let html = ssg_html(&page(
        "Meter(value: 0.5, high: 0.9, label: \"Disk\")\n    Sparkline([1, 3, 2], label: \"Week\").bar\n    RelativeTime(\"2020-01-01T00:00:00Z\")",
    ));
    assert!(
        html.contains("<meter class=\"wf-meter\" value=\"0.5\" high=\"0.9\" aria-label=\"Disk\">"),
        "{html}"
    );
    assert!(
        html.contains(
            "<span class=\"wf-sparkline wf-sparkline--bar\" role=\"img\" aria-label=\"Week\"><svg"
        ),
        "{html}"
    );
    assert!(html.contains("<rect "), "{html}");
    assert!(
        html.contains("datetime=\"2020-01-01T00:00:00Z\"") && html.contains("January 1, 2020"),
        "{html}"
    );
}

#[test]
fn a_literal_shape_is_a_rule_and_a_live_one_follows_its_state() {
    let src = page(
        "state cols = 2\n    Gallery(columns: cols, ratio: 4 / 3) { Text(\"x\") }\n    AspectRatio(ratio: 16 / 9) { Text(\"y\") }\n    Button(\"More\") { on click { cols = cols + 1 } }",
    );
    let js = spa_generated(&src);
    assert!(js.contains("--wf-columns"), "{js}");
    let html = ssg_html(&src);
    assert!(
        !html.contains("aspect-ratio: 16"),
        "a literal ratio is a class: {html}"
    );
}

#[test]
fn the_ones_a_running_page_needs_are_handed_to_the_runtime() {
    let js = spa_generated(&page(
        "state sizes: [Number] = [30, 70]\n    state p = 10\n    CircularProgress(value: p, label: \"Upload\")\n    AvatarGroup(max: 2) { Avatar(initials: \"A\") }\n    ScrollArea(height: \"100px\", label: \"Log\") { Text(\"x\") }\n    Sticky { Text(\"s\") }\n    PanelGroup(bind: sizes) {\n        PanelGroup.Panel(min: 20) { Text(\"a\") }\n        PanelGroup.Panel { Text(\"b\") }\n    }\n    Button(\"Go\") { on click { p = 20 } }",
    ));
    for call in [
        "WF.ring(",
        "WF.avatarGroup(n, 2)",
        "WF.scrollArea(n)",
        "WF.sticky(n)",
        "WF.panelGroup(",
        "addEventListener(\"resize\"",
    ] {
        assert!(js.contains(call), "{call}: {js}");
    }
}
