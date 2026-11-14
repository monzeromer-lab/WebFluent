# Element

> A custom element a script defines, placed by its tag: Element("stripe-pricing-table", publishable-key: key) { on ready { … } }. Every named argument is an attribute, every on handler an event it fires, and the block its children.

The page: https://webfluent.monzeromer.dev/reference/element

A custom element a script defines, placed by its tag: Element("stripe-pricing-table", publishable-key: key) { on ready { … } }. Every named argument is an attribute, every on handler an event it fires, and the block its children.

```wf
Element(_ tag: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `tag` | `String  (positional)` | The custom element's tag: lower case, with a hyphen, as every custom element's is |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
Element("relative-time", datetime: "2026-03-14T09:30:00Z") { Text("March 14") }
Element("stripe-pricing-table", publishableKey: "pk_test_…") { on ready { log("prices shown") } }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [E116](https://webfluent.monzeromer.dev/errors/E116.md): A tag that cannot be a custom element's
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
