# Item

> A row of a list, a settings page or a search's results: Item.Media at its start, its title: and description:, then its block, and Item.Actions at its end.

The page: https://webfluent.monzeromer.dev/reference/item

A row of a list, a settings page or a search's results: Item.Media at its start, its title: and description:, then its block, and Item.Actions at its end.

```wf
Item(title: String, description: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `title:` | `String` | What it is |
| `description:` | `String` | A line more, muted |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Item.Item.Media`
- `Item.Item.Content`
- `Item.Item.Actions`

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
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
