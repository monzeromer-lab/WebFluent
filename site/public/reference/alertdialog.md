# AlertDialog

> A <dialog> that asks before something that cannot be undone: role="alertdialog", its body read out with its title, and closed by Escape or its own buttons, not a press outside. For a question asked from a handler, await ask(…) draws one.

The page: https://webfluent.monzeromer.dev/reference/alertdialog

A <dialog> that asks before something that cannot be undone: role="alertdialog", its body read out with its title, and closed by Escape or its own buttons, not a press outside. For a question asked from a handler, await ask(…) draws one.

```wf
AlertDialog(visible: State, title: String, dismiss: Enum, …) { … }
```

## Example

```wf
state sure = false
state said = "Draft: 3 unsaved edits"
Button("Discard changes").danger.outlined { on click { sure = true } }
AlertDialog(visible: sure, title: "Discard your changes?") {
    Text("Your 3 edits to the pricing page will be lost.")
    AlertDialog.Footer {
        Button("Keep editing") { on click { sure = false } }
        Button("Discard").danger { on click { sure = false  said = "Draft discarded" } }
    }
}
Text(said).muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape, a press on the backdrop or its close button shuts it; a condition opens it but is not written |
| `title:` | `String` | The heading at the top, and the accessible name; a value that reads state follows it; a `AlertDialog.Title` block draws the heading instead |
| `dismiss:` | `one of its cases` | What closes it besides the page's own button and `visible:`; an alert dialog is answered, so a press beside it does nothing unless it says `.any`. Written as a flag, `.any`, or by name, `dismiss: .any`.

- `.any`: Escape, or a press outside it
- `.escape` (the default): Escape only: a press outside is the page's
- `.none`: Neither: only the page's own button closes it, for what must be answered |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control; when it closes the focus goes back where it was |

## Flags

`.any` `.escape` `.none`

## Events

- `close`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `AlertDialog`.

### AlertDialog.Title

What the alert dialog's heading draws, in place of its title:'s words, which stay its name.

```wf
AlertDialog.Title { … }
```

### AlertDialog.Footer

Its answers: the button that loses nothing first.

```wf
AlertDialog.Footer { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
