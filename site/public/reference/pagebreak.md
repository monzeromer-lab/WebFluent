# PageBreak

> Forces a new PDF page.

The page: https://webfluent.monzeromer.dev/reference/pagebreak

Forces a new PDF page.

```wf
PageBreak
```

## Example

```wf
Document(title: "Report") {
    Heading("Summary").h1
    Paragraph("The quarter in one page.")
    PageBreak
    Heading("Appendix: the figures").h2
    Paragraph("Every number, month by month.")
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
