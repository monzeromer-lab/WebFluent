# NavigationMenu.Link

> An entry that is a link, beside the ones that open panels.

The page: https://webfluent.monzeromer.dev/reference/navigationmenu-link

An entry that is a link, beside the ones that open panels.

```wf
NavigationMenu.Link(_ label: String, to: Path, active: Enum)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The link's words |
| `to:` | `Path  (required)` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `active:` | `.prefix` | How the link decides it is the current one |

## Flags

`.prefix`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
