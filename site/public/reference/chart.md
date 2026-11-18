# Chart

> A chart drawn as vector graphics when the page is built: bars, lines, areas, a pie or a donut over a list of records.

The page: https://webfluent.monzeromer.dev/reference/chart

A chart drawn as vector graphics when the page is built: bars, lines, areas, a pie or a donut over a list of records.

```wf
Chart(kind: Enum, data: [Json], x: String, …)
```

## Example

```wf
Chart(kind: .bar, data: [
    { month: "Jan", web: 41, store: 22 },
    { month: "Feb", web: 48, store: 25 },
    { month: "Mar", web: 56, store: 24 },
    { month: "Apr", web: 61, store: 30 },
], x: "month", y: ["web", "store"], unit: "k").legend
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `kind:` | `one of its cases` | What it draws (also written `type:`). Written as a flag, `.bar`, or by name, `kind: .bar`.

- `.bar` (the default): Bars, one per row; grouped for several series, or `stacked` (the default)
- `.line`: A line through the rows per series, a dot at each of up to 24 rows
- `.area`: A line with the area under it filled at 18% opacity
- `.pie`: Slices of a whole, one per row, from the first `y` key
- `.donut`: A pie with a hole |
| `data:` | `[Json]` | The rows: a list of records (`{ month: "Jan", web: 41 }`) or of numbers, known when the page is built (`E109` on a page otherwise) |
| `x:` | `String` | The key of each row's label along the bottom (a pie's legend) |
| `y:` | `String or [String]` | The key of each row's value, or a list of keys for several series; a pie or donut uses the first |
| `colors:` | `[String]` | The series' colours in order (a pie's: one a slice), repeated when there are more; one that is not a CSS colour is replaced by the default |
| `width:` | `Number` | The drawing's width in its own units (its 11-unit text scales with it); on a page the SVG fills its box's width, so with `height:` it sets the shape |
| `height:` | `Number` | The drawing's height in its own units; with `width:` it sets the shape |
| `ink:` | `String` | The colour of its axis labels, value labels and legend text, a CSS colour |
| `grid:` | `String` | The colour of its horizontal grid lines, a CSS colour |
| `legend:` | `Bool` | Name the series above a bar, line or area chart (only with two or more), or list each slice with its value beside a pie; `legend: false` leaves it out. Written as a flag, `.legend`, or `legend: true`. |
| `stacked:` | `Bool` | Stack a bar chart's series in one bar a row; other kinds ignore it. Written as a flag, `.stacked`, or `stacked: true`. |
| `labels:` | `Bool` | Write each bar's value above it (bars over 12 units tall) or each slice's percentage on it (slices over about 17°); lines and areas ignore it. Written as a flag, `.labels`, or `labels: true`. |
| `unit:` | `String` | Written after each value on the axis and in a pie's legend: `ms`, `%` |

## Flags

`.bar` `.line` `.area` `.pie` `.donut` `.legend` `.stacked` `.labels`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.2.0.
