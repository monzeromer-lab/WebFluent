use std::collections::HashMap;

pub fn default_tokens() -> HashMap<String, String> {
    let mut t = HashMap::new();

    // Colors
    t.insert("color-primary".into(), "#3B82F6".into());
    t.insert("color-secondary".into(), "#64748B".into());
    t.insert("color-success".into(), "#22C55E".into());
    t.insert("color-danger".into(), "#EF4444".into());
    t.insert("color-warning".into(), "#F59E0B".into());
    t.insert("color-info".into(), "#06B6D4".into());
    t.insert("color-background".into(), "#FFFFFF".into());
    t.insert("color-surface".into(), "#F8FAFC".into());
    t.insert("color-text".into(), "#0F172A".into());
    t.insert("color-text-muted".into(), "#64748B".into());
    t.insert("color-border".into(), "#E2E8F0".into());

    // Typography
    t.insert("font-family".into(), "Inter, system-ui, -apple-system, sans-serif".into());
    t.insert("font-family-mono".into(), "'JetBrains Mono', 'Fira Code', monospace".into());
    t.insert("font-size-xs".into(), "0.75rem".into());
    t.insert("font-size-sm".into(), "0.875rem".into());
    t.insert("font-size-base".into(), "1rem".into());
    t.insert("font-size-lg".into(), "1.25rem".into());
    t.insert("font-size-xl".into(), "1.5rem".into());
    t.insert("font-size-2xl".into(), "2rem".into());
    t.insert("font-size-3xl".into(), "2.5rem".into());
    t.insert("font-weight-normal".into(), "400".into());
    t.insert("font-weight-medium".into(), "500".into());
    t.insert("font-weight-bold".into(), "700".into());
    t.insert("line-height-tight".into(), "1.25".into());
    t.insert("line-height-normal".into(), "1.5".into());
    t.insert("line-height-loose".into(), "1.75".into());

    // Spacing
    t.insert("spacing-xs".into(), "0.25rem".into());
    t.insert("spacing-sm".into(), "0.5rem".into());
    t.insert("spacing-md".into(), "1rem".into());
    t.insert("spacing-lg".into(), "1.5rem".into());
    t.insert("spacing-xl".into(), "2rem".into());
    t.insert("spacing-2xl".into(), "3rem".into());
    t.insert("spacing-3xl".into(), "4rem".into());

    // Border radius
    t.insert("radius-none".into(), "0".into());
    t.insert("radius-sm".into(), "0.25rem".into());
    t.insert("radius-md".into(), "0.5rem".into());
    t.insert("radius-lg".into(), "1rem".into());
    t.insert("radius-xl".into(), "1.5rem".into());
    t.insert("radius-full".into(), "9999px".into());

    // Shadows
    t.insert("shadow-none".into(), "none".into());
    t.insert("shadow-sm".into(), "0 1px 2px 0 rgba(0,0,0,0.05)".into());
    t.insert("shadow-md".into(), "0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)".into());
    t.insert("shadow-lg".into(), "0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)".into());
    t.insert("shadow-xl".into(), "0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1)".into());

    // Breakpoints
    t.insert("screen-sm".into(), "640px".into());
    t.insert("screen-md".into(), "768px".into());
    t.insert("screen-lg".into(), "1024px".into());
    t.insert("screen-xl".into(), "1280px".into());

    // Transitions
    t.insert("transition-fast".into(), "150ms ease".into());
    t.insert("transition-normal".into(), "250ms ease".into());
    t.insert("transition-slow".into(), "350ms ease".into());

    // Animation
    t.insert("animation-duration-fast".into(), "150ms".into());
    t.insert("animation-duration-normal".into(), "300ms".into());
    t.insert("animation-duration-slow".into(), "500ms".into());
    t.insert("animation-easing-default".into(), "cubic-bezier(0.4, 0, 0.2, 1)".into());
    t.insert("animation-easing-bounce".into(), "cubic-bezier(0.68, -0.55, 0.265, 1.55)".into());
    t.insert("animation-easing-spring".into(), "cubic-bezier(0.175, 0.885, 0.32, 1.275)".into());

    t
}

pub fn dark_tokens() -> HashMap<String, String> {
    let mut t = HashMap::new();
    t.insert("color-background".into(), "#0F172A".into());
    t.insert("color-surface".into(), "#1E293B".into());
    t.insert("color-text".into(), "#F1F5F9".into());
    t.insert("color-text-muted".into(), "#94A3B8".into());
    t.insert("color-border".into(), "#334155".into());
    t
}

pub fn minimal_tokens() -> HashMap<String, String> {
    let mut t = HashMap::new();
    t.insert("color-primary".into(), "#000000".into());
    t.insert("color-secondary".into(), "#666666".into());
    t.insert("color-background".into(), "#FFFFFF".into());
    t.insert("color-surface".into(), "#FAFAFA".into());
    t.insert("color-text".into(), "#111111".into());
    t.insert("color-text-muted".into(), "#888888".into());
    t.insert("color-border".into(), "#EEEEEE".into());
    t.insert("radius-sm".into(), "0".into());
    t.insert("radius-md".into(), "0".into());
    t.insert("radius-lg".into(), "0".into());
    t.insert("shadow-sm".into(), "none".into());
    t.insert("shadow-md".into(), "none".into());
    t.insert("shadow-lg".into(), "none".into());
    t
}

pub fn brutalist_tokens() -> HashMap<String, String> {
    let mut t = HashMap::new();
    t.insert("color-primary".into(), "#FF0000".into());
    t.insert("color-secondary".into(), "#000000".into());
    t.insert("color-background".into(), "#FFFFFF".into());
    t.insert("color-surface".into(), "#F0F0F0".into());
    t.insert("color-text".into(), "#000000".into());
    t.insert("color-text-muted".into(), "#333333".into());
    t.insert("color-border".into(), "#000000".into());
    t.insert("font-family".into(), "'Courier New', monospace".into());
    t.insert("radius-sm".into(), "0".into());
    t.insert("radius-md".into(), "0".into());
    t.insert("radius-lg".into(), "0".into());
    t.insert("shadow-sm".into(), "2px 2px 0 #000".into());
    t.insert("shadow-md".into(), "4px 4px 0 #000".into());
    t.insert("shadow-lg".into(), "8px 8px 0 #000".into());
    t
}

pub fn get_theme_tokens(theme_name: &str) -> HashMap<String, String> {
    let mut tokens = default_tokens();
    let overrides = match theme_name {
        "dark" => dark_tokens(),
        "minimal" => minimal_tokens(),
        "brutalist" => brutalist_tokens(),
        _ => HashMap::new(), // "default" or unknown -> no overrides
    };
    for (k, v) in overrides {
        tokens.insert(k, v);
    }
    tokens
}
