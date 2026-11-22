## Styling

### Style Blocks

```wf
Card {
    style {
        background: #f0f0f0
        border-radius: 1rem
        padding: 2rem
        box-shadow: 0 2px 8px rgba(0,0,0,0.1)
        transition: all 200ms ease
    }
}
```

Style properties use CSS names (hyphenated). A value is raw CSS to the end
of its line or a `;` — no quotes — so anything CSS accepts is written as
CSS: `rgba(0,0,0,0.1)`, `0 4px 12px`, `"Inter", sans-serif`. Two more
things may appear in a value: `$token`, a design token (`$surface`, or the
short name a property's group resolves — `padding: $xl` is `$spacing-xl`),
and `{expr}`, a splice that reads state. A value that reads state follows
it: `style { width: {pct}% }` or `style { background: {tone} }` repaints
whenever `pct` or `tone` changes; a value without a splice compiles to a
stylesheet rule the element carries by class.

### Style in PDF and slides

A PDF or a deck is laid out from the same HTML and CSS the page would
get, so a `style { }` block, a theme and the project's `.css` files mean
there what they mean on the web — every property the engine draws, on any
element (see [PDF Output](#pdf-output)). `@media print { }` applies to
paper only; a rule only a screen has (`:hover`, a width query) does not.

### Responsive values

Any layout prop takes one value per breakpoint. It compiles to a class and
a media query — no JavaScript, no resize listener, and the right layout in
the very first paint:

```wf
Grid(columns: { base: 1, md: 2, lg: 3 }, gap: { base: .sm, lg: .lg }) { … }
Row(direction: { base: .column, md: .row }) { … }
Column(span: { base: 12, md: 6 }) { … }
```

The steps are `base`, `sm`, `md`, `lg`, `xl`, and each is a **design
token** — `screen-sm` … `screen-xl` — so a theme moves them and every
media query, and `viewport.md`, move with it. A step a value omits keeps
whatever the step below it said.

The same names are sugar inside a `style { }` block: `@md { … }` is
`@media (min-width: $screen-md)`.

```wf
Heading("Title").h1 {
    style {
        font-size: 24px
        @md { font-size: 34px }
    }
}
```

**A layout stays as it is written.** The engine used to reflow every `Row`
into a column and every `Grid` into one column below 768px, with
`!important` — which took twelve overrides and five `!important`s to undo
on one site. It does not any more. A layout that should reflow says so:

```wf
Row.stacks { … }                              // a column on a narrow screen
Grid(columns: { base: 1, md: 3 }) { … }       // or say what it does at each width
```

`wf migrate` adds `.stacks` to every `Row`, `Grid` and `Column` of a
WebFluent 2 project, so the old behaviour carries over, written down.

`viewport` stays for what is genuinely behavioural — a drawer instead of a
sidebar — and follows `matchMedia`, which fires once when the answer
changes, rather than on every pixel of a drag.

### Responsive Styles with @media

Style blocks support `@media` queries for responsive design:

```wf
Sidebar {
    style {
        width: 260px
        position: fixed

        @media (max-width: 768px) {
            display: none
        }
    }
}

Heading("Title").h1 {
    style {
        font-size: 3rem

        @media (max-width: 768px) {
            font-size: 1.5rem
        }
    }
}
```

A `@media` block is compiled into `styles.css` under a class named by its
content, and the element carries the class — so a hundred identical cards
share one rule, and the static paint has it before JavaScript runs. Values
inside it take `$tokens` like any other style value.

### Nested rules

What an inline style cannot say — how an element looks while hovered,
focused, pressed or disabled — is a nested rule in the same `style { }`,
in CSS nesting syntax: `&` is the element.

```wf
Button("Publish changes") {
    style {
        background: $color-primary
        color: $color-surface
        &:hover { background: $color-secondary }
        &:active { transform: translateY(1px) }
        &:focus-visible { outline: 2px solid $color-primary; outline-offset: 2px }
        &:disabled { opacity: 0.45 }
    }
}
Input(placeholder: "shop.example.com").text {
    style { &::placeholder { color: $color-text-muted } }
}
Link("Home", to: "/") {
    style { &[aria-current="page"] { font-weight: 600 } }
}
```

Any selector CSS nesting allows: `&:hover`, `&:focus-visible` (the keyboard
focus ring, not a ring on every click), `&:active`, `&:disabled`,
`&::placeholder`, `&:focus-within`, and the ARIA states a control keeps in
its attributes — `&[aria-current="page"]`, which the router sets on a
`Link`; `&[aria-pressed="true"]`, `&[aria-selected="true"]`,
`&[aria-checked="true"]`, `&[aria-expanded="true"]`,
`&[aria-invalid="true"]` — so the rule keys off the attribute assistive
technology reads. These are stylesheet rules, so their values must be
known at build time: literals and tokens, not state. They override the
element's base declarations while the state or media condition holds (they
are emitted `!important`, since the base is inline); when a state rule and a
media rule set the same property, the media rule wins.

A rule that has to vary per element reads a custom property the element sets
— a custom property can follow state, the rule cannot:

```wf
Button(label) {
    style {
        --hover-bg: {hoverColor}  // state, prop or derived
        &:hover { background: var(--hover-bg) }
    }
}
```

A custom property of the element's own is read with `var(--name)`; `$name`
is reserved for design tokens.

### Stylesheets

What no `style { }` block can say — a selector over several elements, a
`@keyframes`, a `@font-face`, a class a dozen elements share by name, a rule
on `html` or `::selection` — goes in a `.css` file, and any `.css` file under
`src/` is part of the build:

```
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
  color: var(--text-secondary);
}
```

```wf
Card(class: "feature feature--first") { ... }
Card(class: tone) { ... }        // a state: the classes follow it
```

The files are read in path order and bundled into `styles.css` after the
engine's rules (so on equal specificity yours win) and before the rules that
`style { }` blocks compile to (which carry tripled specificity, as the
inline styles they replace beat any sheet). They are minified with the rest.
`var(--token)` reaches every design token the theme declares. There is no
scoping: a class is global, and named on purpose.

`meta.stylesheets` in the config still links a sheet that is *not* built —
a file in `public/` or a URL — ahead of `styles.css`.


A `.js` file under `src/` is part of the build the same way — see
[Your own JavaScript](#your-own-javascript).

### Themes

A theme is written in WebFluent, in your own `src/`:

```wf
theme Brand {
    color-primary: #3B82F6
    color-secondary: #8B5CF6
    font-family: Inter, sans-serif
    radius-md: 0.5rem
}
```

Every token you do not name keeps its baseline value, so a theme is only as
large as the difference you want. A theme may also declare tokens of its own
(`surface-raised: #131519`, `viz-1: #ff6a2b`); every token becomes a custom
property on `:root`, usable from any style block as `$surface-raised` and
from a stylesheet as `var(--surface-raised)`. Declare one and it is used
automatically; declare several and pick one with `"theme": { "name":
"Brand" }`.

Every built-in draws from these tokens alone, so a theme restyles all of
them, new ones included. What is written on a tone is a token too —
`color-on-primary` (and `-secondary`, `-success`, `-danger`, `-warning`,
`-info`) is a primary button's, badge's, pressed toggle's or selected day's
label — so a theme with a light primary names a dark one (`A13` checks the
pair). Alerts, focus rings and highlights are mixed from the tone and the
page's colours, so a dark theme needs nothing more; `color-backdrop` dims
the page behind a modal, a dialog, a sheet and a drawer. A `Modal` or
`Dialog` is as wide as its `<dialog>`, so `class:` or `style { width: … }`
on it sizes what it shows.

The baseline holds a stacking scale — `z-sticky`, `z-drawer`, `z-dropdown`,
`z-modal`, `z-toast`, `z-tooltip`, `z-skip-link` — for a page's own sticky
header or drawer to sit where it should against the built-ins (modals,
popovers, menus and tooltips are in the browser's top layer and need none),
and the least a control reaches: `target-min` (24px, WCAG 2.2's floor) and
`target-min-touch` (44px, where the pointer is a finger). Every built-in
control is at least that size; `style { min-block-size: $target-min }` gives a
control of your own the same.

**Dark mode.** Declare a second theme with the tokens that change and name
it in the config: `"theme": { "name": "Brand", "dark": "Night" }`. Its
tokens apply under `prefers-color-scheme: dark`, and whenever the reader
chose — `setTheme("dark")`, `setTheme("light")`, `setTheme("system")`,
kept across visits; `theme` reads the choice.

```wf
theme Night { color-background: #0B1220  color-text: #E5E7EB  color-text-muted: #9CA3AF  color-surface: #111827 }
```

**Keyframes.** `animation Name { from { … } 50% { … } to { … } }` declares
keyframes: play them with `animate: .Name` or `exit: .Name` on any element,
or write `animation: Name 1s infinite` in a style. `@container (…) { }`
inside a style block is a container query, passed through as written.

The baseline names system fonts on purpose. A theme that names a web font
lists where to fetch it, and every page links it ahead of `styles.css` with a
`preconnect` to its origin:

```json
{ "meta": { "fonts": ["https://fonts.googleapis.com/css2?family=Manrope:wght@400..800&display=swap"] } }
```

`meta.stylesheets` links extra stylesheets the same way — a file in `public/`
by site-relative path, or a URL. It is for the few things no element-level
`style { }` can say (`html { background }`, `::selection`), not for component
styling, which belongs in `.wf` source.

For values a machine supplies (a deploy pipeline injecting a brand colour),
`theme.tokens` in `webfluent.app.json` still applies, on top of the theme:

```json
{ "theme": { "tokens": { "color-primary": "#8B5CF6" } } }
```

