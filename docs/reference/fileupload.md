# FileUpload

> File input; handle the chosen files in on change.

The page: https://webfluent.monzeromer.dev/reference/fileupload

File input; handle the chosen files in on change.

```wf
FileUpload(accept: String, label: String, multiple: Bool, …) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `accept:` | `String` | Accepted types, e.g. `image/*` or `.pdf,.doc` |
| `label:` | `String` | Visible label; also the control's accessible name |
| `multiple:` | `Bool` | Accept several files |
| `disabled:` | `Bool` | Whether the control is inert |

## Flags

`.multiple` `.disabled` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
FileUpload(accept: "image/*", label: "Choose a picture") { on change(e) { log(e.target.files[0].name) } }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
