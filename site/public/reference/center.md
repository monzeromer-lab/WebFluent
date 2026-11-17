# Center

> Its children in the middle of it, across and down.

The page: https://webfluent.monzeromer.dev/reference/center

Its children in the middle of it, across and down.

```wf
Center { … }
```

## Example

```wf
Center {
    style { height: 200px; background: $color-surface; border-radius: $radius-md }
    Stack(gap: .sm, align: .center) {
        Spinner
        Text("Loading your projects…").muted
    }
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
