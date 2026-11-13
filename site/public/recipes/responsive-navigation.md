# Make navigation responsive

> Show navigation links on a wide screen and a menu button on a narrow one, by branching on viewport.

The page: https://webfluent.monzeromer.dev/recipes/responsive-navigation

A row of links fits a laptop but not a phone. To show the links on a wide
screen and a menu button that opens them on a narrow one, branch on
`viewport`:

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
        IconButton(icon: "menu", label: "Open navigation") { on click { open = !open } }
        show open {
            Stack(gap: .sm, animate: .slideDown) {
                Link("Home", to: "/")
                Link("Docs", to: "/docs")
                Link("Pricing", to: "/pricing")
            }
        }
    }
}
```

## How it works

- [`viewport.md`](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#the-browser-as-values)
  is true while the window is at least the `md` breakpoint wide. The
  runtime updates it when a breakpoint is crossed, so the `if` swaps the
  row for the button as the window narrows.
- On a narrow screen, the `IconButton` toggles `open`. Its `label:` is the
  button's accessible name, since the icon has no text.
- `show open` keeps the links in the page and hides them while `open` is
  false. `animate: .slideDown` plays as they appear.

The pre-rendered page does not know the window's width, so what depends on
`viewport` is drawn once the page is live. For layout alone — a row that
becomes a column — prefer
[responsive values](https://webfluent.monzeromer.dev/learn/13-styling.md#responsive-values), which are
right in the first paint.
