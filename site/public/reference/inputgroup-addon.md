# InputGroup.Addon

> Text beside the input: https://, .com, kg.

The page: https://webfluent.monzeromer.dev/reference/inputgroup-addon

Text beside the input: https://, .com, kg.

```wf
InputGroup.Addon(_ text: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The text |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state q = ""
InputGroup {
    Icon("search")
    Input(bind: q, aria-label: "Search")
    InputGroup.Clear
}
state site = ""
InputGroup { InputGroup.Addon("https://")  Input(bind: site, aria-label: "Site") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
