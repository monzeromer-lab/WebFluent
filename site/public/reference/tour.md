# Tour

> A tour of the page while open: holds: each step in a card at its target: (a ref:), lit through a cut-out in the dimmed page, with Back, Next and Done; Escape ends it and writes false back. on finish hears Done, on close any end.

The page: https://webfluent.monzeromer.dev/reference/tour

A tour of the page while open: holds: each step in a card at its target: (a ref:), lit through a cut-out in the dimmed page, with Back, Next and Done; Escape ends it and writes false back. on finish hears Done, on close any end.

```wf
Tour(open: State) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `open:` | `State  (required)` | Whether the tour is showing; written `false` when it ends |

## Events

- `close`
- `finish`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Tour.Tour.Step`

## Example

```wf
state touring = false
Button("New", ref: newBtn)
Button("Take the tour") { on click { touring = true } }
Tour(open: touring) {
    Tour.Step(target: newBtn, title: "New") { Text("Start a project here.") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
