# IconButton

> Icon-only button. label is its accessible name, never visible text.

The page: https://webfluent.monzeromer.dev/reference/iconbutton

Icon-only button. label is its accessible name, never visible text.

```wf
IconButton(icon: String, label: String, size: Enum, …) { … }
```

## Example

```wf
state stars = 0
Row(gap: .sm, align: .center) {
    IconButton(icon: "star", label: "Star this project").primary { on click { stars = stars + 1 } }
    IconButton(icon: "edit", label: "Edit")
    IconButton(icon: "copy", label: "Copy link")
    IconButton(icon: "trash", label: "Delete").danger
    Text("{stars} stars").muted
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `icon:` | `String` | The icon it shows, one of the built-in icon names; a value that reads state follows it |
| `label:` | `String` | Its accessible name (`aria-label`) and tooltip (`title`, unless one is written), never visible text; without it the tooltip is the icon's name and it has no name (`A02`) |
| `size:` | `one of its cases` | Its square size. Written as a flag, `.sm`, or by name, `size: .sm`.

- `.sm`: 28px
- `.md` (the default): 36px, the default
- `.lg`: 48px |
| `tone:` | `one of its cases` | The icon's colour; the button stays transparent. Written as a flag, `.primary`, or by name, `tone: .primary`.

- `.primary`: The brand colour
- `.danger`: Red: destructive |
| `type:` | `one of its cases` | What it does inside a form; any button not `.submit` or `.reset` never submits one. Written as a flag, `.button`, or by name, `type: .button`.

- `.button` (the default): A plain button
- `.submit`: Submits the enclosing form
- `.reset`: Resets the enclosing form |
| `disabled:` | `Bool` | Whether it is inert: it cannot be pressed or focused. Written as a flag, `.disabled`, or `disabled: true`. |

## Flags

`.sm` `.md` `.lg` `.primary` `.danger` `.button` `.submit` `.reset` `.disabled`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on click`: The element was clicked or activated

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A02](https://webfluent.monzeromer.dev/errors/A02.md): An icon button with no name
- [V08](https://webfluent.monzeromer.dev/errors/V08.md): An icon the runtime does not draw

Before WebFluent 5.0.
