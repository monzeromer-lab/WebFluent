# 12. Styling

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

The baseline tokens, by group:

| Group | Tokens |
|---|---|
| Color | `color-primary`, `color-secondary`, `color-success`, `color-danger`, `color-warning`, `color-info`, `color-background`, `color-surface`, `color-text`, `color-text-muted`, `color-border` |
| Font | `font-family`, `font-family-mono`, `font-size-xs` … `font-size-3xl`, `font-weight-normal/medium/bold`, `line-height-tight/normal/loose` |
| Spacing | `spacing-xs`, `spacing-sm`, `spacing-md`, `spacing-lg`, `spacing-xl`, `spacing-2xl`, `spacing-3xl` |
| Radius | `radius-none`, `radius-sm`, `radius-md`, `radius-lg`, `radius-xl`, `radius-full` |
| Shadow | `shadow-none`, `shadow-sm`, `shadow-md`, `shadow-lg`, `shadow-xl` |
| Motion | `transition-fast`, `transition-normal`, `transition-slow` |

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
  (`$color-primary`, `$surface-raised`) works anywhere. A token the theme
  does not have is an error.
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
readers get none ([chapter 13](13-motion.md)).

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

## Enum props as hooks

A component's enum props are written to its root element as `data-<prop>`,
so one stylesheet rule styles every variant ([chapter 8](08-components.md#enum-props)):

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
`Spacer` and `Divider` with their `gap:`, `align:`, `justify:`,
`columns:` props ([chapter 4](04-elements.md#layout)), plus the responsive
`viewport` values ([chapter 5](05-state-and-reactivity.md#the-browser-as-values)).

```wf
page Layout(path: "/") {
    Container {
        Stack(gap: .lg) {
            Row(justify: .between, align: .center) {
                Heading("Projects").h2
                Button("New").primary.sm
            }
            Grid(columns: if viewport.md { 3 } else { 1 }, gap: .md) {
                for n in 1..=6 { Card { Text("Project {n}") } }
            }
        }
    }
}
```

## What the build emits

`styles.css` holds the tokens (`:root { --color-primary: … }`), the dark
overrides, the structural and baseline CSS of the built-ins you used (and
only those), and your bundled stylesheets. Each page's own `style` blocks go
to `pages/<Name>.css`, loaded with the page. Unused built-in CSS is pruned.

## Next

[Motion](13-motion.md).
