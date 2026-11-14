# TagInput

> Words typed into a field, each a chip once Enter or a comma ends it; Backspace in the empty field takes the last back, and each chip has a button to take it out.

The page: https://webfluent.monzeromer.dev/reference/taginput

Words typed into a field, each a chip once Enter or a comma ends it; Backspace in the empty field takes the last back, and each chip has a button to take it out.

```wf
TagInput(bind: State, label: String, placeholder: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The list of the words |
| `label:` | `String  (required)` | What the words are, for a screen reader |
| `placeholder:` | `String` | Shown while the field is empty |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state tags: [String] = ["rust"]
TagInput(bind: tags, label: "Tags", placeholder: "Add a tag")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
