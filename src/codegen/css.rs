use crate::themes::{self, BuiltinCss};
use std::collections::HashMap;

/// Generate the CSS stylesheet with design tokens and the full built-in design.
///
/// Merges the theme's base tokens with any custom overrides, then appends all
/// component CSS rules and animation keyframes. This is [`generate_css_with`] at
/// [`BuiltinCss::Full`] — the historical behaviour, kept for callers that have no
/// opinion (`wf build`, the template engine).
pub fn generate_css(theme_name: &str, custom_tokens: &HashMap<String, String>) -> String {
    generate_css_with(theme_name, custom_tokens, BuiltinCss::Full)
}

/// [`generate_css`] with control over how much built-in design is emitted.
///
/// The `:root` token block is identical in both modes, so `var(--spacing-md)` and a
/// bare-keyword `style { background: surface }` resolve the same either way. What
/// changes is the sheet underneath it: [`BuiltinCss::Structural`] drops the baseline
/// look and the named theme's palette overrides, leaving layout and mechanics.
pub fn generate_css_with(
    theme_name: &str,
    custom_tokens: &HashMap<String, String>,
    builtin: BuiltinCss,
) -> String {
    // A named theme (`dark`, `brutalist`) exists to restyle the baseline sheet. With
    // no baseline to restyle it would only impose a palette the author didn't ask
    // for, so structural builds start from the neutral token set.
    let mut tokens = match builtin {
        BuiltinCss::Full => themes::get_theme_tokens(theme_name),
        BuiltinCss::Structural => themes::tokens::default_tokens(),
    };

    // Apply custom overrides
    for (k, v) in custom_tokens {
        tokens.insert(k.clone(), v.clone());
    }

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

    fn full(theme: &str) -> String {
        generate_css_with(theme, &HashMap::new(), BuiltinCss::Full)
    }

    fn structural(theme: &str) -> String {
        generate_css_with(theme, &HashMap::new(), BuiltinCss::Structural)
    }

    #[test]
    fn default_mode_is_the_historical_full_sheet() {
        assert_eq!(generate_css("default", &HashMap::new()), full("default"));
        assert_eq!(BuiltinCss::default(), BuiltinCss::Full);
    }

    #[test]
    fn both_modes_publish_the_same_tokens() {
        // The palette pipeline (and every `var(--…)` in .wf style blocks) reads
        // :root, so removing the baseline design must not remove a single token.
        for name in themes::tokens::default_tokens().keys() {
            let decl = format!("--{name}:");
            assert!(structural("default").contains(&decl), "missing {name}");
        }
    }

    #[test]
    fn structural_keeps_layout_and_mechanics() {
        let css = structural("default");
        for rule in [
            ".wf-row { display: flex",
            ".wf-col--6 { flex: 0 0 50%",
            ".wf-stack { display: flex; flex-direction: column",
            ".wf-modal.open { display: flex; }",
            ".wf-tab-page.active { display: block; }",
            "@keyframes wf-fadeIn",
        ] {
            assert!(css.contains(rule), "structural sheet dropped `{rule}`");
        }
    }

    #[test]
    fn structural_drops_the_baseline_look() {
        let css = structural("default");
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
        let css = structural("default");
        assert!(!css.contains("outline: none"));
        assert!(!css.contains("appearance: none"));
    }

    #[test]
    fn named_themes_only_repaint_the_full_sheet() {
        assert!(full("dark").contains("--color-background: #0F172A"));
        // Structural has no baseline for `dark` to restyle, so it stays neutral.
        assert!(structural("dark").contains("--color-background: #FFFFFF"));
    }

    #[test]
    fn custom_tokens_win_in_both_modes() {
        let mut custom = HashMap::new();
        custom.insert("color-primary".to_string(), "#ff00ff".to_string());
        assert!(generate_css_with("dark", &custom, BuiltinCss::Full).contains("--color-primary: #ff00ff"));
        assert!(
            generate_css_with("dark", &custom, BuiltinCss::Structural)
                .contains("--color-primary: #ff00ff")
        );
    }
}
