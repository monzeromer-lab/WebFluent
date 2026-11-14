# BottomNavigation

> A phone's tab bar along the bottom of the screen: each entry an icon over its words, at least 44px to press, clear of the home indicator (the safe-area inset); the current route's is marked.

The page: https://webfluent.monzeromer.dev/reference/bottomnavigation

A phone's tab bar along the bottom of the screen: each entry an icon over its words, at least 44px to press, clear of the home indicator (the safe-area inset); the current route's is marked.

```wf
BottomNavigation(label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The bar's name for a screen reader (unset, `Main`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `BottomNavigation.BottomNavigation.Item`

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
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
