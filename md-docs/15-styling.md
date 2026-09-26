# 15. Styling

<!--
route: guide/styling
group: building
blurb: Every built-in ships styled; a theme changes the whole site; a style block is real CSS scoped to its element.
description: Themes and tokens, style blocks with splices, nesting and media queries, transitions, classes and stylesheets, dark mode, responsive layout.
-->

Every built-in ships styled, and a theme changes the whole site by changing
its tokens. When you need more, an element takes a `style { }` block of
real CSS, scoped to that element, with tokens and live values spliced in.

## Design tokens and the theme

A theme is a set of tokens, each of which becomes a CSS custom property
(`--color-primary`). A `theme` declaration overrides what the baseline
carries; every token not listed keeps its baseline value:

```wf
theme Brand {
    color-primary: #0F766E
    color-secondary: #134E4A
    color-background: #FFFFFF
    color-surface: #F8FAFC
    color-text: #0F172A
    color-text-muted: #64748B
    color-border: #E2E8F0
    font-family: 'Inter', system-ui, sans-serif
    font-family-mono: 'JetBrains Mono', monospace
    radius-md: 14px
    spacing-lg: 2.5rem
    shadow-md: 0 4px 12px rgba(0,0,0,0.08)
}
```

A project with one theme uses it; with several, `webfluent.app.json` chooses:
`"theme": { "name": "Brand" }`. Values are raw CSS to the end of the line.

The baseline carries 71 tokens in groups — colour, font, spacing, radius,
shadow, motion, easing, breakpoints, code and layout — every one a CSS custom
property on `:root`, readable from a style block as `$name` and from a
stylesheet as `var(--name)`. [Design tokens](41-design-tokens.md) lists each
with its baseline value. Moving a breakpoint moves every media query the
build writes with it.

A theme may add tokens of its own (`surface-raised: #131519`); they become
custom properties like the rest. The contrast lint (`A13`) checks text
against background pairs in the theme and warns below WCAG AA.

`theme.builtin` in `webfluent.app.json`: `full` (default) ships the baseline look of
every built-in you use; `structural` ships only their layout and mechanics
— focus rings, overlays, the grid — for a site that brings its whole
design.

## `style { }`

```wf
page Styled(path: "/") {
    state accent = "#D97706"
    state wide = false
    Card {
        style {
            padding: $lg                       // a token, resolved by the property: spacing-lg
            background: $surface               // color-surface
            border: 1px solid $border
            border-radius: $radius-lg
            box-shadow: $shadow-md
            border-left: 4px solid {accent}    // a live value: repaints when `accent` changes
            max-width: {if wide { "100%" } else { "40rem" }}
            &:hover { box-shadow: $shadow-lg }
            & > p { margin: 0 }
            @media (max-width: 640px) { padding: $sm }
        }
        Text("Styled card")
        Button("Widen") { on click { wide = !wide } }
    }
}
```

- A style block holds CSS declarations, one per line or `;`-separated. No
  quotes around values — `padding: 2rem`, not `"2rem"`.
- `$token` names a token. A short name is resolved by the property:
  `padding: $lg` is `spacing-lg`, `color: $primary` is `color-primary`,
  `border-radius: $lg` is `radius-lg`, `box-shadow: $md` is `shadow-md`,
  `font-size: $xl` is `font-size-xl`, `transition: $fast`. A full name
  (`$color-primary`, `$surface-raised`) works anywhere. Check a token's
  spelling: one no theme defines resolves to nothing, and the browser drops
  the declaration.
- `{expr}` splices a live value; the declaration updates when the value
  changes. A whole value or a part of one (`{n}px`).
- `&:hover`, `&:focus-visible`, `&.active`, `& > p`, `&[aria-current]`
  nest as CSS nesting does; `@media (…) { }` nests too.
- The block is scoped: it becomes a class on that element alone, written to
  the page's stylesheet at build time. Splices become custom properties set
  by the runtime.

The static parts of every `style { }` are compiled to CSS; the linter warns
when a declaration is invalid CSS.

## `transition { }`

```wf
page Fade(path: "/") {
    state on = false
    Card {
        style { opacity: {if on { 1 } else { 0.4 }}; transform: {if on { "none" } else { "scale(0.98)" }} }
        transition { opacity: $fast; transform: 200ms ease-out }
        Text("Fades")
    }
    Button("Toggle") { on click { on = !on } }
}
```

`transition { prop: duration easing }` sets CSS transitions for the
element; a `$fast/$normal/$slow` token or a raw value. Reduced-motion
readers get none ([chapter 20](20-motion.md)).

## Classes and your own stylesheet

`class: "name"` adds a class from your own stylesheet; a `.css` file in
`src/` is bundled into `styles.css`, and `meta.stylesheets` in `webfluent.app.json`
links external ones.

```wf
page Classes(path: "/") {
    state active = true
    Container(class: "hero") {
        Heading("Welcome", class: if active { "title is-on" } else { "title" }).h1
    }
}
```

The value may be an expression, so a class can follow state. Built-ins wear
`wf-*` classes (`wf-btn`, `wf-btn--primary`, `wf-card`), stable across
releases, so a stylesheet may target them.

## When your rule and a built-in's disagree

Three layers of CSS meet on a page, in this order of strength:

1. **The built-ins' own rules** (`.wf-btn`, `.wf-card`), shipped in
   `styles.css`.
2. **Your stylesheets** (`src/**/*.css`), bundled after them — so on equal
   specificity, yours win.
3. **`style { }` blocks**, compiled to rules with tripled specificity, and
   nested state and media rules marked `!important` — so an element's own
   block beats any sheet, as the inline style it replaces would.

So: to restyle one element, use its `style { }`. To restyle every button,
write `.wf-btn { … }` in a stylesheet, or change the tokens in a theme — the
built-ins read tokens, so `color-primary` recolours every primary button,
link and focus ring at once. `theme.builtin: "structural"` drops the
built-ins' look entirely and keeps only their layout and mechanics, for a
project that brings its whole design.

## Enum props as hooks

A component's enum props are written to its root element as `data-<prop>`,
so one stylesheet rule styles every variant ([chapter 11](11-components.md#enum-props)):

```css
.note[data-tone="loud"] { font-weight: 700; color: var(--color-danger) }
```

## Dark mode

A second `theme` declaration named in `webfluent.app.json` as `"theme": { "name":
"Light", "dark": "Night" }` stands in for dark mode. Its tokens override the
light theme's under the reader's system preference
(`prefers-color-scheme: dark`) and whenever `setTheme("dark")` was called;
`setTheme("light")` forces light, `setTheme("system")` follows the system
again. The choice is remembered across visits and applied before the first
paint, so a dark reader never sees a light flash.

```wf
theme Light {
    color-background: #FFFFFF
    color-text: #0F172A
}

theme Night {
    color-background: #0B1220
    color-surface: #111827
    color-text: #E5E7EB
    color-text-muted: #9CA3AF
    color-border: #1F2937
}

page Prefs(path: "/") {
    Text("You are on the {theme} theme")
    Row(gap: .sm) {
        Button("Light") { on click { setTheme("light") } }
        Button("Dark") { on click { setTheme("dark") } }
        Button("System") { on click { setTheme("system") } }
    }
}
```

Everything that uses tokens — the built-ins, your `$token` styles — switches
with it; a color written raw does not, so prefer tokens. The `theme` value
is `"light"`, `"dark"` or `"system"`.

## Layout without CSS

Most layout needs no `style` at all: `Stack`, `Row`, `Grid`, `Container`,
`Spacer` and `Divider` with their `gap:`, `align:`, `justify:` and
`columns:` props ([chapter 7](07-elements.md#layout-elements)).

```wf
page Layout(path: "/", title: "Layout", description: "A grid of projects.") {
    Container {
        Stack(gap: .lg) {
            Row(justify: .between, align: .center) {
                Heading("Projects").h2
                Button("New").primary.sm
            }
            Grid(columns: { base: 1, md: 2, lg: 3 }, gap: .md) {
                for n in 1..=6 { Card { Text("Project {n}") } }
            }
        }
    }
}
```

## Responsive values

Any layout prop takes **one value per breakpoint**:

```wf
Grid(columns: { base: 1, md: 2, lg: 3 }, gap: { base: .sm, lg: .lg }) { … }
Row(direction: { base: .column, md: .row }) { … }
Column(span: { base: 12, md: 6 }) { … }
```

That compiles to one class and one media query per step — no JavaScript, no
resize listener, and **the right layout in the very first paint**, where a
`viewport` test could only paint one branch and swap after hydration.

The steps are `base`, `sm`, `md`, `lg`, `xl`, and each is a design token —
`screen-sm` … `screen-xl`. Move one in your theme and every media query
moves with it, including `viewport.md`:

```wf
theme Brand { screen-md: 900px }
```

A step a value omits keeps whatever the step below it said, so
`{ base: 1, lg: 3 }` is one column until `lg`.

The same names are sugar inside a `style { }` block:

```wf
Heading("Title").h1 {
    style {
        font-size: 24px
        @md { font-size: 34px }
    }
}
```

### A layout stays as it is written

The engine used to reflow **every** `Row` into a column and **every**
`Grid` into one column below 768px, with `!important`. It does not any
more. A layout that should reflow says so:

```wf
Row.stacks { … }                          // a column on a narrow screen
Grid.stacks { … }                         // one column on a narrow screen
Grid(columns: { base: 1, md: 3 }) { … }   // or say what it does at each width
```

`wf migrate` adds `.stacks` to every `Row`, `Grid` and `Column` of a
WebFluent 2 project, so the old behaviour carries over — written down,
where you can see it and change it.

`viewport` ([chapter 8](08-state-and-reactivity.md#the-browser-as-values))
stays for what is genuinely behavioural — a drawer instead of a sidebar —
and follows `matchMedia`, which fires once when the answer changes rather
than on every pixel of a drag.

## What the build emits

`styles.css` holds the tokens something in the output uses
(`:root { --color-primary: … }`), the dark overrides, the CSS of the built-ins
you used (and only those), your bundled stylesheets, and the compiled
`style { }` rules that more than one page reaches. A rule only one page can
reach goes to `pages/<Name>.css`, loaded with that page.
[Performance](31-performance.md) shows how to see what each weighs.

## Next

[Forms and validation](16-forms.md).
