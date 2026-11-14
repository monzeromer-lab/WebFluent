# Item.Content

> Its title, description and block. Drawn for it.

The page: https://webfluent.monzeromer.dev/reference/item-content

Its title, description and block. Drawn for it.

```wf
Item.Content { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state on = true
Item(title: "Notifications", description: "Sent to your phone") {
    Item.Media { Icon("bell") }
    Item.Actions { Switch(bind: on, label: "On") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
