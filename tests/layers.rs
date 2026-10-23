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
        js.contains("= WF.uid(\"wf-tip\");")
            && js.contains("role: \"tooltip\", id: _e")
            && js.contains("}, \"Save\");"),
        "the positional text is the tip, under an id of its own at run time: {js}"
    );
    assert!(
        js.contains(
            "WF.el(\"kbd\", { className: \"wf-tooltip__shortcut\" }, WF.keyLabel(\"mod+s\"))"
        ),
        "{js}"
    );
    assert!(
        js.contains("{ keys: \"mod+s\", delay: 400, placement: \"bottom\" });"),
        "a tooltip's `delay:` is how long it waits, not an animation's, and its keys are its control's: {js}"
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

#[test]
fn a_menu_state_opens_has_no_button_and_each_menu_drawn_has_its_own_list() {
    let src = page(
        "state open = true\n    Button(\"Here\", ref: here)\n    Menu(visible: open, anchor: here, label: \"Choose\") { Menu.Item { Text(\"One\") } }\n    for n in [\"a\", \"b\"] by n {\n        Menu(trigger: n) { Menu.Item { Text(n) } }\n    }",
    );
    let js = spa_generated(&src);
    assert!(
        js.contains("role: \"menu\", \"aria-label\": \"Choose\""),
        "named by its label: {js}"
    );
    assert!(
        !js.contains("}, \"Choose\");"),
        "no button says the label: {js}"
    );
    assert!(
        js.contains("WF.uid(\"wf-menu\")"),
        "an id for each menu drawn: {js}"
    );
    assert!(js.contains("wf-menu__trigger"), "{js}");
}

#[test]
fn a_box_a_choice_and_a_switch_are_painted_with_their_inputs() {
    let src = page(
        "state on = true\n    state size = \"md\"\n    Switch(bind: on, label: \"Wrap\", id: \"wrap\")\n    Checkbox(checked: false, label: \"Box\", id: \"box\") { on change { on = !on } }\n    Radio(bind: size, value: \"md\", label: \"Medium\", id: \"md\")",
    );
    let html = ssg_html(&src);
    assert!(html.contains("<label class=\"wf-switch\"><input id=\"wrap\" type=\"checkbox\" role=\"switch\" aria-checked=\"true\" checked><span class=\"wf-switch__track\">"), "{html}");
    assert!(
        html.contains(
            "<label class=\"wf-checkbox\"><input id=\"box\" type=\"checkbox\">Box</label>"
        ),
        "{html}"
    );
    assert!(
        html.contains(
            "<input id=\"md\" name=\"size\" type=\"radio\" value=\"md\" checked>Medium</label>"
        ),
        "{html}"
    );
}

/// The line that builds the element `class` names, and what is drawn after it
/// up to the next element of that kind: the attributes it is given, and the
/// handlers attached to it.
fn element_line<'a>(js: &'a str, class: &str) -> &'a str {
    js.lines()
        .find(|l| l.contains(&format!("{{ className: \"{class}")))
        .unwrap_or_else(|| panic!("no element of class {class}: {js}"))
}

fn var_of(line: &str) -> &str {
    line.trim()
        .trim_start_matches("const ")
        .split(' ')
        .next()
        .unwrap()
}

#[test]
fn a_layer_takes_the_attributes_and_handlers_any_element_takes() {
    let src = page(
        "state open = false\n    state keys = 0\n    state why = \"\"\n    Button(\"Anchor\", ref: anchor)\n    \
         Popover(visible: open, anchor: anchor, label: \"Choices\", role: \"listbox\", tabindex: \"-1\", data-popup: \"row\", aria-describedby: \"hint\", title: \"Pick one\", id: \"pop\", ref: pop) {\n        \
             on keydown(e) { keys = keys + 1 }\n        \
             on pointerdown { keys = keys + 10 }\n        \
             on close(reason) { why = reason }\n        \
             Text(\"one\")\n    }\n    \
         Menu(trigger: \"View\", role: \"group\", data-kind: \"view\", tabindex: \"-1\") {\n        \
             on keydown(e) { keys = keys + 1 }\n        \
             Menu.Item { Text(\"x\") }\n    }\n    \
         Menu(visible: open, at: { x: 0, y: 0 }, role: \"listbox\", aria-label: \"Actions\", data-kind: \"ctx\") {\n        \
             on keydown(e) { keys = keys + 1 }\n        \
             on close(reason) { why = reason }\n        \
             Menu.Item { Text(\"y\") }\n    }\n    \
         Tooltip(\"Save\", data-tip: \"save\", id: \"tip\") {\n        \
             on mouseenter { keys = keys + 1 }\n        \
             Button(\"Save\")\n    }\n    \
         Modal(visible: open, title: \"M\", role: \"alertdialog\", data-kind: \"modal\", tabindex: \"-1\") {\n        \
             on keydown(e) { keys = keys + 1 }\n        \
             Text(\"m\")\n    }\n    \
         Dialog(visible: open, title: \"D\", data-kind: \"dialog\", aria-describedby: \"body\") {\n        \
             on keydown(e) { keys = keys + 1 }\n        \
             Text(\"d\", id: \"body\")\n    }\n    \
         Text(\"{keys} {why}\")",
    );
    // `C04` says an unknown named argument is written to the element as an
    // attribute; these are all known to every element, and none is reported.
    assert_eq!(checks(&src), "");
    let js = spa_generated(&src);

    // A popover: the author's role in place of `dialog`, and the rest.
    let pop = element_line(&js, "wf-popover");
    assert!(
        pop.contains("role: \"listbox\"") && !pop.contains("role: \"dialog\""),
        "its author's role, and only that: {pop}"
    );
    for attr in [
        "\"aria-label\": \"Choices\"",
        "tabindex: \"-1\"",
        "\"data-popup\": \"row\"",
        "\"aria-describedby\": \"hint\"",
        "title: \"Pick one\"",
        "id: \"pop\"",
        "ref: pop",
    ] {
        assert!(pop.contains(attr), "{attr} on the popover: {pop}");
    }
    let v = var_of(pop);
    assert!(
        js.contains(&format!(
            "{v}.addEventListener(\"keydown\", (e) => {{ _keys.set((_keys() + 1)); }});"
        )),
        "its `on keydown` is its element's: {js}"
    );
    assert!(
        js.contains(&format!("{v}.addEventListener(\"pointerdown\"")),
        "{js}"
    );
    assert!(
        !js.contains(&format!("{v}.addEventListener(\"close\"")),
        "`on close` is the layer's, heard once: {js}"
    );
    assert!(
        js.contains("onClose: (reason) => { _why.set(reason); }"),
        "{js}"
    );

    // A menu at its button: on its root.
    let menu = element_line(&js, "wf-menu\"");
    for attr in [
        "role: \"group\"",
        "\"data-kind\": \"view\"",
        "tabindex: \"-1\"",
    ] {
        assert!(menu.contains(attr), "{attr} on the menu: {menu}");
    }
    assert!(
        js.contains(&format!("{}.addEventListener(\"keydown\"", var_of(menu))),
        "{js}"
    );
    assert!(
        !menu.contains("trigger") && !menu.contains("visible"),
        "what the menu reads is not an attribute: {menu}"
    );

    // A menu state opens: its list is the whole of it.
    let ctx = js
        .lines()
        .find(|l| l.contains("WF.el(\"ul\"") && l.contains("\"data-kind\": \"ctx\""))
        .unwrap_or_else(|| panic!("{js}"));
    assert!(
        ctx.contains("role: \"listbox\"") && !ctx.contains("role: \"menu\""),
        "{ctx}"
    );
    assert!(ctx.contains("\"aria-label\": \"Actions\""), "{ctx}");
    assert!(!ctx.contains("at:") && !ctx.contains("visible:"), "{ctx}");
    assert!(
        js.contains(&format!("{}.addEventListener(\"keydown\"", var_of(ctx))),
        "{js}"
    );

    // A tooltip: on its wrapper.
    let tip = element_line(&js, "wf-tooltip\"");
    assert!(
        tip.contains("\"data-tip\": \"save\"") && tip.contains("id: \"tip\""),
        "{tip}"
    );
    assert!(
        js.contains(&format!("{}.addEventListener(\"mouseenter\"", var_of(tip))),
        "{js}"
    );

    // A modal and a dialog: on the `<dialog>`, beside the name its title gives.
    let modal = element_line(&js, "wf-modal\"");
    for attr in [
        "\"aria-labelledby\": \"wf-dlg-",
        "role: \"alertdialog\"",
        "\"data-kind\": \"modal\"",
        "tabindex: \"-1\"",
    ] {
        assert!(modal.contains(attr), "{attr} on the modal: {modal}");
    }
    assert!(
        !modal.contains("title:"),
        "its title is its heading: {modal}"
    );
    let dialog = element_line(&js, "wf-dialog\"");
    assert!(
        dialog.contains("\"data-kind\": \"dialog\"")
            && dialog.contains("\"aria-describedby\": \"body\""),
        "{dialog}"
    );
    for el in [modal, dialog] {
        assert!(
            js.contains(&format!("{}.addEventListener(\"keydown\"", var_of(el))),
            "{js}"
        );
    }
}

#[test]
fn a_layers_attributes_are_painted_as_the_live_page_writes_them() {
    let src = page(
        "state open = false\n    Button(\"Anchor\", ref: anchor)\n    \
         Popover(visible: open, anchor: anchor, label: \"Choices\", role: \"listbox\", data-popup: \"row\", id: \"pop\") { Text(\"one\") }\n    \
         Popover(visible: open, anchor: anchor, label: \"Branches\") { Text(\"main\") }\n    \
         Menu(trigger: \"View\", role: \"group\", data-kind: \"view\", id: \"view\") { Menu.Item { Text(\"x\") } }\n    \
         Menu(visible: open, at: { x: 0, y: 0 }, role: \"listbox\", label: \"Mine\", aria-label: \"Actions\", data-kind: \"ctx\") { Menu.Item { Text(\"y\") } }\n    \
         Tooltip(\"Saves the draft\", data-tip: \"save\") { Button(\"Save\") }",
    );
    let html = ssg_html(&src);
    assert!(
        html.contains("<div popover=\"manual\" class=\"wf-popover\" aria-label=\"Choices\" role=\"listbox\" data-popup=\"row\" id=\"pop\">"),
        "the author's role, and the label as its name: {html}"
    );
    assert!(
        html.contains(
            "popover=\"manual\" role=\"dialog\" class=\"wf-popover\" aria-label=\"Branches\""
        ),
        "a dialog when nothing says otherwise: {html}"
    );
    assert!(
        !html.contains(">\n                Branches") && !html.contains("Choices\n"),
        "a label is a name, not words inside it: {html}"
    );
    assert!(
        html.contains("<div class=\"wf-menu\" role=\"group\" data-kind=\"view\" id=\"view\">"),
        "{html}"
    );
    assert!(
        html.contains("<ul class=\"wf-menu__items\" role=\"listbox\" aria-label=\"Actions\" data-kind=\"ctx\" hidden>"),
        "the author's role and name in place of the list's own: {html}"
    );
    assert!(
        html.contains("<div class=\"wf-tooltip\" data-tip=\"save\">"),
        "{html}"
    );
    assert!(
        html.contains(
            "<span class=\"wf-tooltip__text\" role=\"tooltip\" hidden>Saves the draft</span>"
        ),
        "the tip is hidden until the script shows it, not words in the page: {html}"
    );
    let wrapper = html.find("class=\"wf-tooltip\"").unwrap();
    let button = html[wrapper..].find("<button").unwrap();
    assert!(
        !html[wrapper..wrapper + button].contains("Saves the draft"),
        "nothing painted before its control: {html}"
    );
}

/// `html:` is one of the runtime's own keys — where `Unsafe.Html`'s markup
/// goes. An author's `html:` is the attribute `C04` says it is, on a layer
/// as on any element; it used to be markup on every element that wrote its
/// unknown arguments through.
#[test]
fn an_html_argument_is_an_attribute_and_never_markup() {
    let src = page(
        "state open = false\n    state markup = \"<b>x</b>\"\n    Button(\"Anchor\", ref: anchor)\n    \
         Stack(html: markup, markdown: \"# x\") { Text(\"s\") }\n    \
         Popover(visible: open, anchor: anchor, html: markup) { Text(\"p\") }\n    \
         Tooltip(\"Save\", html: markup) { Button(\"Save\") }\n    \
         Modal(visible: open, title: \"M\", html: markup) { Text(\"m\") }\n    \
         Menu(trigger: \"View\", highlight: markup) { Menu.Item(html: markup) { Text(\"x\") } }\n    \
         IconButton(icon: \"x\", label: \"Close\", html: markup)\n    \
         Element(\"x-card\", html: markup)\n    \
         Unsafe.Html(markup)",
    );
    let js = spa_generated(&src);
    for (class, key) in [
        ("wf-stack", "\"attr:html\": () => _markup()"),
        ("wf-stack", "\"attr:markdown\": \"# x\""),
        ("wf-popover", "\"attr:html\": () => _markup()"),
        ("wf-tooltip\"", "\"attr:html\": () => _markup()"),
        ("wf-modal\"", "\"attr:html\": () => _markup()"),
        ("wf-menu\"", "\"attr:highlight\": () => _markup()"),
        ("wf-menu__item\"", "\"attr:html\": () => _markup()"),
        ("wf-icon-btn", "\"attr:html\": () => _markup()"),
    ] {
        let line = js
            .lines()
            .find(|l| l.contains(&format!("className: \"{class}")))
            .unwrap_or_else(|| panic!("no {class}: {js}"));
        assert!(line.contains(key), "{key} on {class}: {line}");
    }
    assert!(
        js.contains("WF.el(\"x-card\", { \"attr:html\": () => _markup()"),
        "a custom element's too: {js}"
    );
    // The one element that puts markup in still does.
    let unsafe_html = js
        .lines()
        .filter(|l| l.contains("html: () => _markup()") && !l.contains("attr:html"))
        .count();
    assert_eq!(unsafe_html, 1, "only `Unsafe.Html` writes markup: {js}");
}

/// Every key the runtime hands a module (`attrHook`) is one the compiler
/// spells `attr:…` for an author, so no attribute can become markup.
#[test]
fn every_runtime_hook_is_a_key_an_author_cannot_reach() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime/modules");
    let mut hooks = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "js") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for piece in text.split("attrHook(\"").skip(1) {
            let name = piece.split('"').next().unwrap().to_string();
            hooks.push((
                name,
                path.file_name().unwrap().to_string_lossy().to_string(),
            ));
        }
    }
    assert!(!hooks.is_empty(), "the hooks are found");
    for (name, file) in hooks {
        assert!(
            webfluent::codegen::js::RUNTIME_KEYS.contains(&name.as_str()),
            "`attrHook(\"{name}\")` in {file} is missing from RUNTIME_KEYS"
        );
    }
}
