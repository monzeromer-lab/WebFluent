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
| `bind:` | `State (required)` | The list state it reads and writes: the `value:` of every option ticked, in the order they were ticked |
| `label:` | `String (required)` | What is being chosen: drawn as the group's legend, which names it for a screen reader |
| `all:` | `String` | The words of a first box that ticks every option or none, mixed while some are ticked; only when the options are written out, not drawn by a `for` or an `if` |

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
| `label` | `String (positional)` | The option's words, beside its box; also its value when it has no `value:` |
| `value:` | `String or Number or Bool` | What the list holds while it is ticked |

## Drawn for it

What `CheckboxGroup` draws itself, by its class; not written.

- `CheckboxGroup.Legend` (`.`): Its legend.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
