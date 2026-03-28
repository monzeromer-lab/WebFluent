use std::collections::HashMap;
use crate::themes;

/// Generate the CSS stylesheet with design tokens and component styles.
///
/// Merges the theme's base tokens with any custom overrides, then appends
/// all component CSS rules and animation keyframes.
pub fn generate_css(theme_name: &str, custom_tokens: &HashMap<String, String>) -> String {
    let mut tokens = themes::get_theme_tokens(theme_name);

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
    let component_styles = themes::component_css();

    format!("{}\n{}", root_block, component_styles)
}
