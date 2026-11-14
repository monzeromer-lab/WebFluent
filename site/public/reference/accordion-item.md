# Accordion.Item

> One section: its title is what a reader presses, its block what opens.

The page: https://webfluent.monzeromer.dev/reference/accordion-item

One section: its title is what a reader presses, its block what opens.

```wf
Accordion.Item(_ title: String, open: Bool, name: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String  (positional)` | What a reader presses |
| `open:` | `Bool` | Open when it is drawn |
| `name:` | `String` | Items of one name open one at a time: what `.single` writes |

## Flags

`.open`

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

Since WebFluent 6.2.0.
