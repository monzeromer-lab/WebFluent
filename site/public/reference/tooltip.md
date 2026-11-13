# Tooltip

> Shows text above everything when its child is hovered (after delay:) or focused from the keyboard.

The page: https://webfluent.monzeromer.dev/reference/tooltip

Shows text above everything when its child is hovered (after delay:) or focused from the keyboard.

```wf
Tooltip(_ text: String, shortcut: String, placement: Enum, delay: Number) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The tooltip text |
| `shortcut:` | `String` | The keys that do the same, `mod+s`, shown as a chip as this system writes them |
| `placement:` | `.bottom .bottomStart .bottomEnd .top .topStart .topEnd .left .leftStart .leftEnd .right .rightStart .rightEnd` | Where it goes against its anchor; it flips to the other side when there is no room. Unset, `.bottomStart` |
| `delay:` | `Number` | How long a hover waits before it shows, in ms. Unset, 500 |

## Flags

`.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Tooltip("Copies the link") { IconButton(icon: "copy", label: "Copy link") }
Tooltip("Save", shortcut: "Ctrl+S", delay: 400, placement: .bottom) { Button("Save") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.0.
