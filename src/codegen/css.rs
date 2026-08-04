use crate::themes::{self, BuiltinCss};
use std::collections::HashMap;

/// Generate the CSS stylesheet from resolved design tokens, with the full
/// built-in design underneath.
///
/// `tokens` comes from [`themes::resolve_tokens`], which has already layered the
/// baseline, the project's `Theme` declaration and any config overrides. This is
/// [`generate_css_with`] at [`BuiltinCss::Full`].
pub fn generate_css(tokens: &HashMap<String, String>) -> String {
    generate_css_with(tokens, BuiltinCss::Full)
}

/// [`generate_css`] with control over how much built-in design is emitted.
///
/// The `:root` token block is identical in both modes, so `var(--spacing-md)` and
/// a bare-keyword `style { background: surface }` resolve the same either way.
/// What changes is the sheet underneath: [`BuiltinCss::Structural`] drops the
/// engine's baseline look, leaving layout and mechanics.
///
/// The author's own tokens apply in both modes. They used to be discarded under
/// `Structural` on the reasoning that a *built-in* palette had no baseline left
/// to restyle — true of the palettes the engine shipped, and exactly wrong for a
/// theme the author wrote, which is the whole design in that mode.
pub fn generate_css_with(tokens: &HashMap<String, String>, builtin: BuiltinCss) -> String {
    // Generate :root block
    let mut root_vars: Vec<String> = tokens
        .iter()
        .map(|(k, v)| format!("  --{}: {};", k, v))
        .collect();
    root_vars.sort();

    let root_block = format!(":root {{\n{}\n}}", root_vars.join("\n"));
    let component_styles = match builtin {
        BuiltinCss::Full => themes::component_css(),
        BuiltinCss::Structural => themes::structural_css(),
    };

    format!("{}\n{}", root_block, component_styles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> HashMap<String, String> {
        themes::tokens::default_tokens()
    }

    fn full() -> String {
        generate_css_with(&baseline(), BuiltinCss::Full)
    }

    fn structural() -> String {
        generate_css_with(&baseline(), BuiltinCss::Structural)
    }

    #[test]
    fn default_mode_is_the_historical_full_sheet() {
        assert_eq!(generate_css(&baseline()), full());
        assert_eq!(BuiltinCss::default(), BuiltinCss::Full);
    }

    #[test]
    fn both_modes_publish_the_same_tokens() {
        // The palette pipeline (and every `var(--…)` in .wf style blocks) reads
        // :root, so removing the baseline design must not remove a single token.
        for name in themes::tokens::default_tokens().keys() {
            let decl = format!("--{name}:");
            assert!(structural().contains(&decl), "missing {name}");
        }
    }

    #[test]
    fn structural_keeps_layout_and_mechanics() {
        let css = structural();
        for rule in [
            ".wf-row { display: flex",
            ".wf-col--6 { flex: 0 0 50%",
            ".wf-stack { display: flex; flex-direction: column",
            // Modal visibility is now the browser's job — `<dialog>` hides itself
            // while closed — so what the sheet still owes is the backdrop.
            ".wf-modal::backdrop",
            ".wf-tab-page.active { display: block; }",
            "@keyframes wf-fadeIn",
            // Reduced motion and a reachable skip link are mechanics, not design.
            "@media (prefers-reduced-motion: reduce)",
            ".wf-skip-link:focus",
        ] {
            assert!(css.contains(rule), "structural sheet dropped `{rule}`");
        }
    }

    #[test]
    fn structural_drops_the_baseline_look() {
        let css = structural();
        // The button's chrome, the card's border, the navbar's frosted fill and the
        // heading type scale are the design the engine used to assume.
        for gone in [
            "backdrop-filter",
            "h1.wf-heading",
            ".wf-btn:hover",
            ".wf-card { background:",
        ] {
            assert!(!css.contains(gone), "structural sheet still emits `{gone}`");
        }
        // …but a variant the author explicitly asked for keeps its colour.
        assert!(css.contains(".wf-btn--primary { background: var(--color-primary)"));
    }

    #[test]
    fn structural_leaves_focus_and_native_affordances_alone() {
        let css = structural();
        assert!(!css.contains("outline: none"));
        assert!(!css.contains("appearance: none"));
    }

    /// A theme the author wrote is their whole design in structural mode, so
    /// unlike the built-in palettes this replaced, it must survive there.
    #[test]
    fn authored_tokens_reach_both_modes() {
        let mut tokens = baseline();
        tokens.insert("color-background".to_string(), "#0F172A".to_string());
        tokens.insert("color-primary".to_string(), "#ff00ff".to_string());

        for mode in [BuiltinCss::Full, BuiltinCss::Structural] {
            let css = generate_css_with(&tokens, mode);
            assert!(css.contains("--color-background: #0F172A"), "{mode:?}");
            assert!(css.contains("--color-primary: #ff00ff"), "{mode:?}");
        }
    }
}
