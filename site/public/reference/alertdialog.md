# AlertDialog

> A <dialog> that asks before something that cannot be undone: role="alertdialog", its body read out with its title, and closed by Escape or its own buttons, not a press outside. For a question asked from a handler, await ask(…) draws one.

The page: https://webfluent.monzeromer.dev/reference/alertdialog

A <dialog> that asks before something that cannot be undone: role="alertdialog", its body read out with its title, and closed by Escape or its own buttons, not a press outside. For a question asked from a handler, await ask(…) draws one.

```wf
AlertDialog(visible: State, title: String, dismiss: Enum, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |
| `dismiss:` | `.any .escape .none` | What closes it besides the page's own button |
| `initialFocus:` | `Any` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control |

## Flags

`.any` `.escape` `.none`

## Events

- `close`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `AlertDialog.AlertDialog.Title`
- `AlertDialog.AlertDialog.Footer`

## Example

```wf
state sure = false
Button("Discard") { on click { sure = true } }
AlertDialog(visible: sure, title: "Discard your changes?") {
    Text("Your edits will be lost.")
    AlertDialog.Footer {
        Button("Keep editing") { on click { sure = false } }
        Button("Discard").danger { on click { sure = false } }
    }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
