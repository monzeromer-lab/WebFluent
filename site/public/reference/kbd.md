# Kbd

> A key or a combination, written as this system writes it: Kbd("mod+k") is ⌘K on a Mac and Ctrl+K elsewhere.

The page: https://webfluent.monzeromer.dev/reference/kbd

A key or a combination, written as this system writes it: Kbd("mod+k") is ⌘K on a Mac and Ctrl+K elsewhere.

```wf
Kbd(_ keys: String)
```

## Example

```wf
Stack(gap: .xs) {
    Row(gap: .xs, align: .center) { Text("Press")  Kbd("mod+k")  Text("to search the docs.") }
    Row(gap: .xs, align: .center) { Text("Send with")  Kbd("shift+Enter")  Text("and close with")  Kbd("Escape") }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `keys` | `String (positional)` | The combination, as `on key` names it (`mod+k`, `shift+Enter`), written as this system writes it: `⌘K` on a Mac, `Ctrl+K` elsewhere and in the static paint |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
