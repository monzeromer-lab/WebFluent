# CheckboxGroup

> Check boxes under a legend whose bind: is the list of the values ticked, with all: a box above them for every one at once — ticked when all are, mixed when some are.

The page: https://webfluent.monzeromer.dev/reference/checkboxgroup

Check boxes under a legend whose bind: is the list of the values ticked, with all: a box above them for every one at once — ticked when all are, mixed when some are.

```wf
CheckboxGroup(bind: State, label: String, all: String) { … }
```

## Example

```wf
state channels: [String] = ["email"]
CheckboxGroup(bind: channels, label: "Notify me by", all: "Every channel") {
    CheckboxGroup.Option("Email", value: "email")
    CheckboxGroup.Option("Text message", value: "sms")
    CheckboxGroup.Option("Push notification", value: "push")
}
Text("Chosen: {channels.join(", ")}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The list of the values ticked |
| `label:` | `String  (required)` | What is being chosen: the legend |
| `all:` | `String` | The words of a box that ticks every option at once: `Select all` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `CheckboxGroup`.

### CheckboxGroup.Option

One box: its words and its value.

```wf
CheckboxGroup.Option(_ label: String, value: String or Number or Bool)
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `value:` | `String or Number or Bool` | What the list holds while it is ticked; unset, its words |

## Drawn for it

What `CheckboxGroup` draws itself, by its class; not written.

- `CheckboxGroup.Legend` (`.`): Its legend.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
