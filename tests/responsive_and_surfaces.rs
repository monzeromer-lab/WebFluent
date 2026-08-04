//! What the stylesheet does to a component, rather than what the codegen emits.
//!
//! Everything else in this repository checks markup. None of it would have caught
//! a sidebar that renders transparent, or navigation that disappears on a phone
//! with no way back — those are properties of the CSS, and the CSS had no tests
//! at all.
//!
//! The rules here are the ones a person notices immediately and a DOM assertion
//! never sees: a panel that floats over content must be opaque; nothing may be
//! hidden without a control to bring it back; a scale must have a range.

use std::collections::HashMap;

use webfluent::codegen::generate_css_with;
use webfluent::themes::{BuiltinCss, tokens::default_tokens};

fn sheets() -> [(&'static str, String); 2] {
    [
        (
            "full",
            generate_css_with(&default_tokens(), BuiltinCss::Full),
        ),
        (
            "structural",
            generate_css_with(&default_tokens(), BuiltinCss::Structural),
        ),
    ]
}

/// The declarations of one rule, as written.
fn rule_body<'a>(css: &'a str, selector: &str) -> Option<&'a str> {
    // Match `selector {` at a boundary, so `.wf-sidebar` does not find
    // `.wf-sidebar__item`.
    let mut rest = css;
    while let Some(i) = rest.find(selector) {
        let after = &rest[i + selector.len()..];
        let boundary = after
            .chars()
            .next()
            .is_none_or(|c| c.is_whitespace() || c == '{' || c == ',');
        if boundary {
            if let Some(open) = after.find('{') {
                if after[..open].trim().is_empty() {
                    let body = &after[open + 1..];
                    if let Some(close) = body.find('}') {
                        return Some(&body[..close]);
                    }
                }
            }
        }
        rest = after;
    }
    None
}

// ─── Surfaces ───────────────────────────────────────────────────────────

/// A panel that sits over page content must be opaque.
///
/// The sidebar had no `background` in structural mode at all, so it rendered
/// transparent and the page showed straight through it. Off-canvas made that
/// worse: it now slides over the content rather than sitting beside it.
#[test]
fn every_overlaying_surface_declares_a_background() {
    let mut failures = Vec::new();
    for (mode, css) in sheets() {
        for selector in [
            ".wf-sidebar",
            ".wf-modal__content",
            ".wf-dialog__content",
            ".wf-dropdown__items",
            ".wf-menu__items",
            ".wf-navbar",
        ] {
            match rule_body(&css, selector) {
                Some(body) if body.contains("background") => {}
                Some(body) => failures.push(format!(
                    "{mode}: {selector} overlays page content but declares no background:\n    {}",
                    body.trim()
                )),
                None => failures.push(format!("{mode}: no rule for {selector} at all")),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "transparent surfaces:\n{}",
        failures.join("\n")
    );
}

/// A surface that lifts out of the flow needs to be distinguishable from what is
/// behind it — by its own fill, a border, or a shadow. A panel whose only
/// boundary is a hairline at 1.23:1 against the page reads as having no edge.
///
/// Elements that stay in the flow are excluded: whether a card has an edge is a
/// design decision, and structural mode exists to leave that to the author.
#[test]
fn a_panel_is_distinguishable_from_the_page_behind_it() {
    let mut failures = Vec::new();
    for (mode, css) in sheets() {
        for selector in [".wf-sidebar", ".wf-modal__content", ".wf-dialog__content"] {
            let Some(body) = rule_body(&css, selector) else {
                failures.push(format!("{mode}: no rule for {selector}"));
                continue;
            };
            let separated = body.contains("border")
                || body.contains("box-shadow")
                || body.contains("background");
            if !separated {
                failures.push(format!("{mode}: {selector} has no visible boundary"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "indistinct panels:\n{}",
        failures.join("\n")
    );
}

// ─── Nothing disappears without a way back ──────────────────────────────

/// Nothing that carries navigation may be hidden by a breakpoint unless the same
/// sheet provides a control to bring it back.
///
/// `.wf-sidebar { display: none }` at 768px and `.wf-navbar__links { display: none }`
/// at 480px meant a WebFluent site on a phone had no navigation whatsoever, and
/// nothing anywhere to reveal it.
#[test]
fn navigation_is_never_hidden_without_a_control() {
    let mut failures = Vec::new();
    for (mode, css) in sheets() {
        for (hidden, control) in [
            (".wf-sidebar", ".wf-sidebar__toggle"),
            (".wf-navbar__links", ".wf-navbar__toggle"),
        ] {
            // Find every `display: none` applied to the element.
            let hides = css
                .match_indices(hidden)
                .filter(|(i, _)| {
                    let after = &css[i + hidden.len()..];
                    after
                        .find('}')
                        .map(|end| after[..end].contains("display: none"))
                        .unwrap_or(false)
                })
                .count();
            if hides == 0 {
                continue;
            }
            let has_control = css.contains(control)
                && rule_body(&css, control).is_some_and(|b| b.contains("display"));
            if !has_control {
                failures.push(format!(
                    "{mode}: {hidden} is hidden but {control} is never shown — the navigation is unreachable"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "unreachable navigation:\n{}",
        failures.join("\n")
    );
}

/// The off-canvas panel must be reachable *and* dismissible: a scrim to click
/// past it, and a state the toggle can drive.
#[test]
fn the_off_canvas_panel_can_be_opened_and_closed() {
    for (mode, css) in sheets() {
        assert!(
            css.contains(".wf-sidebar[data-open=\"true\"]"),
            "{mode}: nothing reveals the sidebar once it is off-canvas"
        );
        assert!(
            css.contains(".wf-sidebar__scrim"),
            "{mode}: an open panel with no scrim cannot be dismissed by tapping past it"
        );
        assert!(
            rule_body(&css, ".wf-sidebar").is_some_and(
                |b| b.contains("transition") || css.contains("transform: translateX(-100%)")
            ),
            "{mode}: the panel has no closed position to slide from"
        );
    }
}

// ─── Fixed sizes ────────────────────────────────────────────────────────

/// The type scale must carry its own range.
///
/// It used to be fixed rems, so a page was only responsive if a media query
/// reset each size — and those rules lose to any inline `style { font-size: … }`
/// an author writes, which is exactly how a hand-styled hero stayed desktop-sized
/// on a phone.
#[test]
fn the_type_scale_is_fluid() {
    let tokens = default_tokens();
    let mut fixed = Vec::new();
    for step in [
        "font-size-lg",
        "font-size-xl",
        "font-size-2xl",
        "font-size-3xl",
    ] {
        let value = tokens.get(step).expect("token");
        if !value.contains("clamp(") {
            fixed.push(format!("--{step} is {value}, with no range to shrink into"));
        }
    }
    assert!(fixed.is_empty(), "fixed type scale:\n{}", fixed.join("\n"));
}

/// A layout must not hardcode a header height in two stylesheets. It did, as
/// `56px`, so a navbar that wrapped to two rows sat on top of the sidebar under
/// it.
#[test]
fn the_header_offset_is_a_token_not_a_constant() {
    for (mode, css) in sheets() {
        let body = rule_body(&css, ".wf-sidebar").expect("sidebar rule");
        assert!(
            !body.contains("56px"),
            "{mode}: the sidebar hardcodes a header height:\n    {}",
            body.trim()
        );
        assert!(
            body.contains("var(--wf-header-height)"),
            "{mode}: the sidebar does not key off the header height token"
        );
    }
    assert!(
        default_tokens().contains_key("wf-header-height"),
        "the token the sheets reference is not defined"
    );
}

// ─── Overflow ───────────────────────────────────────────────────────────

/// Content wider than the viewport must scroll rather than push the page sideways.
#[test]
fn wide_content_scrolls_instead_of_breaking_the_layout() {
    let mut failures = Vec::new();
    for (mode, css) in sheets() {
        for (selector, why) in [
            (".wf-navbar__links", "a long link list"),
            (".wf-sidebar", "a tall navigation panel"),
        ] {
            let Some(body) = rule_body(&css, selector) else {
                continue;
            };
            if !body.contains("overflow") && !body.contains("flex-wrap") {
                failures.push(format!(
                    "{mode}: {selector} ({why}) neither wraps nor scrolls"
                ));
            }
        }
        // An image must never be the thing that widens a page.
        let img = rule_body(&css, ".wf-image").expect("image rule");
        if !img.contains("max-width: 100%") {
            failures.push(format!("{mode}: .wf-image can exceed its container"));
        }
    }
    assert!(
        failures.is_empty(),
        "overflow risks:\n{}",
        failures.join("\n")
    );
}

/// Every breakpoint the sheet uses must be one of the documented set, so the
/// layout changes at predictable widths rather than at whatever number the last
/// person typed.
#[test]
fn breakpoints_are_consistent_across_both_sheets() {
    let mut seen: HashMap<String, Vec<&str>> = HashMap::new();
    for (mode, css) in sheets() {
        let mut rest = css.as_str();
        while let Some(i) = rest.find("@media (max-width:") {
            let after = &rest[i..];
            let end = after.find(')').unwrap_or(after.len());
            seen.entry(after[..end].to_string()).or_default().push(mode);
            rest = &after[end..];
        }
    }
    let widths: Vec<&String> = seen.keys().collect();
    for w in &widths {
        assert!(
            w.contains("1024px") || w.contains("768px") || w.contains("480px"),
            "unexpected breakpoint {w} — the documented set is 1024/768/480"
        );
    }
    assert!(!widths.is_empty(), "the sheet has no breakpoints at all");
}

/// Both sheets must respond at the same widths, or a structural build reflows
/// somewhere a full build does not.
#[test]
fn both_sheets_break_at_the_same_widths() {
    let [(_, full), (_, structural)] = sheets();
    let widths = |css: &str| {
        let mut out: Vec<String> = Vec::new();
        let mut rest = css;
        while let Some(i) = rest.find("@media (max-width:") {
            let after = &rest[i..];
            let end = after.find(')').unwrap_or(after.len());
            let w = after[..end].to_string();
            if !out.contains(&w) {
                out.push(w);
            }
            rest = &after[end..];
        }
        out.sort();
        out
    };
    assert_eq!(
        widths(&full),
        widths(&structural),
        "the two sheets reflow at different widths"
    );
}

// ─── Interaction ────────────────────────────────────────────────────────

/// Every sheet must give keyboard focus a visible indicator.
///
/// The structural sheet had no focus rule of any kind, so in the mode designed
/// for authors who supply their own CSS, a keyboard user got whatever the
/// browser happened to do underneath it — often nothing.
#[test]
fn both_sheets_indicate_keyboard_focus() {
    for (mode, css) in sheets() {
        assert!(
            css.contains(":focus-visible"),
            "{mode}: nothing in the sheet indicates focus"
        );
        assert!(
            css.contains("outline: 2px solid var(--color-primary)"),
            "{mode}: the focus indicator is not an outline a user can see"
        );
    }
}

/// No rule may remove a focus indicator without putting one back.
///
/// `.wf-input` and `.wf-select` set `outline: none` and relied on a `:focus`
/// border. That is the pattern MDN names as the thing not to do, and `:focus`
/// also fires on a mouse click, so the ring appeared when nobody asked for it.
#[test]
fn nothing_removes_the_focus_outline() {
    let mut failures = Vec::new();
    for (mode, css) in sheets() {
        for (sel, body) in css.split('}').filter_map(|chunk| chunk.split_once('{')) {
            if body.contains("outline: none") || body.contains("outline:none") {
                failures.push(format!("{mode}: {} removes the focus outline", sel.trim()));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "focus removed:\n{}",
        failures.join("\n")
    );
}

/// A pointer target must be at least 24×24, per WCAG 2.2 §2.5.8.
///
/// A carousel dot was 8px square and a checkbox 18px — reachable with a mouse,
/// a coin toss with a thumb.
#[test]
fn pointer_targets_meet_the_minimum_size() {
    let minimum = 24;
    let mut failures = Vec::new();
    for (mode, css) in sheets() {
        for selector in [
            ".wf-carousel__dot",
            ".wf-tag__remove",
            ".wf-alert__dismiss",
            ".wf-checkbox",
            ".wf-radio",
            ".wf-switch",
        ] {
            let Some(body) = rule_body(&css, selector) else {
                continue;
            };
            let sized = ["width", "height", "min-width", "min-height"]
                .iter()
                .filter_map(|prop| {
                    let at = body.find(&format!("{prop}:"))?;
                    let tail = &body[at + prop.len() + 1..];
                    let num: String = tail
                        .trim_start()
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    num.parse::<u32>().ok()
                })
                .collect::<Vec<_>>();
            if sized.iter().any(|v| *v < minimum) {
                failures.push(format!(
                    "{mode}: {selector} has a dimension below {minimum}px ({sized:?})"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "targets too small:\n{}",
        failures.join("\n")
    );
}

/// The z-index ladder must be ordered and free of collisions between layers that
/// can be on screen together.
#[test]
fn the_stacking_order_is_coherent() {
    for (mode, css) in sheets() {
        // A selector may carry its z-index inside a media query rather than in
        // its base rule, so scan every rule the selector opens.
        let read = |sel: &str| -> Option<u32> {
            let mut best = None;
            let mut rest = css.as_str();
            while let Some(i) = rest.find(sel) {
                let after = &rest[i + sel.len()..];
                if let Some(open) = after.find('{') {
                    if let Some(close) = after.find('}') {
                        if open < close {
                            let body = &after[open + 1..close];
                            if let Some(at) = body.find("z-index:") {
                                if let Ok(v) = body[at + 8..]
                                    .trim_start()
                                    .chars()
                                    .take_while(|c| c.is_ascii_digit())
                                    .collect::<String>()
                                    .parse::<u32>()
                                {
                                    best = Some(v);
                                }
                            }
                        }
                    }
                }
                rest = after;
            }
            best
        };
        let navbar = read(".wf-navbar").expect("navbar z-index");
        let scrim = read(".wf-sidebar__scrim").expect("scrim z-index");
        let sidebar = read(".wf-sidebar").expect("sidebar z-index");
        let toast = read(".wf-toast-container").expect("toast z-index");

        assert!(
            scrim > navbar,
            "{mode}: the scrim sits under the navbar it should cover"
        );
        assert!(
            sidebar > scrim,
            "{mode}: the panel sits under its own scrim"
        );
        assert!(
            toast > sidebar,
            "{mode}: a toast would be hidden behind an open panel"
        );
    }
}
