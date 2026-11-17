# Editable

> Text that turns into a field when pressed: Enter or leaving keeps what was typed, Escape puts it back; bind: holds the text.

The page: https://webfluent.monzeromer.dev/reference/editable

Text that turns into a field when pressed: Enter or leaving keeps what was typed, Escape puts it back; bind: holds the text.

```wf
Editable(bind: State, label: String, placeholder: String)
```

## Example

```wf
state project = "Quarterly report"
Editable(bind: project, label: "Project name", placeholder: "Name the project")
Text("Saved as “{project}”").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The text |
| `label:` | `String  (required)` | What the text is, for a screen reader |
| `placeholder:` | `String` | Shown while it is empty |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
