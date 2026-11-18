# FileUpload

> File input; bind: holds what was chosen, and on change hears it.

The page: https://webfluent.monzeromer.dev/reference/fileupload

File input; bind: holds what was chosen, and on change hears it.

```wf
FileUpload(accept: String, label: String, multiple: Bool, …) { … }
```

## Example

```wf
state photo: File? = null
FileUpload(bind: photo, accept: "image/*", label: "Profile picture")
if let p = photo {
    Text("Chosen: {p.name}").muted
} else {
    Text("No picture chosen").muted
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `accept:` | `String` | The kinds of file it offers: MIME types or extensions, comma-separated (`image/*`, `.pdf,.doc`) |
| `label:` | `String` | The label drawn above the file input and its accessible name; with `.dropzone`, the words of its Choose button |
| `multiple:` | `Bool` | Several files at once: `bind:` holds a `[File]`, and with `.dropzone` each choice adds to the list. Written as a flag, `.multiple`, or `multiple: true`. |
| `disabled:` | `Bool` | Whether the control is inert: it cannot be focused or changed, and a form does not send it. Written as a flag, `.disabled`, or `disabled: true`. |
| `bind:` | `State` | The state the choice is written to: a `File?` (`null` when none), or a `[File]` with `.multiple` |
| `dropzone:` | `Bool` | An area files are dropped on or chosen into, each listed with its name, a preview if it is an image, and a button to take it out. Written as a flag, `.dropzone`, or `dropzone: true`. |

## Flags

`.multiple` `.disabled` `.dropzone`

## DOM events it answers

Any other DOM event works too, in an `on … { }` handler.

- `on change`: The value was committed

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
