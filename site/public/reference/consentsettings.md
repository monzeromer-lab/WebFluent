# ConsentSettings

> A button that asks the reader again — a footer's "Cookie settings": it forgets their answer and shows the ConsentBanner. A script already loaded stays until the page is loaded again.

The page: https://webfluent.monzeromer.dev/reference/consentsettings

A button that asks the reader again — a footer's "Cookie settings": it forgets their answer and shows the ConsentBanner. A script already loaded stays until the page is loaded again.

```wf
ConsentSettings(_ label: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | What it says (unset, `Cookie settings`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
ConsentBanner("We would like to count visits.")
ConsentSettings("Cookie settings")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
