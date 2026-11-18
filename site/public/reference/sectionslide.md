# SectionSlide

> Full-bleed coloured band with a centred label.

The page: https://webfluent.monzeromer.dev/reference/sectionslide

Full-bleed coloured band with a centred label.

```wf
SectionSlide(_ label: String, tone: Enum)
```

## Example

```wf
Presentation {
    SectionSlide("Part one: the numbers").primary
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | The label, an `<h2>` centred on the band |
| `tone:` | `one of its cases` | The colour of the band, which fills the slide; the label is white (black on `warning`). Written as a flag, `.primary`, or by name, `tone: .primary`.

- `.primary`: The theme's primary colour
- `.success`: Green
- `.danger`: Red
- `.warning`: Amber, with black text
- `.info`: Blue |

## Flags

`.primary` `.success` `.danger` `.warning` `.info`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
