# ScrollArea

> A box that scrolls within height:, with a thin scrollbar, its edges fading where there is more, and a Tab stop so a keyboard can scroll it.

The page: https://webfluent.monzeromer.dev/reference/scrollarea

A box that scrolls within height:, with a thin scrollbar, its edges fading where there is more, and a Tab stop so a keyboard can scroll it.

```wf
ScrollArea(height: String, label: String, horizontal: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `height:` | `String` | How tall it may grow before it scrolls, `300px` |
| `label:` | `String  (required)` | What it holds, for a screen reader |
| `horizontal:` | `Bool` | It scrolls across rather than down |

## Flags

`.horizontal`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
ScrollArea(height: "120px", label: "Log") {
    for n in 1..=20 { Text("Line {n}") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
