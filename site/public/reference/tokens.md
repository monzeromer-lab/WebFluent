# Design tokens

> Every baseline design token with its default value, how a theme overrides one, and how a style block or a stylesheet reads it, by full or short name.

The page: https://webfluent.monzeromer.dev/reference/tokens

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
([Command line](https://webfluent.monzeromer.dev/reference/cli.md#wf-theme)).

## Every built-in follows the theme

The built-ins draw their colours, sizes, radii, fonts and motion from these
tokens and nothing else, so a theme restyles every one of them, the ones a
page adds later included. A tone's label is a token too: `color-on-primary`
is what a primary button, badge, pressed toggle, current page or selected
day writes on `color-primary`, and so for `secondary`, `success`, `danger`,
`warning` and `info`. A theme with a light primary names a dark
`color-on-primary`, and `A13` holds each pair to WCAG AA where a page draws
it. An alert's tint, a focus ring and a highlighted match are worked out from
the tone and the page's colours (`color-mix`), so a dark theme gets dark
alerts without naming any; `color-backdrop` dims the page behind a modal, a
dialog, a sheet and a drawer. The only colours that stay fixed are the ones
over a picture: a carousel's and a lightbox's controls.

## The baseline

The tables below are written from the compiler's baseline theme, and the
test suite checks them against it.

<!-- tokens -->

### Colour

| Token | Baseline | What it is for |
|---|---|---|
| `color-backdrop` | `rgb(0 0 0 / 0.5)` | What dims the page behind a modal, a dialog, a sheet and a drawer |
| `color-background` | `#FFFFFF` | The page behind everything |
| `color-border` | `#E2E8F0` | A line between things: a field's edge, a divider, a card's border |
| `color-danger` | `#EF4444` | Failed, destructive, invalid: a danger button, an error message |
| `color-info` | `#06B6D4` | For information: an info badge or alert, a note callout |
| `color-on-danger` | `#FFFFFF` | Text and icons written on `color-danger`; A13 checks the pair |
| `color-on-info` | `#FFFFFF` | Text and icons written on `color-info`; A13 checks the pair |
| `color-on-primary` | `#FFFFFF` | Text and icons written on `color-primary`; A13 checks the pair |
| `color-on-secondary` | `#FFFFFF` | Text and icons written on `color-secondary`; A13 checks the pair |
| `color-on-success` | `#FFFFFF` | Text and icons written on `color-success`; A13 checks the pair |
| `color-on-warning` | `#000000` | Text and icons written on `color-warning`; A13 checks the pair |
| `color-primary` | `#3B82F6` | The main tone: a primary button, a link, a focus ring, what is chosen or current |
| `color-secondary` | `#64748B` | A quieter tone, for a secondary button or badge |
| `color-success` | `#22C55E` | Done, passing, saved: a success badge, alert or toast |
| `color-surface` | `#F8FAFC` | What sits on the page: a card, a menu, a popover, a table header |
| `color-text` | `#0F172A` | The text |
| `color-text-muted` | `#64748B` | Text that matters less: a hint, a caption, `.muted` |
| `color-warning` | `#F59E0B` | Needs attention: a warning badge or alert, a rating's stars |

### Font

| Token | Baseline | What it is for |
|---|---|---|
| `font-family` | `system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif` | The text's typeface |
| `font-family-mono` | `ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace` | Code's typeface: `Code`, `CodeBlock`, `Kbd` |
| `font-size-2xl` | `clamp(1.5rem, 1.25rem + 1.25vw, 2rem)` | A heading (`h3`, `h2`), growing with the screen |
| `font-size-3xl` | `clamp(1.75rem, 1.38rem + 1.88vw, 2.5rem)` | The page's heading (`h1`), growing with the screen |
| `font-size-base` | `1rem` | Body text |
| `font-size-lg` | `clamp(1.125rem, 1.06rem + 0.31vw, 1.25rem)` | Large text: `.lg`, a lede |
| `font-size-sm` | `0.875rem` | Small text: `.sm`, a hint, a table header |
| `font-size-xl` | `clamp(1.25rem, 1.13rem + 0.63vw, 1.5rem)` | A small heading (`h4`) |
| `font-size-xs` | `0.75rem` | The smallest text: a badge, a caption |
| `font-weight-bold` | `700` | `.bold`, a heading, a title |
| `font-weight-medium` | `500` | A label's or a button's weight |
| `font-weight-normal` | `400` | Body text's weight |
| `line-height-loose` | `1.75` | Line height for long reading |
| `line-height-normal` | `1.5` | Body text's line height |
| `line-height-tight` | `1.25` | Headings' line height |

### Spacing

| Token | Baseline | What it is for |
|---|---|---|
| `spacing-2xl` | `clamp(2rem, 1.5rem + 2.5vw, 3rem)` | A section's spacing, growing with the screen |
| `spacing-3xl` | `clamp(2.5rem, 1.75rem + 3.75vw, 4rem)` | A page's largest spacing, growing with the screen |
| `spacing-lg` | `1.5rem` | A large gap: `gap: .lg`, between sections |
| `spacing-md` | `1rem` | The usual gap: `gap: .md`, a card's padding |
| `spacing-sm` | `0.5rem` | A small gap: `gap: .sm`, a control's padding |
| `spacing-xl` | `clamp(1.5rem, 1.25rem + 1.25vw, 2rem)` | An extra-large gap, growing with the screen |
| `spacing-xs` | `0.25rem` | The smallest gap: `gap: .xs`, `$xs` in a padding |

### Radius

| Token | Baseline | What it is for |
|---|---|---|
| `radius-full` | `9999px` | A pill or a circle: `.pill`, an avatar |
| `radius-lg` | `1rem` | Rounder corners: a modal, a sheet |
| `radius-md` | `0.5rem` | The usual corners: a button, a field, a card |
| `radius-none` | `0` | Square corners |
| `radius-sm` | `0.25rem` | Slightly rounded: a badge, a checkbox, code |
| `radius-xl` | `1.5rem` | Very round corners |

### Shadow

| Token | Baseline | What it is for |
|---|---|---|
| `shadow-lg` | `0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -4px rgba(0,0,0,0.1)` | A popover, a dropdown, a toast |
| `shadow-md` | `0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)` | `Card.elevated`, a menu |
| `shadow-none` | `none` | No shadow |
| `shadow-sm` | `0 1px 2px 0 rgba(0,0,0,0.05)` | A slight lift: a button, a field |
| `shadow-xl` | `0 20px 25px -5px rgba(0,0,0,0.1), 0 8px 10px -6px rgba(0,0,0,0.1)` | A modal, a dialog, a sheet |

### Motion

| Token | Baseline | What it is for |
|---|---|---|
| `animation-duration-fast` | `150ms` | `.fast` and `speed: .fast` |
| `animation-duration-normal` | `300ms` | An animation's length when it says nothing |
| `animation-duration-slow` | `500ms` | `.slow` and `speed: .slow` |
| `animation-easing-bounce` | `cubic-bezier(0.68, -0.55, 0.265, 1.55)` | `easing: .bouncy` |
| `animation-easing-default` | `cubic-bezier(0.4, 0, 0.2, 1)` | An animation's easing when it says nothing |
| `animation-easing-spring` | `cubic-bezier(0.175, 0.885, 0.32, 1.275)` | `easing: .spring`, sampled into `linear()` |
| `transition-fast` | `150ms ease` | A quick change: a hover, a press |
| `transition-normal` | `250ms ease` | The usual change: an open, a slide |
| `transition-slow` | `350ms ease` | A deliberate change |

### Easing

| Token | Baseline | What it is for |
|---|---|---|
| `ease-in` | `cubic-bezier(0.4, 0, 1, 1)` | `easing: .easeIn`: something leaving |
| `ease-out` | `cubic-bezier(0, 0, 0.2, 1)` | `easing: .easeOut`: something arriving |
| `ease-spring` | `linear(0, 0.006, 0.079, 0.234, 0.44, 0.652, 0.828, 0.947, 1.008, 1.025, 1.016, 1)` | A spring that overshoots and settles |
| `ease-standard` | `cubic-bezier(0.2, 0, 0, 1)` | `easing: .standard`: the default for motion that moves |

### Breakpoints

| Token | Baseline | What it is for |
|---|---|---|
| `screen-lg` | `1024px` | Where `lg` starts: `{ lg: … }`, `@lg { }`, `viewport.lg` |
| `screen-md` | `768px` | Where `md` starts: `{ md: … }`, `@md { }`, `viewport.md` |
| `screen-sm` | `640px` | Where `sm` starts: `{ sm: … }`, `@sm { }`, `viewport.sm` |
| `screen-xl` | `1280px` | Where `xl` starts: `{ xl: … }`, `@xl { }`, `viewport.xl` |

### Code

| Token | Baseline | What it is for |
|---|---|---|
| `syntax-comment` | `#959BA1` | `CodeBlock`'s comments |
| `syntax-function` | `#6FD3E6` | `CodeBlock`'s calls and element names |
| `syntax-keyword` | `#8FBDE7` | `CodeBlock`'s keywords |
| `syntax-number` | `#E0B070` | `CodeBlock`'s numbers |
| `syntax-punct` | `#B3BAC0` | `CodeBlock`'s punctuation |
| `syntax-string` | `#7FD6A8` | `CodeBlock`'s strings |

### Terminal

| Token | Baseline | What it is for |
|---|---|---|
| `term-bg` | `#1B1F22` | A terminal block's background |
| `term-dim` | `#959BA1` | A terminal block's quieter text: a prompt, a comment |
| `term-ink` | `#E6E9EC` | A terminal block's text |

### Stacking

| Token | Baseline | What it is for |
|---|---|---|
| `z-drawer` | `200` | Where a drawer, a `Sidebar` on a narrow screen, stacks |
| `z-dropdown` | `1000` | Where a dropdown not in the top layer stacks |
| `z-modal` | `1300` | Where a modal not in the top layer stacks |
| `z-skip-link` | `10000` | Where the skip link stacks, above everything |
| `z-sticky` | `100` | Where a sticky header or `Sticky` stacks |
| `z-toast` | `1400` | Where the toasts stack |
| `z-tooltip` | `1500` | Where a tooltip not in the top layer stacks |

### Targets

| Token | Baseline | What it is for |
|---|---|---|
| `target-min` | `24px` | The least size of anything a pointer presses (WCAG 2.2's 24px) |
| `target-min-touch` | `44px` | The least size of anything a finger presses (44px) |

### Layout

| Token | Baseline | What it is for |
|---|---|---|
| `wf-header-height` | `56px` | How far a sticky element sits below the page's sticky header |

87 tokens in all.

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

[Runtime API](https://webfluent.monzeromer.dev/reference/runtime.md) — `window.WF`, the runtime every compiled
page calls.
