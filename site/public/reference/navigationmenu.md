# NavigationMenu

> A site's main navigation whose entries open panels (a mega menu) as disclosures: one open at a time, the arrow keys along the row, Down into a panel, Escape, a press outside or the focus leaving closes it.

The page: https://webfluent.monzeromer.dev/reference/navigationmenu

A site's main navigation whose entries open panels (a mega menu) as disclosures: one open at a time, the arrow keys along the row, Down into a panel, Escape, a press outside or the focus leaving closes it.

```wf
NavigationMenu(label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The navigation's name for a screen reader (unset, `Main`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `NavigationMenu.NavigationMenu.Item`
- `NavigationMenu.NavigationMenu.Link`
- `NavigationMenu.NavigationMenu.List`
- `NavigationMenu.NavigationMenu.Trigger`
- `NavigationMenu.NavigationMenu.Panel`

## Example

```wf
NavigationMenu(label: "Main") {
    NavigationMenu.Item("Products") {
        Link("Editor", to: "/")
        Link("Compiler", to: "/")
    }
    NavigationMenu.Link("Pricing", to: "/")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
