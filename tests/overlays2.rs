//! The second group of overlays: a `HoverCard`'s card, a `LoadingOverlay`'s
//! cover and a `Tour`'s steps are painted hidden for the runtime to show;
//! a `Lightbox` and a `ConsentBanner` are handed to it; a `meta.scripts`
//! entry marked `"load": "consent"` has no tag in the page. What a real
//! browser does with them is `tests/browser/overlays2.mjs`.

use crate::common::{page, spa_generated, ssg_html};

#[test]
fn a_hover_card_and_a_loading_overlay_are_painted_for_the_runtime() {
    let src = page(
        "state saving = false\n    HoverCard(delay: 200) {\n        Link(\"@ada\", to: \"/\")\n        HoverCard.Content { Text(\"Ada\") }\n    }\n    LoadingOverlay(visible: saving, label: \"Saving\") { Text(\"Region\") }\n    Button(\"Go\") { on click { saving = true } }",
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<div class=\"wf-hover-card__content\" hidden>"),
        "{html}"
    );
    assert!(
        html.contains("<div class=\"wf-loading-overlay\" aria-busy=\"false\">"),
        "{html}"
    );
    assert!(html.contains(">Saving</p>"), "{html}");
    let js = spa_generated(&src);
    assert!(js.contains("WF.hoverCard(n, 200)"), "{js}");
    assert!(js.contains("WF.loadingOverlay(n, (() =>"), "{js}");
}

#[test]
fn a_tour_hands_the_runtime_its_targets_and_writes_open_back() {
    let src = page(
        "state touring = false\n    Button(\"New\", ref: newBtn)\n    Button(\"Tour\") { on click { touring = true } }\n    Tour(open: touring) {\n        Tour.Step(target: newBtn, title: \"New\") { Text(\"Start here.\") }\n    }",
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<div class=\"wf-tour__step\" hidden>"),
        "{html}"
    );
    assert!(html.contains("<h2 class=\"wf-heading\">New</h2>"), "{html}");
    let js = spa_generated(&src);
    assert!(js.contains("WF.tour(n, (() =>"), "{js}");
    assert!(js.contains("addEventListener(\"close\""), "{js}");
}

#[test]
fn a_lightbox_and_a_consent_banner_are_handed_to_the_runtime() {
    let src = page(
        "Lightbox(label: \"Trip\") { Image(src: \"https://example.com/a.jpg\", alt: \"A\") }\n    ConsentBanner(\"Count visits?\", accept: \"Yes\")",
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<section class=\"wf-consent\" aria-label=\"Cookies\" hidden>"),
        "{html}"
    );
    assert!(
        html.contains("data-consent=\"decline\"") && html.contains(">Yes</button>"),
        "{html}"
    );
    let js = spa_generated(&src);
    assert!(js.contains("WF.lightbox(n, \"Trip\")"), "{js}");
    assert!(js.contains("WF.consentBanner(n)"), "{js}");
}
