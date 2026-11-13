# Table.Cell

> A cell. header makes it a <th> anywhere, as Table.Head does for its cells.

The page: https://webfluent.monzeromer.dev/reference/table-cell

A cell. header makes it a <th> anywhere, as Table.Head does for its cells.

```wf
Table.Cell(_ content: Any, header: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `Any  (positional)` | The cell's content |
| `header:` | `Bool` | Render as a `<th>` |

## Flags

`.header` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Table(caption: "Deploys") {
    Table.Head { Table.Row { Table.Cell("Region")  Table.Cell("Status") } }
    Table.Body { Table.Row { Table.Cell("fra1")  Table.Cell("Healthy") } }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.0.
