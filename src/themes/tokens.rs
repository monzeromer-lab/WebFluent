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
    // System fonts, not webfonts. The baseline used to name Inter and JetBrains
    // Mono while shipping no `@font-face`, no preload and no `font-display` — so
    // every site silently fell back unless the reader happened to have them
    // installed, which also meant it looked different to its author than to its
    // audience. A theme that wants a webfont can name one and load it.
    t.insert(
        "font-family".into(),
        "system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif".into(),
    );
    t.insert(
        "font-family-mono".into(),
        "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace".into(),
    );
    // Fluid from the smallest phone to the widest desktop, via `clamp()`.
    //
    // These were fixed rems, so responsiveness depended entirely on media queries
    // resetting them — and those rules lose to any inline `style { font-size: … }`
    // an author writes, which is how a hand-styled hero stayed desktop-sized on a
    // phone. A scale that carries its own range needs no rule to override, and
    // nothing to lose to. Each clamp holds the old value as its maximum, so a wide
    // viewport renders exactly as before.
    t.insert("font-size-xs".into(), "0.75rem".into());
    t.insert("font-size-sm".into(), "0.875rem".into());
    t.insert("font-size-base".into(), "1rem".into());
    t.insert(
        "font-size-lg".into(),
        "clamp(1.125rem, 1.06rem + 0.31vw, 1.25rem)".into(),
    );
    t.insert(
        "font-size-xl".into(),
        "clamp(1.25rem, 1.13rem + 0.63vw, 1.5rem)".into(),
    );
    t.insert(
        "font-size-2xl".into(),
        "clamp(1.5rem, 1.25rem + 1.25vw, 2rem)".into(),
    );
    t.insert(
        "font-size-3xl".into(),
        "clamp(1.75rem, 1.38rem + 1.88vw, 2.5rem)".into(),
    );
    // The easings a design names, so a project says `easing: $ease-spring`
    // rather than repeating a curve. `linear(…)` is how a spring is
    // written as an easing function: it overshoots and settles, which no
    // cubic Bézier can say.
    t.insert("ease-standard".into(), "cubic-bezier(0.2, 0, 0, 1)".into());
    t.insert("ease-out".into(), "cubic-bezier(0, 0, 0.2, 1)".into());
    t.insert("ease-in".into(), "cubic-bezier(0.4, 0, 1, 1)".into());
    t.insert(
        "ease-spring".into(),
        "linear(0, 0.006, 0.079, 0.234, 0.44, 0.652, 0.828, 0.947, 1.008, 1.025, 1.016, 1)".into(),
    );
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
    // The large steps are the ones that crowd a narrow screen, so they scale too.
    t.insert(
        "spacing-xl".into(),
        "clamp(1.5rem, 1.25rem + 1.25vw, 2rem)".into(),
    );
    t.insert(
        "spacing-2xl".into(),
        "clamp(2rem, 1.5rem + 2.5vw, 3rem)".into(),
    );
    t.insert(
        "spacing-3xl".into(),
        "clamp(2.5rem, 1.75rem + 3.75vw, 4rem)".into(),
    );

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
    t.insert(
        "shadow-md".into(),
        "0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)".into(),
    );
    t.insert(
        "shadow-lg".into(),
        "0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)".into(),
    );
    t.insert(
        "shadow-xl".into(),
        "0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1)".into(),
    );

    // The height a sticky header occupies, which a sidebar has to sit below.
    // This used to be a 56px constant in two stylesheets, so a navbar that wrapped
    // to two rows ended up overlapping the sidebar under it.
    t.insert("wf-header-height".into(), "56px".into());

    // Breakpoints
    t.insert("screen-sm".into(), "640px".into());
    t.insert("screen-md".into(), "768px".into());
    t.insert("screen-lg".into(), "1024px".into());
    t.insert("screen-xl".into(), "1280px".into());

    // A console stays dark in both themes: code blocks, terminals and the
    // syntax colours of a `Code(…, language:)` or a Markdown fence.
    t.insert("term-bg".into(), "#1B1F22".into());
    t.insert("term-ink".into(), "#E6E9EC".into());
    t.insert("term-dim".into(), "#959BA1".into());
    t.insert("syntax-keyword".into(), "#8FBDE7".into());
    t.insert("syntax-string".into(), "#7FD6A8".into());
    t.insert("syntax-number".into(), "#E0B070".into());
    t.insert("syntax-function".into(), "#6FD3E6".into());
    t.insert("syntax-punct".into(), "#B3BAC0".into());
    t.insert("syntax-comment".into(), "#959BA1".into());

    // Transitions
    t.insert("transition-fast".into(), "150ms ease".into());
    t.insert("transition-normal".into(), "250ms ease".into());
    t.insert("transition-slow".into(), "350ms ease".into());

    // Animation
    t.insert("animation-duration-fast".into(), "150ms".into());
    t.insert("animation-duration-normal".into(), "300ms".into());
    t.insert("animation-duration-slow".into(), "500ms".into());
    t.insert(
        "animation-easing-default".into(),
        "cubic-bezier(0.4, 0, 0.2, 1)".into(),
    );
    t.insert(
        "animation-easing-bounce".into(),
        "cubic-bezier(0.68, -0.55, 0.265, 1.55)".into(),
    );
    t.insert(
        "animation-easing-spring".into(),
        "cubic-bezier(0.175, 0.885, 0.32, 1.275)".into(),
    );

    t
}
