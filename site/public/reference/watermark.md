# Watermark

> Large, faint, rotated text behind each PDF page's content.

The page: https://webfluent.monzeromer.dev/reference/watermark

Large, faint, rotated text behind each PDF page's content.

```wf
Watermark(_ text: String, on: Enum)
```

## Example

```wf
Document(title: "Contract draft") {
    Watermark("DRAFT")
    Heading("Service agreement").h1
    Paragraph("This draft is for review and is not binding.")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String (positional)` | The text: 72pt, bold, 8% black, rotated behind the content (`.wf-watermark` restyles it) |
| `on:` | `one of its cases` | Which pages it is drawn on. Written as a flag, `.all`, or by name, `on: .all`.

- `.all` (the default): Every page
- `.first`: The first page only
- `.rest`: Every page but the first
- `.odd`: Odd pages (1, 3, 5 …)
- `.even`: Even pages |

## Flags

`.all` `.first` `.rest` `.odd` `.even`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.2.0.
