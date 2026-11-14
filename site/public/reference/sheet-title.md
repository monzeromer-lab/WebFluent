# Sheet.Title

> What the sheet's heading draws, in place of its title:'s words, which stay its name.

The page: https://webfluent.monzeromer.dev/reference/sheet-title

What the sheet's heading draws, in place of its title:'s words, which stay its name.

```wf
Sheet.Title { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state filters = false
state sort = "new"
Button("Filters") { on click { filters = true } }
Sheet(visible: filters, title: "Filters", snap: [0.5, 1]).closable {
    Radio(bind: sort, value: "new", label: "Newest")
    Radio(bind: sort, value: "price", label: "Price")
    Sheet.Footer { Button("Show results").primary { on click { filters = false } } }
}
state details = false
Sheet(visible: details, side: .end, title: "Details") { Text("Opened 3 days ago") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
