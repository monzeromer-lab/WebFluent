# Code

> Inline code, or a code block with block; language: colours it.

The page: https://webfluent.monzeromer.dev/reference/code

Inline code, or a code block with block; language: colours it.

```wf
Code(_ content: Any, block: Bool, language: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `Any  (positional)` | The code |
| `block:` | `Bool` | A multi-line block |
| `language:` | `String` | Colours the code as this language: `wf`, `json`, `bash` or `css` |

## Flags

`.block`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Text("Run ")
Code("wf serve")
Code("page Home(path: \"/\") { Text(\"Hi\") }", language: "wf").block
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
