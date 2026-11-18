# Tooltip

> Shows text above everything when its child is hovered (after delay:) or focused from the keyboard.

The page: https://webfluent.monzeromer.dev/reference/tooltip

Shows text above everything when its child is hovered (after delay:) or focused from the keyboard.

```wf
Tooltip(_ text: String, shortcut: String, placement: Enum, delay: Number) { … }
```

## Example

```wf
Row(gap: .sm) {
    Tooltip("Copy the link") { IconButton(icon: "copy", label: "Copy link") }
    Tooltip("Edit the page", shortcut: "mod+e") { IconButton(icon: "edit", label: "Edit") }
    Tooltip("Delete it for good", placement: .bottom, delay: 200) {
        IconButton(icon: "trash", label: "Delete").danger
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String (positional)` | The tooltip's words, shown above everything at the control in its block and linked to it by `aria-describedby` |
| `shortcut:` | `String` | The keys that do the same, `mod+s`, shown as a chip as this system writes them and set as the control's `aria-keyshortcuts` |
| `placement:` | `one of its cases` | Where it goes against its control; it flips to the other side when there is no room. Written as a flag, `.bottom`, or by name, `placement: .bottom`.

- `.bottom`: Below, centred
- `.bottomStart`: Below, from its start
- `.bottomEnd`: Below, to its end
- `.top` (the default): Above, centred (the default)
- `.topStart`: Above, from its start
- `.topEnd`: Above, to its end
- `.left`: Before it, centred
- `.leftStart`: Before it, from its top
- `.leftEnd`: Before it, to its bottom
- `.right`: After it, centred
- `.rightStart`: After it, from its top
- `.rightEnd`: After it, to its bottom |
| `delay:` | `Number` | How long a hover waits before it shows, in ms; keyboard focus shows it at once |

## Flags

`.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
