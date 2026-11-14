# BottomNavigation.Item

> One entry: its words, where it goes, and its icon.

The page: https://webfluent.monzeromer.dev/reference/bottomnavigation-item

One entry: its words, where it goes, and its icon.

```wf
BottomNavigation.Item(_ label: String, to: Path, active: Enum, icon: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `to:` | `Path  (required)` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `active:` | `.prefix` | How the link decides it is the current one |
| `icon:` | `String` | The icon over its words |

## Flags

`.prefix`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
BottomNavigation(label: "Sections") {
    BottomNavigation.Item("Home", to: "/", icon: "home")
    BottomNavigation.Item("Search", to: "/", icon: "search")
    BottomNavigation.Item("Me", to: "/", icon: "user")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
