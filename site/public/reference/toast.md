# Toast

> Temporary notification.

The page: https://webfluent.monzeromer.dev/reference/toast

Temporary notification.

```wf
Toast(_ message: String, tone: Enum)
```

## Example

```wf
state archived = 0
Row(gap: .sm) {
    Button("Archive conversation") {
        on click {
            archived = archived + 1
            toast("Conversation archived", action: "Undo", onAction: () => toast("Restored"))
        }
    }
    Button("Save draft").primary { on click { toast("Draft saved", tone: .success) } }
}
Text("{archived} archived").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `message` | `String (positional)` | The message, shown above everything for 3 s once the element is drawn, and said to a screen reader; the element itself draws nothing in place |
| `tone:` | `one of its cases` | The kind of notice, which sets its colours; `.danger` is announced at once, the rest politely. Written as a flag, `.success`, or by name, `tone: .success`.

- `.success`: Green: done, healthy
- `.danger`: Red: destructive, failed
- `.warning`: Amber: attention
- `.info` (the default): Blue: neutral notice |

## Flags

`.success` `.danger` `.warning` `.info`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
