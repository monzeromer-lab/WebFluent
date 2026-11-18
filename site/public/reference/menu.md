# Menu

> A menu above everything: opened by its trigger: button, or by visible: at an anchor: or a point at: — a context menu. Escape, a press outside or a choice close it, write false back to visible: and fire on close.

The page: https://webfluent.monzeromer.dev/reference/menu

A menu above everything: opened by its trigger: button, or by visible: at an anchor: or a point at: — a context menu. Escape, a press outside or a choice close it, write false back to visible: and fire on close.

```wf
Menu(trigger: String, label: String, visible: State, …) { … }
```

## Example

```wf
state said = "Nothing chosen yet"
state pinned = false
Menu(trigger: "File actions").bottomStart {
    Menu.Item(shortcut: "F2") {
        on click { said = "Rename" }
        Text("Rename")
    }
    Menu.Item(shortcut: "mod+d") {
        on click { said = "Duplicate" }
        Text("Duplicate")
    }
    Menu.Item(checked: pinned) {
        on click { pinned = !pinned }
        Text("Pinned")
    }
    Menu.Sub(label: "Move to") {
        Menu.Item { on click { said = "Moved to Archive" }  Text("Archive") }
        Menu.Item { on click { said = "Moved to Drafts" }  Text("Drafts") }
    }
    Menu.Divider
    Menu.Item(shortcut: "mod+backspace").danger {
        on click { said = "Delete" }
        Text("Delete")
    }
}
Text(said).muted.sm
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `trigger:` | `String` | The words on the button that opens it |
| `label:` | `String` | The menu list's accessible name (`aria-label`); it is not the button's words, which `trigger:` gives |
| `visible:` | `State` | A Bool state that opens and closes it: the button sets it, and Escape, a press outside or a choice write `false`. With no `trigger:`, state alone opens it |
| `anchor:` | `Element or Point` | What it is placed against: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }` |
| `at:` | `Point` | A point to place it at, `{ x: e.clientX, y: e.clientY }`: where the pointer asked for a context menu |
| `placement:` | `one of its cases` | Where it goes against its anchor; it flips to the other side when there is no room, and slides to stay on screen. Written as a flag, `.bottom`, or by name, `placement: .bottom`.

- `.bottom`: Below, centred
- `.bottomStart` (the default): Below, lined up with its start edge (the right in a right-to-left page)
- `.bottomEnd`: Below, lined up with its end edge
- `.top`: Above, centred
- `.topStart`: Above, lined up with its start edge
- `.topEnd`: Above, lined up with its end edge
- `.left`: To its left, centred
- `.leftStart`: To its left, from its top
- `.leftEnd`: To its left, to its bottom
- `.right`: To its right, centred
- `.rightStart`: To its right, from its top
- `.rightEnd`: To its right, to its bottom |
| `offset:` | `Number` | The gap between it and its anchor, in px |

## Flags

`.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd`

## Events

- `close`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Menu`.

### Menu.Item

One entry of the menu; on click is what choosing it does.

```wf
Menu.Item(shortcut: String, checked: Bool, disabled: Bool, …) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `shortcut:` | `String` | The keys that do the same, `mod+b`, shown at its end as this system writes them (`⌘B`, `Ctrl+B`); in a `Menubar` the keys also run it |
| `checked:` | `Bool` | A check mark that follows the value (`menuitemcheckbox`, `aria-checked`): the item turns something on and off. Written as a flag, `.checked`, or `checked: true`. |
| `disabled:` | `Bool` | Shown dimmed (`aria-disabled`), skipped by the arrow keys, and cannot be chosen. Written as a flag, `.disabled`, or `disabled: true`. |
| `danger:` | `Bool` | What it does cannot be taken back — Delete, Discard: drawn in the danger colour. Written as a flag, `.danger`, or `danger: true`. |

Flags: `.checked` `.disabled` `.danger`

### Menu.Sub

An entry that opens a menu of its own beside it.

```wf
Menu.Sub(label: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | The words of the entry that opens its menu beside it |

### Menu.Divider

A rule between groups of entries.

```wf
Menu.Divider
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
