//! Every built-in takes what any element takes. `C04` says a named argument
//! a built-in has no prop for is written to its element as an attribute, and
//! an `on` handler is the element's; a built-in its own emitter draws — a
//! long list, a splitter, tabs, a sidebar, a chart, a control in a box — used
//! to drop `role:`, `tabindex:`, `aria-*`, `data-*` and every handler. Each
//! case here gives one of them all four and a handler, and holds the live
//! page, the pre-render and the template engine to writing them on the same
//! element: its root, or the input of a control drawn in a box.

mod common;

use common::{Backend, Elem, elems, page, spa_generated, ssg_html, template_html};

/// What every case is given, and what each backend must write.
const GIVEN: &str = "role: \"r\", tabindex: \"-1\", data-k: \"v\", aria-describedby: \"d\"";
const WANT: &[(&str, &str)] = &[
    ("role", "r"),
    ("tabindex", "-1"),
    ("data-k", "v"),
    ("aria-describedby", "d"),
];
const HANDLER: &str = "on keydown(e) { keys = keys + 1 }";

/// Where a built-in's attributes go.
#[derive(Clone, Copy, Debug)]
enum At {
    /// Its root, the element of this class.
    Root(&'static str),
    /// The input inside the element of this class: the control itself,
    /// where its handlers are.
    Input(&'static str),
}

fn find(els: &[Elem], at: At) -> Option<&Elem> {
    match at {
        At::Root(class) => els.iter().find(|e| e.has_class(class)),
        At::Input(class) => {
            let i = els.iter().position(|e| e.has_class(class))?;
            els[i..].iter().find(|e| e.tag == "input")
        }
    }
}

fn source(call: &str) -> String {
    let body = call.replace("{attrs}", GIVEN).replace("{on}", HANDLER);
    page(&format!(
        "    state keys = 0\n    state open = false\n    state w = 100\n    state n = 5\n    {body}\n    Text(\"{{keys}}\")"
    ))
}

fn findings(src: &str) -> Vec<String> {
    let program = webfluent::parse_source(src, "t.wf").unwrap();
    let checked = webfluent::sema::types::check(&program, &|_| "t.wf".to_string());
    checked
        .findings
        .errors
        .iter()
        .chain(checked.findings.warnings.iter())
        .map(|e| e.to_string())
        .filter(|e| e.contains("C04") || e.contains("V09"))
        .collect()
}

/// `call` written with `{attrs}` where the attributes go and `{on}` in its
/// block: every backend in `backends` writes them on the element `at` names,
/// and on the live page the handler is attached there.
fn takes(call: &str, at: At, backends: &[Backend]) {
    let src = source(call);
    assert_eq!(findings(&src), Vec::<String>::new(), "{call}");
    for &backend in backends {
        let els = elems(backend, &src);
        let el = find(&els, at).unwrap_or_else(|| {
            panic!(
                "no {at:?} in {}:\n{}",
                backend.name(),
                common::render(backend, &src)
            )
        });
        for (key, value) in WANT {
            assert_eq!(
                el.attr(key),
                Some(*value),
                "`{key}` on {at:?} of `{call}` in {}: {}",
                backend.name(),
                el.raw
            );
        }
    }
    // The handler is the element's: an `on:` key of its attributes, or a
    // listener on it.
    let js = spa_generated(&src);
    let els = elems(Backend::Spa, &src);
    let el = find(&els, at).unwrap();
    let line = js
        .lines()
        .find(|l| l.contains(el.raw.as_str()))
        .unwrap_or_else(|| panic!("{js}"));
    let var = line
        .trim()
        .trim_start_matches("const ")
        .split(' ')
        .next()
        .unwrap();
    let body = "(e) => { _keys.set((_keys() + 1)); }";
    assert!(
        line.contains(&format!("\"on:keydown\": {body}"))
            || js.contains(&format!("{var}.addEventListener(\"keydown\", {body});")),
        "`on keydown` reaches {at:?} of `{call}`: {js}"
    );
}

const ALL: &[Backend] = &[Backend::Spa, Backend::Ssg, Backend::Template];

#[test]
fn a_virtual_list_takes_them_on_its_box() {
    takes(
        "VirtualList(rowHeight: 20, {attrs}) {\n        {on}\n        for r in [1, 2] by r { Text(\"{r}\") }\n    }",
        At::Root("wf-virtual-list"),
        ALL,
    );
}

#[test]
fn a_splitter_takes_them_on_its_handle() {
    takes(
        "Splitter(bind: w, min: 10, {attrs}) { {on} }",
        At::Root("wf-splitter"),
        ALL,
    );
}

#[test]
fn tabs_take_them_on_their_box() {
    takes(
        "Tabs({attrs}) {\n        {on}\n        TabPage(\"A\") { Text(\"a\") }\n    }",
        At::Root("wf-tabs"),
        ALL,
    );
}

#[test]
fn a_sidebar_takes_them_on_its_panel() {
    takes(
        "Sidebar({attrs}) {\n        {on}\n        Sidebar.Item(to: \"/\") { Text(\"Home\") }\n    }",
        At::Root("wf-sidebar"),
        ALL,
    );
}

#[test]
fn a_breadcrumb_takes_them_on_its_trail() {
    takes(
        "Breadcrumb({attrs}) {\n        {on}\n        Breadcrumb.Item { Text(\"Here\") }\n    }",
        At::Root("wf-breadcrumb"),
        ALL,
    );
}

#[test]
fn an_avatar_takes_them() {
    takes(
        "Avatar(initials: \"MO\", {attrs}) { {on} }",
        At::Root("wf-avatar"),
        ALL,
    );
}

#[test]
fn a_skeleton_takes_them() {
    takes(
        "Skeleton(height: \"1rem\", {attrs}) { {on} }",
        At::Root("wf-skeleton"),
        ALL,
    );
}

#[test]
fn a_carousel_takes_them_on_its_region() {
    takes(
        "Carousel(label: \"Shots\", {attrs}) {\n        {on}\n        Carousel.Slide { Text(\"1\") }\n    }",
        At::Root("wf-carousel"),
        ALL,
    );
}

#[test]
fn a_spacer_takes_them() {
    takes("Spacer({attrs}) { {on} }", At::Root("wf-spacer"), ALL);
}

#[test]
fn an_icon_button_takes_them() {
    takes(
        "IconButton(icon: \"close\", label: \"Close\", {attrs}) { {on} }",
        At::Root("wf-icon-btn"),
        ALL,
    );
}

#[test]
fn a_chart_drawn_at_build_takes_them_on_its_figure() {
    takes(
        "Chart(kind: .bar, data: [1, 2, 3], {attrs}) { {on} }",
        At::Root("wf-chart"),
        ALL,
    );
}

#[test]
fn a_qr_code_takes_them_on_its_figure() {
    takes(
        "QrCode(\"https://example.com\", {attrs}) { {on} }",
        At::Root("wf-qr"),
        ALL,
    );
}

#[test]
fn a_video_with_captions_takes_them_and_hears_its_handlers() {
    takes(
        "Video(src: \"/a.mp4\", captions: \"/a.vtt\", controls: true, {attrs}) { {on} }",
        At::Root("wf-video"),
        &[Backend::Spa, Backend::Ssg],
    );
}

#[test]
fn a_slider_takes_them_on_its_input() {
    takes(
        "Slider(bind: n, label: \"Size\", {attrs}) { {on} }",
        At::Input("wf-slider"),
        ALL,
    );
}

#[test]
fn a_date_picker_takes_them_on_its_input() {
    takes(
        "DatePicker(label: \"When\", {attrs}) { {on} }",
        At::Input("wf-datepicker"),
        ALL,
    );
}

#[test]
fn a_file_upload_takes_them_on_its_input() {
    takes(
        "FileUpload(label: \"Files\", {attrs}) { {on} }",
        At::Input("wf-file-upload"),
        ALL,
    );
}

#[test]
fn a_switch_takes_them_on_its_input() {
    takes(
        "Switch(bind: open, label: \"Wrap\", {attrs}) { {on} }",
        At::Input("wf-switch"),
        &[Backend::Spa, Backend::Ssg],
    );
}

#[test]
fn a_checkbox_takes_them_on_its_input() {
    takes(
        "Checkbox(bind: open, label: \"Wrap\", {attrs}) { {on} }",
        At::Input("wf-checkbox"),
        &[Backend::Spa, Backend::Ssg],
    );
}

#[test]
fn a_radio_takes_them_on_its_input() {
    takes(
        "Radio(bind: w, value: 100, label: \"Wide\", {attrs}) { {on} }",
        At::Input("wf-radio"),
        &[Backend::Spa, Backend::Ssg],
    );
}

#[test]
fn a_modal_and_a_dialog_take_them_on_the_dialog() {
    takes(
        "Modal(visible: open, title: \"M\", {attrs}) {\n        {on}\n        Text(\"m\")\n    }",
        At::Root("wf-modal"),
        ALL,
    );
    takes(
        "Dialog(visible: open, title: \"D\", {attrs}) {\n        {on}\n        Text(\"d\")\n    }",
        At::Root("wf-dialog"),
        ALL,
    );
}

#[test]
fn a_popover_a_menu_and_a_tooltip_take_them_on_their_roots() {
    takes(
        "Button(\"Anchor\", ref: anchor)\n    Popover(visible: open, anchor: anchor, {attrs}) {\n        {on}\n        Text(\"p\")\n    }",
        At::Root("wf-popover"),
        ALL,
    );
    takes(
        "Menu(trigger: \"View\", {attrs}) {\n        {on}\n        Menu.Item { Text(\"x\") }\n    }",
        At::Root("wf-menu"),
        ALL,
    );
    takes(
        "Tooltip(\"Save\", {attrs}) {\n        {on}\n        Button(\"Save\")\n    }",
        At::Root("wf-tooltip"),
        ALL,
    );
}

#[test]
fn a_submenu_entry_takes_them_in_place_of_its_own() {
    let src = source(
        "Menu(trigger: \"View\") {\n        Menu.Sub(label: \"More\", {attrs}) {\n            {on}\n            Menu.Item { Text(\"x\") }\n        }\n    }",
    );
    let js = spa_generated(&src);
    let line = js
        .lines()
        .find(|l| l.contains("wf-menu__sub"))
        .unwrap_or_else(|| panic!("{js}"));
    for attr in [
        "role: \"r\"",
        "tabindex: \"-1\"",
        "\"data-k\": \"v\"",
        "\"aria-describedby\": \"d\"",
    ] {
        assert!(line.contains(attr), "{attr}: {line}");
    }
    assert!(
        !line.contains("role: \"menuitem\"") && !line.contains("tabindex: \"-1\", tabindex"),
        "the author's role and tabindex take the place of its own: {line}"
    );
    let var = line
        .trim()
        .trim_start_matches("const ")
        .split(' ')
        .next()
        .unwrap();
    assert!(
        js.contains(&format!("{var}.addEventListener(\"keydown\"")),
        "{js}"
    );
}

/// Given nothing, each writes what it wrote before: the same line, with no
/// listener — the generated code of a program that gives none is the same.
#[test]
fn given_nothing_each_writes_what_it_wrote_before() {
    for (call, line) in [
        (
            "VirtualList(rowHeight: 20, label: \"Rows\") { for r in [1] by r { Text(\"{r}\") } }",
            "WF.el(\"div\", { className: \"wf-virtual-list\", \"aria-label\": \"Rows\" });",
        ),
        (
            "Splitter(bind: w)",
            "WF.el(\"div\", { className: \"wf-splitter\" });",
        ),
        (
            "Tabs { TabPage(\"A\") { Text(\"a\") } }",
            "WF.el(\"div\", { className: \"wf-tabs\" });",
        ),
        (
            "Breadcrumb { Breadcrumb.Item { Text(\"Here\") } }",
            "WF.el(\"nav\", { className: \"wf-breadcrumb\", \"aria-label\": \"breadcrumb\" });",
        ),
        (
            "Switch(bind: open, label: \"S\")",
            "WF.el(\"input\", { type: \"checkbox\", role: \"switch\", checked: () => !!_open(), \"aria-checked\": () => _open() ? \"true\" : \"false\", \"on:change\": () => { _open.set(!_open()); } });",
        ),
        (
            "Slider(bind: n)",
            "WF.el(\"input\", { type: \"range\", min: 0, max: 100, step: 1, value: () => _n(), \"on:input\": (event) => { _n.set(Number(event.target.value));  } });",
        ),
        (
            "Menu(trigger: \"M\") { Menu.Sub(label: \"More\") { Menu.Item { Text(\"x\") } } }",
            "WF.el(\"li\", { className: \"wf-menu__item wf-menu__sub\", role: \"menuitem\", tabindex: \"-1\" }, WF.el(\"span\", {}, \"More\")",
        ),
    ] {
        let js = spa_generated(&source(call));
        assert!(js.contains(line), "`{call}` wrote:\n{js}");
        assert!(!js.contains("addEventListener(\"keydown\""), "{js}");
    }
}

/// A sidebar's toggle points at the panel by its id: the author's, when
/// there is one, in place of the one made for it.
#[test]
fn a_sidebar_s_toggle_points_at_the_id_its_author_gave() {
    let js = spa_generated(&source(
        "Sidebar(id: \"nav\") { Sidebar.Item(to: \"/\") { Text(\"Home\") } }",
    ));
    assert!(
        js.contains("WF.el(\"aside\", { className: \"wf-sidebar\", id: \"nav\" })"),
        "{js}"
    );
    assert!(js.contains("\"aria-controls\": \"nav\""), "{js}");
    assert!(
        !js.contains("wf-sidebar-"),
        "no id of its own beside it: {js}"
    );
    let plain = spa_generated(&source(
        "Sidebar { Sidebar.Item(to: \"/\") { Text(\"Home\") } }",
    ));
    assert!(plain.contains("id: \"wf-sidebar-"), "{plain}");
    assert!(
        plain.contains("\"aria-controls\": \"wf-sidebar-"),
        "{plain}"
    );
}

/// A control drawn in a box: what is the whole control's — `hidden:`,
/// `lang:`, `dir:` — is its box's, and everything else its input's; a
/// switch's `role:` takes the place of `switch`.
#[test]
fn a_control_in_a_box_keeps_hidden_lang_and_dir_on_its_box() {
    let src = source(
        "Switch(bind: open, label: \"Wrap\", role: \"menuitemcheckbox\", title: \"Wraps lines\", hidden: true, lang: \"ar\")\n    Slider(bind: n, label: \"Size\", id: \"size\", disabled: true, dir: \"rtl\")",
    );
    for backend in [Backend::Spa, Backend::Ssg] {
        let els = elems(backend, &src);
        let label = find(&els, At::Root("wf-switch")).unwrap();
        let input = find(&els, At::Input("wf-switch")).unwrap();
        assert!(
            label.attr("hidden").is_some(),
            "{}: {}",
            backend.name(),
            label.raw
        );
        assert_eq!(
            label.attr("lang"),
            Some("ar"),
            "{}: {}",
            backend.name(),
            label.raw
        );
        assert_eq!(
            input.attr("role"),
            Some("menuitemcheckbox"),
            "{}: {}",
            backend.name(),
            input.raw
        );
        assert_eq!(
            input.attr("title"),
            Some("Wraps lines"),
            "{}: {}",
            backend.name(),
            input.raw
        );
        assert!(
            input.attr("hidden").is_none() && input.attr("lang").is_none(),
            "{}",
            input.raw
        );
        let slider = find(&els, At::Root("wf-slider")).unwrap();
        let range = find(&els, At::Input("wf-slider")).unwrap();
        assert_eq!(
            slider.attr("dir"),
            Some("rtl"),
            "{}: {}",
            backend.name(),
            slider.raw
        );
        assert_eq!(
            range.attr("id"),
            Some("size"),
            "{}: {}",
            backend.name(),
            range.raw
        );
        assert!(
            range.attr("disabled").is_some(),
            "a disabled slider is: {}",
            range.raw
        );
    }
    let js = spa_generated(&src);
    assert!(!js.contains("role: \"switch\""), "{js}");
}

/// What every element takes beside its attributes — a lifetime — is the
/// shared tail's on an icon button too: written among its attributes as
/// well, `mount:` ran once more as an attribute's value, with no element.
#[test]
fn an_icon_button_s_lifetime_is_not_an_attribute() {
    let js = spa_generated(&source(
        "IconButton(icon: \"close\", label: \"Close\", mount: (b) => console.log(b))",
    ));
    let line = js
        .lines()
        .find(|l| l.contains("wf-icon-btn"))
        .unwrap_or_else(|| panic!("{js}"));
    assert!(!line.contains("mount:"), "{line}");
    assert_eq!(
        js.matches("console.log(b)").count(),
        1,
        "once, by WF.attach: {js}"
    );
    assert!(js.contains("WF.attach("), "{js}");
}

/// A chart or a QR code drawn when the page is built takes what any
/// element takes beside its attributes too: `class:`, a style block.
#[test]
fn a_chart_drawn_at_build_takes_its_class_and_style() {
    let js = spa_generated(&source(
        "QrCode(\"https://example.com\", class: \"mine\") {\n        style { width: 4rem }\n    }",
    ));
    assert!(js.contains(".classList.add(\"mine\")"), "{js}");
    assert!(
        js.contains(".classList.add(\"wf-s"),
        "its style block's class: {js}"
    );
}

/// The pre-render and the template engine paint a modal's or a dialog's
/// title as the live page draws it: its heading, which names it — not the
/// `title` attribute, the tooltip a pointer shows.
#[test]
fn a_modal_s_title_is_painted_as_its_heading() {
    let src = source(
        "Modal(visible: open, title: \"Explain\", data-k: \"v\") {\n        Text(\"Body\")\n        Modal.Footer { Button(\"OK\") }\n    }\n    Dialog(visible: open, title: \"Sure?\") { Text(\"D\") }",
    );
    for (name, html) in [
        ("pre-render", ssg_html(&src)),
        ("template", template_html(&src)),
    ] {
        assert!(!html.contains("title=\"Explain\""), "{name}: {html}");
        let open = html
            .find("<dialog class=\"wf-modal\"")
            .unwrap_or_else(|| panic!("{name}: {html}"));
        let tag = &html[open..open + html[open..].find('>').unwrap()];
        let id = tag
            .split("aria-labelledby=\"")
            .nth(1)
            .and_then(|r| r.split('"').next())
            .unwrap_or_else(|| panic!("{name}: named by its heading: {tag}"));
        assert!(tag.contains("data-k=\"v\""), "{name}: {tag}");
        assert!(
            html.contains(&format!(
                "<div class=\"wf-modal__header\"><h3 id=\"{id}\">Explain</h3></div>"
            )),
            "{name}: {html}"
        );
        let body = html.find("<div class=\"wf-modal__body\">").unwrap();
        let footer = html.find("<div class=\"wf-modal__footer\">").unwrap();
        assert!(
            html[body..footer].contains("Body") && html[footer..].contains("OK"),
            "{name}: the body, then the footer: {html}"
        );
        assert!(
            html.contains("<h3 id=\"") && html.contains(">Sure?</h3>"),
            "{name}: a dialog's too: {html}"
        );
        let ids: Vec<&str> = html
            .split("<h3 id=\"")
            .skip(1)
            .map(|r| r.split('"').next().unwrap())
            .collect();
        assert_eq!(ids.len(), 2, "{name}");
        assert_ne!(ids[0], ids[1], "{name}: each heading its own id");
    }
}

/// A long list's, a splitter's and a carousel's `label:` is their name, on
/// the live page and so in the pre-render and the template engine — not
/// words painted inside them; a breadcrumb named by its author is named
/// once.
#[test]
fn a_label_that_names_is_painted_as_a_name() {
    let src = source(
        "VirtualList(label: \"Results\") { for r in [1] by r { Text(\"row {r}\") } }\n    Splitter(bind: w, label: \"Resize the panel\")\n    Carousel(label: \"Shots\") { Carousel.Slide { Text(\"1\") } }\n    Breadcrumb(aria-label: \"Trail\") { Breadcrumb.Item { Text(\"Here\") } }\n    Breadcrumb { Breadcrumb.Item { Text(\"There\") } }",
    );
    for backend in [Backend::Ssg, Backend::Template] {
        let out = common::render(backend, &src);
        let els = elems(backend, &src);
        for (class, name) in [
            ("wf-virtual-list", "Results"),
            ("wf-splitter", "Resize the panel"),
            ("wf-carousel", "Shots"),
        ] {
            let el = find(&els, At::Root(class)).unwrap();
            assert_eq!(
                el.attr("aria-label"),
                Some(name),
                "{}: {}",
                backend.name(),
                el.raw
            );
        }
        assert!(
            !out.contains(">Resize the panel<") && !out.contains("Results\n"),
            "{}: {out}",
            backend.name()
        );
        let trails: Vec<&Elem> = els
            .iter()
            .filter(|e| e.has_class("wf-breadcrumb"))
            .collect();
        assert_eq!(
            trails[0].attr("aria-label"),
            Some("Trail"),
            "{}",
            trails[0].raw
        );
        assert_eq!(
            trails[0].raw.matches("aria-label").count(),
            1,
            "named once: {}",
            trails[0].raw
        );
        assert_eq!(
            trails[1].attr("aria-label"),
            Some("breadcrumb"),
            "as the live page names it: {}",
            trails[1].raw
        );
    }
}

/// An attribute the component has no prop for — `C04`'s — is painted as
/// the live page writes it, on any element: `target:` on a link, `colspan:`
/// on a cell, an author's own.
#[test]
fn an_attribute_with_no_prop_is_painted_as_the_live_page_writes_it() {
    let src = source(
        "Link(\"Docs\", to: \"/docs\", target: \"_blank\", rel: \"noopener\")\n    Stack(id: \"s\", lang: \"ar\", hidden: true, draggable: true, spellcheck: false) { Text(\"x\") }",
    );
    for backend in [Backend::Ssg, Backend::Template] {
        let els = elems(backend, &src);
        let link = els.iter().find(|e| e.has_class("wf-link")).unwrap();
        assert_eq!(
            link.attr("target"),
            Some("_blank"),
            "{}: {}",
            backend.name(),
            link.raw
        );
        assert_eq!(
            link.attr("rel"),
            Some("noopener"),
            "{}: {}",
            backend.name(),
            link.raw
        );
        let stack = els.iter().find(|e| e.has_class("wf-stack")).unwrap();
        assert_eq!(
            stack.attr("id"),
            Some("s"),
            "{}: {}",
            backend.name(),
            stack.raw
        );
        assert_eq!(
            stack.attr("lang"),
            Some("ar"),
            "{}: {}",
            backend.name(),
            stack.raw
        );
        assert!(
            stack.attr("hidden").is_some(),
            "{}: {}",
            backend.name(),
            stack.raw
        );
        assert_eq!(
            stack.attr("draggable"),
            Some("true"),
            "{}: {}",
            backend.name(),
            stack.raw
        );
        assert!(
            stack.attr("spellcheck").is_none(),
            "`false` is no attribute, as on the live page: {}",
            stack.raw
        );
    }
}

/// `on close` on a modal or a dialog is its own event — the `<dialog>`'s
/// `close`, however it was closed — so it is not `V09`, and the registry
/// offers it.
#[test]
fn on_close_is_an_event_a_modal_and_a_dialog_fire() {
    let src = source(
        "state closes = 0\n    Modal(visible: open, title: \"M\") {\n        on close { closes = closes + 1 }\n        Text(\"m\")\n    }\n    Dialog(visible: open, title: \"D\") {\n        on close { closes = closes + 1 }\n        Text(\"d\")\n    }",
    );
    assert_eq!(findings(&src), Vec::<String>::new());
    for name in ["Modal", "Dialog"] {
        let sig = webfluent::registry::component(name).unwrap();
        assert!(sig.events.contains(&"close"), "{name}");
    }
    let js = spa_generated(&src);
    let modal = js
        .lines()
        .find(|l| l.contains("WF.el(\"dialog\", { className: \"wf-modal\""))
        .unwrap();
    let var = modal
        .trim()
        .trim_start_matches("const ")
        .split(' ')
        .next()
        .unwrap();
    assert!(
        js.contains(&format!("{var}.addEventListener(\"close\"")),
        "the dialog's own close event: {js}"
    );
}

/// A four-pixel PNG, for a build to make a picture of.
const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 4, 0, 0, 0, 4, 8, 2, 0,
    0, 0, 38, 147, 9, 41, 0, 0, 0, 16, 73, 68, 65, 84, 120, 156, 99, 224, 47, 203, 131, 35, 6, 226,
    56, 0, 133, 51, 15, 49, 121, 135, 76, 10, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

/// An `Image` of a picture the program declares is a `<picture>` the
/// runtime builds: what any element takes is its image's — where its `alt:`
/// is and its `load` fires — on the live page and in the pre-render.
#[test]
fn a_picture_s_image_takes_them() {
    let dir = std::env::temp_dir().join(format!("wf-picture-attrs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(dir.join("src/hero.png"), PNG).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "p", "build": { "ssg": true, "minify": false } }"#,
    )
    .unwrap();
    let body = source("Image(hero, alt: \"The harbour\", {attrs}) { {on} }")
        .replace("page P(", "image hero = \"hero.png\"\n\npage P(")
        .replace("title: \"T\")", "title: \"T\", description: \"D\")");
    std::fs::write(dir.join("src/pages/P.wf"), body).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("Build complete"), "{said}");
    let mut js = std::fs::read_to_string(dir.join("build/app.js")).unwrap();
    if let Ok(pages) = std::fs::read_dir(dir.join("build/pages")) {
        for page in pages.flatten() {
            if page.path().extension().is_some_and(|e| e == "js") {
                js.push_str(&std::fs::read_to_string(page.path()).unwrap());
            }
        }
    }
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let line = js
        .lines()
        .find(|l| l.contains("WF.picture("))
        .unwrap_or_else(|| panic!("{js}"));
    assert!(
        line.contains("attrs: { role: \"r\", tabindex: \"-1\", \"data-k\": \"v\", \"aria-describedby\": \"d\", \"on:keydown\": (e) => { _keys.set((_keys() + 1)); } }"),
        "{line}"
    );
    let img = html
        .split("<img ")
        .nth(1)
        .and_then(|r| r.split('>').next())
        .unwrap_or_else(|| panic!("{html}"));
    for attr in [
        "role=\"r\"",
        "tabindex=\"-1\"",
        "data-k=\"v\"",
        "aria-describedby=\"d\"",
        "alt=\"The harbour\"",
    ] {
        assert!(img.contains(attr), "{attr} on the painted image: {img}");
    }
}
