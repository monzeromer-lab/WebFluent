//! What a `style { }` block says that CSS will not take: a property no
//! browser knows (`colr:`), and a design token the theme does not declare
//! (`$brnad`), which compiles to a `var(--…)` that is never set.

use crate::diagnostics::Diagnostic;
use crate::linter::vocabulary::levenshtein;
use crate::parser::ast::*;
use std::collections::HashMap;

/// Every style finding in the lowered `program`. `tokens` is the theme as
/// this build resolves it; without one, tokens are not checked.
pub fn lint_styles(
    program: &Program,
    tokens: Option<&HashMap<String, String>>,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        let body = match decl {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            _ => continue,
        };
        let file = file_of(index);
        walk(body, &file, tokens, &mut out);
    }
    out
}

fn walk(
    stmts: &[Statement],
    file: &str,
    tokens: Option<&HashMap<String, String>>,
    out: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            if let Some(style) = &ui.style_block {
                let all = style
                    .properties
                    .iter()
                    .chain(style.media_queries.iter().flat_map(|m| m.properties.iter()))
                    .chain(style.pseudo_blocks.iter().flat_map(|p| p.properties.iter()));
                for prop in all {
                    property(prop, file, tokens, out);
                }
            }
            walk(&ui.children, file, tokens, out);
            for fill in &ui.slot_fills {
                walk(&fill.body, file, tokens, out);
            }
        }
        for body in stmt.kind.bodies() {
            walk(body, file, tokens, out);
        }
    }
}

fn property(
    prop: &StyleProperty,
    file: &str,
    tokens: Option<&HashMap<String, String>>,
    out: &mut Vec<Diagnostic>,
) {
    let (line, col) = (
        prop.span.line.max(1) as usize,
        prop.span.col.max(1) as usize,
    );
    let name = crate::codegen::style_tokens::canonical_style_prop(&prop.name);
    // V05: a property no browser knows does nothing.
    if !known_property(&name) {
        let near = CSS_PROPERTIES
            .iter()
            .map(|p| (levenshtein(&name, p), *p))
            .filter(|(d, _)| *d <= 2)
            .min()
            .map(|(_, p)| format!("Did you mean `{p}`?"))
            .unwrap_or_else(|| "A custom property is written `--name`".to_string());
        out.push(
            Diagnostic::coded(
                "V05",
                format!(
                    "`{}` is not a CSS property, so the browser ignores it",
                    prop.name
                ),
                file,
                line,
                col,
            )
            .with_hint(near),
        );
    }
    // V06: a token the theme does not declare.
    let Some(tokens) = tokens else {
        return;
    };
    let mut used = Vec::new();
    collect_tokens(&prop.value, &mut used);
    for token in used {
        if tokens.contains_key(&token) {
            continue;
        }
        let mut names: Vec<&String> = tokens.keys().collect();
        names.sort();
        let near = names
            .iter()
            .map(|n| (levenshtein(&token, n), *n))
            .filter(|(d, _)| *d <= 3)
            .min()
            .map(|(_, n)| format!("Did you mean `${n}`?"))
            .unwrap_or_else(|| {
                "Declare it in your `theme { … }`, or use one the theme has".to_string()
            });
        out.push(
            Diagnostic::coded(
                "V06",
                format!("`${token}` is no token the theme declares, so it is never set"),
                file,
                line,
                col,
            )
            .with_hint(near),
        );
    }
}

fn collect_tokens(e: &Expr, out: &mut Vec<String>) {
    if let Expr::Token(name) = e {
        out.push(name.clone());
    }
    for child in e.children() {
        collect_tokens(child, out);
    }
}

/// Whether CSS has the property: one of the standard ones, a custom
/// property, or one with a vendor's prefix.
fn known_property(name: &str) -> bool {
    name.starts_with("--")
        || name.starts_with("-webkit-")
        || name.starts_with("-moz-")
        || name.starts_with("-ms-")
        || CSS_PROPERTIES.contains(&name)
}

/// The CSS properties a browser knows.
const CSS_PROPERTIES: &[&str] = &[
    "accent-color",
    "align-content",
    "align-items",
    "align-self",
    "all",
    "animation",
    "animation-delay",
    "animation-direction",
    "animation-duration",
    "animation-fill-mode",
    "animation-iteration-count",
    "animation-name",
    "animation-play-state",
    "animation-timing-function",
    "animation-composition",
    "animation-timeline",
    "appearance",
    "aspect-ratio",
    "backdrop-filter",
    "backface-visibility",
    "background",
    "background-attachment",
    "background-blend-mode",
    "background-clip",
    "background-color",
    "background-image",
    "background-origin",
    "background-position",
    "background-position-x",
    "background-position-y",
    "background-repeat",
    "background-size",
    "block-size",
    "border",
    "border-block",
    "border-block-color",
    "border-block-end",
    "border-block-end-color",
    "border-block-end-style",
    "border-block-end-width",
    "border-block-start",
    "border-block-start-color",
    "border-block-start-style",
    "border-block-start-width",
    "border-block-style",
    "border-block-width",
    "border-bottom",
    "border-bottom-color",
    "border-bottom-left-radius",
    "border-bottom-right-radius",
    "border-bottom-style",
    "border-bottom-width",
    "border-collapse",
    "border-color",
    "border-end-end-radius",
    "border-end-start-radius",
    "border-image",
    "border-image-outset",
    "border-image-repeat",
    "border-image-slice",
    "border-image-source",
    "border-image-width",
    "border-inline",
    "border-inline-color",
    "border-inline-end",
    "border-inline-end-color",
    "border-inline-end-style",
    "border-inline-end-width",
    "border-inline-start",
    "border-inline-start-color",
    "border-inline-start-style",
    "border-inline-start-width",
    "border-inline-style",
    "border-inline-width",
    "border-left",
    "border-left-color",
    "border-left-style",
    "border-left-width",
    "border-radius",
    "border-right",
    "border-right-color",
    "border-right-style",
    "border-right-width",
    "border-spacing",
    "border-start-end-radius",
    "border-start-start-radius",
    "border-style",
    "border-top",
    "border-top-color",
    "border-top-left-radius",
    "border-top-right-radius",
    "border-top-style",
    "border-top-width",
    "border-width",
    "bottom",
    "box-decoration-break",
    "box-shadow",
    "box-sizing",
    "break-after",
    "break-before",
    "break-inside",
    "caption-side",
    "caret-color",
    "clear",
    "clip",
    "clip-path",
    "color",
    "color-scheme",
    "column-count",
    "column-fill",
    "column-gap",
    "column-rule",
    "column-rule-color",
    "column-rule-style",
    "column-rule-width",
    "column-span",
    "column-width",
    "columns",
    "contain",
    "contain-intrinsic-size",
    "contain-intrinsic-width",
    "contain-intrinsic-height",
    "container",
    "container-name",
    "container-type",
    "content",
    "content-visibility",
    "counter-increment",
    "counter-reset",
    "counter-set",
    "cursor",
    "direction",
    "display",
    "empty-cells",
    "fill",
    "fill-opacity",
    "fill-rule",
    "filter",
    "flex",
    "flex-basis",
    "flex-direction",
    "flex-flow",
    "flex-grow",
    "flex-shrink",
    "flex-wrap",
    "float",
    "font",
    "font-family",
    "font-feature-settings",
    "font-kerning",
    "font-optical-sizing",
    "font-size",
    "font-size-adjust",
    "font-stretch",
    "font-style",
    "font-synthesis",
    "font-variant",
    "font-variant-caps",
    "font-variant-ligatures",
    "font-variant-numeric",
    "font-variation-settings",
    "font-weight",
    "gap",
    "grid",
    "grid-area",
    "grid-auto-columns",
    "grid-auto-flow",
    "grid-auto-rows",
    "grid-column",
    "grid-column-end",
    "grid-column-start",
    "grid-row",
    "grid-row-end",
    "grid-row-start",
    "grid-template",
    "grid-template-areas",
    "grid-template-columns",
    "grid-template-rows",
    "hanging-punctuation",
    "height",
    "hyphens",
    "image-rendering",
    "inline-size",
    "inset",
    "inset-block",
    "inset-block-end",
    "inset-block-start",
    "inset-inline",
    "inset-inline-end",
    "inset-inline-start",
    "isolation",
    "justify-content",
    "justify-items",
    "justify-self",
    "left",
    "letter-spacing",
    "line-break",
    "line-clamp",
    "line-height",
    "list-style",
    "list-style-image",
    "list-style-position",
    "list-style-type",
    "margin",
    "margin-block",
    "margin-block-end",
    "margin-block-start",
    "margin-bottom",
    "margin-inline",
    "margin-inline-end",
    "margin-inline-start",
    "margin-left",
    "margin-right",
    "margin-top",
    "marker",
    "mask",
    "mask-image",
    "mask-mode",
    "mask-position",
    "mask-repeat",
    "mask-size",
    "max-block-size",
    "max-height",
    "max-inline-size",
    "max-width",
    "min-block-size",
    "min-height",
    "min-inline-size",
    "min-width",
    "mix-blend-mode",
    "object-fit",
    "object-position",
    "offset",
    "opacity",
    "order",
    "orphans",
    "outline",
    "outline-color",
    "outline-offset",
    "outline-style",
    "outline-width",
    "overflow",
    "overflow-anchor",
    "overflow-wrap",
    "overflow-x",
    "overflow-y",
    "overflow-clip-margin",
    "overscroll-behavior",
    "overscroll-behavior-x",
    "overscroll-behavior-y",
    "padding",
    "padding-block",
    "padding-block-end",
    "padding-block-start",
    "padding-bottom",
    "padding-inline",
    "padding-inline-end",
    "padding-inline-start",
    "padding-left",
    "padding-right",
    "padding-top",
    "page-break-after",
    "page-break-before",
    "page-break-inside",
    "paint-order",
    "perspective",
    "perspective-origin",
    "place-content",
    "place-items",
    "place-self",
    "pointer-events",
    "position",
    "print-color-adjust",
    "quotes",
    "resize",
    "right",
    "rotate",
    "row-gap",
    "scale",
    "scroll-behavior",
    "scroll-margin",
    "scroll-margin-block",
    "scroll-margin-bottom",
    "scroll-margin-inline",
    "scroll-margin-left",
    "scroll-margin-right",
    "scroll-margin-top",
    "scroll-padding",
    "scroll-padding-block",
    "scroll-padding-bottom",
    "scroll-padding-inline",
    "scroll-padding-left",
    "scroll-padding-right",
    "scroll-padding-top",
    "scroll-snap-align",
    "scroll-snap-stop",
    "scroll-snap-type",
    "scrollbar-color",
    "scrollbar-gutter",
    "scrollbar-width",
    "shape-outside",
    "stroke",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-opacity",
    "stroke-width",
    "tab-size",
    "table-layout",
    "text-align",
    "text-align-last",
    "text-decoration",
    "text-decoration-color",
    "text-decoration-line",
    "text-decoration-style",
    "text-decoration-thickness",
    "text-emphasis",
    "text-indent",
    "text-justify",
    "text-orientation",
    "text-overflow",
    "text-rendering",
    "text-shadow",
    "text-transform",
    "text-underline-offset",
    "text-underline-position",
    "text-wrap",
    "top",
    "touch-action",
    "transform",
    "transform-box",
    "transform-origin",
    "transform-style",
    "transition",
    "transition-behavior",
    "transition-delay",
    "transition-duration",
    "transition-property",
    "transition-timing-function",
    "translate",
    "unicode-bidi",
    "user-select",
    "vertical-align",
    "view-transition-name",
    "visibility",
    "white-space",
    "white-space-collapse",
    "widows",
    "width",
    "will-change",
    "word-break",
    "word-spacing",
    "word-wrap",
    "writing-mode",
    "z-index",
    "zoom",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_property_is_known_custom_or_prefixed() {
        assert!(known_property("background-color"));
        assert!(known_property("--hover-bg"));
        assert!(known_property("-webkit-line-clamp"));
        assert!(!known_property("colr"));
    }
}
