# Sidebar

> Side navigation. A Sidebar.Item whose to matches the current route is marked .active and aria-current="page".

The page: https://webfluent.monzeromer.dev/reference/sidebar

Side navigation. A Sidebar.Item whose to matches the current route is marked .active and aria-current="page".

```wf
Sidebar { … }
```

## Example

```wf
Row {
    style { height: 360px; border: 1px solid $color-border }
    Sidebar {
        style { height: 100% }
        Sidebar.Header { Text("Docs").bold }
        Sidebar.Item(to: "/", icon: "home") { Text("Home") }
        Sidebar.Item(to: "/learn", icon: "book", active: .prefix) { Text("Learn") }
        Sidebar.Item(to: "/reference", icon: "list", active: .prefix) { Text("Reference") }
        Sidebar.Item(to: "/recipes", icon: "layers") { Text("Recipes") }
        Sidebar.Divider
        Sidebar.Item(to: "/playground", icon: "play") { Text("Playground") }
    }
    Stack(gap: .sm) {
        style { flex: 1; padding: 24px }
        Heading("Reference").h3
        Text("Every built-in, with its props and an example.").muted
    }
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Sidebar`.

### Sidebar.Header

The sidebar's heading area.

```wf
Sidebar.Header { … }
```

### Sidebar.Item

One navigation entry.

```wf
Sidebar.Item(to: Path, icon: String, active: Enum) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `to:` | `Path` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `icon:` | `String` | Icon shown before the label |
| `active:` | `.prefix` | How the link decides it is the current one |

Flags: `.prefix`

### Sidebar.Divider

A rule between groups of items.

```wf
Sidebar.Divider
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
