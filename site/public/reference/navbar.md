# Navbar

> Top navigation bar with brand, links and action areas.

The page: https://webfluent.monzeromer.dev/reference/navbar

Top navigation bar with brand, links and action areas.

```wf
Navbar { … }
```

## Example

```wf
Navbar {
    Navbar.Brand { Link("WebFluent", to: "/") }
    Navbar.Links {
        Link("Learn", to: "/learn", active: .prefix)
        Link("Reference", to: "/reference", active: .prefix)
        Link("Recipes", to: "/recipes")
    }
    Navbar.Actions {
        ThemeToggle
        Button("Try it").primary.sm { on click { navigate("/playground") } }
    }
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Navbar`.

### Navbar.Brand

The brand area, usually a link home.

```wf
Navbar.Brand { … }
```

### Navbar.Links

The navigation links; collapses behind a toggle on small screens.

```wf
Navbar.Links { … }
```

### Navbar.Actions

Buttons at the end of the bar.

```wf
Navbar.Actions { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
