# Listbox.Option

> One option: its words, and the value bind: holds when it is chosen.

The page: https://webfluent.monzeromer.dev/reference/listbox-option

One option: its words, and the value bind: holds when it is chosen.

```wf
Listbox.Option(_ label: String, value: Any, icon: String, aria-disabled: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label` | `String  (positional)` | Its words |
| `value:` | `Any  (required)` | What `bind:` holds when it is chosen |
| `icon:` | `String` | An icon before the words |
| `aria-disabled:` | `String` | `"true"`: shown, passed over, never chosen |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

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
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
