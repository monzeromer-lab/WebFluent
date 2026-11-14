# DataTable

> Rows of records with their columns declared once: a column sorts when it is .sortable (its header a button, aria-sort saying which way), cell(r) { … } draws a cell, empty { … } shows when there are no rows, and bind: chooses rows (a list with .multiple, a box for each and one for all).

The page: https://webfluent.monzeromer.dev/reference/datatable

Rows of records with their columns declared once: a column sorts when it is .sortable (its header a button, aria-sort saying which way), cell(r) { … } draws a cell, empty { … } shows when there are no rows, and bind: chooses rows (a list with .multiple, a box for each and one for all).

```wf
DataTable(rows: Any, key: Any, bind: State, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `rows:` | `Any  (required)` | The records |
| `key:` | `Any  (required)` | What tells two rows apart: `r => r.id` |
| `bind:` | `State` | The chosen row's key, or a list of them with `.multiple` |
| `multiple:` | `Bool` | Several rows at once |
| `label:` | `String  (required)` | What the table is: its caption, read by a screen reader |

## Flags

`.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `DataTable.DataTable.Column`

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
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
