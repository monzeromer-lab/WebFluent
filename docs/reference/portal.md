# Portal

> Its block, drawn into a node somebody else's code owns: an element a script hands over, or a ref: handle. When into: names another node the block moves there with its nodes and state; null keeps it aside until a node comes back; it goes when the page, branch or list item it is written in leaves. What is inside belongs to where it is written: its handlers, emit and lifetimes.

The page: https://webfluent.monzeromer.dev/reference/portal

Its block, drawn into a node somebody else's code owns: an element a script hands over, or a ref: handle. When into: names another node the block moves there with its nodes and state; null keeps it aside until a node comes back; it goes when the page, branch or list item it is written in leaves. What is inside belongs to where it is written: its handlers, emit and lifetimes.

```wf
Portal(into: Any) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `into:` | `Any  (required)` | Where it draws: an element, a `ref:` handle, or `null` to keep it aside; read again whenever what it reads changes |

## Flags

`.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Stack(ref: panel)
Portal(into: panel) { Text("Drawn inside the stack") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C01](https://webfluent.monzeromer.dev/errors/C01.md): A required prop or field left out
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.10.0.
