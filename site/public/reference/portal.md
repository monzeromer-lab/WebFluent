# Portal

> Its block, drawn into a node somebody else's code owns: an element a script hands over, or a ref: handle. When into: names another node the block moves there with its nodes and state; null keeps it aside until a node comes back; it goes when the page, branch or list item it is written in leaves. What is inside belongs to where it is written: its handlers, emit and lifetimes.

The page: https://webfluent.monzeromer.dev/reference/portal

Its block, drawn into a node somebody else's code owns: an element a script hands over, or a ref: handle. When into: names another node the block moves there with its nodes and state; null keeps it aside until a node comes back; it goes when the page, branch or list item it is written in leaves. What is inside belongs to where it is written: its handlers, emit and lifetimes.

```wf
Portal(into: Element?) { … }
```

## Example

```wf
state count = 0
Stack(gap: .sm, ref: panel) {
    style { padding: 1rem; border: 1px dashed $color-border; border-radius: $radius-md }
    Text("A node somebody else's code owns:").muted.sm
}
Portal(into: panel) {
    Row(gap: .sm, align: .center) {
        Text("Clicked {count} times")
        Button("Click").primary.sm { on click { count = count + 1 } }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `into:` | `Element? (required)` | Where it draws: an element, a `ref:` handle, or `null` to keep it aside; read again whenever what it reads changes |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C01](https://webfluent.monzeromer.dev/errors/C01.md): A required prop or field left out
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.10.0.
