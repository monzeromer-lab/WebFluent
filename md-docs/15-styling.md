# 15. Styling

<!--
route: guide/styling
group: building
blurb: Restyle the whole site with a theme, one element with a style block, and many elements with your own stylesheet — and make the layout responsive.
description: Themes and tokens, style blocks with live values, nested rules and media queries, transitions, classes and stylesheets, dark mode, responsive layout.
-->

This chapter shows you how to change the way your site looks, from the whole
site down to one element. Every built-in element already ships styled, so
you only write styles for what you want to change.

You will be able to:

- change colours, fonts and spacing across the site with a `theme`;
- style one element with a `style { }` block, including live values, hover
  states and media queries;
- share rules between elements with classes and your own `.css` files;
- add dark mode;
- make a layout change with the width of the screen.

## Before you start

You need a project that builds (see [Getting started](02-getting-started.md))
and you should know how to place elements and pass them props
([Elements](07-elements.md)). Styles that follow state use `state` values
([State and reactivity](08-state-and-reactivity.md)).

Three tools cover almost everything. Pick the narrowest one that does the
job:

| To change… | Use | Section |
|---|---|---|
| The whole site: every button, card and link | a `theme` | [Change the whole site with a theme](#change-the-whole-site-with-a-theme) |
| One element | a `style { }` block on it | [Style one element](#style-one-element) |
| A group of elements, or something a block cannot say | a class and a `.css` file | [Classes and your own stylesheet](#classes-and-your-own-stylesheet) |

## Change the whole site with a theme

To change the colours, fonts, spacing or corner radius of everything at
once, declare a `theme`. A theme sets *design tokens* — named values such as
`color-primary` or `radius-md` that every built-in reads.

This theme gives the site a teal brand colour, the Inter font and rounder
corners:

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

Every primary button, link and focus ring now uses `#0F766E`, because the
built-ins read `color-primary` rather than a fixed colour.

How a theme works:

- Each value is raw CSS, to the end of its line. Do not quote it.
- You list only the tokens you change. Every token you leave out keeps its
  *baseline* value — the default the compiler ships.
- The baseline has 71 tokens, in groups: colour, font, spacing, radius,
  shadow, motion, easing, breakpoints, code and layout.
  [Design tokens](42-design-tokens.md) lists each one with its baseline
  value.
- Every token becomes a CSS custom property on `:root` (`--color-primary`).
  You read it from a style block as `$color-primary` and from a stylesheet
  as `var(--color-primary)`.
- A theme may add tokens of its own, such as `surface-raised: #131519`. They
  become custom properties like the rest.
- The breakpoints are tokens too, so moving one moves every media query the
  build writes (see [Responsive values](#responsive-values)).

If the project declares one theme, the build uses it. If it declares
several, choose one in `webfluent.app.json`:

```json
{ "theme": { "name": "Brand" } }
```

The contrast lint, `A13`, checks the text and background pairs of your theme
and warns when one falls below the WCAG AA contrast ratio. Run `wf theme` to
see every token, where it comes from and which pairs it checked.

To keep the built-ins' layout and mechanics but drop their look — for a site
that brings its whole design — set `theme.builtin` in `webfluent.app.json`:

| `theme.builtin` | What the build ships for each built-in you use |
|---|---|
| `"full"` (default) | Its layout, its mechanics and its baseline look |
| `"structural"` | Only its layout and mechanics: focus rings, overlays, the grid |

## Style one element

To change one element, give it a `style { }` block. The block holds CSS
declarations, and they apply to that element only.

This card uses tokens for its spacing and colours, a live value for its
accent border and width, a hover rule, a rule for its paragraphs and a media
query:

```wf
page Styled(path: "/styled", title: "Styled card", description: "A card with its own style.") {
    state accent = "#D97706"
    state wide = false

    Heading("Styled card").h1
    Card {
        style {
            padding: $lg                       // spacing-lg, from the property
            background: $surface              // color-surface
            border: 1px solid $border
            border-radius: $radius-lg
            box-shadow: $shadow-md
            border-left: 4px solid {accent}    // repaints when `accent` changes
            max-width: {if wide { "100%" } else { "40rem" }}
            &:hover { box-shadow: $shadow-lg }
            & > p { margin: 0 }
            @media (max-width: 640px) { padding: $sm }
        }
        Text("Hover me, or widen me.")
        Button("Widen") { on click { wide = !wide } }
        Button("Make it blue") { on click { accent = "#2563EB" } }
    }
}
```

The card gets a deeper shadow on hover. **Widen** lets it fill the page, and
**Make it blue** recolours its left border at once.

### Write declarations

- Write one declaration per line, or several on one line separated by `;`.
- Write values as CSS, with no quotes: `padding: 2rem`, not `"2rem"`.
- Never separate two declarations with a comma. `width: 2rem, flex: none` is
  `E002`, because a style block is CSS, not a map.

### Use a token

`$name` reads a design token. A full name, such as `$color-primary` or
`$surface-raised`, works on any property. A short name is resolved by the
property it is written on:

| You write | It means |
|---|---|
| `padding: $lg` (and `margin`, `gap`, `width`, `height`, `top` …) | `spacing-lg` |
| `color: $primary`, `background: $surface`, `border: 1px solid $border` | `color-primary`, `color-surface`, `color-border` |
| `border-radius: $lg` | `radius-lg` |
| `box-shadow: $md` | `shadow-md` |
| `font-size: $xl` | `font-size-xl` |
| `transition: $fast` | `transition-fast` |

A token that the theme does not declare is `V06`, an error, because the
browser would drop the declaration. A property name that CSS does not have,
such as `colr:`, is `V05`, a warning that suggests the nearest property.

### Splice a live value

`{expr}` puts a value into a declaration. When the value reads state, the
declaration updates whenever that state changes. A splice may be the whole
value (`{accent}`) or part of one (`{pct}%`).

A splice that is a bare number for a length, such as `width: {pct}`, is
`V07`: the browser drops a length with no unit. Write `width: {pct}%` or
`width: {pct}px`.

### Style a state with a nested rule

A nested rule styles the element while it is in some state. `&` is the
element itself, as in CSS nesting:

- `&:hover`, `&:focus-visible` (the keyboard focus ring), `&:active`,
  `&:disabled`, `&::placeholder`, `&:focus-within`;
- `&.active`, `& > p`;
- the ARIA states a control keeps in its attributes:
  `&[aria-current="page"]`, which the router sets on the current `Link`,
  and `&[aria-pressed="true"]`, `&[aria-selected="true"]`,
  `&[aria-checked="true"]`, `&[aria-expanded="true"]`,
  `&[aria-invalid="true"]`.

`@media (…) { }` and `@container (…) { }` nest the same way.

A nested rule is compiled into the stylesheet, so its values must be known
at build time: literals and tokens, not state. To make a hover colour follow
state, set a custom property on the element and read it in the rule:

```wf
page Swatch(path: "/swatch", title: "Swatch", description: "A hover colour that follows state.") {
    state hoverColor = "#0F766E"

    Heading("Swatch").h1
    Button("Hover me") {
        style {
            --hover-bg: {hoverColor}
            &:hover { background: var(--hover-bg) }
        }
    }
    Button("Use amber") { on click { hoverColor = "#D97706" } }
}
```

Read your own custom property with `var(--name)`. `$name` is only for design
tokens.

### How the build compiles a style block

- The parts of a block that do not read state become a stylesheet rule,
  under a class named by the block's content. The element carries the
  class, so a hundred identical cards share one rule and the pre-rendered
  page has it before any script runs.
- A declaration that reads state is set on the element by the runtime and
  updated when the state changes.
- Nested state rules and media rules are marked `!important`, so they win
  over the element's base declarations while they apply. When a state rule
  and a media rule set the same property, the media rule wins.

## Animate a style change

To make a property change smoothly instead of jumping, add a
`transition { }` block. Each line names a property, a duration and an
optional easing.

This card fades and grows when you press **Toggle**:

```wf
page Fade(path: "/fade", title: "Fade", description: "A card that fades in and out.") {
    state active = false

    Heading("Fade").h1
    Card {
        style { opacity: {if active { 1 } else { 0.4 }}; transform: {if active { "none" } else { "scale(0.98)" }} }
        transition { opacity: $fast; transform: 200ms ease-out }
        Text("Fades")
    }
    Button("Toggle") { on click { active = !active } }
}
```

A duration may be a token — `$fast`, `$normal` or `$slow` — or a raw value
such as `200ms ease-out`. A reader who asked their system for less motion
gets no transition. Enter and exit animations, and other motion, are in
[Motion](20-motion.md).

## Classes and your own stylesheet

To share a rule between many elements, or to write what a `style { }` block
cannot say, put it in a `.css` file and add its class with `class:`.

Use a stylesheet for:

- a selector over several elements;
- a `@keyframes` or a `@font-face`;
- a class that a dozen elements share by name;
- a rule on `html`, `body` or `::selection`.

### Add a stylesheet

Every `.css` file anywhere under `src/` is part of the build. You can keep
one global file, or put a file beside the component it styles:

```text
src/
├── styles.css            # global rules
└── components/
    ├── Card.wf
    └── card.css          # beside the component it styles
```

```css
/* src/components/card.css */
.feature {
  container-type: inline-size;
}
.feature--first::before {
  content: counter(feature, upper-roman);
  color: var(--color-text-muted);
}
```

The build reads the files in path order, minifies them and bundles them
into `styles.css`. `var(--token)` reaches every design token. There is no
scoping: a class is global, so name it on purpose.

To link a sheet the build does not compile — a file in `public/` or a URL —
list it in `meta.stylesheets` in `webfluent.app.json`. It is linked ahead of
`styles.css`.

### Add a class

`class: "name"` adds a class to the element. It does not replace the
element's own classes: `Container(class: "hero")` renders
`class="wf-container hero"`.

The value may be an expression, so a class can follow state:

```wf
page Welcome(path: "/welcome", title: "Welcome", description: "A heading whose class follows state.") {
    state active = true

    Container(class: "hero") {
        Heading("Welcome", class: if active { "title is-on" } else { "title" }).h1
        Button("Switch") { on click { active = !active } }
    }
}
```

When several classes each have their own condition, pass a map or a list:

- In a **map**, each key is a class, and it is on while its value is true.
- A **list** joins strings and maps, and skips anything that is `null`.

```wf
page Chores(path: "/chores", title: "Chores", description: "Classes that follow state.") {
    state done = false
    state urgent = true
    state tone = "calm"

    Heading("Chores").h1
    Text("Water the plants", class: { "is-done": done, "is-urgent": urgent })
    Container(class: ["panel", tone, if urgent { "is-loud" }]) { Text("Due today") }
    Button("Done") { on click { done = !done } }
}
```

The rules for a class value:

- A map's values must be conditions (`Bool`).
- A bare condition is not a class: `class: active` is `T01`. Write
  `{ "is-active": active }`, which says what `active` turns on.
- When the value changes, only the classes that `class:` itself named come
  off. A class a script added with `classList.add` stays.

### Target the built-ins

Each built-in wears `wf-*` classes, such as `wf-btn`, `wf-btn--primary` and
`wf-card`. They are stable across releases, so a stylesheet may target them.

Do not add another built-in's class to an element with `class:`. It brings
that built-in's rules with it, and draws `V04`.

## Override a built-in's look

To decide which rule wins, know the three layers of CSS on a page. From
weakest to strongest:

1. **The built-ins' own rules** (`.wf-btn`, `.wf-card`), shipped in
   `styles.css`.
2. **Your stylesheets** (`src/**/*.css`), bundled after them. On equal
   specificity, yours win.
3. **`style { }` blocks**, compiled to rules with tripled specificity, with
   nested state and media rules marked `!important`. An element's own block
   beats any stylesheet, as the inline style it replaces would.

So choose the tool by how far the change reaches:

- To restyle **one element**, use its `style { }` block.
- To restyle **every button**, write `.wf-btn { … }` in a stylesheet, or
  change the tokens in a theme. Because the built-ins read tokens,
  `color-primary` recolours every primary button, link and focus ring at
  once.
- To drop the built-ins' look entirely, set `theme.builtin: "structural"`
  (see [Change the whole site with a theme](#change-the-whole-site-with-a-theme)).

## Style a component's variants

To style each variant of your own component from a stylesheet, use its enum
props. A component writes each enum prop to its root element as a
`data-<prop>` attribute, such as `data-tone="loud"`, so one rule styles one
variant ([Components](11-components.md#enum-props)):

```css
.note[data-tone="loud"] { font-weight: 700; color: var(--color-danger) }
```

## Dark mode

To add dark mode, declare a second theme with the tokens that change, and
name both themes in `webfluent.app.json`:

```json
{ "theme": { "name": "Light", "dark": "Night" } }
```

The dark theme's tokens replace the light theme's in two cases:

- when the reader's system prefers dark (`prefers-color-scheme: dark`) and
  the reader has not chosen light;
- whenever the reader chose dark with `setTheme("dark")`.

This page declares both themes and lets the reader choose:

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

page Prefs(path: "/prefs", title: "Appearance", description: "Choose light or dark.") {
    Heading("Appearance").h1
    Text("You are on the {theme} theme")
    Row(gap: .sm) {
        Button("Light") { on click { setTheme("light") } }
        Button("Dark") { on click { setTheme("dark") } }
        Button("System") { on click { setTheme("system") } }
    }
}
```

The text under the heading reads `You are on the dark theme` after you press
**Dark**.

- `setTheme("light")` forces light, `setTheme("dark")` forces dark, and
  `setTheme("system")` follows the system again.
- `theme` reads the reader's choice: `"light"`, `"dark"` or `"system"`.
- The choice is kept in the browser's storage across visits. The page's
  script applies it before it draws the page. The system preference needs
  no script: it is a media query in the stylesheet, so it applies from the
  first paint.

> [!WARNING]
> Only what reads tokens switches with the theme. A colour written raw, such
> as `background: #FFFFFF`, stays the same in dark mode. Use tokens such as
> `$surface` and `$text` for anything that should change.

## Lay out a page without CSS

Most layout needs no `style` at all. `Stack`, `Row`, `Grid`, `Container`,
`Spacer` and `Divider` take `gap:`, `align:`, `justify:` and `columns:`
props ([Elements](07-elements.md#layout-elements)).

This page puts a title and a button on one line, with a grid of cards below
that has one, two or three columns depending on the screen:

```wf
page Layout(path: "/projects", title: "Projects", description: "A grid of projects.") {
    Container {
        Stack(gap: .lg) {
            Row(justify: .between, align: .center) {
                Heading("Projects").h1
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

To make a layout prop change with the screen width, give it one value per
breakpoint instead of one value:

```wf
Grid(columns: { base: 1, md: 2, lg: 3 }, gap: { base: .sm, lg: .lg }) { … }
Row(direction: { base: .column, md: .row }) { … }
Column(span: { base: 12, md: 6 }) { … }
```

The build compiles each step to a class and a media query. No JavaScript
runs and no resize listener is added, so the first paint already has the
right layout. A `viewport` test, by contrast, paints one branch and swaps
it after the page's script starts.

- The steps are `base`, `sm`, `md`, `lg` and `xl`.
- A step you leave out keeps the value of the step below it, so
  `{ base: 1, lg: 3 }` is one column until `lg`.
- Each step is a design token, `screen-sm` to `screen-xl`. Move one in your
  theme and every media query moves with it, `viewport.md` included:

```wf
theme Brand { screen-md: 900px }
```

The same names work inside a `style { }` block: `@md { … }` means
`@media (min-width: $screen-md)`.

```wf
Heading("Title").h1 {
    style {
        font-size: 24px
        @md { font-size: 34px }
    }
}
```

### Make a layout reflow on a narrow screen

A `Row`, `Grid` or `Column` stays as you write it at every width. To make
one reflow on a narrow screen, add the `.stacks` flag, or say what it does
at each width with a responsive value:

```wf
Row.stacks { … }                          // a column on a narrow screen
Grid.stacks { … }                         // one column on a narrow screen
Grid(columns: { base: 1, md: 3 }) { … }   // or say what it does at each width
```

What `.stacks` does:

| Element | Below 1024px | Below 768px |
|---|---|---|
| `Row.stacks` | unchanged | a column |
| `Grid.stacks` | two columns | one column |
| `Column.stacks` | half width, when its span is 3 or 4 | full width |

Older versions of the engine reflowed every `Row` and `Grid` this way below
768px, with `!important`, whether you asked or not. They no longer do.
`wf migrate` adds `.stacks` to every `Row`, `Grid` and `Column` of a
WebFluent 2 project, so the old behaviour carries over, written where you
can see and change it.

Keep `viewport` ([State and reactivity](08-state-and-reactivity.md#the-browser-as-values))
for what changes behaviour, such as a drawer instead of a sidebar. It
follows `matchMedia`, which fires once when the answer changes, not on every
pixel of a drag.

## Find your styles in the build output

The build writes your styles to two places:

- `styles.css` holds the tokens something in the output uses
  (`:root { --color-primary: … }`), the dark theme's overrides, the rules of
  the built-ins you used (and only those), your bundled stylesheets, and the
  compiled `style { }` rules that more than one page reaches.
- `pages/<Name>.css` holds the `style { }` rules that only one page can
  reach. The page loads it after `styles.css`.

[Performance](31-performance.md) shows how to see what each file weighs.

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| `E002`: a `,` after the value of a property | Declarations are separated by a new line or `;`, never `,`. |
| `V06`: a token the theme does not declare | Check the token's spelling against `wf theme`, or declare it in your theme. |
| `V05`: a property CSS does not have | Fix the property name; the warning names the nearest one. |
| `V07`: a bare number spliced as a length | Add the unit: `width: {pct}%`. |
| A hover rule ignores a state value | Nested rules are compiled at build time. Set a custom property from state and read it with `var(--…)` in the rule. |
| Your stylesheet rule loses to an element | The element's `style { }` block is stronger than any stylesheet. Move the rule into the block, or remove the declaration from the block. |
| Part of the page stays light in dark mode | That part uses a raw colour. Replace it with a token. |
| `V04`: a `wf-*` class in `class:` | Give your element its own class instead of a built-in's. |
| A `Row` or `Grid` does not reflow on a phone | Add `.stacks`, or give the prop a responsive value. |

## Next

[Forms and validation](16-forms.md).
