# Sidebar

> Side navigation. A Sidebar.Item whose to matches the current route is marked .active and aria-current="page".

The page: https://webfluent.monzeromer.dev/reference/sidebar

Side navigation. A Sidebar.Item whose to matches the current route is marked .active and aria-current="page".

```wf
Sidebar { … }
```

## Flags

`.dropInside` `.trapFocus`

## Slots

- `children`

## Parts

- `Sidebar.Sidebar.Header`
- `Sidebar.Sidebar.Item`
- `Sidebar.Sidebar.Divider`

## Example

```wf
Sidebar {
    Sidebar.Header { Text("Console").bold }
    Sidebar.Item(to: "/", icon: "home") { Text("Overview") }
    Sidebar.Divider
    Sidebar.Item(to: "/settings", icon: "settings") { Text("Settings") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
