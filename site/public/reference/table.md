# Table

> Data table. Cells inside Table.Head render as <th scope="col">; caption is the table's accessible name, visually hidden.

The page: https://webfluent.monzeromer.dev/reference/table

Data table. Cells inside Table.Head render as <th scope="col">; caption is the table's accessible name, visually hidden.

```wf
Table(caption: String) { … }
```

## Example

```wf
state regions = [{ name: "fra1", latency: 42, status: "Healthy" }, { name: "iad1", latency: 88, status: "Healthy" }, { name: "sin1", latency: 190, status: "Degraded" }]
Table(caption: "Regions") {
    Table.Head {
        Table.Row { Table.Cell("Region")  Table.Cell("Latency")  Table.Cell("Status") }
    }
    Table.Body {
        for r in regions by r.name {
            Table.Row {
                Table.Cell(r.name).header
                Table.Cell("{r.latency} ms")
                Table.Cell { Badge(r.status, tone: if r.status == "Healthy" { .success } else { .warning }) }
            }
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `caption:` | `String` | The table's accessible name, drawn as a `<caption>` that is visually hidden |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Table`.

### Table.Head

Header section; its cells become <th scope="col">.

```wf
Table.Head { … }
```

### Table.Body

The body of a table: its rows of data.

```wf
Table.Body { … }
```

### Table.Row

A row of cells.

```wf
Table.Row { … }
```

### Table.Cell

A cell. header makes it a <th> anywhere, as Table.Head does for its cells.

```wf
Table.Cell(_ content: String or Number or Bool, header: Bool) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `content` | `String or Number or Bool (positional)` | The cell's text: a string, number or boolean; unset, its block |
| `header:` | `Bool` | Drawn as a header cell, `<th scope="col">`, outside a `Table.Head` too (a cell a component draws). Written as a flag, `.header`, or `header: true`. |

Flags: `.header`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A10](https://webfluent.monzeromer.dev/errors/A10.md): A table with no header row

Before WebFluent 5.0.
