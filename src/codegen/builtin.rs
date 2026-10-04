//! The built-in component table as data — one source for every renderer.
//!
//! This used to live three times: once in `codegen::js` for the SPA bundle, once
//! in `codegen::ssg` for the static paint, and once in `template` for the library
//! renderer. Nothing compared them, so the same `.wf` source could render
//! `<input class="wf-slider">` statically and a `<div class="wf-slider">` wrapper
//! at runtime, and hydration would find a DOM it did not build. The three copies
//! had already drifted on nine components and eleven modifiers.
//!
//! Adding a component or a modifier now means editing one table, the way
//! [`crate::parser::MODIFIER_KEYWORDS`] did for the modifier vocabulary.

use crate::parser::ast::{Arg, Expr};

/// The HTML tag and base class a built-in component renders as.
///
/// Components with no class of their own (`Thead`, `Option`, the table parts)
/// return `""`; callers must not emit an empty `class` attribute for them.
///
/// Unknown names fall back to `("div", "")` — the parser has already reported an
/// unknown component, and a renderer is not the place to fail a build.
pub fn builtin_to_html(name: &str) -> (&'static str, &'static str) {
    match name {
        // ─── Layout ──────────────────────────────────────
        "Container" => ("div", "wf-container"),
        // A node handed to somebody else's code.
        "Host" => ("div", "wf-host"),
        // `Unsafe.html(markup)`: the one element whose content is markup.
        "UnsafeHtml" => ("div", "wf-html"),
        "Row" => ("div", "wf-row"),
        "Column" => ("div", "wf-col"),
        "Grid" => ("div", "wf-grid"),
        "Stack" => ("div", "wf-stack"),
        "Spacer" => ("div", "wf-spacer"),
        "Divider" => ("hr", "wf-divider"),

        // ─── Navigation ──────────────────────────────────
        "Navbar" => ("nav", "wf-navbar"),
        "Sidebar" => ("aside", "wf-sidebar"),
        "Breadcrumb" => ("nav", "wf-breadcrumb"),
        "Link" => ("a", "wf-link"),
        "Menu" => ("div", "wf-menu"),
        "Tabs" => ("div", "wf-tabs"),
        "TabPage" => ("div", "wf-tab-page"),

        // ─── Data display ────────────────────────────────
        "Card" => ("div", "wf-card"),
        "Table" => ("table", "wf-table"),
        "Thead" => ("thead", ""),
        "Tbody" => ("tbody", ""),
        "Trow" => ("tr", ""),
        "Tcell" => ("td", ""),
        "List" => ("ul", "wf-list"),
        "Badge" => ("span", "wf-badge"),
        "Avatar" => ("div", "wf-avatar"),
        "Tooltip" => ("div", "wf-tooltip"),
        "Tag" => ("span", "wf-tag"),

        // ─── Data input ──────────────────────────────────
        "Input" => ("input", "wf-input"),
        "Select" => ("select", "wf-select"),
        "Option" => ("option", ""),
        "Checkbox" => ("label", "wf-checkbox"),
        "Radio" => ("label", "wf-radio"),
        "Switch" => ("label", "wf-switch"),
        // The next three wrap a label and an input, so their root is the wrapper
        // in every backend. The static renderers used to emit the bare `<input>`,
        // which hydration could not reconcile with the wrapper the SPA builds.
        "Slider" => ("div", "wf-slider"),
        "DatePicker" => ("div", "wf-datepicker"),
        "FileUpload" => ("div", "wf-file-upload"),
        "Textarea" => ("textarea", "wf-input wf-textarea"),
        "Form" => ("form", "wf-form"),

        // ─── Feedback ────────────────────────────────────
        "Alert" => ("div", "wf-alert"),
        "Toast" => ("div", "wf-toast"),
        // `<dialog>`, driven by `showModal()`. The browser then supplies focus
        // trapping, an inert background, Escape-to-close, `aria-modal` and the
        // `::backdrop` — every one of which a `div` with an `.open` class has to
        // reimplement, and none of which this engine had.
        "Modal" => ("dialog", "wf-modal"),
        "Dialog" => ("dialog", "wf-dialog"),
        "Spinner" => ("div", "wf-spinner"),
        "Progress" => ("progress", "wf-progress"),
        "Skeleton" => ("div", "wf-skeleton"),

        // ─── Actions ─────────────────────────────────────
        "Button" => ("button", "wf-btn"),
        "IconButton" => ("button", "wf-icon-btn"),
        "ButtonGroup" => ("div", "wf-btn-group"),
        "Dropdown" => ("div", "wf-dropdown"),

        // ─── Media ───────────────────────────────────────
        "Image" => ("img", "wf-image"),
        "Video" => ("video", "wf-video"),
        "Audio" => ("audio", "wf-audio"),
        "Icon" => ("i", "wf-icon"),
        "Carousel" => ("div", "wf-carousel"),

        // ─── Typography ──────────────────────────────────
        "Text" => ("p", "wf-text"),
        // The level modifier picks the real tag; see [`heading_tag`].
        "Heading" => ("h2", "wf-heading"),
        "Code" => ("code", "wf-code"),
        "Blockquote" => ("blockquote", "wf-blockquote"),
        "Markdown" => ("div", "wf-markdown"),

        // ─── Document structure ──────────────────────────
        // These come from the PDF vocabulary but are ordinary HTML on the web,
        // and `App { … Footer }` is the documented site-shell idiom. The SPA and
        // SSG renderers used to drop them to an unclassed `<div>`.
        "Section" => ("section", "wf-section"),
        "Paragraph" => ("p", "wf-text"),
        "Document" => ("div", "wf-document"),
        "Header" => ("header", "wf-header"),
        "Footer" => ("footer", "wf-footer"),
        "Background" => ("div", "wf-background"),
        "Watermark" => ("div", "wf-watermark"),
        "PageBreak" => ("div", "wf-page-break"),
        "Chart" => ("figure", "wf-chart"),
        "QrCode" => ("figure", "wf-qr"),
        "TableOfContents" => ("nav", "wf-toc"),

        // ─── Slides ──────────────────────────────────────
        // A deck is a paged document of fixed pages; each slide kind is a
        // section the engine sizes to the slide.
        "Presentation" => ("div", "wf-presentation"),
        "Slide" => ("section", "wf-slide"),
        "TitleSlide" => ("section", "wf-title-slide"),
        "SectionSlide" => ("section", "wf-section-slide"),
        "TwoColumn" => ("section", "wf-two-column"),
        "ImageSlide" => ("section", "wf-image-slide"),

        // ─── Routing ─────────────────────────────────────
        "Router" => ("div", "wf-router"),
        "Route" => ("div", ""),

        _ => ("div", ""),
    }
}

/// The class a modifier contributes, given the component's base class.
///
/// Returns `""` for modifiers that are not classes at all — input types become a
/// `type=` attribute (see [`input_type`]), and heading levels become the tag (see
/// [`heading_tag`]).
/// The CSS timing function an easing name stands for, in a `transition`
/// block and in the `easing:` prop alike; a name that is not one of the
/// engine's is passed through as written.
pub fn easing_css(name: &str) -> &str {
    match name {
        "ease" => "ease",
        "linear" => "linear",
        "easeIn" => "ease-in",
        "easeOut" => "ease-out",
        "easeInOut" => "ease-in-out",
        // The named easings are design tokens, so a theme retunes them
        // once and every animation follows.
        "spring" => "var(--ease-spring)",
        "standard" => "var(--ease-standard)",
        "bouncy" => "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
        "smooth" => "cubic-bezier(0.4, 0, 0.2, 1)",
        other => other,
    }
}

pub fn modifier_to_class(base_class: &str, modifier: &str) -> String {
    match modifier {
        // ─── Size ────────────────────────────────────────
        "small" => format!("{}--small", base_class),
        "medium" => String::new(), // the default; no class needed
        "large" => format!("{}--large", base_class),

        // ─── Colour ──────────────────────────────────────
        "primary" => format!("{}--primary", base_class),
        "secondary" => format!("{}--secondary", base_class),
        "success" => format!("{}--success", base_class),
        "danger" => format!("{}--danger", base_class),
        "warning" => format!("{}--warning", base_class),
        "info" => format!("{}--info", base_class),

        // A layout that asks to reflow on a narrow screen. One class,
        // shared by every layout, because the rule is the same for all of
        // them and a reader of the sheet should see it once.
        "stacks" => "wf-stacks".to_string(),

        // ─── Shape ───────────────────────────────────────
        "rounded" => format!("{}--rounded", base_class),
        "pill" => format!("{}--pill", base_class),
        "square" => format!("{}--square", base_class),
        "circle" => format!("{}--circle", base_class),

        // ─── Spacer sizes ────────────────────────────────
        "xs" | "sm" | "lg" | "xl" => format!("{}--{}", base_class, modifier),
        "md" => String::new(), // the default height

        // ─── Elevation ───────────────────────────────────
        "flat" => format!("{}--flat", base_class),
        "elevated" => format!("{}--elevated", base_class),
        "outlined" => format!("{}--outlined", base_class),

        // ─── Width ───────────────────────────────────────
        "full" => format!("{}--full", base_class),
        "fit" => format!("{}--fit", base_class),
        "fluid" => format!("{}--fluid", base_class),

        // ─── Text ────────────────────────────────────────
        // These style the text of whatever they are put on, so they always name
        // the typography class rather than a per-component variant.
        "bold" => "wf-text--bold".to_string(),
        "italic" => "wf-text--italic".to_string(),
        "underline" => "wf-text--underline".to_string(),
        "uppercase" => "wf-text--uppercase".to_string(),
        "lowercase" => "wf-text--lowercase".to_string(),
        "left" => "wf-text--left".to_string(),
        "center" => "wf-text--center".to_string(),
        "right" => "wf-text--right".to_string(),
        "heading" => "wf-text--heading".to_string(),
        "subtitle" => "wf-text--subtitle".to_string(),
        "muted" => "wf-text--muted".to_string(),

        // ─── Heading level ───────────────────────────────
        // No class: the stylesheet sizes headings by tag (`h1.wf-heading`), so a
        // `wf-heading--h1` class was inert everywhere it was emitted. The level
        // selects the tag instead — see [`heading_tag`].
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => String::new(),

        // ─── Other variants ──────────────────────────────
        "dismissible" => format!("{}--dismissible", base_class),
        "block" => format!("{}--block", base_class),
        "bordered" => format!("{}--bordered", base_class),

        // ─── Attributes, not classes ─────────────────────
        "text" | "email" | "password" | "number" | "search" | "tel" | "url" | "date" | "time"
        | "datetime" | "color" | "submit" | "reset" | "required" | "controls" | "autoplay"
        | "multiple" => String::new(),

        // ─── Tags, not classes ───────────────────────────
        // `List(ordered)` is an `<ol>`, `Tcell(header)` a `<th>`; see [`element_tag`].
        "ordered" | "header" => String::new(),

        // ─── Animation ───────────────────────────────────
        // Pure CSS keyframes, so they apply in static output too. The template
        // renderer used to drop them on the grounds that it emits no JS.
        "fadeIn" | "fadeOut" | "slideUp" | "slideDown" | "slideLeft" | "slideRight" | "scaleIn"
        | "scaleOut" | "bounce" | "shake" | "pulse" | "spin" | "expand" | "collapse" => {
            format!("wf-animate-{}", modifier)
        }
        "fast" => "wf-animate--fast".to_string(),
        "slow" => "wf-animate--slow".to_string(),

        _ => String::new(),
    }
}

/// The utility classes a layout element's `gap:`, `align:` and `justify:`
/// arguments select.
///
/// `Row(gap: md)` used to emit `wf-row--gap-md`, a class no stylesheet defined,
/// and `align`/`justify` only knew three values between them. One utility
/// family per axis, shared by `Row`, `Stack` and `Grid`, keeps the argument and
/// the rule in step across every backend.
pub fn layout_arg_classes(args: &[Arg]) -> Vec<String> {
    let mut classes = Vec::new();
    for arg in args {
        let Arg::Named(key, val) = arg else { continue };
        let value = match val {
            Expr::Identifier(id) => id.as_str(),
            Expr::StringLiteral(s) => s.as_str(),
            _ => continue,
        };
        let ok = match key.as_str() {
            "gap" => matches!(value, "xs" | "sm" | "md" | "lg" | "xl"),
            "align" => matches!(value, "start" | "center" | "end" | "stretch" | "baseline"),
            "justify" => matches!(
                value,
                "start" | "center" | "end" | "between" | "around" | "evenly"
            ),
            _ => false,
        };
        if ok {
            classes.push(format!("wf-{}--{}", key, value));
        }
    }
    classes
}

/// The classes a literal `class:` argument names, split on whitespace.
pub fn author_classes(args: &[Arg]) -> Vec<String> {
    args.iter()
        .find_map(|a| match a {
            Arg::Named(k, v) if k == "class" => literal_classes(v),
            _ => None,
        })
        .unwrap_or_default()
}

/// The classes a `class:` value names when every part of it is written out:
/// a string, a map whose values are `true`/`false`, a list of those, with
/// `null` and `false` skipped. `None` when any part reads a value.
pub fn literal_classes(value: &Expr) -> Option<Vec<String>> {
    fn push(text: &str, out: &mut Vec<String>) {
        for c in text.split_whitespace() {
            if !out.iter().any(|o| o == c) {
                out.push(c.to_string());
            }
        }
    }
    fn walk(value: &Expr, out: &mut Vec<String>) -> Option<()> {
        match value {
            Expr::StringLiteral(s) => push(s, out),
            Expr::Null | Expr::BoolLiteral(false) => {}
            Expr::ListLiteral(items) => {
                for item in items {
                    walk(item, out)?;
                }
            }
            Expr::MapLiteral(pairs) => {
                for (key, on) in pairs {
                    match on {
                        _ if key == "..." => return None,
                        Expr::BoolLiteral(true) => push(key.trim_matches('"'), out),
                        Expr::BoolLiteral(false) => {}
                        _ => return None,
                    }
                }
            }
            _ => return None,
        }
        Some(())
    }
    let mut out = Vec::new();
    walk(value, &mut out)?;
    Some(out)
}

/// The classes a `class:` value names, from what the build knows of it:
/// a string, a map (each key whose value is truthy) or a list of either.
pub fn static_classes(value: &crate::codegen::static_eval::Static, out: &mut Vec<String>) {
    use crate::codegen::static_eval::Static;
    match value {
        Static::Null | Static::Bool(false) => {}
        Static::List(items) => {
            for item in items {
                static_classes(item, out);
            }
        }
        Static::Map(pairs) => {
            for (key, on) in pairs {
                if on.truthy() {
                    static_classes(&Static::Str(key.trim_matches('"').to_string()), out);
                }
            }
        }
        other => {
            for c in other.to_text().split_whitespace() {
                if !out.iter().any(|o| o == c) {
                    out.push(c.to_string());
                }
            }
        }
    }
}

/// The full class list for an element: its base class plus every modifier class.
pub fn class_list(base_class: &str, modifiers: &[String]) -> Vec<String> {
    let mut classes = Vec::new();
    if !base_class.is_empty() {
        classes.push(base_class.to_string());
    }
    for m in modifiers {
        let c = modifier_to_class(base_class, m);
        if !c.is_empty() && !classes.contains(&c) {
            classes.push(c);
        }
    }
    classes
}

/// The heading level a `Heading`'s modifiers select, defaulting to `h2`.
///
/// Heading level is an outline and SEO concern, not a font size, so it has to
/// reach the tag. The SPA renderer used to leave every heading an `<h2>`.
pub fn heading_tag(modifiers: &[String]) -> &'static str {
    for m in modifiers {
        match m.as_str() {
            "h1" => return "h1",
            "h2" => return "h2",
            "h3" => return "h3",
            "h4" => return "h4",
            "h5" => return "h5",
            "h6" => return "h6",
            _ => {}
        }
    }
    "h2"
}

/// The HTML tag an element renders as, once its modifiers have had their say.
///
/// Two built-ins let a modifier choose the tag rather than a class: a
/// `Heading`'s level (`h1`…`h6`) is part of the document outline, and a
/// `List(ordered)` is an `<ol>`. Every HTML backend used to repeat the heading
/// case inline and none handled the list, so the three could drift apart.
pub fn element_tag(name: &str, modifiers: &[String]) -> &'static str {
    match name {
        "Heading" => heading_tag(modifiers),
        "List" if modifiers.iter().any(|m| m == "ordered") => "ol",
        // A header cell by declaration, for a cell a component renders where
        // the emitter cannot see the enclosing Thead.
        "Tcell" if modifiers.iter().any(|m| m == "header") => "th",
        _ => builtin_to_html(name).0,
    }
}

/// The element a `Host(tag: "…")` is made of.
///
/// An allow-list, not the author's string: the tag is written into the
/// document, and a library asking for a `canvas` or an `svg` is the whole
/// of what this is for. Anything else is a `div`, which is what it was.
pub fn host_tag(named: Option<&str>) -> &'static str {
    let named = named.unwrap_or("div");
    HOST_TAGS
        .iter()
        .find(|t| **t == named)
        .copied()
        .unwrap_or("div")
}

/// The elements a `Host` may be made of. A tag outside it is a `V10`; the
/// registry's description of `tag:` names each (a test holds the two).
pub const HOST_TAGS: &[&str] = &[
    "div", "span", "canvas", "svg", "section", "figure", "pre", "p", "ul", "table",
];

/// The `type=` value an input modifier selects, if it selects one.
///
/// `datetime` is spelled `datetime-local` in HTML. Both static renderers used to
/// omit `datetime` from the match they then tested it inside, so the branch was
/// unreachable and the attribute never appeared.
pub fn input_type(modifier: &str) -> Option<&'static str> {
    Some(match modifier {
        "text" => "text",
        "email" => "email",
        "password" => "password",
        "number" => "number",
        "search" => "search",
        "tel" => "tel",
        "url" => "url",
        "date" => "date",
        "time" => "time",
        "datetime" => "datetime-local",
        "color" => "color",
        "submit" => "submit",
        "reset" => "reset",
        _ => return None,
    })
}

/// The implicit ARIA role a built-in carries, if the tag does not already give
/// it one.
///
/// An `Alert` is a live region: a screen reader has to be told about it when it
/// appears, or the user never learns the thing failed. `role="alert"` is
/// assertive and interrupts, which is right for a failure and wrong for a
/// confirmation — so the severity picks between `alert` and the polite `status`.
/// A landmark's accessible name, where the tag alone leaves it ambiguous.
///
/// `Navbar` and `Breadcrumb` both render `<nav>`, and a page with both gives a
/// screen-reader user two identical "navigation" entries to choose between.
pub fn landmark_label(name: &str) -> Option<&'static str> {
    match name {
        "Navbar" => Some("Main"),
        "Breadcrumb" => Some("Breadcrumb"),
        _ => None,
    }
}

pub fn implicit_role(name: &str, modifiers: &[String]) -> Option<&'static str> {
    match name {
        "Alert" => Some(
            if modifiers
                .iter()
                .any(|m| m == "danger" || m == "error" || m == "warning")
            {
                "alert"
            } else {
                "status"
            },
        ),
        "Spinner" => Some("status"),
        _ => None,
    }
}

/// HTML void elements — no children, no closing tag.
pub const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Whether `tag` is a void element, and so cannot be given children.
pub fn is_void(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag)
}

#[cfg(test)]
mod tests {

    #[test]
    fn the_registry_names_every_tag_a_host_makes() {
        let host = crate::registry::component("Host").unwrap();
        let tag = host.props.iter().find(|p| p.name == "tag").unwrap();
        for t in HOST_TAGS {
            assert!(
                tag.summary.contains(&format!("`{t}`")),
                "`tag:` does not name `{t}`"
            );
        }
    }

    use super::*;

    /// The registry exists so the renderers cannot disagree. Everything below is
    /// a property of the table itself.
    #[test]
    fn heading_level_reaches_the_tag_and_not_a_class() {
        assert_eq!(heading_tag(&["h1".to_string()]), "h1");
        assert_eq!(heading_tag(&["h6".to_string()]), "h6");
        assert_eq!(heading_tag(&[]), "h2", "an unqualified Heading is an h2");
    }

    #[test]
    fn element_tag_folds_the_modifier_chosen_tags() {
        let m = |s: &str| vec![s.to_string()];
        assert_eq!(element_tag("Heading", &m("h3")), "h3");
        assert_eq!(element_tag("List", &m("ordered")), "ol");
        assert_eq!(element_tag("List", &[]), "ul");
        assert_eq!(element_tag("Card", &m("elevated")), "div");
        assert_eq!(
            modifier_to_class("wf-heading", "h1"),
            "",
            "the level must not also become a class the stylesheet never defines"
        );
    }

    #[test]
    fn datetime_maps_to_the_html_spelling() {
        assert_eq!(input_type("datetime"), Some("datetime-local"));
        assert_eq!(input_type("primary"), None);
    }

    #[test]
    fn text_modifiers_are_component_independent() {
        // Put on any component, these style its text, so they name the same class.
        for base in ["wf-text", "wf-btn", "wf-card"] {
            assert_eq!(modifier_to_class(base, "underline"), "wf-text--underline");
            assert_eq!(modifier_to_class(base, "right"), "wf-text--right");
        }
    }

    #[test]
    fn class_list_starts_with_the_base_class_and_drops_empties() {
        let classes = class_list(
            "wf-btn",
            &["primary".into(), "medium".into(), "large".into()],
        );
        assert_eq!(classes, vec!["wf-btn", "wf-btn--primary", "wf-btn--large"]);
    }

    #[test]
    fn a_classless_component_yields_no_class_at_all() {
        let (tag, class) = builtin_to_html("Thead");
        assert_eq!((tag, class), ("thead", ""));
        assert!(
            class_list(class, &[]).is_empty(),
            "no empty class attribute"
        );
    }

    #[test]
    fn the_wrapping_input_components_root_on_their_own_class() {
        for name in ["Slider", "DatePicker", "FileUpload"] {
            let (tag, class) = builtin_to_html(name);
            assert_eq!(tag, "div", "{name} roots on its wrapper");
            assert!(class.starts_with("wf-"), "{name} carries its own class");
        }
    }
}
