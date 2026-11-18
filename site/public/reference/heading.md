# Heading

> Heading; level picks the tag (h2 when none is given).

The page: https://webfluent.monzeromer.dev/reference/heading

Heading; level picks the tag (h2 when none is given).

```wf
Heading(_ content: String or Number or Bool, level: Enum, align: Enum, muted: Bool)
```

## Example

```wf
Stack(gap: .xs) {
    Heading("Pricing").h1
    Heading("Plans for every team").h2.muted
    Heading("What every plan has").h3
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `String or Number or Bool (positional)` | The heading text: a string, a number or a boolean |
| `level:` | `one of its cases` | The tag (`<h1>` … `<h6>`), its place in the outline, and its size. Written as a flag, `.h1`, or by name, `level: .h1`.

- `.h1`: The page's one top heading, `font-size-3xl`
- `.h2` (the default): A section heading, `font-size-2xl` (the default)
- `.h3`: A sub-section, `font-size-xl`
- `.h4`: `font-size-lg`
- `.h5`: `font-size-base`
- `.h6`: `font-size-sm` |
| `align:` | `one of its cases` | Where its lines sit. Written as a flag, `.left`, or by name, `align: .left`.

- `.left`: To the start of the line (`text-align: start`; the right in a right-to-left page)
- `.center`: Centred
- `.right`: To the end of the line (`text-align: end`; the left in a right-to-left page) |
| `muted:` | `Bool` | In `color-text-muted`. Written as a flag, `.muted`, or `muted: true`. |

## Flags

`.h1` `.h2` `.h3` `.h4` `.h5` `.h6` `.left` `.center` `.right` `.muted`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [A07](https://webfluent.monzeromer.dev/errors/A07.md): An empty heading
- [A11](https://webfluent.monzeromer.dev/errors/A11.md): A skipped heading level
- [A12](https://webfluent.monzeromer.dev/errors/A12.md): A page without exactly one h1

Before WebFluent 5.0.
