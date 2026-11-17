# Popconfirm

> A question beside the button in its block before what it does: the button opens it, on confirm hears yes, on cancel no or Escape; the focus starts on the answer that loses nothing.

The page: https://webfluent.monzeromer.dev/reference/popconfirm

A question beside the button in its block before what it does: the button opens it, on confirm hears yes, on cancel no or Escape; the focus starts on the answer that loses nothing.

```wf
Popconfirm(title: String, yes: String, no: String, …) { … }
```

## Example

```wf
state files = ["notes.md", "draft.md", "todo.md"]
Stack(gap: .sm) {
    for f in files by f {
        Row(gap: .sm, align: .center, justify: .between) {
            Text(f)
            Popconfirm(title: "Delete {f}?", yes: "Delete", no: "Keep").danger {
                on confirm { files = files.filter(x => x != f) }
                Button("Delete").sm.outlined
            }
        }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `title:` | `String  (required)` | The question |
| `yes:` | `String` | The yes button's words (unset, `Yes`) |
| `no:` | `String` | The no button's words (unset, `Cancel`) |
| `danger:` | `Bool` | What yes does cannot be taken back: drawn in the danger colour |

## Flags

`.danger`

## Events

- `confirm`
- `cancel`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
