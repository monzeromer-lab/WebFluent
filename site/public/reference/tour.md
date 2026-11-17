# Tour

> A tour of the page while open: holds: each step in a card at its target: (a ref:), lit through a cut-out in the dimmed page, with Back, Next and Done; Escape ends it and writes false back. on finish hears Done, on close any end.

The page: https://webfluent.monzeromer.dev/reference/tour

A tour of the page while open: holds: each step in a card at its target: (a ref:), lit through a cut-out in the dimmed page, with Back, Next and Done; Escape ends it and writes false back. on finish hears Done, on close any end.

```wf
Tour(open: State, back: String, next: String, …) { … }
```

## Example

```wf
state touring = false
state toured = "Not toured yet"
Row(gap: .sm) {
    Button("New project", ref: newBtn).primary
    Button("Import", ref: importBtn)
    Button("Take the tour") { on click { touring = true } }
}
Tour(open: touring) {
    on finish { toured = "Tour finished" }
    Tour.Step(target: newBtn, title: "Start here") { Text("Make a project from a template.") }
    Tour.Step(target: importBtn, title: "Bring your work") { Text("Import a folder you already have.") }
}
Text(toured).muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `open:` | `State  (required)` | Whether the tour is showing; written `false` when it ends |
| `back:` | `String` | The back button's words (unset, `Back`) |
| `next:` | `String` | The next button's words (unset, `Next`) |
| `done:` | `String` | The last step's button's words (unset, `Done`) |
| `close:` | `String` | The close button's name (unset, `End the tour`) |
| `of:` | `String` | The word in `2 of 5` (unset, `of`) |

## Events

- `close`
- `finish`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Tour`.

### Tour.Step

One step: what it points at, its title, and its block.

```wf
Tour.Step(target: Element, title: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `target:` | `Element  (required)` | The element it points at: a `ref:` |
| `title:` | `String` | Its heading |

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
