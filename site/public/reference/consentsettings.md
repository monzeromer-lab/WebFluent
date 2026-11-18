# ConsentSettings

> A button that asks the reader again — a footer's "Cookie settings": it forgets their answer and shows the ConsentBanner. A script already loaded stays until the page is loaded again.

The page: https://webfluent.monzeromer.dev/reference/consentsettings

A button that asks the reader again — a footer's "Cookie settings": it forgets their answer and shows the ConsentBanner. A script already loaded stays until the page is loaded again.

```wf
ConsentSettings(_ label: String)
```

## Example

```wf
Row(gap: .md, align: .center) {
    Text("© 2026 Ledger").muted.sm
    Link("Privacy", to: "/learn")
    ConsentSettings("Cookie settings")
}
ConsentBanner("We would like to count visits.", accept: "Allow", decline: "No thanks")
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | The button's words; pressing it forgets the reader's answer and shows the page's `ConsentBanner` again |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
