# TransferList

> Two lists side by side — what can be chosen and what is — with buttons that move the selected across, or all of them; bind: holds the keys of the chosen.

The page: https://webfluent.monzeromer.dev/reference/transferlist

Two lists side by side — what can be chosen and what is — with buttons that move the selected across, or all of them; bind: holds the keys of the chosen.

```wf
TransferList(bind: State, options: Any, text: Any, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The keys of what is chosen |
| `options:` | `Any  (required)` | Everything that can be chosen |
| `text:` | `Any` | What an option says: `o => o.name` (unset, the option) |
| `key:` | `Any` | What identifies an option: `o => o.id` (unset, the option) |
| `label:` | `String  (required)` | What is being chosen |
| `from:` | `String` | The first list's heading (unset, `Available`) |
| `to:` | `String` | The second list's heading (unset, `Chosen`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state picked: [String] = ["b"]
TransferList(bind: picked, options: [{ id: "a", name: "Ada" }, { id: "b", name: "Grace" }, { id: "c", name: "Alan" }], text: p => p.name, key: p => p.id, label: "Reviewers")
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
