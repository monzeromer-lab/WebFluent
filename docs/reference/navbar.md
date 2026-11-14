# Navbar

> Top navigation bar with brand, links and action areas.

The page: https://webfluent.monzeromer.dev/reference/navbar

Top navigation bar with brand, links and action areas.

```wf
Navbar { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Navbar.Navbar.Brand`
- `Navbar.Navbar.Links`
- `Navbar.Navbar.Actions`

## Example

```wf
Navbar {
    Navbar.Brand { Link("Acme", to: "/") }
    Navbar.Links { Link("Docs", to: "/docs")  Link("Pricing", to: "/pricing") }
    Navbar.Actions { Button("Sign in").sm }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
