# Toggle

> A button that stays pressed while its state holds — bold in an editor, a filter that is on: aria-pressed, a press flipping bind:.

The page: https://webfluent.monzeromer.dev/reference/toggle

A button that stays pressed while its state holds — bold in an editor, a filter that is on: aria-pressed, a press flipping bind:.

```wf
Toggle(_ label: String, bind: State, icon: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `bind:` | `State  (required)` | The `Bool` state it shows and flips |
| `icon:` | `String` | An icon before the words |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state bold = false
Toggle("Bold", bind: bold, icon: "type")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
