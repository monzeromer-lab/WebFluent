# TransferList

> Two lists side by side — what can be chosen and what is — with buttons that move the selected across, or all of them; bind: holds the keys of the chosen.

The page: https://webfluent.monzeromer.dev/reference/transferlist

Two lists side by side — what can be chosen and what is — with buttons that move the selected across, or all of them; bind: holds the keys of the chosen.

```wf
TransferList(bind: State, options: [Item], text: (Item) -> String, …)
```

## Example

```wf
state picked: [String] = ["ada"]
TransferList(bind: picked, options: [
    { id: "ada", name: "Ada Lovelace" },
    { id: "grace", name: "Grace Hopper" },
    { id: "alan", name: "Alan Turing" },
    { id: "katherine", name: "Katherine Johnson" },
], text: p => p.name, key: p => p.id, label: "Reviewers", from: "Everyone")
Text("Reviewers: {picked.join(", ")}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State (required)` | A list state holding the keys of the chosen options, kept in the order of `options:`; anything else reads as nothing chosen |
| `options:` | `[Item] (required)` | Everything that can be chosen, in the order both lists show it |
| `text:` | `(Item) -> String` | What an option says: `o => o.name` |
| `key:` | `(Item) -> String or (Item) -> Number` | What identifies an option, and what `bind:` holds for it: `o => o.id` |
| `label:` | `String (required)` | What is being chosen: the name of the whole control |
| `from:` | `String` | The heading over the list of what is not chosen |
| `to:` | `String` | The heading over the list of what is chosen |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
