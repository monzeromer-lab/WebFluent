# Router

> Where the current page renders. May sit at any depth inside app; pages declare their own paths.

The page: https://webfluent.monzeromer.dev/reference/router

Where the current page renders. May sit at any depth inside app; pages declare their own paths.

```wf
Router(transition: Enum, duration: String) { … }
```

## Example

```wf
Stack {
    Navbar {
        Navbar.Brand { Link("Ledger", to: "/") }
        Navbar.Links {
            Link("Learn", to: "/learn")
            Link("Reference", to: "/reference")
        }
    }
    Router(transition: .fade, duration: "200ms")
    Text("© 2026 Ledger").muted.sm
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `transition:` | `one of its cases` | How a route change moves between pages, through the browser's View Transitions where it has them; none for a reader who asks for less motion. Written as a flag, `.none`, or by name, `transition: .none`.

- `.none` (the default): The next page replaces the last at once
- `.fade`: The last page fades out, the next fades in
- `.slide`: The last page slides out to the left, the next in from the right |
| `duration:` | `String` | How long each half of the transition plays, a CSS time, `200ms` |

## Flags

`.none` `.fade` `.slide`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [R03](https://webfluent.monzeromer.dev/errors/R03.md): An app with no Router, or with two

Before WebFluent 5.0.
