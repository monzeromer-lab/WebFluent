# Heading

> Heading; level picks the tag (h2 when none is given).

The page: https://webfluent.monzeromer.dev/reference/heading

Heading; level picks the tag (h2 when none is given).

```wf
Heading(_ content: Any, level: Enum, align: Enum, muted: Bool)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `Any  (positional)` | The heading text |
| `level:` | `.h1 .h2 .h3 .h4 .h5 .h6` | Heading level: the tag, and its place in the outline |
| `align:` | `.left .center .right` | Text alignment |
| `muted:` | `Bool` | Dimmed, for secondary text |

## Flags

`.h1` `.h2` `.h3` `.h4` `.h5` `.h6` `.left` `.center` `.right` `.muted` `.dropInside` `.trapFocus`

## Example

```wf
Heading("Pricing").h1
Heading("Frequently asked").h2
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [A07](https://webfluent.monzeromer.dev/errors/A07.md): An empty heading
- [A11](https://webfluent.monzeromer.dev/errors/A11.md): A skipped heading level
- [A12](https://webfluent.monzeromer.dev/errors/A12.md): A page without exactly one h1

Since WebFluent 5.0.
