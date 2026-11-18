# Modal

> Modal <dialog>, opened by a state flag; the browser supplies focus trapping and Escape. on close runs when it closes, however it was closed, and the focus goes back to what had it.

The page: https://webfluent.monzeromer.dev/reference/modal

Modal <dialog>, opened by a state flag; the browser supplies focus trapping and Escape. on close runs when it closes, however it was closed, and the focus goes back to what had it.

```wf
Modal(visible: State, title: String, closable: Bool, …) { … }
```

## Example

```wf
state open = false
state name = "Q3 report"
Button("Rename") { on click { open = true } }
Modal(visible: open, title: "Rename the report").closable {
    on close { open = false }
    Input(bind: name, label: "Name")
    Modal.Footer {
        Button("Cancel") { on click { open = false } }
        Button("Save").primary { on click { open = false } }
    }
}
Text("Report: {name}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape, a press on the backdrop or its close button shuts it; a condition opens it but is not written |
| `title:` | `String` | The heading at the top, and the accessible name; a value that reads state follows it; a `Modal.Title` block draws the heading instead |
| `closable:` | `Bool` | A close button (×) in the heading row, which closes it as Escape does; with no `title:` the row holds the button alone. Written as a flag, `.closable`, or `closable: true`. |
| `dismiss:` | `one of its cases` | What closes it besides the page's own button and `visible:`. Written as a flag, `.any`, or by name, `dismiss: .any`.

- `.any` (the default): Escape, or a press outside it
- `.escape`: Escape only: a press outside is the page's
- `.none`: Neither: only the page's own button closes it, for what must be answered |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control; when it closes the focus goes back where it was |

## Flags

`.closable` `.any` `.escape` `.none`

## Events

- `close`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Modal`.

### Modal.Title

What the modal's heading draws, in place of its title:'s words, which stay its name: Modal.Title { Text { "Rename "  Code(name) } }.

```wf
Modal.Title { … }
```

### Modal.Footer

The modal's action row.

```wf
Modal.Footer { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A08](https://webfluent.monzeromer.dev/errors/A08.md): A dialog with no title

Before WebFluent 5.0.
