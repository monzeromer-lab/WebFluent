# Kbd

> A key or a combination, written as this system writes it: Kbd("mod+k") is ⌘K on a Mac and Ctrl+K elsewhere.

The page: https://webfluent.monzeromer.dev/reference/kbd

A key or a combination, written as this system writes it: Kbd("mod+k") is ⌘K on a Mac and Ctrl+K elsewhere.

```wf
Kbd(_ keys: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `keys` | `String  (positional)` | The combination, as `on key` names it: `mod+k`, `shift+Enter` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Text { Text("Press ")  Kbd("mod+k")  Text(" to search") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
