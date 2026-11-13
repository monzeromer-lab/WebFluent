# SectionSlide

> Full-bleed coloured band with a centred label.

The page: https://webfluent.monzeromer.dev/reference/sectionslide

Full-bleed coloured band with a centred label.

```wf
SectionSlide(_ label: String, tone: Enum)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The label |
| `tone:` | `.primary .success .danger .warning .info` | Colour of the band |

## Flags

`.primary` `.success` `.danger` `.warning` `.info` `.dropInside` `.trapFocus`

## Example

```wf
Presentation { SectionSlide("Numbers").primary }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.0.
