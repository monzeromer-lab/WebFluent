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
| `visible:` | `State` | State flag that opens and closes it |
| `title:` | `String` | Heading shown at the top |
| `closable:` | `Bool` | A close button in the heading row, which closes it as Escape does |
| `side:` | `.bottom .start .end` | The edge it comes in from |
| `snap:` | `[Number]` | A bottom sheet's heights, as fractions of the window's, `[0.5, 1]`: it opens at the first |
| `label:` | `String` | Its accessible name, when it has no `title:` |
| `dismiss:` | `.any .escape .none` | What closes it besides the page's own button |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, what says `autofocus`, else its first control |

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
