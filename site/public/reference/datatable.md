# DataTable

> Rows of records with their columns declared once: a column sorts when it is .sortable (its header a button, aria-sort saying which way), cell(r) { … } draws a cell, empty { … } shows when there are no rows, and bind: chooses rows (a list with .multiple, a box for each and one for all).

The page: https://webfluent.monzeromer.dev/reference/datatable

Rows of records with their columns declared once: a column sorts when it is .sortable (its header a button, aria-sort saying which way), cell(r) { … } draws a cell, empty { … } shows when there are no rows, and bind: chooses rows (a list with .multiple, a box for each and one for all).

```wf
DataTable(rows: [Item], key: (Item) -> String or (Item) -> Number, bind: State, …) { … }
```

## Example

```wf
state deploys = [{ id: "d1", service: "checkout", region: "fra1", minutes: 4, status: "Live" }, { id: "d2", service: "search", region: "iad1", minutes: 11, status: "Failed" }, { id: "d3", service: "accounts", region: "sin1", minutes: 7, status: "Live" }]
state chosen: [String] = []
DataTable(rows: deploys, key: d => d.id, bind: chosen, label: "Deploys").multiple {
    DataTable.Column("Service", value: d => d.service).sortable
    DataTable.Column("Region", value: d => d.region)
    DataTable.Column("Minutes", value: d => d.minutes).sortable
    DataTable.Column("Status") {
        cell(d) { Badge(d.status, tone: if d.status == "Live" { .success } else { .danger }) }
    }
}
Text("{chosen.length} chosen").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `rows:` | `[Item]  (required)` | The records |
| `key:` | `(Item) -> String or (Item) -> Number  (required)` | What tells two rows apart: `r => r.id` |
| `bind:` | `State` | The chosen row's key, or a list of them with `.multiple` |
| `multiple:` | `Bool` | Several rows at once |
| `label:` | `String  (required)` | What the table is: its caption, read by a screen reader |
| `filter:` | `String` | Only the rows whose columns hold this text, its case ignored |
| `pageSize:` | `Number` | Rows a page: the rest are on pages behind Previous and Next |

## Flags

`.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `DataTable`.

### DataTable.Column

One column: its title, its value: (what it shows and sorts by), and a cell(r) { … } that draws it instead.

```wf
DataTable.Column(_ title: String, value: (Item) -> String or (Item) -> Number or (Item) -> Bool or (Item) -> Money, sortable: Bool)
```

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String  (positional)` | Its header |
| `value:` | `(Item) -> String or (Item) -> Number or (Item) -> Bool or (Item) -> Money` | What the column holds for a row: `r => r.service` |
| `sortable:` | `Bool` | Its header sorts the rows by its `value:` |

Flags: `.sortable`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
