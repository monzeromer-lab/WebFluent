//! What the author writes as text reaches the generated JavaScript as a
//! string, whatever it holds: a quote in an animation's duration or easing
//! cannot end the string and start code.

mod common;
use common::*;

const ESCAPE: &str = r#"1s\"); window.pwned = (\""#;

/// Whether a quote the text holds ended its string: `"); window.pwned` with
/// no backslash before the quote.
fn broke_out(js: &str) -> bool {
    js.match_indices("\"); window.pwned")
        .any(|(at, _)| !js[..at].ends_with('\\'))
}

#[test]
fn an_animation_setting_stays_a_string() {
    for key in ["duration", "delay", "stagger", "easing", "exit"] {
        // A stagger spaces out the items of a list.
        let src = page(&format!(
            r#"state xs = [1, 2]
    for x in xs {{ Text("x", {key}: "{ESCAPE}").fadeIn }}"#
        ));
        let js = raw_output(Backend::Spa, &src);
        assert!(js.contains("window.pwned"), "{key}: {js}");
        assert!(!broke_out(&js), "{key}: the text ended its string: {js}");
    }
}

#[test]
fn an_animate_statement_stays_a_string() {
    let src = page(&format!(
        r#"Card(ref: panel) {{ on mouseenter {{ animate(panel, "pulse", "{ESCAPE}") }} }}"#
    ));
    let js = raw_output(Backend::Spa, &src);
    assert!(js.contains("window.pwned"), "{js}");
    assert!(!broke_out(&js), "{js}");
}
