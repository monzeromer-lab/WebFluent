# Meter

> A level against its range (<meter>): a disk's use, a password's strength. Past high: or under low: it says so in its colour; optimum: says which end is good.

The page: https://webfluent.monzeromer.dev/reference/meter

A level against its range (<meter>): a disk's use, a password's strength. Past high: or under low: it says so in its colour; optimum: says which end is good.

```wf
Meter(value: Number, min: Number, max: Number, …)
```

## Example

```wf
state used = 0.72
Stack(gap: .sm) {
    Text("Disk: {used:.percent} used")
    Meter(value: used, max: 1, high: 0.9, optimum: 0, label: "Disk")
    Slider(bind: used, min: 0, max: 1, step: 0.01, label: "Used")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | The level |
| `min:` | `Number` | The bottom of the range (unset, 0) |
| `max:` | `Number` | The top of the range (unset, 1) |
| `low:` | `Number` | Under this, it is low |
| `high:` | `Number` | Over this, it is high |
| `optimum:` | `Number` | The best level |
| `label:` | `String` | What it measures, for a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
