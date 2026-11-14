# Listbox

> A list of options always shown, one chosen (selection follows the arrow keys) or, .multiple, several: Space chooses, Shift with an arrow takes the next one in too, Ctrl or ⌘ with A takes them all; a letter goes to the next option that starts with it.

The page: https://webfluent.monzeromer.dev/reference/listbox

A list of options always shown, one chosen (selection follows the arrow keys) or, .multiple, several: Space chooses, Shift with an arrow takes the next one in too, Ctrl or ⌘ with A takes them all; a letter goes to the next option that starts with it.

```wf
Listbox(bind: State, label: String, multiple: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The chosen value, or a list of them with `.multiple` |
| `label:` | `String  (required)` | What is being chosen, for a screen reader |
| `multiple:` | `Bool` | Several at once: `bind:` is a list |

## Flags

`.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Listbox.Listbox.Option`

## Example

```wf
state fruit = "apple"
Listbox(bind: fruit, label: "Fruit") {
    Listbox.Option("Apple", value: "apple")
    Listbox.Option("Banana", value: "banana")
    Listbox.Option("Cherry", value: "cherry")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
