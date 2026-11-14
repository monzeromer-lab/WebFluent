# DescriptionList.Details

> A value. Drawn for it.

The page: https://webfluent.monzeromer.dev/reference/descriptionlist-details

A value. Drawn for it.

```wf
DescriptionList.Details(_ text: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The value |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
DescriptionList.grid {
    DescriptionList.Item("Region", value: "eu-west")
    DescriptionList.Item("Status") { Badge("Live").success }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
