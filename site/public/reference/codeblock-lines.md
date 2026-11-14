# CodeBlock.Lines

> The line numbers.

The page: https://webfluent.monzeromer.dev/reference/codeblock-lines

The line numbers.

```wf
CodeBlock.Lines(_ text: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The numbers, a line each |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
CodeBlock("page Home(path: \"/\") {\n    Text(\"Hello\")\n}", language: "wf", title: "Home.wf").copy.lines
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
