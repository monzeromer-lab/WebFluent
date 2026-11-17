# ButtonGroup

> Buttons joined into one row.

The page: https://webfluent.monzeromer.dev/reference/buttongroup

Buttons joined into one row.

```wf
ButtonGroup { … }
```

## Example

```wf
state range = "Week"
ButtonGroup {
    Button("Day", outlined: range != "Day") { on click { range = "Day" } }
    Button("Week", outlined: range != "Week") { on click { range = "Week" } }
    Button("Month", outlined: range != "Month") { on click { range = "Month" } }
}
Text("Showing one {range}").muted
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
