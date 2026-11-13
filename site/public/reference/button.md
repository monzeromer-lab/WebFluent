# Button

> Button. Its block holds what it shows; on click what it does.

The page: https://webfluent.monzeromer.dev/reference/button

Button. Its block holds what it shows; on click what it does.

```wf
Button(_ label: String, tone: Enum, size: Enum, shape: Enum, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The label |
| `tone:` | `.primary .secondary .success .danger .warning .info` | Colour |
| `size:` | `.sm .md .lg` | Its size |
| `shape:` | `.rounded .pill` | Corner shape |
| `width:` | `.full` | Width |
| `outlined:` | `Bool` | A border and no fill |
| `type:` | `.button .submit .reset` | Its role inside a form |
| `to:` | `Path` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.rounded` `.pill` `.full` `.outlined` `.button` `.submit` `.reset` `.disabled` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Button("Save").primary { on click { log("saved") } }
Button("Delete").danger.outlined.sm
Button("Send", type: .submit).primary.full
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [A05](https://webfluent.monzeromer.dev/errors/A05.md): A button with no text

Since WebFluent 5.0.
