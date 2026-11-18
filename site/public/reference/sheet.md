# Sheet

> A modal <dialog> that comes in from an edge: the bottom (a phone's filters, a share list) or the start or end of the line. Dragged by its handle or heading toward its edge it closes; a bottom sheet with snap: points settles at the nearest.

The page: https://webfluent.monzeromer.dev/reference/sheet

A modal <dialog> that comes in from an edge: the bottom (a phone's filters, a share list) or the start or end of the line. Dragged by its handle or heading toward its edge it closes; a bottom sheet with snap: points settles at the nearest.

```wf
Sheet(visible: State, title: String, closable: Bool, …) { … }
```

## Example

```wf
state filters = false
state sort = "new"
state inStock = true
Button("Filters") { on click { filters = true } }
Sheet(visible: filters, title: "Filters", snap: [0.5, 1]).closable {
    Radio(bind: sort, value: "new", label: "Newest first")
    Radio(bind: sort, value: "price", label: "Lowest price")
    Switch(bind: inStock, label: "In stock only")
    Sheet.Footer { Button("Show results").primary.full { on click { filters = false } } }
}
Text("Sorted by {sort}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape, a press on the backdrop or its close button shuts it; a condition opens it but is not written |
| `title:` | `String` | The heading at the top, and the accessible name; a value that reads state follows it; a `Sheet.Title` block draws the heading instead |
| `closable:` | `Bool` | A close button (×) in the heading row, which closes it as Escape does; with no `title:` the row holds the button alone. Written as a flag, `.closable`, or `closable: true`. |
| `side:` | `one of its cases` | The edge it slides in from, and is dragged back toward to close. Written as a flag, `.bottom`, or by name, `side: .bottom`.

- `.bottom` (the default): From the bottom of the window, with a drag handle
- `.start`: From where a line starts: the left, the right in a right-to-left page
- `.end`: From where a line ends: the right, the left in a right-to-left page |
| `snap:` | `[Number]` | A bottom sheet's heights, as fractions of the window's (`[0.5, 1]`): it opens at the first written, a drag settles at the nearest; values outside 0–1 are dropped |
| `label:` | `String` | Its accessible name when it has no `title:`; ignored when it has one |
| `dismiss:` | `one of its cases` | What closes it besides the page's own button and `visible:`. Written as a flag, `.any`, or by name, `dismiss: .any`.

- `.any` (the default): Escape, or a press outside it
- `.escape`: Escape only: a press outside is the page's
- `.none`: Neither, and its handle does not drag it shut: only the page's own button closes it, for what must be answered |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control; when it closes the focus goes back where it was |

## Flags

`.closable` `.bottom` `.start` `.end` `.any` `.escape` `.none`

## Events

- `close`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Sheet`.

### Sheet.Title

What the sheet's heading draws, in place of its title:'s words, which stay its name.

```wf
Sheet.Title { … }
```

### Sheet.Footer

The sheet's action row, kept in view below what scrolls.

```wf
Sheet.Footer { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
