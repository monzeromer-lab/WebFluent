# Tabs.Page

> One tab and its panel.

The page: https://webfluent.monzeromer.dev/reference/tabs-page

One tab and its panel.

```wf
Tabs.Page(_ label: String, value: Any, disabled: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The tab's label |
| `value:` | `Any` | What the Tabs' `bind:` holds while this tab is open |
| `disabled:` | `Bool` | Shown, but not opened: the arrow keys pass over it |

## Flags

`.disabled`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Tabs {
    Tabs.Page("Profile") { Text("Your details") }
    Tabs.Page("Billing") { Text("Your plan") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
