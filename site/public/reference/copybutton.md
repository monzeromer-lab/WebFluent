# CopyButton

> A button that copies value: to the clipboard, says Copied for a moment, and tells a screen reader so.

The page: https://webfluent.monzeromer.dev/reference/copybutton

A button that copies value: to the clipboard, says Copied for a moment, and tells a screen reader so.

```wf
CopyButton(_ label: String, value: Any, copied: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words; unset, `Copy` |
| `value:` | `Any  (required)` | What it copies |
| `copied:` | `String` | What it says once it has copied; unset, `Copied` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
CopyButton(value: "https://example.com/invite/42")
CopyButton("Copy link", value: "https://example.com", copied: "Link copied")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
