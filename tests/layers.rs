//! Layers (W5): `Popover`, `Menu` — at its button, at an anchor, at a
//! point, with submenus — and `Tooltip`, drawn in the top layer by
//! `WF.layer`. What a real browser does with them is
//! `tests/browser/layers.mjs`; this holds the build to what it hands the
//! runtime and what it paints.

mod common;

use common::{page, spa_generated, ssg_html};

fn checks(src: &str) -> String {
    webfluent::sema::types::check(&webfluent::parse_source(src, "t.wf").unwrap(), &|_| {
        "t.wf".to_string()
    })
    .findings
    .errors
    .iter()
    .map(|e| e.to_string())
    .collect()
}

#[test]
fn a_popover_is_a_layer_at_its_anchor_that_writes_its_state_back() {
    let src = page(
        "state open = false\n    state why = \"\"\n    Button(\"Branches\", ref: branches) { on click { open = !open } }\n    Popover(visible: open, anchor: branches, placement: .bottomStart, label: \"Branches\", id: \"pop\").arrow {\n        on close(reason) { why = reason }\n        Text(\"main\")\n    }",
    );
    let js = spa_generated(&src);
    assert!(js.contains("WF.layer("), "{js}");
    assert!(
        js.contains("() => _open(), (v) => _open.set(v), { anchor: () => "),
        "the state is read and written back: {js}"
    );
    assert!(js.contains("placement: \"bottomStart\""), "{js}");
    assert!(
        js.contains("onClose: (reason) => { _why.set(reason); }"),
        "{js}"
    );
    assert!(
        js.contains("className: \"wf-popover wf-popover--arrow\", role: \"dialog\", \"aria-label\": \"Branches\", id: \"pop\""),
        "its name, its id and its arrow: {js}"
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("popover=\"manual\" role=\"dialog\""),
        "painted hidden, as the browser hides a popover: {html}"
    );
}

#[test]
fn a_menu_at_its_button_is_drawn_in_the_top_layer() {
    let src = page(
        "state wrap = false\n    Menu(trigger: \"View\", placement: .bottomEnd) {\n        Menu.Item(shortcut: \"mod+b\") {\n            on click { wrap = !wrap }\n            Text(\"Toggle sidebar\")\n        }\n        Menu.Item(checked: wrap) { Text(\"Word wrap\") }\n        Menu.Item(disabled: true) { Text(\"Zen mode\") }\n        Menu.Divider()\n    }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("\"aria-haspopup\": \"menu\""),
        "the button says what it opens: {js}"
    );
    let call = js
        .lines()
        .find(|l| l.contains("WF.menu("))
        .unwrap_or_else(|| panic!("{js}"));
    assert!(call.contains("{ placement: \"bottomEnd\" }"), "{call}");
    assert!(
        js.find("WF.menu(").unwrap() < js.find("Toggle sidebar").unwrap(),
        "the list is a menu before its items are drawn into it: {js}"
    );
    // The item's shortcut is drawn at its end and said once.
    assert!(
        js.contains("WF.el(\"kbd\", { className: \"wf-menu__shortcut\", \"aria-hidden\": \"true\" }, WF.keyLabel(\"mod+b\"))"),
        "{js}"
    );
    assert!(
        js.contains(".setAttribute(\"aria-keyshortcuts\", WF.keyAria(\"mod+b\"))"),
        "{js}"
    );
    assert!(
        js.contains(
            "\"aria-checked\": () => ((_wrap()) ? \"true\" : \"false\"), role: \"menuitemcheckbox\""
        ),
        "a checked item is a checkbox item that follows its state: {js}"
    );
    assert!(
        js.contains("\"aria-disabled\": ((true) ? \"true\" : \"false\"), role: \"menuitem\""),
        "{js}"
    );
    assert!(
        !js.contains("shortcut: \"mod+b\""),
        "a shortcut is not an attribute: {js}"
    );
}

#[test]
fn a_context_menu_is_opened_by_state_at_a_point() {
    let src = page(
        "state shown = false\n    state point = { x: 0, y: 0 }\n    state said = \"\"\n    Card {\n        on contextmenu(e) {\n            e.preventDefault()\n            point = { x: e.clientX, y: e.clientY }\n            shown = true\n        }\n        Text(\"Right-click here\")\n    }\n    Menu(visible: shown, at: point, id: \"context\") {\n        on close(reason) { said = reason }\n        Menu.Item { Text(\"Rename\") }\n    }",
    );
    assert_eq!(checks(&src), "", "the point and the handler check");
    let js = spa_generated(&src);
    assert!(
        js.contains(
            "WF.el(\"ul\", { className: \"wf-menu__items\", role: \"menu\", id: \"context\" })"
        ),
        "the list is the whole menu: {js}"
    );
    assert!(
        js.contains(", null, ") && js.contains("() => _shown(), (v) => _shown.set(v), { anchor: () => _point(), onClose: (reason) => { _said.set(reason); } });"),
        "no button; the state, the point and why it closed: {js}"
    );
}

#[test]
fn a_submenu_opens_beside_the_entry_of_the_menu_it_is_in() {
    let src = page(
        "Menu(trigger: \"View\") {\n        Menu.Sub(label: \"Appearance\") {\n            Menu.Item { Text(\"Light\") }\n            Menu.Sub(label: \"More\") {\n                Menu.Item { Text(\"Sepia\") }\n            }\n        }\n    }",
    );
    let js = spa_generated(&src);
    let menu = js.find("WF.menu(").unwrap_or_else(|| panic!("{js}"));
    let subs: Vec<&str> = js.lines().filter(|l| l.contains("WF.submenu(")).collect();
    assert_eq!(subs.len(), 2, "{js}");
    let first = js.find("WF.submenu(").unwrap();
    assert!(menu < first, "after the menu it is in: {js}");
    // The outer submenu's list is the inner one's owner.
    let outer_list = subs[0]
        .trim()
        .trim_start_matches("WF.submenu(")
        .split(", ")
        .nth(1)
        .unwrap()
        .to_string();
    assert!(
        subs[1].trim().ends_with(&format!(", {outer_list});")),
        "{subs:?}"
    );
    assert!(js.contains("\"wf-menu__item wf-menu__sub\""), "{js}");
}

#[test]
fn a_tooltip_names_its_shortcut_and_its_delay_is_its_own() {
    let src = page(
        "Tooltip(\"Save\", shortcut: \"mod+s\", delay: 400, placement: .bottom) {\n        Button(\"Save\")\n    }",
    );
    assert_eq!(checks(&src), "");
    let js = spa_generated(&src);
    assert!(
        js.contains("role: \"tooltip\", id: \"wf-tip-") && js.contains("}, \"Save\");"),
        "the positional text is the tip: {js}"
    );
    assert!(
        js.contains(
            "WF.el(\"kbd\", { className: \"wf-tooltip__shortcut\" }, WF.keyLabel(\"mod+s\"))"
        ),
        "{js}"
    );
    assert!(
        js.contains("{ delay: 400, placement: \"bottom\" });"),
        "a tooltip's `delay:` is how long it waits, not an animation's: {js}"
    );
    assert!(!js.contains("data-wf-delay"), "{js}");
}

#[test]
fn what_a_layer_draws_with_is_kept_only_when_a_layer_is_used() {
    let css = |body: &str| {
        let program = webfluent::sema::lower(webfluent::parse_source(&page(body), "t.wf").unwrap());
        webfluent::themes::prune_css(
            webfluent::themes::component_css(),
            &webfluent::themes::Usage::of(&program),
        )
    };
    let plain = css("Text(\"hi\")");
    assert!(
        !plain.contains(".wf-layer"),
        "a page of text draws no layer"
    );
    assert!(!plain.contains(".wf-popover"));
    for body in [
        "state o = false\n    Popover(visible: o) { Text(\"x\") }",
        "Menu(trigger: \"M\") { Menu.Item { Text(\"x\") } }",
        "Tooltip(\"t\") { Button(\"b\") }",
    ] {
        let kept = css(body);
        assert!(kept.contains(".wf-layer {"), "{body}");
        assert!(kept.contains(".wf-layer:not([data-open])"), "{body}");
    }
    assert!(
        css("state o = false\n    Popover(visible: o) { Text(\"x\") }").contains(".wf-popover {")
    );
}

#[test]
fn a_menu_item_that_cannot_be_taken_back_says_so() {
    let src = page(
        "Menu(trigger: \"File\") {\n        Menu.Item(shortcut: \"F2\") { Text(\"Rename\") }\n        Menu.Divider\n        Menu.Item.danger { Text(\"Delete\") }\n        Menu.Item(danger: true) { Text(\"Discard\") }\n    }",
    );
    assert!(checks(&src).is_empty(), "{}", checks(&src));
    let js = spa_generated(&src);
    assert_eq!(
        js.matches("wf-menu__item--danger").count(),
        2,
        "both, as a flag and as a prop: {js}"
    );
    let html = ssg_html(&src);
    assert_eq!(html.matches("wf-menu__item--danger").count(), 2, "{html}");
}

#[test]
fn a_middle_click_and_focus_within_are_events_of_any_element() {
    let src = page(
        "state closed = 0\n    state inside = false\n    Row {\n        on auxclick(e) { if e.button == 1 { closed = closed + 1 } }\n        on focusin { inside = true }\n        on focusout { inside = false }\n        Text(\"{closed} {inside}\")\n    }",
    );
    let dir = std::env::temp_dir().join(format!("wf-events-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), "{\"name\":\"e\"}").unwrap();
    std::fs::write(dir.join("src/pages/P.wf"), &src).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["check", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    let said =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(!said.contains("V09"), "no V09: {said}");
    let _ = std::fs::remove_dir_all(&dir);
    let js = spa_generated(&src);
    for event in ["auxclick", "focusin", "focusout"] {
        assert!(js.contains(event), "{event} is listened to: {js}");
    }
}
