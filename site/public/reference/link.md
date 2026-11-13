# Link

> Client-side navigation link; its label or its block is the link text.

The page: https://webfluent.monzeromer.dev/reference/link

Client-side navigation link; its label or its block is the link text.

```wf
Link(_ label: String, to: Path, active: Enum) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The link text |
| `to:` | `Path` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `active:` | `.prefix` | How the link decides it is the current one |

## Flags

`.prefix` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Link("About us", to: "/about")
Link("Guide", to: "/docs", active: .prefix)
Link(to: "https://example.com", target: "_blank") { Text("Example") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E107](https://webfluent.monzeromer.dev/errors/E107.md): A URL a browser would run
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [R01](https://webfluent.monzeromer.dev/errors/R01.md): A route to nothing
- [A06](https://webfluent.monzeromer.dev/errors/A06.md): A link with no text

Since WebFluent 5.0.
