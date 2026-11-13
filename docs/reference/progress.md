# Progress

> Progress bar; a value that reads state moves with it.

The page: https://webfluent.monzeromer.dev/reference/progress

Progress bar; a value that reads state moves with it.

```wf
Progress(value: Number, max: Number)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `value:` | `Number` | Current value |
| `max:` | `Number` | Value at 100% |

## Flags

`.dropInside` `.trapFocus`

## Example

```wf
Progress(value: 40, max: 100)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
