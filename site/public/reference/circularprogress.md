# CircularProgress

> A ring that fills to value: of max:, announced as a progress bar; with no value:, it turns, for work of no known length. Its block is drawn in the middle.

The page: https://webfluent.monzeromer.dev/reference/circularprogress

A ring that fills to value: of max:, announced as a progress bar; with no value:, it turns, for work of no known length. Its block is drawn in the middle.

```wf
CircularProgress(value: Number, max: Number, label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | How far along; unset, it turns |
| `max:` | `Number` | The value when full (unset, 100) |
| `label:` | `String` | What is progressing, for a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state done = 40
CircularProgress(value: done, label: "Upload") { Text("{done}%").sm }
Button("More") { on click { done = done + 20 } }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
