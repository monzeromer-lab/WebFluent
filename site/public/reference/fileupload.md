# FileUpload

> File input; bind: holds what was chosen, and on change hears it.

The page: https://webfluent.monzeromer.dev/reference/fileupload

File input; bind: holds what was chosen, and on change hears it.

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
| `bind:` | `State` | The chosen files: a `File`, or a list of them with `.multiple` |
| `dropzone:` | `Bool` | An area files are dropped on or chosen into, each shown with its preview and a button to take it out |

## Flags

`.multiple` `.disabled` `.dropzone`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on change`: The value was committed

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
state photo: File? = null
FileUpload(bind: photo, accept: "image/*", label: "Choose a picture")
if let p = photo { Text(p.name) }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
