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
        "Icon" => ("i", "wf-icon"),
        "Carousel" => ("div", "wf-carousel"),

        // ─── Typography ──────────────────────────────────
        "Text" => ("p", "wf-text"),
        // The level modifier picks the real tag; see [`heading_tag`].
        "Heading" => ("h2", "wf-heading"),
        "Code" => ("code", "wf-code"),
        "Blockquote" => ("blockquote", "wf-blockquote"),

        // ─── Document structure ──────────────────────────
        // These come from the PDF vocabulary but are ordinary HTML on the web,
        // and `App { … Footer }` is the documented site-shell idiom. The SPA and
        // SSG renderers used to drop them to an unclassed `<div>`.
        "Section" => ("section", "wf-section"),
        "Paragraph" => ("p", "wf-text"),
        "Document" => ("div", "wf-document"),
        "Header" => ("header", "wf-header"),
        "Footer" => ("footer", "wf-footer"),

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

        // ─── Shape ───────────────────────────────────────
        "rounded" => format!("{}--rounded", base_class),
        "pill" => format!("{}--pill", base_class),
        "square" => format!("{}--square", base_class),

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
        | "datetime" | "color" | "submit" | "reset" | "required" | "controls" | "autoplay" => {
            String::new()
        }

        // ─── Animation ───────────────────────────────────
        // Pure CSS keyframes, so they apply in static output too. The template
        // renderer used to drop them on the grounds that it emits no JS.
        "fadeIn" | "fadeOut" | "slideUp" | "slideDown" | "slideLeft" | "slideRight" | "scaleIn"
        | "scaleOut" | "bounce" | "shake" | "pulse" | "spin" => format!("wf-animate-{}", modifier),
        "fast" => "wf-animate--fast".to_string(),
        "slow" => "wf-animate--slow".to_string(),

        _ => String::new(),
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
    use super::*;

    /// The registry exists so the renderers cannot disagree. Everything below is
    /// a property of the table itself.
    #[test]
    fn heading_level_reaches_the_tag_and_not_a_class() {
        assert_eq!(heading_tag(&["h1".to_string()]), "h1");
        assert_eq!(heading_tag(&["h6".to_string()]), "h6");
        assert_eq!(heading_tag(&[]), "h2", "an unqualified Heading is an h2");
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
