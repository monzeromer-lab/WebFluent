# DataTable.Column

> One column: its title, its value: (what it shows and sorts by), and a cell(r) { … } that draws it instead.

The page: https://webfluent.monzeromer.dev/reference/datatable-column

One column: its title, its value: (what it shows and sorts by), and a cell(r) { … } that draws it instead.

```wf
DataTable.Column(_ title: String, value: Any, sortable: Bool)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String  (positional)` | Its header |
| `value:` | `Any` | What the column holds for a row: `r => r.service` |
| `sortable:` | `Bool` | Its header sorts the rows by its `value:` |

## Flags

`.sortable`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state deploys = [{ id: "1", service: "web", status: "live" }, { id: "2", service: "api", status: "failed" }]
state chosen: [String] = []
DataTable(rows: deploys, key: d => d.id, bind: chosen, label: "Deploys").multiple {
    empty { EmptyState(icon: "list", title: "No deploys yet") }
    DataTable.Column("Service", value: d => d.service).sortable
    DataTable.Column("Status") { cell(d) { Badge(d.status) } }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
