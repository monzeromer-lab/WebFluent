# Sticky

> A box that sticks to the top of its scrolling parent (offset: below it) as the page scrolls past, marked data-stuck while it is stuck.

The page: https://webfluent.monzeromer.dev/reference/sticky

A box that sticks to the top of its scrolling parent (offset: below it) as the page scrolls past, marked data-stuck while it is stuck.

```wf
Sticky(offset: String, bottom: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `offset:` | `String` | How far below the top it sticks, `64px` (unset, 0) |
| `bottom:` | `Bool` | It sticks to the bottom instead |

## Flags

`.bottom`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Sticky(offset: "8px") { Text("Stays in view").bold }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
