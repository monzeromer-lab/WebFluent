# Chart

> A chart drawn as vector graphics when the page is built: bars, lines, areas, a pie or a donut over a list of records.

The page: https://webfluent.monzeromer.dev/reference/chart

A chart drawn as vector graphics when the page is built: bars, lines, areas, a pie or a donut over a list of records.

```wf
Chart(kind: Enum, data: Any, x: String, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `kind:` | `.bar .line .area .pie .donut` | What it draws |
| `data:` | `Any` | The rows: a list of records, or of numbers |
| `x:` | `String` | The key of each row's label |
| `y:` | `Any` | The key of each row's value, or a list of keys for several series |
| `colors:` | `Any` | The series' colours, in order |
| `width:` | `Number` | Its width in user units (it scales to its box) |
| `height:` | `Number` | Its height in user units |
| `ink:` | `String` | The colour of its labels |
| `grid:` | `String` | The colour of its grid lines |
| `legend:` | `Bool` | Name the series |
| `stacked:` | `Bool` | Stack a bar chart's series |
| `labels:` | `Bool` | Write each bar's or slice's value on it |
| `unit:` | `String` | A unit after each value on the axis: `ms`, `%` |

## Flags

`.bar` `.line` `.area` `.pie` `.donut` `.legend` `.stacked` `.labels` `.dropInside` `.trapFocus`

## Example

```wf
Chart(kind: .bar, data: [{ month: "Jan", web: 12 }, { month: "Feb", web: 18 }], x: "month", y: "web", unit: "k")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.2.0.
