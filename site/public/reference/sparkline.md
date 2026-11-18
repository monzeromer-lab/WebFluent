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
| `data` | `[Number] (positional)` | The numbers, a list, drawn as an SVG when the page is built, so they must be known then (`E109` otherwise); scaled from the least to the most |
| `bar:` | `Bool` | Bars from zero rather than a line. Written as a flag, `.bar`, or `bar: true`. |
| `width:` | `Number` | Its width in px; 0 or less is the default |
| `height:` | `Number` | Its height in px; 0 or less is the default |
| `color:` | `String` | The line's or the bars' colour, a CSS colour (`#0F766E`, `var(--color-primary)`); anything else is the default, the text's colour (`currentColor`) |
| `label:` | `String` | What it shows, as the picture's accessible name (`role="img"`) |

## Flags

`.bar`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
