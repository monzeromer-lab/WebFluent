# Skeleton

> Placeholder shape shown while content loads.

The page: https://webfluent.monzeromer.dev/reference/skeleton

Placeholder shape shown while content loads.

```wf
Skeleton(height: String, width: String, size: String, …)
```

## Example

```wf
Row(gap: .md, align: .center) {
    Skeleton(size: "48px").circle
    Stack(gap: .sm) {
        Skeleton(height: "16px", width: "180px")
        Skeleton(height: "12px", width: "260px")
        Skeleton(height: "12px", width: "220px")
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `height:` | `String` | Its height, a CSS length (`"20px"`, `"1.5rem"`); a value that reads state follows it |
| `width:` | `String` | Its width, a CSS length (`"200px"`, `"60%"`) |
| `size:` | `String` | The diameter of a `.circle`, a CSS length, used for any of `width:` and `height:` not given; ignored without `.circle` |
| `circle:` | `Bool` | Round instead of a rounded rectangle, for an avatar's place. Written as a flag, `.circle`, or `circle: true`. |

## Flags

`.circle`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
