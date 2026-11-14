# Accordion.Panel

> What opens: the item's block. Drawn for it.

The page: https://webfluent.monzeromer.dev/reference/accordion-panel

What opens: the item's block. Drawn for it.

```wf
Accordion.Panel { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Accordion.single {
    Accordion.Item("Shipping") { Text("Two to four days.") }
    Accordion.Item("Returns", open: true) { Text("Thirty days.") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
