# EmptyState.Title

> Its title.

The page: https://webfluent.monzeromer.dev/reference/emptystate-title

Its title.

```wf
EmptyState.Title(_ text: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The title |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
EmptyState(icon: "folder", title: "No projects yet", text: "Create one to get started.") {
    Button("New project").primary
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
