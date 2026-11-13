# Tabs.Page

> One tab and its panel.

The page: https://webfluent.monzeromer.dev/reference/tabs-page

One tab and its panel.

```wf
Tabs.Page(_ label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | The tab's label |

## Flags

`.dropInside` `.trapFocus`

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

Since WebFluent 5.0.
