# EmptyState

> What a list or a page shows when it has nothing yet: an icon, a title, a sentence and, in its block, what to do about it.

The page: https://webfluent.monzeromer.dev/reference/emptystate

What a list or a page shows when it has nothing yet: an icon, a title, a sentence and, in its block, what to do about it.

```wf
EmptyState(icon: String, title: String, text: String) { … }
```

## Example

```wf
state count = 0
EmptyState(icon: "folder", title: "No projects yet", text: "A project holds your pages, components and stores.") {
    Button("New project").primary { on click { count = count + 1 } }
    Link("Read the guide", to: "/learn")
}
if count > 0 { Text("{count} project(s) asked for").muted }
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

Written under its name, inside `EmptyState`.

### EmptyState.Title

Its title.

```wf
EmptyState.Title(_ text: String)
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The title |

### EmptyState.Text

Its sentence.

```wf
EmptyState.Text(_ text: String)
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The sentence |

### EmptyState.Actions

What to do: the block it was given.

```wf
EmptyState.Actions { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
