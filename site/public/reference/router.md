# Router

> Where the current page renders. May sit at any depth inside app; pages declare their own paths.

The page: https://webfluent.monzeromer.dev/reference/router

Where the current page renders. May sit at any depth inside app; pages declare their own paths.

```wf
Router(transition: Enum, duration: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `transition:` | `.none .fade .slide` | How a route change moves between pages |
| `duration:` | `String` | How long each half of the transition plays, e.g. `200ms` |

## Flags

`.none` `.fade` `.slide` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
app {
    Navbar { Navbar.Brand { Link("Acme", to: "/") } }
    Router(transition: .fade, duration: "200ms")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [R03](https://webfluent.monzeromer.dev/errors/R03.md): An app with no Router, or with two

Since WebFluent 5.0.
