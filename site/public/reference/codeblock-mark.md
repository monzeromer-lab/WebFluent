# CodeBlock.Mark

> One line's mark. Drawn for it.

The page: https://webfluent.monzeromer.dev/reference/codeblock-mark

One line's mark. Drawn for it.

```wf
CodeBlock.Mark
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
CodeBlock("page Home(path: \"/\") {\n    Text(\"Hello\")\n}", language: "wf", title: "Home.wf").copy.lines
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
