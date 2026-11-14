# DescriptionList

> Terms and what they are (<dl>): a column of pairs, or .grid, each term beside its value.

The page: https://webfluent.monzeromer.dev/reference/descriptionlist

Terms and what they are (<dl>): a column of pairs, or .grid, each term beside its value.

```wf
DescriptionList(grid: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `grid:` | `Bool` | Each term beside its value |

## Flags

`.grid`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `DescriptionList.DescriptionList.Item`
- `DescriptionList.DescriptionList.Term`
- `DescriptionList.DescriptionList.Details`

## Example

```wf
DescriptionList.grid {
    DescriptionList.Item("Region", value: "eu-west")
    DescriptionList.Item("Status") { Badge("Live").success }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
