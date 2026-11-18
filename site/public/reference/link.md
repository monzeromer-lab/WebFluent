# Link

> Client-side navigation link; its label or its block is the link text.

The page: https://webfluent.monzeromer.dev/reference/link

Client-side navigation link; its label or its block is the link text.

```wf
Link(_ label: String, to: Path, active: Enum) { … }
```

## Example

```wf
Stack(gap: .sm) {
    Link("Start the guide", to: "/learn")
    Link("Every component", to: "/reference", active: .prefix)
    Link(to: "/recipes") {
        Icon("book")
        Text("Recipes for common screens")
    }
    Link("The source on GitHub", to: "https://github.com/webfluent/webfluent", target: "_blank")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | The link's words; a block written after it may be its content instead |
| `to:` | `Path` | The route or address it goes to, navigated in place; while it matches the current route the link carries `.active` and `aria-current="page"` |
| `active:` | `one of its cases` | How it decides it is the current page: unset, the route must equal `to`; `.prefix` also matches routes beneath it. Written as a flag, `.prefix`, or by name, `active: .prefix`.

- `.prefix`: Also current on every route beneath `to`: `/docs` for `/docs/forms` |

## Flags

`.prefix`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on click`: The element was clicked or activated

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E107](https://webfluent.monzeromer.dev/errors/E107.md): A URL a browser would run
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [R01](https://webfluent.monzeromer.dev/errors/R01.md): A route to nothing
- [A06](https://webfluent.monzeromer.dev/errors/A06.md): A link with no text

Before WebFluent 5.0.
