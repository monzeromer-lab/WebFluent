# HoverCard

> A preview card at a link or a button: shown after delay: ms of hover, at once on focus; it stays while the pointer is on it, and goes when the pointer leaves both, on Escape or on a press outside.

The page: https://webfluent.monzeromer.dev/reference/hovercard

A preview card at a link or a button: shown after delay: ms of hover, at once on focus; it stays while the pointer is on it, and goes when the pointer leaves both, on Escape or on a press outside.

```wf
HoverCard(delay: Number) { … }
```

## Example

```wf
Text("Reviewed by") {
    HoverCard(delay: 300) {
        Link("@ada", to: "/showcase")
        HoverCard.Content {
            Row(gap: .sm, align: .center) {
                Avatar(initials: "AL").primary
                Stack(gap: .xs) {
                    Text("Ada Lovelace").bold
                    Text("Wrote the first program.").muted.sm
                }
            }
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `delay:` | `Number` | How long the pointer rests on the control before the card shows, in ms; focus shows it at once, and it goes 300 ms after the pointer leaves both |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `HoverCard`.

### HoverCard.Content

What the card shows. The rest of the block is what it is at.

```wf
HoverCard.Content { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
