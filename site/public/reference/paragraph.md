# Paragraph

> Block of text with paragraph spacing.

The page: https://webfluent.monzeromer.dev/reference/paragraph

Block of text with paragraph spacing.

```wf
Paragraph(_ content: String or Number or Bool, tone: Enum, size: Enum, align: Enum, …)
```

## Example

```wf
Document(title: "Q3 report") {
    Heading("Summary").h1
    Paragraph("Revenue grew 15% on the quarter, led by the new self-serve plan.")
    Paragraph("Churn held at 2.1%; support response times halved.").muted
    Paragraph("Figures are unaudited.").sm.italic
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `String or Number or Bool (positional)` | The text, a `<p>`: a string, a number or a boolean |
| `tone:` | `one of its cases` | Its colour, a tone of the theme. Written as a flag, `.primary`, or by name, `tone: .primary`.

- `.primary`: The theme's primary colour
- `.secondary`: The theme's secondary colour
- `.success`: Green: done, healthy
- `.danger`: Red: destructive, failed
- `.warning`: Amber: needs attention
- `.info`: Blue: a neutral notice |
| `size:` | `one of its cases` | Its font size. Written as a flag, `.sm`, or by name, `size: .sm`.

- `.sm`: `font-size-sm`, 0.875rem
- `.md` (the default): The size it inherits (the default)
- `.lg`: `font-size-lg`, about 1.125–1.25rem |
| `align:` | `one of its cases` | Where its lines sit. Written as a flag, `.left`, or by name, `align: .left`.

- `.left`: To the start of the line (`text-align: start`; the right in a right-to-left page)
- `.center`: Centred
- `.right`: To the end of the line (`text-align: end`; the left in a right-to-left page) |
| `transform:` | `one of its cases` | The case its letters are drawn in. Written as a flag, `.uppercase`, or by name, `transform: .uppercase`.

- `.uppercase`: ALL CAPS
- `.lowercase`: all lower case |
| `kind:` | `one of its cases` | A typographic role. Written as a flag, `.heading`, or by name, `kind: .heading`.

- `.heading`: Bold, at `font-size-lg`
- `.subtitle`: Small (`font-size-sm`) and muted |
| `bold:` | `Bool` | Bold. Written as a flag, `.bold`, or `bold: true`. |
| `italic:` | `Bool` | Italic. Written as a flag, `.italic`, or `italic: true`. |
| `underline:` | `Bool` | Underlined. Written as a flag, `.underline`, or `underline: true`. |
| `muted:` | `Bool` | In `color-text-muted`. Written as a flag, `.muted`, or `muted: true`. |

## Flags

`.primary` `.secondary` `.success` `.danger` `.warning` `.info` `.sm` `.md` `.lg` `.left` `.center` `.right` `.uppercase` `.lowercase` `.heading` `.subtitle` `.bold` `.italic` `.underline` `.muted`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
