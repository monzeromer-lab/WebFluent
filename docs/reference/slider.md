# Slider

> Range input. aria-* and its other attributes reach the input; with aria-valuetext the raw number is not shown beside the track.

The page: https://webfluent.monzeromer.dev/reference/slider

Range input. aria-* and its other attributes reach the input; with aria-valuetext the raw number is not shown beside the track.

```wf
Slider(bind: State, min: Number, max: Number, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | State variable the control reads and writes (two-way) |
| `min:` | `Number` | Lowest value |
| `max:` | `Number` | Highest value |
| `step:` | `Number` | Granularity |
| `label:` | `String` | Visible label; also the control's accessible name |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.disabled` `.dropInside` `.trapFocus`

## Example

```wf
state volume = 50
Slider(bind: volume, min: 0, max: 100, step: 5, label: "Volume")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A04](https://webfluent.monzeromer.dev/errors/A04.md): A control with no label

Since WebFluent 5.0.
