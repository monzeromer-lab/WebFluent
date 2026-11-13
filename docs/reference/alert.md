# Alert

> Inline notice; a danger or warning one is announced as an alert.

The page: https://webfluent.monzeromer.dev/reference/alert

Inline notice; a danger or warning one is announced as an alert.

```wf
Alert(_ message: String, tone: Enum) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `message` | `String  (positional)` | The message |
| `tone:` | `.success .danger .warning .info` | Kind of notice |

## Flags

`.success` `.danger` `.warning` `.info` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Alert("Your changes were saved.").success
Alert("The server is not answering.").danger
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 5.0.
