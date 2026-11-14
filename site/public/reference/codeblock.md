# CodeBlock

> A block of code with a bar above it: a file name, a copy button (.copy), and line numbers (.lines). language: colours it, as on Code.

The page: https://webfluent.monzeromer.dev/reference/codeblock

A block of code with a bar above it: a file name, a copy button (.copy), and line numbers (.lines). language: colours it, as on Code.

```wf
CodeBlock(_ code: String, language: String, title: String, copy: Bool, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `code` | `String  (positional)` | The code |
| `language:` | `String` | Colours the code as this language: `wf`, `json`, `bash` or `css` |
| `title:` | `String` | The file name or the language, in the bar |
| `copy:` | `Bool` | A button in the bar that copies the code |
| `lines:` | `Bool` | Line numbers beside it |

## Flags

`.copy` `.lines`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Parts

- `CodeBlock.CodeBlock.Bar`
- `CodeBlock.CodeBlock.Lines`

## Example

```wf
CodeBlock("page Home(path: \"/\") {\n    Text(\"Hello\")\n}", language: "wf", title: "Home.wf").copy.lines
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
