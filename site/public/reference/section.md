# Section

> Groups content with spacing; on the web, a <section>.

The page: https://webfluent.monzeromer.dev/reference/section

Groups content with spacing; on the web, a <section>.

```wf
Section { … }
```

## Example

```wf
Document(title: "Service agreement") {
    Section {
        Heading("1. Payment").h2
        Paragraph("Invoices are due within 30 days of issue.")
    }
    Section {
        style { break-inside: avoid }
        Heading("2. Signatures").h2
        Paragraph("Signed for Northwind Ltd: ______________________")
    }
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
