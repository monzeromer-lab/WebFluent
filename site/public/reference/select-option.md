# Select.Option

> One choice: the visible label, and the value sent with the form.

The page: https://webfluent.monzeromer.dev/reference/select-option

One choice: the visible label, and the value sent with the form.

```wf
Select.Option(_ label: String, value: Any)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The visible label |
| `value:` | `Any` | The value `bind` takes when this one is chosen; the label when absent |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state plan = "free"
Select(bind: plan, label: "Plan") {
    Select.Option("Free", value: "free")
    Select.Option("Team", value: "team")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
