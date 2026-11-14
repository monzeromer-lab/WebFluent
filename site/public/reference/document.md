# Document

> Root of a PDF document: the paper, the margins and what the file says about itself.

The page: https://webfluent.monzeromer.dev/reference/document

Root of a PDF document: the paper, the margins and what the file says about itself.

```wf
Document(page_size: Enum, size: String, landscape: Bool, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `page_size:` | `.A4 .A3 .A5 .Letter .Legal` | Paper size (also `size:`) |
| `size:` | `String` | Paper size: `A4`, `Letter`, `Tabloid`, or two lengths, `210mm 297mm` |
| `landscape:` | `Bool` | Turn the paper on its side |
| `margin:` | `Any` | The page margins: one length, or top right bottom left (`72`, `20mm 25mm`) |
| `title:` | `String` | The file's title, shown by a reader in place of its name |
| `author:` | `String` | The file's author; several separated by commas |
| `subject:` | `String` | What the document is about |
| `keywords:` | `String` | Comma-separated keywords |
| `lang:` | `String` | The document's language, for screen readers and hyphenation |

## Flags

`.A4` `.A3` `.A5` `.Letter` `.Legal` `.landscape`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Document(page_size: "A4") {
    Section { Heading("Report").h1  Paragraph("Revenue grew.") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
