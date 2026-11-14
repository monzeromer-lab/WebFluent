# Host

> A bare node handed to somebody else's code, with a lifetime. mount: runs with the node each time it is made, update: again whenever the state it reads changes, and cleanup: when the page, branch or list item it belongs to leaves. Every other element takes the same three.

The page: https://webfluent.monzeromer.dev/reference/host

A bare node handed to somebody else's code, with a lifetime. mount: runs with the node each time it is made, update: again whenever the state it reads changes, and cleanup: when the page, branch or list item it belongs to leaves. Every other element takes the same three.

```wf
Host(tag: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `tag:` | `String` | The element to make: `div` (the default), `span`, `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul` or `table` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Host(tag: "canvas", mount: (node) => node.getContext("2d"), cleanup: (ctx) => log("gone"))
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [V10](https://webfluent.monzeromer.dev/errors/V10.md): A Host tag it does not make

Before WebFluent 5.0.
