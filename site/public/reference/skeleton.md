# Skeleton

> Placeholder shape shown while content loads.

The page: https://webfluent.monzeromer.dev/reference/skeleton

Placeholder shape shown while content loads.

```wf
Skeleton(height: String, width: String, size: String, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `height:` | `String` | CSS height |
| `width:` | `String` | CSS width |
| `size:` | `String` | Diameter of a `circle` |
| `circle:` | `Bool` | A round placeholder |

## Flags

`.circle`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
Skeleton(height: "20px", width: "200px")
Skeleton(size: "48px").circle
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
