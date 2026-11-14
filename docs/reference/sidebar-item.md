# Sidebar.Item

> One navigation entry.

The page: https://webfluent.monzeromer.dev/reference/sidebar-item

One navigation entry.

```wf
Sidebar.Item(to: Path, icon: String, active: Enum) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `to:` | `Path` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `icon:` | `String` | Icon shown before the label |
| `active:` | `.prefix` | How the link decides it is the current one |

## Flags

`.prefix`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on click`: The element was clicked or activated

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

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

Before WebFluent 5.0.
