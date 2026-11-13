# IconButton

> Icon-only button. label is its accessible name, never visible text.

The page: https://webfluent.monzeromer.dev/reference/iconbutton

Icon-only button. label is its accessible name, never visible text.

```wf
IconButton(icon: String, label: String, size: Enum, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `icon:` | `String` | One of the built-in icon names |
| `label:` | `String` | Accessible name (`aria-label` and `title`) |
| `size:` | `.sm .md .lg` | Its size |
| `tone:` | `.primary .danger` | Colour |
| `type:` | `.button .submit .reset` | Its role inside a form |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.sm` `.md` `.lg` `.primary` `.danger` `.button` `.submit` `.reset` `.disabled` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
IconButton(icon: "close", label: "Close")
IconButton(icon: "edit", label: "Edit the title").primary.sm
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A02](https://webfluent.monzeromer.dev/errors/A02.md): An icon button with no name
- [V08](https://webfluent.monzeromer.dev/errors/V08.md): An icon the runtime does not draw

Since WebFluent 5.0.
