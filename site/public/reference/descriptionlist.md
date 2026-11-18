# DescriptionList

> Terms and what they are (<dl>): a column of pairs, or .grid, each term beside its value.

The page: https://webfluent.monzeromer.dev/reference/descriptionlist

Terms and what they are (<dl>): a column of pairs, or .grid, each term beside its value.

```wf
DescriptionList(grid: Bool) { … }
```

## Example

```wf
DescriptionList.grid {
    DescriptionList.Item("Service", value: "checkout-api")
    DescriptionList.Item("Region", value: "eu-west-1")
    DescriptionList.Item("Replicas", value: 3)
    DescriptionList.Item("Status") { Badge("Live").success }
    DescriptionList.Item("Last deploy") { RelativeTime(@2026-11-14T16:05Z) }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `grid:` | `Bool` | Each term beside its value, in two columns (the terms as wide as the widest); unset, each term above its value. Written as a flag, `.grid`, or `grid: true`. |

## Flags

`.grid`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `DescriptionList`.

### DescriptionList.Item

One pair: its term, and value: or its block.

```wf
DescriptionList.Item(_ term: String, value: String or Number or Bool) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `term` | `String (positional)` | What it is: the term (`<dt>`) |
| `value:` | `String or Number or Bool` | Its value, shown as text in the `<dd>`; unset, the item's block is the value |

## Drawn for it

What `DescriptionList` draws itself, by its class; not written.

- `DescriptionList.Term` (`.`): A term.
- `DescriptionList.Details` (`.`): A value.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
