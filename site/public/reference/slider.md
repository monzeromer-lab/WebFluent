# Slider

> Range input. aria-* and its other attributes reach the input; with aria-valuetext the raw number is not shown beside the track.

The page: https://webfluent.monzeromer.dev/reference/slider

Range input. aria-* and its other attributes reach the input; with aria-valuetext the raw number is not shown beside the track.

```wf
Slider(bind: State, min: Number, max: Number, …)
```

## Example

```wf
state volume = 60
Slider(bind: volume, min: 0, max: 100, step: 5, label: "Volume")
Text("Volume: {volume}%").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State` | The `Number` state it reads and writes as it is dragged; with `.range` a `[Number]` holding `[low, high]`. Its value is shown beside the track |
| `min:` | `Number` | The value at the start of the track |
| `max:` | `Number` | The value at the end of the track |
| `step:` | `Number` | How far one move of the thumb, or one arrow key, changes the value |
| `range:` | `Bool` | Two thumbs, a low and a high that cannot pass each other: `bind:` holds `[low, high]`, read as `[min, max]` until it holds two numbers. Written as a flag, `.range`, or `range: true`. |
| `label:` | `String` | The label drawn above the track and the input's accessible name; with `.range` no label is drawn, and it names the group and each thumb (`Price, lowest`) |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it. Written as a flag, `.disabled`, or `disabled: true`. |

## Flags

`.range` `.disabled`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on input`: The value changed, on every keystroke
- `on change`: The value was committed

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A04](https://webfluent.monzeromer.dev/errors/A04.md): A control with no label

Before WebFluent 5.0.
