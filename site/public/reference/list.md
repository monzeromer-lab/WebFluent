# List

> List of its children; ordered for a numbered list.

The page: https://webfluent.monzeromer.dev/reference/list

List of its children; ordered for a numbered list.

```wf
List(ordered: Bool) { … }
```

## Example

```wf
Stack(gap: .md) {
    List {
        List.Item { Text("Unlimited projects") }
        List.Item { Text("Custom domains") }
        List.Item { Text("Priority support").bold }
    }
    List.ordered {
        List.Item { Code("wf init shop") }
        List.Item { Text("Write your first page") }
        List.Item { Code("wf serve") }
    }
}
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

Written under its name, inside `List`.

### List.Item

One entry (<li>).

```wf
List.Item { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
