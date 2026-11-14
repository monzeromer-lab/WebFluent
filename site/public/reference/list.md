# List

> List of its children; ordered for a numbered list.

The page: https://webfluent.monzeromer.dev/reference/list

List of its children; ordered for a numbered list.

```wf
List(ordered: Bool) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `ordered:` | `Bool` | Numbered (`<ol>`) |

## Flags

`.ordered`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `List.List.Item`

## Example

```wf
List {
    List.Item { Text("Milk") }
    List.Item { Text("Eggs") }
}
List.ordered { List.Item { Text("First") } }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
