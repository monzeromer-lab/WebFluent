# TableOfContents

> The document's headings with the page each lands on, each a link to it.

The page: https://webfluent.monzeromer.dev/reference/tableofcontents

The document's headings with the page each lands on, each a link to it.

```wf
TableOfContents(levels: Number, title: String)
```

## Example

```wf
Document(title: "Employee handbook") {
    Heading("Employee handbook").h1
    TableOfContents(levels: 2, title: "Contents")
    PageBreak
    Heading("Your first week").h2
    Heading("Equipment").h3
    Heading("Time off").h2
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `levels:` | `Number` | The deepest heading level listed: `1` is the `h1`s only, `2` adds the `h2`s |
| `title:` | `String` | A heading (`<h2>`) above the list |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.2.0.
