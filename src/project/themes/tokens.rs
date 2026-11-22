use std::collections::HashMap;

pub fn default_tokens() -> HashMap<String, String> {
    let mut t = HashMap::new();

    // Colors
    t.insert("color-primary".into(), "#2563EB".into());
    t.insert("color-secondary".into(), "#64748B".into());
    t.insert("color-success".into(), "#15803D".into());
    t.insert("color-danger".into(), "#DC2626".into());
    t.insert("color-warning".into(), "#F59E0B".into());
    t.insert("color-info".into(), "#0E7490".into());
    t.insert("color-background".into(), "#FFFFFF".into());
    t.insert("color-surface".into(), "#F8FAFC".into());
    t.insert("color-text".into(), "#0F172A".into());
    t.insert("color-text-muted".into(), "#64748B".into());
    t.insert("color-border".into(), "#E2E8F0".into());
    // What is written on a tone: a primary button's label, a success
    // badge's, a selected day's. A theme whose primary is light names a dark
    // one, and every built-in drawn on that tone follows.
    t.insert("color-on-primary".into(), "#FFFFFF".into());
    t.insert("color-on-secondary".into(), "#FFFFFF".into());
    t.insert("color-on-success".into(), "#FFFFFF".into());
    t.insert("color-on-danger".into(), "#FFFFFF".into());
    t.insert("color-on-warning".into(), "#000000".into());
    t.insert("color-on-info".into(), "#FFFFFF".into());
    // What dims the page behind a modal, a dialog, a sheet and a drawer.
    t.insert("color-backdrop".into(), "rgb(0 0 0 / 0.5)".into());

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

    // Stacking: one scale, so a page's own sticky header, drawer or
    // overlay sits where it should against the built-ins'. Modals,
    // popovers, menus and tooltips are drawn in the browser's top layer
    // and need none.
    t.insert("z-sticky".into(), "100".into());
    t.insert("z-drawer".into(), "200".into());
    t.insert("z-dropdown".into(), "1000".into());
    t.insert("z-modal".into(), "1300".into());
    t.insert("z-toast".into(), "1400".into());
    t.insert("z-tooltip".into(), "1500".into());
    t.insert("z-skip-link".into(), "10000".into());

    // Targets: the least a control reaches, for a pointer and for a finger.
    t.insert("target-min".into(), "24px".into());
    t.insert("target-min-touch".into(), "44px".into());

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

/// What a baseline token is for, in one line: the docs' token tables, `wf
/// theme --json` and the editor's hover read it from here. `None` for a
/// name that is not a baseline token (a theme's own).
pub fn describe(name: &str) -> Option<&'static str> {
    Some(match name {
        "color-primary" => {
            "The main tone: a primary button, a link, a focus ring, what is chosen or current"
        }
        "color-secondary" => "A quieter tone, for a secondary button or badge",
        "color-success" => "Done, passing, saved: a success badge, alert or toast",
        "color-danger" => "Failed, destructive, invalid: a danger button, an error message",
        "color-warning" => "Needs attention: a warning badge or alert, a rating's stars",
        "color-info" => "For information: an info badge or alert, a note callout",
        "color-background" => "The page behind everything",
        "color-surface" => "What sits on the page: a card, a menu, a popover, a table header",
        "color-text" => "The text",
        "color-text-muted" => "Text that matters less: a hint, a caption, `.muted`",
        "color-border" => "A line between things: a field's edge, a divider, a card's border",
        "color-on-primary" => "Text and icons written on `color-primary`; A13 checks the pair",
        "color-on-secondary" => "Text and icons written on `color-secondary`; A13 checks the pair",
        "color-on-success" => "Text and icons written on `color-success`; A13 checks the pair",
        "color-on-danger" => "Text and icons written on `color-danger`; A13 checks the pair",
        "color-on-warning" => "Text and icons written on `color-warning`; A13 checks the pair",
        "color-on-info" => "Text and icons written on `color-info`; A13 checks the pair",
        "color-backdrop" => "What dims the page behind a modal, a dialog, a sheet and a drawer",
        "font-family" => "The text's typeface",
        "font-family-mono" => "Code's typeface: `Code`, `CodeBlock`, `Kbd`",
        "font-size-xs" => "The smallest text: a badge, a caption",
        "font-size-sm" => "Small text: `.sm`, a hint, a table header",
        "font-size-base" => "Body text",
        "font-size-lg" => "Large text: `.lg`, a lede",
        "font-size-xl" => "A small heading (`h4`)",
        "font-size-2xl" => "A heading (`h3`, `h2`), growing with the screen",
        "font-size-3xl" => "The page's heading (`h1`), growing with the screen",
        "font-weight-normal" => "Body text's weight",
        "font-weight-medium" => "A label's or a button's weight",
        "font-weight-bold" => "`.bold`, a heading, a title",
        "line-height-tight" => "Headings' line height",
        "line-height-normal" => "Body text's line height",
        "line-height-loose" => "Line height for long reading",
        "spacing-xs" => "The smallest gap: `gap: .xs`, `$xs` in a padding",
        "spacing-sm" => "A small gap: `gap: .sm`, a control's padding",
        "spacing-md" => "The usual gap: `gap: .md`, a card's padding",
        "spacing-lg" => "A large gap: `gap: .lg`, between sections",
        "spacing-xl" => "An extra-large gap, growing with the screen",
        "spacing-2xl" => "A section's spacing, growing with the screen",
        "spacing-3xl" => "A page's largest spacing, growing with the screen",
        "radius-none" => "Square corners",
        "radius-sm" => "Slightly rounded: a badge, a checkbox, code",
        "radius-md" => "The usual corners: a button, a field, a card",
        "radius-lg" => "Rounder corners: a modal, a sheet",
        "radius-xl" => "Very round corners",
        "radius-full" => "A pill or a circle: `.pill`, an avatar",
        "shadow-none" => "No shadow",
        "shadow-sm" => "A slight lift: a button, a field",
        "shadow-md" => "`Card.elevated`, a menu",
        "shadow-lg" => "A popover, a dropdown, a toast",
        "shadow-xl" => "A modal, a dialog, a sheet",
        "transition-fast" => "A quick change: a hover, a press",
        "transition-normal" => "The usual change: an open, a slide",
        "transition-slow" => "A deliberate change",
        "animation-duration-fast" => "`.fast` and `speed: .fast`",
        "animation-duration-normal" => "An animation's length when it says nothing",
        "animation-duration-slow" => "`.slow` and `speed: .slow`",
        "animation-easing-default" => "An animation's easing when it says nothing",
        "animation-easing-bounce" => "`easing: .bouncy`",
        "animation-easing-spring" => "`easing: .spring`, sampled into `linear()`",
        "ease-standard" => "`easing: .standard`: the default for motion that moves",
        "ease-out" => "`easing: .easeOut`: something arriving",
        "ease-in" => "`easing: .easeIn`: something leaving",
        "ease-spring" => "A spring that overshoots and settles",
        "screen-sm" => "Where `sm` starts: `{ sm: … }`, `@sm { }`, `viewport.sm`",
        "screen-md" => "Where `md` starts: `{ md: … }`, `@md { }`, `viewport.md`",
        "screen-lg" => "Where `lg` starts: `{ lg: … }`, `@lg { }`, `viewport.lg`",
        "screen-xl" => "Where `xl` starts: `{ xl: … }`, `@xl { }`, `viewport.xl`",
        "wf-header-height" => "How far a sticky element sits below the page's sticky header",
        "term-bg" => "A terminal block's background",
        "term-ink" => "A terminal block's text",
        "term-dim" => "A terminal block's quieter text: a prompt, a comment",
        "syntax-keyword" => "`CodeBlock`'s keywords",
        "syntax-string" => "`CodeBlock`'s strings",
        "syntax-number" => "`CodeBlock`'s numbers",
        "syntax-function" => "`CodeBlock`'s calls and element names",
        "syntax-punct" => "`CodeBlock`'s punctuation",
        "syntax-comment" => "`CodeBlock`'s comments",
        "z-sticky" => "Where a sticky header or `Sticky` stacks",
        "z-drawer" => "Where a drawer, a `Sidebar` on a narrow screen, stacks",
        "z-dropdown" => "Where a dropdown not in the top layer stacks",
        "z-modal" => "Where a modal not in the top layer stacks",
        "z-toast" => "Where the toasts stack",
        "z-tooltip" => "Where a tooltip not in the top layer stacks",
        "z-skip-link" => "Where the skip link stacks, above everything",
        "target-min" => "The least size of anything a pointer presses (WCAG 2.2's 24px)",
        "target-min-touch" => "The least size of anything a finger presses (44px)",
        _ => return None,
    })
}

#[cfg(test)]
mod describe_tests {
    #[test]
    fn every_baseline_token_says_what_it_is_for() {
        let missing: Vec<String> = super::default_tokens()
            .into_keys()
            .filter(|k| super::describe(k).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "these tokens say nothing of what they are for: {missing:?}"
        );
    }
}
