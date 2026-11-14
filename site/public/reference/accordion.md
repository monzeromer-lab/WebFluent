# Accordion

> Sections a reader opens one by one, each a native <details>: Accordion.Item("Shipping") { … }. .single keeps one open at a time.

The page: https://webfluent.monzeromer.dev/reference/accordion

Sections a reader opens one by one, each a native <details>: Accordion.Item("Shipping") { … }. .single keeps one open at a time.

```wf
Accordion(single: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `single:` | `Bool` | One open at a time: opening one closes the others |

## Flags

`.single`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Accordion.Accordion.Item`
- `Accordion.Accordion.Summary`
- `Accordion.Accordion.Panel`

## Example

```wf
Accordion.single {
    Accordion.Item("Shipping") { Text("Two to four days.") }
    Accordion.Item("Returns", open: true) { Text("Thirty days.") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
