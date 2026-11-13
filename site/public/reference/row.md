# Row

> Horizontal flex layout. It stays a row at every width unless .stacks or a responsive direction says otherwise.

The page: https://webfluent.monzeromer.dev/reference/row

Horizontal flex layout. It stays a row at every width unless .stacks or a responsive direction says otherwise.

```wf
Row(gap: Enum, align: Enum, justify: Enum, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `gap:` | `.xs .sm .md .lg .xl` | Space between children |
| `align:` | `.start .center .end .stretch .baseline` | Cross-axis alignment |
| `justify:` | `.start .center .end .between .around .evenly` | Main-axis distribution |
| `direction:` | `.row .column` | Which way its children run; takes a value per breakpoint |
| `stacks:` | `Bool` | Reflow on a narrow screen: a Row becomes a column, a Grid one column |

## Flags

`.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly` `.row` `.column` `.stacks` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Row(gap: .md, align: .center, justify: .between) {
    Text("Left")
    Button("Right")
}
Row(direction: { base: .column, md: .row }) { Text("a")  Text("b") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
