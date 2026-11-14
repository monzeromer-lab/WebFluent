# Make navigation responsive

> Show navigation links on a wide screen and a menu button on a narrow one, by branching on viewport.

The page: https://webfluent.monzeromer.dev/recipes/responsive-navigation

A row of links fits a laptop but not a phone. To show the links on a wide
screen and a menu button that opens them on a narrow one, branch on
`viewport`:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
page Nav(path: "/", title: "Home", description: "Navigation that fits the screen.") {
    state open = false
    Heading("Home").h1
    if viewport.md {
        Row(gap: .md) {
            Link("Home", to: "/")
            Link("Docs", to: "/docs")
            Link("Pricing", to: "/pricing")
        }
    } else {
        IconButton(icon: "menu", label: "Open navigation", aria-expanded: open, aria-controls: "nav-menu") {
            on click { open = !open }
        }
        show open {
            Stack(gap: .sm, animate: .slideDown, id: "nav-menu") {
                Link("Home", to: "/")
                Link("Docs", to: "/docs")
                Link("Pricing", to: "/pricing")
            }
        }
    }
}

page Docs(path: "/docs", title: "Docs", description: "How it works.") { Heading("Docs").h1 }
page Pricing(path: "/pricing", title: "Pricing", description: "What it costs.") { Heading("Pricing").h1 }
```

## How it works

- [`viewport.md`](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#the-browser-as-values)
  is true while the window is at least the `md` breakpoint wide. The
  runtime updates it when a breakpoint is crossed, so the `if` swaps the
  row for the button as the window narrows.
- On a narrow screen, the `IconButton` toggles `open`. Its `label:` is the
  button's accessible name, since the icon has no text, and
  `aria-expanded: open` tells a screen reader whether the menu it controls
  (`aria-controls`, the `Stack`'s `id`) is open.
- `show open` keeps the links in the page and hides them while `open` is
  false. `animate: .slideDown` plays as they appear.

The pre-rendered page does not know the window's width, so what depends on
`viewport` is drawn once the page is live. For layout alone — a row that
becomes a column — prefer
[responsive values](https://webfluent.monzeromer.dev/learn/13-styling.md#responsive-values), which are
right in the first paint.

`Docs` and `Pricing` are there so every link lands on a page; in a site of
your own the links name the pages you have, and the navigation usually
lives in `app`, around the `Router`, so every page shows it.
