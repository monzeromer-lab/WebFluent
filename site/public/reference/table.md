# Table

> Data table. Cells inside Table.Head render as <th scope="col">; caption is the table's accessible name, visually hidden.

The page: https://webfluent.monzeromer.dev/reference/table

Data table. Cells inside Table.Head render as <th scope="col">; caption is the table's accessible name, visually hidden.

```wf
Table(caption: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `caption:` | `String` | Accessible name of the table (rendered visually hidden) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Table.Table.Head`
- `Table.Table.Body`
- `Table.Table.Row`
- `Table.Table.Cell`

## Example

```wf
Table(caption: "Deploys") {
    Table.Head { Table.Row { Table.Cell("Region")  Table.Cell("Status") } }
    Table.Body { Table.Row { Table.Cell("fra1")  Table.Cell("Healthy") } }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A10](https://webfluent.monzeromer.dev/errors/A10.md): A table with no header row

Before WebFluent 5.0.
