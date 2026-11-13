# Sidebar.Divider

> A rule between groups of items.

The page: https://webfluent.monzeromer.dev/reference/sidebar-divider

A rule between groups of items.

```wf
Sidebar.Divider
```

## Flags

`.dropInside` `.trapFocus`

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
