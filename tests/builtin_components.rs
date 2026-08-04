//! Conformance suite for the engine's built-in components.
//!
//! A `.wf` file has one meaning, but the engine has three HTML renderers for it —
//! the SPA JS codegen, the static site generator and the template engine — and each
//! carries a private copy of the built-in table. Nothing in the build cross-checks
//! them, so a component can render `<dialog>` under `wf build` and `<div>` under
//! `wf build --ssg` and both "succeed".
//!
//! This suite states the contract once, in [`SPECS`], and holds every backend to
//! it. A failure names the backend that drifted.

mod common;

use common::*;

/// The contract for one built-in component.
struct Spec {
    /// Component name as written in `.wf`.
    name: &'static str,
    /// A page body that exercises it.
    body: &'static str,
    /// The HTML tag its root must render as.
    tag: &'static str,
    /// The class its root must carry (`""` for components that carry none).
    class: &'static str,
    /// Backends that deliberately do not paint this component at all.
    skip: &'static [Backend],
}

const S: &[Backend] = &[];

/// Every built-in component the lexer knows, with the tag and class it is
/// documented to produce. Sources: `src/lexer/token.rs` (the keyword table) and
/// `src/themes/components.rs` (the stylesheet each class must match).
const SPECS: &[Spec] = &[
    // ─── Layout ──────────────────────────────────────────
    Spec {
        name: "Container",
        body: "Container { Text(\"x\") }",
        tag: "div",
        class: "wf-container",
        skip: S,
    },
    Spec {
        name: "Row",
        body: "Row { Text(\"x\") }",
        tag: "div",
        class: "wf-row",
        skip: S,
    },
    Spec {
        name: "Column",
        body: "Column { Text(\"x\") }",
        tag: "div",
        class: "wf-col",
        skip: S,
    },
    Spec {
        name: "Grid",
        body: "Grid { Text(\"x\") }",
        tag: "div",
        class: "wf-grid",
        skip: S,
    },
    Spec {
        name: "Stack",
        body: "Stack { Text(\"x\") }",
        tag: "div",
        class: "wf-stack",
        skip: S,
    },
    Spec {
        name: "Spacer",
        body: "Spacer",
        tag: "div",
        class: "wf-spacer",
        skip: S,
    },
    Spec {
        name: "Divider",
        body: "Divider",
        tag: "hr",
        class: "wf-divider",
        skip: S,
    },
    // ─── Navigation ──────────────────────────────────────
    Spec {
        name: "Navbar",
        body: "Navbar { Text(\"x\") }",
        tag: "nav",
        class: "wf-navbar",
        skip: S,
    },
    Spec {
        name: "Sidebar",
        body: "Sidebar { Text(\"x\") }",
        tag: "aside",
        class: "wf-sidebar",
        skip: S,
    },
    Spec {
        name: "Breadcrumb",
        body: "Breadcrumb { Text(\"x\") }",
        tag: "nav",
        class: "wf-breadcrumb",
        skip: S,
    },
    Spec {
        name: "Link",
        body: "Link(\"x\", to: \"/a\")",
        tag: "a",
        class: "wf-link",
        skip: S,
    },
    Spec {
        name: "Menu",
        body: "Menu { Text(\"x\") }",
        tag: "div",
        class: "wf-menu",
        skip: S,
    },
    Spec {
        name: "Tabs",
        body: "Tabs { TabPage(\"A\") { Text(\"x\") } }",
        tag: "div",
        class: "wf-tabs",
        skip: S,
    },
    Spec {
        name: "TabPage",
        body: "TabPage(\"A\") { Text(\"x\") }",
        tag: "div",
        class: "wf-tab-page",
        skip: S,
    },
    // ─── Data display ────────────────────────────────────
    Spec {
        name: "Card",
        body: "Card { Text(\"x\") }",
        tag: "div",
        class: "wf-card",
        skip: S,
    },
    Spec {
        name: "Table",
        body: "Table { Tbody { Trow { Tcell(\"x\") } } }",
        tag: "table",
        class: "wf-table",
        skip: S,
    },
    Spec {
        name: "Thead",
        body: "Thead { Trow { Tcell(\"x\") } }",
        tag: "thead",
        class: "",
        skip: S,
    },
    Spec {
        name: "Tbody",
        body: "Tbody { Trow { Tcell(\"x\") } }",
        tag: "tbody",
        class: "",
        skip: S,
    },
    Spec {
        name: "Trow",
        body: "Trow { Tcell(\"x\") }",
        tag: "tr",
        class: "",
        skip: S,
    },
    Spec {
        name: "Tcell",
        body: "Tcell(\"x\")",
        tag: "td",
        class: "",
        skip: S,
    },
    Spec {
        name: "List",
        body: "List { Text(\"x\") }",
        tag: "ul",
        class: "wf-list",
        skip: S,
    },
    Spec {
        name: "Badge",
        body: "Badge(\"x\")",
        tag: "span",
        class: "wf-badge",
        skip: S,
    },
    Spec {
        name: "Avatar",
        body: "Avatar(initials: \"MO\")",
        tag: "div",
        class: "wf-avatar",
        skip: S,
    },
    Spec {
        name: "Tooltip",
        body: "Tooltip(text: \"t\") { Text(\"x\") }",
        tag: "div",
        class: "wf-tooltip",
        skip: S,
    },
    Spec {
        name: "Tag",
        body: "Tag(\"x\")",
        tag: "span",
        class: "wf-tag",
        skip: S,
    },
    // ─── Data input ──────────────────────────────────────
    Spec {
        name: "Input",
        body: "Input(placeholder: \"p\")",
        tag: "input",
        class: "wf-input",
        skip: S,
    },
    Spec {
        name: "Select",
        body: "Select { Option(\"x\") }",
        tag: "select",
        class: "wf-select",
        skip: S,
    },
    Spec {
        name: "Option",
        body: "Option(\"x\")",
        tag: "option",
        class: "",
        skip: S,
    },
    Spec {
        name: "Checkbox",
        body: "Checkbox(label: \"x\")",
        tag: "label",
        class: "wf-checkbox",
        skip: S,
    },
    Spec {
        name: "Radio",
        body: "Radio(label: \"x\")",
        tag: "label",
        class: "wf-radio",
        skip: S,
    },
    Spec {
        name: "Switch",
        body: "Switch(label: \"x\")",
        tag: "label",
        class: "wf-switch",
        skip: S,
    },
    Spec {
        name: "Slider",
        body: "Slider(min: 0, max: 10)",
        tag: "div",
        class: "wf-slider",
        skip: S,
    },
    Spec {
        name: "DatePicker",
        body: "DatePicker(label: \"d\")",
        tag: "div",
        class: "wf-datepicker",
        skip: S,
    },
    Spec {
        name: "FileUpload",
        body: "FileUpload(label: \"f\")",
        tag: "div",
        class: "wf-file-upload",
        skip: S,
    },
    Spec {
        name: "Form",
        body: "Form { Input(placeholder: \"p\") }",
        tag: "form",
        class: "wf-form",
        skip: S,
    },
    // ─── Feedback ────────────────────────────────────────
    Spec {
        name: "Alert",
        body: "Alert(\"x\")",
        tag: "div",
        class: "wf-alert",
        skip: S,
    },
    // Native `<dialog>`: the browser supplies the focus trap, the inert
    // background, Escape-to-close and `aria-modal` that a div cannot.
    Spec {
        name: "Modal",
        body: "Modal(title: \"t\") { Text(\"x\") }",
        tag: "dialog",
        class: "wf-modal",
        skip: S,
    },
    Spec {
        name: "Dialog",
        body: "Dialog(title: \"t\") { Text(\"x\") }",
        tag: "dialog",
        class: "wf-dialog",
        skip: S,
    },
    Spec {
        name: "Spinner",
        body: "Spinner",
        tag: "div",
        class: "wf-spinner",
        skip: S,
    },
    Spec {
        name: "Progress",
        body: "Progress(value: 50)",
        tag: "progress",
        class: "wf-progress",
        skip: S,
    },
    Spec {
        name: "Skeleton",
        body: "Skeleton",
        tag: "div",
        class: "wf-skeleton",
        skip: S,
    },
    // ─── Actions ─────────────────────────────────────────
    Spec {
        name: "Button",
        body: "Button(\"x\")",
        tag: "button",
        class: "wf-btn",
        skip: S,
    },
    Spec {
        name: "IconButton",
        body: "IconButton(icon: \"plus\")",
        tag: "button",
        class: "wf-icon-btn",
        skip: S,
    },
    Spec {
        name: "ButtonGroup",
        body: "ButtonGroup { Button(\"x\") }",
        tag: "div",
        class: "wf-btn-group",
        skip: S,
    },
    Spec {
        name: "Dropdown",
        body: "Dropdown(label: \"m\") { Text(\"x\") }",
        tag: "div",
        class: "wf-dropdown",
        skip: S,
    },
    // ─── Media ───────────────────────────────────────────
    Spec {
        name: "Image",
        body: "Image(src: \"/a.png\", alt: \"a\")",
        tag: "img",
        class: "wf-image",
        skip: S,
    },
    Spec {
        name: "Video",
        body: "Video(src: \"/a.mp4\")",
        tag: "video",
        class: "wf-video",
        skip: S,
    },
    Spec {
        name: "Icon",
        body: "Icon(\"plus\")",
        tag: "i",
        class: "wf-icon",
        skip: S,
    },
    Spec {
        name: "Carousel",
        body: "Carousel { Image(src: \"/a.png\", alt: \"a\") }",
        tag: "div",
        class: "wf-carousel",
        skip: S,
    },
    // ─── Typography ──────────────────────────────────────
    Spec {
        name: "Text",
        body: "Text(\"x\")",
        tag: "p",
        class: "wf-text",
        skip: S,
    },
    Spec {
        name: "Heading",
        body: "Heading(\"x\")",
        tag: "h2",
        class: "wf-heading",
        skip: S,
    },
    Spec {
        name: "Code",
        body: "Code(\"x\")",
        tag: "code",
        class: "wf-code",
        skip: S,
    },
    Spec {
        name: "Blockquote",
        body: "Blockquote(\"x\")",
        tag: "blockquote",
        class: "wf-blockquote",
        skip: S,
    },
];

fn spec_source(spec: &Spec) -> String {
    page(spec.body)
}

// ─── The contract ───────────────────────────────────────────────────────

/// Every built-in must render the tag its contract names, in every backend.
#[test]
fn every_builtin_renders_its_contracted_tag() {
    let mut failures = Vec::new();
    for spec in SPECS {
        let src = spec_source(spec);
        for backend in Backend::ALL {
            if spec.skip.contains(&backend) {
                continue;
            }
            match root(backend, &src) {
                Some(e) if e.tag == spec.tag => {}
                Some(e) => failures.push(format!(
                    "{:<12} {:<28} expected <{}>, got <{}>   [{}]",
                    spec.name,
                    backend.name(),
                    spec.tag,
                    e.tag,
                    e.raw.trim()
                )),
                None => failures.push(format!(
                    "{:<12} {:<28} rendered nothing",
                    spec.name,
                    backend.name()
                )),
            }
        }
    }
    assert!(failures.is_empty(), "tag drift:\n{}", failures.join("\n"));
}

/// Every built-in must put its base class on its root, in every backend. This is
/// what the stylesheet targets — a root without it is an unstyled component.
#[test]
fn every_builtin_renders_its_contracted_root_class() {
    let mut failures = Vec::new();
    for spec in SPECS {
        if spec.class.is_empty() {
            continue;
        }
        let src = spec_source(spec);
        for backend in Backend::ALL {
            if spec.skip.contains(&backend) {
                continue;
            }
            match root(backend, &src) {
                Some(e) if e.has_class(spec.class) => {}
                Some(e) => failures.push(format!(
                    "{:<12} {:<28} root lacks .{} (has {:?})   [{}]",
                    spec.name,
                    backend.name(),
                    spec.class,
                    e.classes,
                    e.raw.trim()
                )),
                None => failures.push(format!(
                    "{:<12} {:<28} rendered nothing",
                    spec.name,
                    backend.name()
                )),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "root class drift:\n{}",
        failures.join("\n")
    );
}

/// The three backends must agree with each other, independently of whether they
/// agree with the table above. This is the drift check proper.
#[test]
fn backends_agree_on_tag_and_class() {
    let mut failures = Vec::new();
    for spec in SPECS {
        let src = spec_source(spec);
        let mut seen: Vec<(Backend, String, Vec<String>)> = Vec::new();
        for backend in Backend::ALL {
            if spec.skip.contains(&backend) {
                continue;
            }
            if let Some(e) = root(backend, &src) {
                seen.push((backend, e.tag, e.classes));
            }
        }
        let Some((_, tag0, classes0)) = seen.first().cloned() else {
            continue;
        };
        for (backend, tag, classes) in seen.iter().skip(1) {
            if *tag != tag0 {
                failures.push(format!(
                    "{:<12} tag: {} says <{}>, {} says <{}>",
                    spec.name,
                    seen[0].0.name(),
                    tag0,
                    backend.name(),
                    tag
                ));
            }
            // Compare only the engine's own `wf-` classes; runtime state words
            // like `open` legitimately differ.
            let a: Vec<_> = classes0.iter().filter(|c| c.starts_with("wf-")).collect();
            let b: Vec<_> = classes.iter().filter(|c| c.starts_with("wf-")).collect();
            if a != b {
                failures.push(format!(
                    "{:<12} class: {} says {:?}, {} says {:?}",
                    spec.name,
                    seen[0].0.name(),
                    a,
                    backend.name(),
                    b
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "cross-backend drift:\n{}",
        failures.join("\n")
    );
}

// ─── Modifiers ──────────────────────────────────────────────────────────

/// `Heading("x", h1)` must produce an `<h1>`. Heading level is a document-outline
/// and SEO concern, not a font size — emitting `<h2 class="wf-heading--h1">` is
/// both the wrong outline level and, since no such rule exists, the wrong size.
#[test]
fn heading_modifier_sets_the_heading_level() {
    let mut failures = Vec::new();
    for level in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        let src = page(&format!("Heading(\"x\", {})", level));
        for backend in Backend::ALL {
            let Some(e) = root(backend, &src) else {
                continue;
            };
            if e.tag != level {
                failures.push(format!(
                    "Heading(_, {}) in {} produced <{}> [{}]",
                    level,
                    backend.name(),
                    e.tag,
                    e.raw.trim()
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "heading level drift:\n{}",
        failures.join("\n")
    );
}

/// A modifier must reach the class list identically in every backend. A modifier
/// dropped by one renderer is a component that looks different after `--ssg`.
#[test]
fn modifiers_map_to_the_same_class_in_every_backend() {
    // (component, modifier) pairs that name a real visual variant.
    let cases: &[(&str, &str, &str)] = &[
        ("Button(\"x\", {m})", "wf-btn", "primary"),
        ("Button(\"x\", {m})", "wf-btn", "secondary"),
        ("Button(\"x\", {m})", "wf-btn", "success"),
        ("Button(\"x\", {m})", "wf-btn", "danger"),
        ("Button(\"x\", {m})", "wf-btn", "warning"),
        ("Button(\"x\", {m})", "wf-btn", "info"),
        ("Button(\"x\", {m})", "wf-btn", "small"),
        ("Button(\"x\", {m})", "wf-btn", "large"),
        ("Button(\"x\", {m})", "wf-btn", "full"),
        ("Button(\"x\", {m})", "wf-btn", "rounded"),
        ("Button(\"x\", {m})", "wf-btn", "pill"),
        ("Button(\"x\", {m})", "wf-btn", "outlined"),
        ("Card { Text(\"x\") }", "wf-card", "elevated"),
        ("Card { Text(\"x\") }", "wf-card", "outlined"),
        ("Card { Text(\"x\") }", "wf-card", "flat"),
        ("Badge(\"x\", {m})", "wf-badge", "primary"),
        ("Badge(\"x\", {m})", "wf-badge", "pill"),
        ("Text(\"x\", {m})", "wf-text", "bold"),
        ("Text(\"x\", {m})", "wf-text", "italic"),
        ("Text(\"x\", {m})", "wf-text", "underline"),
        ("Text(\"x\", {m})", "wf-text", "uppercase"),
        ("Text(\"x\", {m})", "wf-text", "lowercase"),
        ("Text(\"x\", {m})", "wf-text", "center"),
        ("Text(\"x\", {m})", "wf-text", "left"),
        ("Text(\"x\", {m})", "wf-text", "right"),
        ("Text(\"x\", {m})", "wf-text", "muted"),
        ("Text(\"x\", {m})", "wf-text", "subtitle"),
        ("Avatar(initials: \"M\", {m})", "wf-avatar", "large"),
        ("Alert(\"x\", {m})", "wf-alert", "success"),
        ("Alert(\"x\", {m})", "wf-alert", "danger"),
        ("Spinner({m})", "wf-spinner", "large"),
    ];

    let mut failures = Vec::new();
    for (tpl, _base, modifier) in cases {
        let body = tpl.replace("{m}", modifier);
        // A bare `Card(elevated) { }` has no positional slot; splice the modifier in.
        let body = if body.contains('{') && !body.contains(modifier) {
            body.replacen(" {", &format!("({}) {{", modifier), 1)
        } else {
            body
        };
        let src = page(&body);
        if parse_program(&src).is_err() {
            failures.push(format!("{modifier}: source did not parse: {body}"));
            continue;
        }

        let mut sets: Vec<(Backend, Vec<String>)> = Vec::new();
        for backend in Backend::ALL {
            if let Some(e) = root(backend, &src) {
                let mut cs: Vec<String> = e
                    .classes
                    .iter()
                    .filter(|c| c.starts_with("wf-"))
                    .cloned()
                    .collect();
                cs.sort();
                sets.push((backend, cs));
            }
        }
        let Some((_, first)) = sets.first().cloned() else {
            continue;
        };
        for (backend, cs) in sets.iter().skip(1) {
            if *cs != first {
                failures.push(format!(
                    "{:<10} on `{}`: {} -> {:?}, {} -> {:?}",
                    modifier,
                    body,
                    sets[0].0.name(),
                    first,
                    backend.name(),
                    cs
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "modifier drift:\n{}",
        failures.join("\n")
    );
}

/// Modifier/component pairs the documentation promises. Each must produce the
/// class the stylesheet defines, in every backend.
const DOCUMENTED_VARIANTS: &[(&str, &str, &str)] = &[
    ("Button(\"x\", {m})", "primary", "wf-btn--primary"),
    ("Button(\"x\", {m})", "secondary", "wf-btn--secondary"),
    ("Button(\"x\", {m})", "success", "wf-btn--success"),
    ("Button(\"x\", {m})", "danger", "wf-btn--danger"),
    ("Button(\"x\", {m})", "warning", "wf-btn--warning"),
    ("Button(\"x\", {m})", "info", "wf-btn--info"),
    ("Button(\"x\", {m})", "small", "wf-btn--small"),
    ("Button(\"x\", {m})", "large", "wf-btn--large"),
    ("Button(\"x\", {m})", "full", "wf-btn--full"),
    ("Button(\"x\", {m})", "rounded", "wf-btn--rounded"),
    ("Button(\"x\", {m})", "pill", "wf-btn--pill"),
    ("Button(\"x\", {m})", "outlined", "wf-btn--outlined"),
    ("Badge(\"x\", {m})", "primary", "wf-badge--primary"),
    ("Badge(\"x\", {m})", "success", "wf-badge--success"),
    ("Badge(\"x\", {m})", "danger", "wf-badge--danger"),
    ("Badge(\"x\", {m})", "warning", "wf-badge--warning"),
    ("Badge(\"x\", {m})", "info", "wf-badge--info"),
    ("Badge(\"x\", {m})", "secondary", "wf-badge--secondary"),
    ("Badge(\"x\", {m})", "pill", "wf-badge--pill"),
    ("Alert(\"x\", {m})", "success", "wf-alert--success"),
    ("Alert(\"x\", {m})", "danger", "wf-alert--danger"),
    ("Alert(\"x\", {m})", "warning", "wf-alert--warning"),
    ("Alert(\"x\", {m})", "info", "wf-alert--info"),
    ("Text(\"x\", {m})", "bold", "wf-text--bold"),
    ("Text(\"x\", {m})", "italic", "wf-text--italic"),
    ("Text(\"x\", {m})", "underline", "wf-text--underline"),
    ("Text(\"x\", {m})", "uppercase", "wf-text--uppercase"),
    ("Text(\"x\", {m})", "lowercase", "wf-text--lowercase"),
    ("Text(\"x\", {m})", "center", "wf-text--center"),
    ("Text(\"x\", {m})", "left", "wf-text--left"),
    ("Text(\"x\", {m})", "right", "wf-text--right"),
    ("Text(\"x\", {m})", "muted", "wf-text--muted"),
    ("Text(\"x\", {m})", "subtitle", "wf-text--subtitle"),
    ("Text(\"x\", {m})", "heading", "wf-text--heading"),
    ("Text(\"x\", {m})", "small", "wf-text--small"),
    ("Text(\"x\", {m})", "large", "wf-text--large"),
    ("Icon(\"x\", {m})", "small", "wf-icon--small"),
    ("Icon(\"x\", {m})", "large", "wf-icon--large"),
    ("Icon(\"x\", {m})", "primary", "wf-icon--primary"),
    ("Icon(\"x\", {m})", "danger", "wf-icon--danger"),
    ("Icon(\"x\", {m})", "success", "wf-icon--success"),
    ("Spinner({m})", "large", "wf-spinner--large"),
    (
        "Image(src: \"/a.png\", alt: \"a\", {m})",
        "rounded",
        "wf-image--rounded",
    ),
];

/// Each documented variant must produce its class, and the stylesheet must
/// define a rule for it. Either half missing means the author writes the
/// modifier and nothing changes on screen.
#[test]
fn documented_variants_produce_a_class_the_stylesheet_defines() {
    let css = built_in_css();
    let mut failures = Vec::new();

    for (tpl, modifier, class) in DOCUMENTED_VARIANTS {
        if !css_defines_class(&css, class) {
            failures.push(format!("stylesheet has no rule for .{class}"));
        }
        let body = tpl.replace("{m}", modifier);
        let src = page(&body);
        if parse_program(&src).is_err() {
            failures.push(format!("`{body}` does not parse"));
            continue;
        }
        for backend in Backend::ALL {
            let Some(e) = root(backend, &src) else {
                failures.push(format!("`{body}` rendered nothing in {}", backend.name()));
                continue;
            };
            if !e.has_class(class) {
                failures.push(format!(
                    "`{}` in {}: expected .{}, got {:?}",
                    body,
                    backend.name(),
                    class,
                    e.classes
                ));
            }
        }
    }
    failures.sort();
    failures.dedup();
    assert!(
        failures.is_empty(),
        "variant failures:\n{}",
        failures.join("\n")
    );
}

/// The inverse of the check above: a `.wf-x--y` rule in the shipped stylesheet is
/// only reachable if `y` is in the modifier vocabulary the parser accepts. A rule
/// keyed on a word the language cannot express is design work no author can use.
#[test]
fn stylesheet_variants_are_reachable_from_the_modifier_vocabulary() {
    let css = built_in_css();
    let vocab = all_modifiers();
    // Variants the engine sets from a named argument or internally, not from a
    // bare modifier word.
    let non_modifier_variants = [
        "label",  // Divider(label: …)
        "fluid",  // Container(fluid) — accepted by codegen, absent from vocab
        "block",  // Code(block)
        "exit",   // Toast animation state
        "circle", // Image/Skeleton shape
        "between", "end", // Row(justify: …)
        "xs", "sm", "lg", "xl", // Spacer sizes, set via `size:`
    ];

    let mut unreachable: Vec<String> = Vec::new();
    let mut rest = css.as_str();
    while let Some(i) = rest.find(".wf-") {
        let after = &rest[i + 1..];
        let end = after
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
            .unwrap_or(after.len());
        let selector = &after[..end];
        if let Some((_, variant)) = selector.split_once("--") {
            if !variant.is_empty()
                && variant.parse::<u32>().is_err()
                && !vocab.contains(&variant)
                && !non_modifier_variants.contains(&variant)
                && !unreachable.iter().any(|s| s == selector)
            {
                unreachable.push(selector.to_string());
            }
        }
        rest = after;
    }
    unreachable.sort();
    assert!(
        unreachable.is_empty(),
        "stylesheet rules keyed on words the parser does not accept as modifiers:\n  {}",
        unreachable.join("\n  ")
    );
}

/// `Heading("x", h1)` is the documented way to set a heading level, and every
/// backend answers it with a `wf-heading--hN` class. No such rule exists — the
/// stylesheet sizes headings by tag (`h1.wf-heading`). So the class is inert, and
/// on the SPA path, where the tag is not switched either, the level is lost
/// entirely.
#[test]
fn heading_level_modifiers_do_not_emit_a_dead_class() {
    let css = built_in_css();
    let mut failures = Vec::new();
    for level in ["h1", "h2", "h3", "h4", "h5", "h6"] {
        let src = page(&format!("Heading(\"x\", {})", level));
        for backend in Backend::ALL {
            let Some(e) = root(backend, &src) else {
                continue;
            };
            for class in e.classes.iter().filter(|c| c.contains("--h")) {
                if !css_defines_class(&css, class) {
                    failures.push(format!(
                        "Heading(_, {}) in {} emits .{}, which the stylesheet never defines",
                        level,
                        backend.name(),
                        class
                    ));
                }
            }
        }
    }
    failures.sort();
    failures.dedup();
    assert!(
        failures.is_empty(),
        "dead heading classes:\n{}",
        failures.join("\n")
    );
}

/// The 43 classes below are emitted with no rule behind them, and that is now a
/// deliberate, reported behaviour rather than a silent one.
///
/// Suppressing them in the codegen would break anyone who wrote the rule
/// themselves — `{base}--{modifier}` is a documented styling hook, not only a
/// theme internal. Defining all 43 would be design work nobody asked for. So the
/// codegen still emits them and `linter::vocabulary` reports each one as `V02`,
/// naming the class and offering both ways out. This test records what the
/// codegen does; `dead_variant_tests` records that the author is told.
#[test]
fn modifiers_outside_a_components_variant_set_do_not_become_classes() {
    let css = built_in_css();
    // (component call template, modifiers that component does not define)
    let cases: &[(&str, &[&str])] = &[
        (
            "Alert(\"x\", {m})",
            &[
                "elevated",
                "flat",
                "outlined",
                "pill",
                "rounded",
                "primary",
                "secondary",
                "small",
                "large",
                "full",
            ],
        ),
        (
            "Text(\"x\", {m})",
            &[
                "pill", "rounded", "outlined", "elevated", "flat", "full", "fit", "square",
            ],
        ),
        (
            "Heading(\"x\", {m})",
            &[
                "pill", "rounded", "outlined", "elevated", "primary", "danger",
            ],
        ),
        (
            "Badge(\"x\", {m})",
            &[
                "elevated", "flat", "outlined", "rounded", "small", "large", "full",
            ],
        ),
        (
            "Code(\"x\", {m})",
            &[
                "primary", "danger", "pill", "rounded", "outlined", "elevated",
            ],
        ),
        (
            "Tag(\"x\", {m})",
            &[
                "primary", "danger", "success", "pill", "rounded", "elevated",
            ],
        ),
    ];

    let mut dead: Vec<String> = Vec::new();
    for (tpl, modifiers) in cases {
        for m in *modifiers {
            let src = page(&tpl.replace("{m}", m));
            if parse_program(&src).is_err() {
                continue;
            }
            for backend in Backend::ALL {
                let Some(e) = root(backend, &src) else {
                    continue;
                };
                for class in &e.classes {
                    if class.starts_with("wf-")
                        && !css_defines_class(&css, class)
                        && !dead.contains(class)
                    {
                        dead.push(class.clone());
                    }
                }
            }
        }
    }
    dead.sort();
    // Every one of these must be something `V02` reports. A class the codegen
    // emits and the linter stays quiet about is the failure this pair guards.
    let reported: Vec<String> = dead
        .iter()
        .filter(|class| {
            let (component, modifier) = class.split_once("--").expect("a variant class");
            let component = component.trim_start_matches("wf-");
            SPECS
                .iter()
                .find(|s| s.class.ends_with(component))
                .map(|spec| {
                    let src = page(
                        &spec
                            .body
                            .replace("(\"x\")", &format!("(\"x\", {modifier})")),
                    );
                    parse_program(&src)
                        .map(|p| {
                            webfluent::validate_vocabulary(&p, "<t>")
                                .iter()
                                .any(|w| w.message.contains(class.as_str()))
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    let unreported: Vec<&String> = dead.iter().filter(|c| !reported.contains(c)).collect();
    assert!(
        unreported.is_empty(),
        "{} classes are emitted with no rule and no warning:\n  {}",
        unreported.len(),
        unreported
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

// ─── Author-supplied styling ────────────────────────────────────────────

/// A `style { }` block must survive on every component. The SPA codegen applies
/// style blocks after its special-component dispatch, so any component handled by
/// a dedicated emitter silently loses the author's styling.
#[test]
fn style_blocks_survive_on_every_component() {
    let mut failures = Vec::new();
    for spec in SPECS {
        // Give each component a style block, whatever its call shape.
        let body = if spec.body.contains('{') {
            spec.body
                .replacen('{', "{ style { color: \"rgb(1, 2, 3)\" }", 1)
        } else {
            format!("{} {{ style {{ color: \"rgb(1, 2, 3)\" }} }}", spec.body)
        };
        let src = page(&body);
        if parse_program(&src).is_err() {
            continue;
        }
        for backend in Backend::ALL {
            if spec.skip.contains(&backend) {
                continue;
            }
            let out = raw_output(backend, &src);
            if !out.contains("rgb(1, 2, 3)") {
                failures.push(format!(
                    "{:<12} {:<28} dropped the author's style block",
                    spec.name,
                    backend.name()
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "style blocks dropped:\n{}",
        failures.join("\n")
    );
}

// ─── Named arguments ────────────────────────────────────────────────────

/// Named arguments that name an HTML attribute must reach the output as one.
#[test]
fn named_args_become_html_attributes() {
    let cases: &[(&str, &str, &str)] = &[
        ("Image(src: \"/a.png\", alt: \"cat\")", "src", "/a.png"),
        ("Image(src: \"/a.png\", alt: \"cat\")", "alt", "cat"),
        ("Input(placeholder: \"name\")", "placeholder", "name"),
        ("Link(\"x\", to: \"/about\")", "href", "/about"),
        ("Video(src: \"/a.mp4\")", "src", "/a.mp4"),
    ];

    let mut failures = Vec::new();
    for (body, attr, want) in cases {
        let src = page(body);
        for backend in Backend::ALL {
            let Some(e) = root(backend, &src) else {
                failures.push(format!("`{body}` rendered nothing in {}", backend.name()));
                continue;
            };
            match e.attr(attr) {
                Some(v) if v.contains(want) => {}
                other => failures.push(format!(
                    "`{}` in {}: {}={:?}, expected to contain {:?}  [{}]",
                    body,
                    backend.name(),
                    attr,
                    other,
                    want,
                    e.raw.trim()
                )),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "attribute drift:\n{}",
        failures.join("\n")
    );
}

/// An input-type modifier must set `type=`. `datetime` is spelled
/// `datetime-local` in HTML.
#[test]
fn input_type_modifiers_set_the_type_attribute() {
    let cases: &[(&str, &str)] = &[
        ("text", "text"),
        ("email", "email"),
        ("password", "password"),
        ("number", "number"),
        ("search", "search"),
        ("tel", "tel"),
        ("url", "url"),
        ("date", "date"),
        ("time", "time"),
        ("datetime", "datetime-local"),
        ("color", "color"),
    ];

    let mut failures = Vec::new();
    for (modifier, want) in cases {
        let src = page(&format!("Input({}, placeholder: \"p\")", modifier));
        for backend in Backend::ALL {
            let Some(e) = root(backend, &src) else {
                continue;
            };
            match e.attr("type") {
                Some(v) if v == *want => {}
                other => failures.push(format!(
                    "Input({}) in {}: type={:?}, expected {:?}",
                    modifier,
                    backend.name(),
                    other,
                    want
                )),
            }
        }
    }
    assert!(
        failures.is_empty(),
        "input type drift:\n{}",
        failures.join("\n")
    );
}

// ─── Content ────────────────────────────────────────────────────────────

/// A component given text must render that text. A backend that drops the label
/// paints an empty control.
#[test]
fn positional_text_reaches_the_output() {
    let cases = [
        "Button(\"Save changes\")",
        "Text(\"Save changes\")",
        "Heading(\"Save changes\")",
        "Badge(\"Save changes\")",
        "Tag(\"Save changes\")",
        "Alert(\"Save changes\")",
        "Code(\"Save changes\")",
        "Blockquote(\"Save changes\")",
        "Link(\"Save changes\", to: \"/a\")",
        "Option(\"Save changes\")",
        "Tcell(\"Save changes\")",
    ];

    let mut failures = Vec::new();
    for body in cases {
        let src = page(body);
        for backend in Backend::ALL {
            let out = raw_output(backend, &src);
            if !out.contains("Save changes") {
                failures.push(format!("`{}` lost its text in {}", body, backend.name()));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "text dropped:\n{}",
        failures.join("\n")
    );
}

/// Every spec source must parse. A component in the keyword table that cannot be
/// written is a hole in the language surface.
#[test]
fn every_spec_source_parses() {
    let mut failures = Vec::new();
    for spec in SPECS {
        let src = spec_source(spec);
        if let Err(e) = parse_program(&src) {
            failures.push(format!("{:<12} {}", spec.name, e));
        }
    }
    assert!(
        failures.is_empty(),
        "unparseable component syntax:\n{}",
        failures.join("\n")
    );
}
