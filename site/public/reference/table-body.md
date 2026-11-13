# Table.Body

> Body section.

The page: https://webfluent.monzeromer.dev/reference/table-body

Body section.

```wf
Table.Body { … }
```

## Flags

`.dropInside` `.trapFocus`

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
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
