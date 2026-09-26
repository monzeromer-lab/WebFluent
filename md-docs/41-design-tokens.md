# 41. Design tokens

<!--
route: guide/tokens
group: reference
blurb: Every token of the baseline theme, with its value: the colours, type, spacing, radii, shadows, motion and breakpoints every built-in is drawn with.
description: Every baseline design token with its default value, how a theme overrides them, and how styles and stylesheets read them.
-->

A design token is a named value — a colour, a size, a duration — that every
built-in is drawn with. A `theme` changes the ones it names and keeps the
rest; each becomes a CSS custom property on `:root`.

```wf
theme Brand {
    color-primary: #0F766E
    radius-md: 14px
    screen-md: 900px
}
```

Read one in a `style { }` block as `$name` — `background: $color-surface`,
or the short form the property's group resolves, `padding: $lg` for
`spacing-lg` — and in a stylesheet as `var(--color-surface)`. A theme may add
tokens of its own (`surface-raised: #131519`) and read them the same way. The
build writes only the tokens something in the output names.

## The baseline

<!-- tokens -->

### Colour

| Token | Baseline |
|---|---|
| `color-background` | `#FFFFFF` |
| `color-border` | `#E2E8F0` |
| `color-danger` | `#EF4444` |
| `color-info` | `#06B6D4` |
| `color-primary` | `#3B82F6` |
| `color-secondary` | `#64748B` |
| `color-success` | `#22C55E` |
| `color-surface` | `#F8FAFC` |
| `color-text` | `#0F172A` |
| `color-text-muted` | `#64748B` |
| `color-warning` | `#F59E0B` |

### Font

| Token | Baseline |
|---|---|
| `font-family` | `system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif` |
| `font-family-mono` | `ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace` |
| `font-size-2xl` | `clamp(1.5rem, 1.25rem + 1.25vw, 2rem)` |
| `font-size-3xl` | `clamp(1.75rem, 1.38rem + 1.88vw, 2.5rem)` |
| `font-size-base` | `1rem` |
| `font-size-lg` | `clamp(1.125rem, 1.06rem + 0.31vw, 1.25rem)` |
| `font-size-sm` | `0.875rem` |
| `font-size-xl` | `clamp(1.25rem, 1.13rem + 0.63vw, 1.5rem)` |
| `font-size-xs` | `0.75rem` |
| `font-weight-bold` | `700` |
| `font-weight-medium` | `500` |
| `font-weight-normal` | `400` |
| `line-height-loose` | `1.75` |
| `line-height-normal` | `1.5` |
| `line-height-tight` | `1.25` |

### Spacing

| Token | Baseline |
|---|---|
| `spacing-2xl` | `clamp(2rem, 1.5rem + 2.5vw, 3rem)` |
| `spacing-3xl` | `clamp(2.5rem, 1.75rem + 3.75vw, 4rem)` |
| `spacing-lg` | `1.5rem` |
| `spacing-md` | `1rem` |
| `spacing-sm` | `0.5rem` |
| `spacing-xl` | `clamp(1.5rem, 1.25rem + 1.25vw, 2rem)` |
| `spacing-xs` | `0.25rem` |

### Radius

| Token | Baseline |
|---|---|
| `radius-full` | `9999px` |
| `radius-lg` | `1rem` |
| `radius-md` | `0.5rem` |
| `radius-none` | `0` |
| `radius-sm` | `0.25rem` |
| `radius-xl` | `1.5rem` |

### Shadow

| Token | Baseline |
|---|---|
| `shadow-lg` | `0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)` |
| `shadow-md` | `0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)` |
| `shadow-none` | `none` |
| `shadow-sm` | `0 1px 2px 0 rgba(0,0,0,0.05)` |
| `shadow-xl` | `0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1)` |

### Motion

| Token | Baseline |
|---|---|
| `animation-duration-fast` | `150ms` |
| `animation-duration-normal` | `300ms` |
| `animation-duration-slow` | `500ms` |
| `animation-easing-bounce` | `cubic-bezier(0.68, -0.55, 0.265, 1.55)` |
| `animation-easing-default` | `cubic-bezier(0.4, 0, 0.2, 1)` |
| `animation-easing-spring` | `cubic-bezier(0.175, 0.885, 0.32, 1.275)` |
| `transition-fast` | `150ms ease` |
| `transition-normal` | `250ms ease` |
| `transition-slow` | `350ms ease` |

### Easing

| Token | Baseline |
|---|---|
| `ease-in` | `cubic-bezier(0.4, 0, 1, 1)` |
| `ease-out` | `cubic-bezier(0, 0, 0.2, 1)` |
| `ease-spring` | `linear(0, 0.006, 0.079, 0.234, 0.44, 0.652, 0.828, 0.947, 1.008, 1.025, 1.016, 1)` |
| `ease-standard` | `cubic-bezier(0.2, 0, 0, 1)` |

### Breakpoints

| Token | Baseline |
|---|---|
| `screen-lg` | `1024px` |
| `screen-md` | `768px` |
| `screen-sm` | `640px` |
| `screen-xl` | `1280px` |

### Code

| Token | Baseline |
|---|---|
| `syntax-comment` | `#959BA1` |
| `syntax-function` | `#6FD3E6` |
| `syntax-keyword` | `#8FBDE7` |
| `syntax-number` | `#E0B070` |
| `syntax-punct` | `#B3BAC0` |
| `syntax-string` | `#7FD6A8` |

### Terminal

| Token | Baseline |
|---|---|
| `term-bg` | `#1B1F22` |
| `term-dim` | `#959BA1` |
| `term-ink` | `#E6E9EC` |

### Layout

| Token | Baseline |
|---|---|
| `wf-header-height` | `56px` |

71 tokens in all.

<!-- /tokens -->

## Short names

In a style block, a short name is resolved by the property it is given to:

| Property | Group | So `$…` means |
|---|---|---|
| `padding`, `margin` and their sides, `gap`, `row-gap`, `column-gap`, `width`, `height`, `min-`/`max-` sizes, `top`/`right`/`bottom`/`left`, `inset` | spacing | `$lg` is `spacing-lg` (`xs` … `3xl`) |
| `color`, `background`, `background-color`, `border-color`, `fill`, `stroke` | colour | `$primary` is `color-primary` (`primary`, `secondary`, `success`, `danger`, `warning`, `info`, `background`, `surface`, `text`, `border`) |
| `font-size` | font size | `$xl` is `font-size-xl` (`xs`, `sm`, `base`, `lg`, `xl`, `2xl`, `3xl`) |
| `line-height` | line height | `$tight` is `line-height-tight` |
| `border-radius` | radius | `$lg` is `radius-lg` |
| `box-shadow` | shadow | `$md` is `shadow-md` |
| `transition` | transition | `$fast` is `transition-fast` |

A full name — `$color-text-muted`, `$spacing-lg`, a token of your own —
works in any property.

## Next

[Runtime API](42-runtime-api.md).
