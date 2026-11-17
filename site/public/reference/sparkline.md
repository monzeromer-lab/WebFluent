# Sparkline

> A tiny chart in a line of text — a line, or .bar — of a list of numbers, drawn as an SVG when the page is built.

The page: https://webfluent.monzeromer.dev/reference/sparkline

A tiny chart in a line of text — a line, or .bar — of a list of numbers, drawn as an SVG when the page is built.

```wf
Sparkline(_ data: [Number], bar: Bool, width: Number, height: Number, …)
```

## Example

```wf
Stack(gap: .sm) {
    Row(gap: .sm, align: .center) {
        Text("Sign-ups this week")
        Sparkline([12, 18, 9, 24, 31, 27, 40], label: "Sign-ups this week")
    }
    Row(gap: .sm, align: .center) {
        Text("Errors by day")
        Sparkline([4, 2, 7, 1, 0, 3, 2], label: "Errors by day").bar
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `data` | `[Number]  (positional)` | The numbers: a list |
| `bar:` | `Bool` | Bars rather than a line |
| `width:` | `Number` | Its width in px (unset, 80) |
| `height:` | `Number` | Its height in px (unset, 20) |
| `color:` | `String` | Its colour (unset, the text's) |
| `label:` | `String` | What it shows, for a screen reader |

## Flags

`.bar`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
