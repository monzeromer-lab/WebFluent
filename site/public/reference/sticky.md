# Sticky

> A box that sticks to the top of its scrolling parent (offset: below it) as the page scrolls past, marked data-stuck while it is stuck.

The page: https://webfluent.monzeromer.dev/reference/sticky

A box that sticks to the top of its scrolling parent (offset: below it) as the page scrolls past, marked data-stuck while it is stuck.

```wf
Sticky(offset: String, bottom: Bool) { … }
```

## Example

```wf
ScrollArea(height: "240px", label: "Changelog") {
    style { border: 1px solid $color-border; border-radius: $radius-md }
    Sticky {
        style { background: $color-background; padding: 0.5rem 0.75rem; border-bottom: 1px solid $color-border }
        Row(justify: .between, align: .center) {
            Text("Changelog").bold
            Badge("7.0").primary
        }
    }
    for v in (0..20).map(i => 19 - i).map(n => "7.0.{n}: fixes and small changes") by v {
        Text(v) { style { padding: 0.5rem 0.75rem } }
    }
}
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

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
