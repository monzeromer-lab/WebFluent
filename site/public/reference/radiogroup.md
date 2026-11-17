# RadioGroup

> One choice of several, as native radio buttons under a legend: the arrow keys move between them, and .cards draws each as a card with its hint.

The page: https://webfluent.monzeromer.dev/reference/radiogroup

One choice of several, as native radio buttons under a legend: the arrow keys move between them, and .cards draws each as a card with its hint.

```wf
RadioGroup(bind: State, label: String, cards: Bool) { … }
```

## Example

```wf
state plan = "team"
RadioGroup(bind: plan, label: "Plan").cards {
    RadioGroup.Option("Free", value: "free", hint: "For trying it out")
    RadioGroup.Option("Team", value: "team", hint: "Up to 20 people, $12 each")
    RadioGroup.Option("Scale", value: "scale", hint: "Unlimited people, single sign-on")
}
Text("Chosen: {plan}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The chosen value |
| `label:` | `String  (required)` | What is being chosen: the group's legend |
| `cards:` | `Bool` | Each option a card, its hint beneath its words |

## Flags

`.cards`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `RadioGroup`.

### RadioGroup.Option

One choice: its words, its value, and a hint said with it.

```wf
RadioGroup.Option(_ label: String, value: String or Number or Bool, hint: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `value:` | `String or Number or Bool  (required)` | What `bind:` holds when it is chosen |
| `hint:` | `String` | A line beneath it, read with it |

## Drawn for it

What `RadioGroup` draws itself, by its class; not written.

- `RadioGroup.Legend` (`.`): Its legend.
- `RadioGroup.Hint` (`.`): An option's hint.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
