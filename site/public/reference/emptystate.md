# EmptyState

> What a list or a page shows when it has nothing yet: an icon, a title, a sentence and, in its block, what to do about it.

The page: https://webfluent.monzeromer.dev/reference/emptystate

What a list or a page shows when it has nothing yet: an icon, a title, a sentence and, in its block, what to do about it.

```wf
EmptyState(icon: String, title: String, text: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `icon:` | `String` | An icon's name, drawn large above the title |
| `title:` | `String  (required)` | What is missing: `No projects yet` |
| `text:` | `String` | A sentence on what to do about it |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `EmptyState.EmptyState.Title`
- `EmptyState.EmptyState.Text`
- `EmptyState.EmptyState.Actions`

## Example

```wf
EmptyState(icon: "folder", title: "No projects yet", text: "Create one to get started.") {
    Button("New project").primary
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
