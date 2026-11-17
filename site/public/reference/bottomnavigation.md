# BottomNavigation

> A phone's tab bar along the bottom of the screen: each entry an icon over its words, at least 44px to press, clear of the home indicator (the safe-area inset); the current route's is marked.

The page: https://webfluent.monzeromer.dev/reference/bottomnavigation

A phone's tab bar along the bottom of the screen: each entry an icon over its words, at least 44px to press, clear of the home indicator (the safe-area inset); the current route's is marked.

```wf
BottomNavigation(label: String) { … }
```

## Example

```wf
Stack(gap: .sm) {
    style { width: 360px; height: 240px; padding: 16px; contain: layout; overflow: hidden; border: 1px solid $color-border; border-radius: 24px }
    Heading("Inbox").h3
    Text("Three new replies since this morning.").muted
    BottomNavigation(label: "Sections") {
        BottomNavigation.Item("Home", to: "/", icon: "home")
        BottomNavigation.Item("Learn", to: "/learn", icon: "book").prefix
        BottomNavigation.Item("Reference", to: "/reference", icon: "list").prefix
        BottomNavigation.Item("Play", to: "/playground", icon: "play")
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The bar's name for a screen reader (unset, `Main`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `BottomNavigation`.

### BottomNavigation.Item

One entry: its words, where it goes, and its icon.

```wf
BottomNavigation.Item(_ label: String, to: Path, active: Enum, icon: String)
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `to:` | `Path  (required)` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `active:` | `.prefix` | How the link decides it is the current one |
| `icon:` | `String` | The icon over its words |

Flags: `.prefix`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
