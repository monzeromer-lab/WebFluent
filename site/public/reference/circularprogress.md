# CircularProgress

> A ring that fills to value: of max:, announced as a progress bar; with no value:, it turns, for work of no known length. Its block is drawn in the middle.

The page: https://webfluent.monzeromer.dev/reference/circularprogress

A ring that fills to value: of max:, announced as a progress bar; with no value:, it turns, for work of no known length. Its block is drawn in the middle.

```wf
CircularProgress(value: Number, max: Number, label: String) { … }
```

## Example

```wf
state done = 40
Row(gap: .lg, align: .center) {
    CircularProgress(value: done, max: 100, label: "Upload") { Text("{done}%").sm }
    Stack(gap: .xs) {
        Text("Uploading photos.zip").bold
        Button("Add 20%").sm { on click { done = if done >= 80 { 100 } else { done + 20 } } }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | How far along, out of `max:`, clamped to the ring; unset or `null`, the ring turns without a value |
| `max:` | `Number` | The value at which the ring is full; `0` or not a number counts as 100 |
| `label:` | `String` | What is progressing: the progress bar's accessible name (`aria-label`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
