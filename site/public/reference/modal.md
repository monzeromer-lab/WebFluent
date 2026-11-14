# Modal

> Modal <dialog>, opened by a state flag; the browser supplies focus trapping and Escape. on close runs when it closes, however it was closed, and the focus goes back to what had it.

The page: https://webfluent.monzeromer.dev/reference/modal

Modal <dialog>, opened by a state flag; the browser supplies focus trapping and Escape. on close runs when it closes, however it was closed, and the focus goes back to what had it.

```wf
Modal(visible: State, title: String, closable: Bool, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |
| `closable:` | `Bool` | A close button in the heading row, which closes it as Escape does |
| `dismiss:` | `.any .escape .none` | What closes it besides the page's own button |
| `initialFocus:` | `Any` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control |

## Flags

`.closable` `.any` `.escape` `.none`

## Events

- `close`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Modal.Modal.Title`
- `Modal.Modal.Footer`

## Example

```wf
state open = false
Button("Delete") { on click { open = true } }
Modal(visible: open, title: "Delete deploys.wf?").closable {
    Modal.Title { Markdown("Delete `deploys.wf`?") }
    Text("This cannot be undone.")
    Modal.Footer { Button("Cancel") { on click { open = false } } }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A08](https://webfluent.monzeromer.dev/errors/A08.md): A dialog with no title

Before WebFluent 5.0.
