# Button

> A button: its label or block is what it shows, on click what it does.

The page: https://webfluent.monzeromer.dev/reference/button

A button: its label or block is what it shows, on click what it does.

```wf
Button(_ label: String, tone: Enum, size: Enum, shape: Enum, …) { … }
```

## Example

```wf
state saved = 0
Row(gap: .sm, justify: .end) {
    Button("Cancel").outlined
    Button("Save draft") { on click { saved = saved + 1 } }
    Button("Publish").primary { on click { toast("Published", tone: .success) } }
}
Text("Saved {saved} times").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | Its words; a block may hold more (an icon, `Text`) after them |
| `tone:` | `one of its cases` | The colour it is filled with, from the theme's tones. Written as a flag, `.primary`, or by name, `tone: .primary`.

- `.primary`: The brand colour: the page's main action
- `.secondary`: The secondary colour
- `.success`: Green: done, healthy
- `.danger`: Red: destructive, cannot be undone
- `.warning`: Amber: attention
- `.info`: Blue: neutral notice |
| `size:` | `one of its cases` | Its padding and text size. Written as a flag, `.sm`, or by name, `size: .sm`.

- `.sm`: Less padding, smaller text
- `.md` (the default): The default size
- `.lg`: More padding, larger text |
| `shape:` | `one of its cases` | Its corners; unset, the theme's `radius-md`. Written as a flag, `.rounded`, or by name, `shape: .rounded`.

- `.rounded`: Rounder corners (`radius-lg`)
- `.pill`: Fully rounded ends (`radius-full`) |
| `width:` | `one of its cases` | How wide it is. Written as a flag, `.full`, or by name, `width: .full`.

- `.full`: The full width of its container |
| `outlined:` | `Bool` | A border and no fill; with `.primary` or `.danger` its words and border take that colour. Written as a flag, `.outlined`, or `outlined: true`. |
| `type:` | `one of its cases` | What it does inside a form; any button not `.submit` or `.reset` never submits one. Written as a flag, `.button`, or by name, `type: .button`.

- `.button` (the default): A plain button
- `.submit`: Submits the enclosing form
- `.reset`: Resets the enclosing form |
| `to:` | `Path` | A route to go to when it is pressed, through the router as a `Link`'s `to:`; it stays a `<button>` |
| `disabled:` | `Bool` | Whether it is inert: it cannot be pressed or focused. Written as a flag, `.disabled`, or `disabled: true`. |
| `loading:` | `Bool` | Busy while it holds — `loading: Checkout.pay.pending`: a spinner in place of its label, at the same width, and a press does nothing. Written as a flag, `.loading`, or `loading: true`. |

## Flags

`.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.rounded` `.pill` `.full` `.outlined` `.button` `.submit` `.reset` `.disabled` `.loading`

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
- [A05](https://webfluent.monzeromer.dev/errors/A05.md): A button with no text

Before WebFluent 5.0.
