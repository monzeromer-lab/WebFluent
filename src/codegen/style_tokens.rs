//! Design-token resolution for `style { }` blocks in the **web** backends (js + ssg).
//!
//! The language card documents that a style value may be a *bare token keyword* —
//! `font-size: xl`, `padding: md`, `background: surface`, `radius: md` — referencing
//! a design token "by suffix". Those tokens are emitted into the stylesheet as CSS
//! custom properties (`--font-size-xl`, `--spacing-md`, `--color-surface`, …; see
//! `themes::tokens`). But the web codegen has no token step: a bare identifier falls
//! through to the reactive-signal path (`_xl()`), which is an *undefined variable* at
//! runtime and white-screens the page (`ReferenceError: … _xl`).
//!
//! This module closes that gap. Given a style property and its value, if the value is
//! a bare identifier naming a known token for that property's group, it resolves to a
//! `var(--<group>-<suffix>)` reference — the value the card always promised. Anything
//! that isn't a recognized token is left untouched, so quoted CSS, numbers, and
//! genuine reactive-state identifiers (a real feature: `font-size: someState`) keep
//! their existing behavior. Resolution is therefore zero-false-positive: only the
//! documented token keywords are intercepted.

use crate::parser::ast::Expr;

/// Canonicalize a style property name, applying the card's aliases so the emitted CSS
/// property is real: `radius` → `border-radius`, `shadow` → `box-shadow`. Everything
/// else is returned unchanged.
pub fn canonical_style_prop(name: &str) -> String {
    match name {
        "radius" => "border-radius".to_string(),
        "shadow" => "box-shadow".to_string(),
        other => other.to_string(),
    }
}

/// If `value` is a bare token keyword valid for the (already-canonicalized) property
/// `css_prop`, return the CSS `var(--…)` reference to emit; otherwise `None` (callers
/// fall back to their normal handling).
pub fn resolve_style_token(css_prop: &str, value: &Expr) -> Option<String> {
    let Expr::Identifier(name) = value else {
        return None;
    };
    let group = token_group(css_prop)?;
    group_suffixes(group)
        .contains(&name.as_str())
        .then(|| format!("var(--{group}-{name})"))
}

/// The design-token group a style property draws from, or `None` if the property
/// takes no token. Mirrors the card's "+ padding background color font-size border
/// width" note and the logical-CSS spacing properties.
fn token_group(css_prop: &str) -> Option<&'static str> {
    let group = match css_prop {
        "font-size" => "font-size",
        "line-height" => "line-height",
        "transition" => "transition",
        "border-radius" => "radius",
        "box-shadow" => "shadow",
        "color" | "background" | "background-color" | "border-color" | "fill" | "stroke" => "color",
        // Spacing: every box-space property (physical + logical), gaps, and sizes.
        p if is_spacing_prop(p) => "spacing",
        _ => return None,
    };
    Some(group)
}

/// Whether a property takes a spacing token (margins, paddings — physical and logical —
/// gaps, and box sizes).
fn is_spacing_prop(p: &str) -> bool {
    matches!(
        p,
        "padding" | "margin" | "gap" | "row-gap" | "column-gap" | "width" | "height"
    ) || p.starts_with("padding-")
        || p.starts_with("margin-")
        || p.starts_with("inset")
        || matches!(p, "top" | "right" | "bottom" | "left")
        || matches!(p, "min-width" | "max-width" | "min-height" | "max-height")
}

/// The valid token suffixes for a group. Kept in lockstep with `themes::tokens`.
fn group_suffixes(group: &str) -> &'static [&'static str] {
    match group {
        "font-size" => &["xs", "sm", "base", "lg", "xl", "2xl", "3xl"],
        "spacing" => &["xs", "sm", "md", "lg", "xl", "2xl", "3xl"],
        // `text-muted` can't be a bare identifier (the `-`), so it's not listed.
        "color" => &[
            "primary", "secondary", "success", "danger", "warning", "info", "background", "surface",
            "text", "border",
        ],
        "radius" => &["none", "sm", "md", "lg", "xl", "full"],
        "shadow" => &["none", "sm", "md", "lg", "xl"],
        "line-height" => &["tight", "normal", "loose"],
        "transition" => &["fast", "normal", "slow"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident(s: &str) -> Expr {
        Expr::Identifier(s.to_string())
    }

    #[test]
    fn resolves_font_size_token() {
        // The exact case from the field report: `font-size: xl` must become a token
        // reference, not a `_xl()` signal call.
        assert_eq!(resolve_style_token("font-size", &ident("xl")).as_deref(), Some("var(--font-size-xl)"));
        assert_eq!(resolve_style_token("font-size", &ident("base")).as_deref(), Some("var(--font-size-base)"));
    }

    #[test]
    fn resolves_spacing_color_radius_shadow() {
        assert_eq!(resolve_style_token("padding", &ident("md")).as_deref(), Some("var(--spacing-md)"));
        assert_eq!(resolve_style_token("padding-inline-start", &ident("lg")).as_deref(), Some("var(--spacing-lg)"));
        assert_eq!(resolve_style_token("background", &ident("surface")).as_deref(), Some("var(--color-surface)"));
        assert_eq!(resolve_style_token("color", &ident("primary")).as_deref(), Some("var(--color-primary)"));
        assert_eq!(resolve_style_token("border-radius", &ident("md")).as_deref(), Some("var(--radius-md)"));
        assert_eq!(resolve_style_token("box-shadow", &ident("lg")).as_deref(), Some("var(--shadow-lg)"));
    }

    #[test]
    fn aliases_map_to_real_css_properties() {
        assert_eq!(canonical_style_prop("radius"), "border-radius");
        assert_eq!(canonical_style_prop("shadow"), "box-shadow");
        assert_eq!(canonical_style_prop("font-size"), "font-size");
    }

    #[test]
    fn unknown_identifier_is_left_for_the_reactive_path() {
        // A non-token identifier (e.g. a declared reactive `state`) must NOT resolve —
        // callers fall through to `_name()`, preserving reactive styles.
        assert_eq!(resolve_style_token("font-size", &ident("mySize")), None);
        assert_eq!(resolve_style_token("color", &ident("brandBlue")), None);
    }

    #[test]
    fn non_token_properties_and_values_are_ignored() {
        // A property with no token group.
        assert_eq!(resolve_style_token("z-index", &ident("xl")), None);
        // A quoted/number value is never a token (only bare identifiers are).
        assert_eq!(resolve_style_token("font-size", &Expr::StringLiteral("1.25rem".into())), None);
        assert_eq!(resolve_style_token("padding", &Expr::NumberLiteral(16.0)), None);
    }
}
