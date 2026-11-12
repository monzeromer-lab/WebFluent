# Styling, themes and layout

A WebFluent page is styled in four places, from the widest reach to the
narrowest. Pick the narrowest one that does the job.

| To change… | Use |
|---|---|
| every page's colours, type, spacing, radius, motion | a `theme` declaration (tokens) |
| every instance of a built-in or of a class | a `.css` file under `src/`, with `class:` |
| one case of your component | its enum prop, styled as `[data-<prop>="<case>"]` in CSS |
| one element | its `style { }` block |

Layout comes first from the layout props (`Row`, `Stack`, `Grid`, `gap:`,
`align:`, `justify:`). They need no CSS at all.

## Contents

- [Tokens](#tokens)
- [A theme, and dark mode](#a-theme-and-dark-mode)
- [style blocks](#style-blocks)
- [Your own stylesheets and class:](#your-own-stylesheets-and-class)
- [Which rule wins](#which-rule-wins)
- [Layout without CSS](#layout-without-css)
- [Responsive](#responsive)
- [Right to left](#right-to-left)
- [Motion](#motion)
- [Mistakes](#mistakes)

## Tokens

A token is a design value with a name. Write it as `$name` in a `style { }`
block and as `var(--name)` in a `.css` file. `wf theme` lists every token the
project has, where each is used, and the contrast of each colour pair.

**Tokens, not raw values.** `background: #FFFFFF` stays white in dark mode,
while `background: $surface` follows the theme. A token a theme doesn't
declare is `V06`.

The baseline's 71 tokens (WebFluent 5.12):

| Group | Tokens |
|---|---|
| colour | `color-primary`, `color-secondary`, `color-success`, `color-danger`, `color-warning`, `color-info`, `color-background`, `color-surface`, `color-text`, `color-text-muted`, `color-border` |
| spacing | `spacing-xs`, `spacing-sm`, `spacing-md`, `spacing-lg`, `spacing-xl`, `spacing-2xl`, `spacing-3xl` |
| font size | `font-size-xs`, `font-size-sm`, `font-size-base`, `font-size-lg`, `font-size-xl`, `font-size-2xl`, `font-size-3xl` |
| font | `font-family`, `font-family-mono`, `font-weight-normal`, `font-weight-medium`, `font-weight-bold` |
| line height | `line-height-tight`, `line-height-normal`, `line-height-loose` |
| radius | `radius-none`, `radius-sm`, `radius-md`, `radius-lg`, `radius-xl`, `radius-full` |
| shadow | `shadow-none`, `shadow-sm`, `shadow-md`, `shadow-lg`, `shadow-xl` |
| transition | `transition-fast`, `transition-normal`, `transition-slow` |
| motion | `animation-duration-fast`, `animation-duration-normal`, `animation-duration-slow`, `animation-easing-default`, `animation-easing-bounce`, `animation-easing-spring`, `ease-standard`, `ease-spring`, `ease-in`, `ease-out` |
| breakpoints | `screen-sm`, `screen-md`, `screen-lg`, `screen-xl` |
| code and terminal | `syntax-keyword`, `syntax-string`, `syntax-number`, `syntax-function`, `syntax-comment`, `syntax-punct`, `term-bg`, `term-ink`, `term-dim` |
| layout | `wf-header-height` |

**Short names** resolve through the property's group:
- `padding: $lg` is `spacing-lg`;
- `color: $text-muted` is `color-text-muted`;
- `border-radius: $md` is `radius-md`;
- `box-shadow: $sm` is `shadow-sm`;
- `font-size: $xl` is `font-size-xl`.

A full name works in any property. A theme may add tokens of its own
(`surface-raised`, `brand`), which then work everywhere, as `$surface-raised`
and `var(--surface-raised)`.

## A theme, and dark mode

A theme names only the tokens it changes, plus any of its own. Declare the
light and dark themes in `src/theme.wf` and name them in the config:
`"theme": { "name": "Day", "dark": "Night" }`.

```wf
/// The brand, in daylight.
theme Day {
    color-primary: #0F766E
    color-background: #FAFAF9
    color-surface: #FFFFFF
    color-text: #1C1917
    color-text-muted: #57534E
    color-border: #E7E5E4
    font-family: Inter, system-ui, sans-serif
    radius-md: 0.5rem
    surface-raised: #F5F5F4
}

/// The same brand at night: only what changes.
theme Night {
    color-primary: #2DD4BF
    color-background: #0C0A09
    color-surface: #1C1917
    color-text: #F5F5F4
    color-text-muted: #A8A29E
    color-border: #292524
    surface-raised: #292524
}
```

- **The reader's choice.** `setTheme("light" | "dark" | "system")` switches
  and remembers the choice, and `theme` reads it. "system" follows
  `prefers-color-scheme` from the first paint, with no script.
- **Contrast.** Check both themes with `wf theme`. A pair below WCAG AA is
  `A13`.
- **Your own design system.** `"theme": { "builtin": "structural" }` keeps
  the built-ins' layout and behaviour and drops their look, for a project that
  styles everything itself.
- **Machine-supplied values.** `theme.tokens` in the config overrides tokens
  from outside, for example a colour a deploy injects.
- **Web fonts.** A theme that names a web font lists it in `meta.fonts` (a
  URL), or ships the file in `public/` with `@font-face` in a `.css` file, so
  it works offline.

## style blocks

```wf
/// A filter that says whether it is on.
component FilterChip(_ label: String, active: Bool = false, accent: String = "var(--color-primary)") {
    event pick()
    Button(label, aria-pressed: active) {
        style {
            padding: $xs $md
            border-radius: $full
            border: 1px solid $border
            background: transparent
            --accent: {accent}
            &:hover { border-color: var(--accent) }
            &[aria-pressed="true"] { background: var(--accent); color: $surface }
            &:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px }
            @md { padding: $sm $lg }
        }
        on click { emit pick() }
    }
}
```

- **The block is CSS.**
  - Hyphenated property names, unquoted values, one declaration per line or
    `;` between them, **never `,`** (`E002`).
  - It comes first in its block, before handlers and children.
- **A value that reads state** is spliced: `width: {pct}%`. A bare number
  spliced as a length has no unit and is dropped (`V07`).
- **Nested rules** (`&:hover`, `&:focus-visible`, `&:disabled`,
  `&::placeholder`, `&[aria-current="page"]`, `&[aria-pressed="true"]`)
  compile to stylesheet rules, so their values must be known at build time.
  For a hover colour that follows state, set a custom property from state
  (`--accent: {accent}`) and read it in the rule (`var(--accent)`).
- **Style ARIA states, not classes.** Style the attribute assistive technology
  reads: `&[aria-pressed="true"]`, `&[aria-expanded="true"]`,
  `&[aria-invalid="true"]`. The look and the meaning can't then drift apart.
- **Media rules.**
  - `@md { … }` is `@media (min-width: $screen-md)`.
  - `@media (…) { … }` and `@container (…) { … }` pass through.
  - A media rule beats a state rule on the same property.
- **Use `&:focus-visible`, not `&:focus`.** It rings the keyboard user, not
  every click.
- **A misspelled property** is `V05`, and the message names the nearest real
  one.

A literal-only block compiles to one shared class rule, so a hundred
identical cards cost one rule. Only spliced values are set on the element.

## Your own stylesheets and class:

What a `style { }` can't say goes in a `.css` file anywhere under `src/`:
- a selector over several elements;
- `@keyframes` and `@font-face`;
- a rule on `html`, `body` or `::selection`;
- a class many elements share.

Every file is bundled into `styles.css` in path order. Keep a feature's sheet
beside its files.

```css
/* src/board/board.css */
.task-row { padding-block: var(--spacing-xs); border-block-end: 1px solid var(--color-border); }
.task-row.is-done a { text-decoration: line-through; color: var(--color-text-muted); }
.stage-badge[data-stage="doing"] { background: var(--color-warning); }
.stage-badge[data-stage="done"] { background: var(--color-success); }
```

- **`class:` adds to the element's own classes** (`wf-card feature`), and it
  follows state:
  - `class: "feature"`;
  - `class: tone`;
  - `class: { "is-open": open, "is-done": task.done }`;
  - `class: ["row", if selected { "is-selected" }]`.

  A bare `Bool` as `class:` is `T01`, so write the map form.
- **Class names.** Plain BEM-ish names (`task-row`, `task-row__title`, a state
  as `is-done`). Every class is global, so name it on purpose.
- **Variants come from enum props.** A component writes each enum prop on its
  root as `data-<prop>="<case>"`, so one rule styles one case and the
  component holds no colour logic.
- **The built-ins' classes.** `wf-btn`, `wf-card`, `wf-btn--primary` and the
  rest are stable, so a sheet may restyle every button through `.wf-btn`.
  Never put one in `class:` (`V04`), because it brings that built-in's rules.
- **Read tokens** with `var(--color-primary)`, `var(--spacing-md)` and so on.

## Which rule wins

From weakest to strongest:
1. the built-ins' rules;
2. your `.css` files (they win on equal specificity);
3. `style { }` blocks (tripled specificity, and nested rules `!important`).

So an element's own block always beats a stylesheet. If a sheet rule seems
ignored, the element's block sets the same property.

## Layout without CSS

- **`Row`** (horizontal), **`Stack`** (vertical) and **`Grid`** take:
  - `gap:` (`.xs .sm .md .lg .xl`);
  - `align:` (`.start .center .end .stretch .baseline`);
  - `justify:` (`.start .center .end .between .around .evenly`).
- **`Grid(columns: 3)`** is a grid, and **`Column(span: 6)`** spans a
  12-column grid.
- **`Container`** centres a max-width column, **`Spacer`** pushes apart, and
  **`Divider`** draws a rule.
- Write the case with its dot: `gap: .md`. `gap: md` reads a name nothing
  declares (`T13`).

## Responsive

A layout stays as written; nothing reflows by surprise. Say what it does at
each width:

```wf
page Gallery(path: "/gallery", title: "Gallery", description: "Every picture, in a grid that reflows.") {
    Heading("Gallery").h1 {
        style {
            font-size: $xl
            @md { font-size: $2xl }
        }
    }
    Grid(columns: { base: 1, md: 2, lg: 3 }, gap: { base: .sm, lg: .lg }) {
        Card { Text("One") }
        Card { Text("Two") }
        Card { Text("Three") }
    }
    Row(gap: .md).stacks {
        Text("Side by side on a wide screen,")
        Text("one above the other on a narrow one.")
    }
}
```

- **The steps** are `base`, `sm`, `md`, `lg`, `xl`, which are the
  `screen-*` tokens. A step a value leaves out keeps the step below.
- **`.stacks`** on a `Row`, `Grid` or `Column` makes it a column on a narrow
  screen.
- **`viewport.md` and friends** are for behaviour, such as a drawer instead of
  a sidebar. Layout is CSS: right in the first paint, with no resize
  listener.

## Right to left

An Arabic, Hebrew, Persian or Urdu locale sets `dir="rtl"` on the page, and
so does a page written in one (`lang: "ar"` in its header); a page with
`lang: "en"` on an Arabic site is left to right.
Everything laid out with `Row`/`Stack`/`Grid` flips by itself. In your CSS
and `style { }`:
- **Logical sides.** `margin-inline-start`, `padding-inline-end`,
  `border-inline-start`, `inset-inline-end`, `text-align: start`, and never
  `left`/`right`.
- **Arrows.** Flip direction icons: `[dir="rtl"] .next-icon { transform: scaleX(-1) }`.
- **Codes and numbers.** Isolate a code or figure inside Arabic text:
  `class: "ltr"` with `.ltr { direction: ltr; unicode-bidi: isolate; }`.

## Motion

- **Enter animations.** `.fadeIn`, `.slideUp`, `.scaleIn`, `.expand` and the
  rest are flags on any element. `exit: .fadeOut` plays when an `if` or
  `for` removes the element, and `stagger: "50ms"` staggers a list.
- **More motion.**
  - `on: .enterView` plays when the element scrolls into view.
  - `count: "600ms"` counts a number to its new value.
  - `shared: "cover-{id}"` carries an element across a route change.
- **Route changes.** `Router(transition: .fade, duration: "200ms")`.
- **Per-property transitions.** A `transition { background: 200ms ease }`
  block inside an element's block.
- **Reduced motion.** A reader who asks for it (`prefers-reduced-motion`)
  gets none, automatically.
- **Inspect.** `wf motion` lists every animation, ease and speed, and the
  project's default (`"motion": { "duration": "180ms" }` in the config).

## Mistakes

| Wrong | Right | Code |
|---|---|---|
| `padding: 1rem, 2rem` | `padding: 1rem 2rem` (or `;` between declarations) | `E002` |
| `color: $brand` with no such token | a token `wf theme` lists, or declare it in the theme | `V06` |
| `colr: red` | `color: $danger` | `V05` |
| `width: {pct}` | `width: {pct}%` | `V07` |
| `Card(class: "wf-btn")` | a class of your own | `V04` |
| `gap: md` | `gap: .md` | `T13` |
| `&:hover { background: {tone} }` | `--hover: {tone}` and `&:hover { background: var(--hover) }` | — |
| `margin-left: $md` | `margin-inline-start: $md` | — |
| `background: #fff` | `background: $surface` | — |
| a `Row` that stays a row on a phone | `Row(…).stacks` or a responsive value | — |
