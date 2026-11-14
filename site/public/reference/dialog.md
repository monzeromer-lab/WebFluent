# Dialog

> Confirmation <dialog>, opened by a state flag. on close runs when it closes, however it was closed.

The page: https://webfluent.monzeromer.dev/reference/dialog

Confirmation <dialog>, opened by a state flag. on close runs when it closes, however it was closed.

```wf
Dialog(visible: State, title: String, closable: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |
| `closable:` | `Bool` | A close button in the heading row, which closes it as Escape does |

## Flags

`.closable`

## Events

- `close`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Dialog.Dialog.Title`

## Example

```wf
state asking = false
Dialog(visible: asking, title: "Leave without saving notes.md?") {
    Dialog.Title { Markdown("Leave without saving `notes.md`?") }
    Text("Your draft will be lost.")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A08](https://webfluent.monzeromer.dev/errors/A08.md): A dialog with no title

Before WebFluent 5.0.
