# CopyButton

> A button that copies value: to the clipboard, says Copied for a moment, and tells a screen reader so.

The page: https://webfluent.monzeromer.dev/reference/copybutton

A button that copies value: to the clipboard, says Copied for a moment, and tells a screen reader so.

```wf
CopyButton(_ label: String, value: String or Number, copied: String)
```

## Example

```wf
Row(gap: .sm, align: .center) {
    Code("https://ledger.example.com/invite/42")
    CopyButton("Copy link", value: "https://ledger.example.com/invite/42", copied: "Link copied")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String (positional)` | Its words |
| `value:` | `String or Number (required)` | What it copies, read when it is pressed; a value that reads state copies what it holds then |
| `copied:` | `String` | What it says for 1.6 s once it has copied, also announced to a screen reader |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
