# 45. Design tokens

<!--
route: guide/tokens
group: reference
blurb: Every token of the baseline theme, with its value: the colours, type, spacing, radii, shadows, motion and breakpoints every built-in is drawn with.
description: Every baseline design token with its default value, how a theme overrides one, and how a style block or a stylesheet reads it, by full or short name.
-->

This chapter lists every design token of the baseline theme with its value,
and how a style reads one. A *design token* is a named value — a colour, a
size, a duration — that every built-in is drawn with. Change a token and
every element drawn with it changes.

A `theme` declaration changes the tokens it names and keeps the baseline
value of every other one. A theme may also add tokens of its own. Each
token becomes a CSS custom property on `:root`:

```wf
theme Brand {
    color-primary: #0F766E
    radius-md: 14px
    screen-md: 900px
    surface-raised: #F1F5F9
}
```

The build writes to `:root` only the tokens something in the output names,
and the tokens those name in turn.

## Reading a token

A token has two spellings, one for each place a style is written:

| Where | Syntax | Example |
|---|---|---|
| A `style { }` block | `$name` | `background: $color-surface` |
| A `style { }` block, short form | `$suffix`, resolved by the property's group ([below](#short-names)) | `padding: $lg` is `spacing-lg` |
| A `.css` file under `src/` | `var(--name)` | `color: var(--color-text-muted)` |

This page reads the tokens of the theme above — one the theme adds, one it
changes, one it keeps, and a breakpoint it moves:

```wf
theme Brand {
    color-primary: #0F766E
    radius-md: 14px
    screen-md: 900px
    surface-raised: #F1F5F9
}

page Pricing(path: "/pricing", title: "Pricing", description: "Plans and prices for every team.") {
    Heading("Pricing").h1
    Card {
        style {
            background: $surface-raised
            padding: $lg
            border-radius: $md
            border: 1px solid $border
            @md { padding: $xl }
        }
        Text("Team plan").bold
    }
}
```

The card's background is `#F1F5F9`, its radius `14px` and its border the
baseline `color-border`. Its padding grows to `spacing-xl` from `900px`
wide, because `@md` reads `screen-md`.

A `$token` the resolved theme does not declare draws the `V06` warning,
since the property would fall back to nothing. `wf theme` lists every token
the build uses, where each is read, and the contrast of the colour pairs
([Command line](41-cli.md#wf-theme)).

## The baseline

The tables below are written from the compiler's baseline theme, and the
test suite checks them against it.

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

In a `style { }` block, a short name is resolved through the group of the
property it is given to: `$lg` on `padding` is `spacing-lg`, and on
`border-radius` it is `radius-lg`.

| Property | Group | Short names | Example |
|---|---|---|---|
| `padding`, `margin` and every `padding-*` and `margin-*` property (physical and logical), `gap`, `row-gap`, `column-gap`, `width`, `height`, `min-width`, `max-width`, `min-height`, `max-height`, `top`, `right`, `bottom`, `left`, `inset` and `inset-*` | spacing | `xs`, `sm`, `md`, `lg`, `xl`, `2xl`, `3xl` | `$lg` is `spacing-lg` |
| `color`, `background`, `background-color`, `border-color`, `fill`, `stroke`, `outline`, `outline-color`, `text-decoration-color`, `caret-color`, `accent-color`, and the `border` shorthands (`border`, `border-top` … `border-inline-end`) | colour | `primary`, `secondary`, `success`, `danger`, `warning`, `info`, `background`, `surface`, `text`, `text-muted`, `border` | `$primary` is `color-primary` |
| `font-size` | font size | `xs`, `sm`, `base`, `lg`, `xl`, `2xl`, `3xl` | `$xl` is `font-size-xl` |
| `line-height` | line height | `tight`, `normal`, `loose` | `$tight` is `line-height-tight` |
| `border-radius` | radius | `none`, `sm`, `md`, `lg`, `xl`, `full` | `$lg` is `radius-lg` |
| `box-shadow` | shadow | `none`, `sm`, `md`, `lg`, `xl` | `$md` is `shadow-md` |
| `transition` | transition | `fast`, `normal`, `slow` | `$fast` is `transition-fast` |

A short name your theme declares as a token of its own means that token,
not the group's: with `md: 20px` in the theme, `$md` is `md` everywhere.

A full name — `$color-text-muted`, `$spacing-lg`, a token of your own — works
in any property.

## Next

[Runtime API](46-runtime-api.md) — `window.WF`, the runtime every compiled
page calls.
