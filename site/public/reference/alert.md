# Alert

> Inline notice; a danger or warning one is announced as an alert.

The page: https://webfluent.monzeromer.dev/reference/alert

Inline notice; a danger or warning one is announced as an alert.

```wf
Alert(_ message: String, tone: Enum) { … }
```

## Example

```wf
Stack(gap: .sm) {
    Alert("Your changes were saved.").success
    Alert("Your trial ends in 3 days.").warning
    Alert("The server is not answering. We will try again in a minute.").danger
    Alert("A new version of the editor is available.").info
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `message` | `String (positional)` | The message; its block, when it has one, follows it |
| `tone:` | `one of its cases` | The kind of notice, which sets its colours; `.danger` and `.warning` are announced at once (`role="alert"`), the rest politely (`role="status"`). Written as a flag, `.success`, or by name, `tone: .success`.

- `.success`: Green: done, healthy
- `.danger`: Red: destructive, failed
- `.warning`: Amber: attention
- `.info`: Blue: neutral notice |

## Flags

`.success` `.danger` `.warning` `.info`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
