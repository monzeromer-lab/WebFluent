# Popover

> A layer at a control: drawn above everything, placed against its anchor:, closed by Escape or a press outside, which write false back to visible: and fire on close.

The page: https://webfluent.monzeromer.dev/reference/popover

A layer at a control: drawn above everything, placed against its anchor:, closed by Escape or a press outside, which write false back to visible: and fire on close.

```wf
Popover(visible: State, anchor: Element or Point, at: Point, …) { … }
```

## Example

```wf
state open = false
state branch = "main"
Button("Branch: {branch}", ref: branchBtn) { on click { open = !open } }
Popover(visible: open, anchor: branchBtn, placement: .bottomStart, label: "Branches").arrow {
    on close(reason) { open = false }
    Stack(gap: .xs) {
        Button("main") { on click { branch = "main"  open = false } }
        Button("release-7.0") { on click { branch = "release-7.0"  open = false } }
        Button("fix-login") { on click { branch = "fix-login"  open = false } }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `State` | Whether it is open: a `Bool` state, written back `false` when Escape or a press outside closes it (as `dismiss:` allows); a condition opens it but is not written |
| `anchor:` | `Element or Point` | What it is placed against and follows as the page scrolls: an element's `ref:`, or a point `{ x: e.clientX, y: e.clientY }`; its control gets `aria-controls` and `aria-expanded` |
| `at:` | `Point` | A point to place it at, `{ x: e.clientX, y: e.clientY }` — where the pointer asked for a context menu; the same option as `anchor:`, so give one or the other |
| `placement:` | `one of its cases` | Where it goes against its anchor; it flips to the other side when there is no room, and slides along its side to stay on screen. Written as a flag, `.bottom`, or by name, `placement: .bottom`.

- `.bottom`: Below, centred
- `.bottomStart` (the default): Below, from its start
- `.bottomEnd`: Below, to its end
- `.top`: Above, centred
- `.topStart`: Above, from its start
- `.topEnd`: Above, to its end
- `.left`: Before it, centred
- `.leftStart`: Before it, from its top
- `.leftEnd`: Before it, to its bottom
- `.right`: After it, centred
- `.rightStart`: After it, from its top
- `.rightEnd`: After it, to its bottom |
| `offset:` | `Number` | The gap between it and its anchor, in px |
| `label:` | `String` | Its accessible name; one drawn as a dialog (no other `role:`) without it is `A18` |
| `arrow:` | `Bool` | A small point on the side toward its anchor, following the placement it lands on. Written as a flag, `.arrow`, or `arrow: true`. |
| `dismiss:` | `one of its cases` | What closes it besides `visible:`; `on close` hears `"escape"` or `"outside"`. Written as a flag, `.any`, or by name, `dismiss: .any`.

- `.any` (the default): Escape, or a press outside it
- `.escape`: Escape only: a press outside is the page's
- `.none`: Neither: only the page's own button closes it, for what must be answered |
| `initialFocus:` | `Element` | What takes the focus when it opens, by its `ref:`; unset, its first control, else the popover itself |

## Flags

`.bottom` `.bottomStart` `.bottomEnd` `.top` `.topStart` `.topEnd` `.left` `.leftStart` `.leftEnd` `.right` `.rightStart` `.rightEnd` `.arrow` `.any` `.escape` `.none`

## Events

- `close`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.9.0.
