# Breadcrumb.Item

> One step of the trail.

The page: https://webfluent.monzeromer.dev/reference/breadcrumb-item

One step of the trail.

```wf
Breadcrumb.Item(to: Path) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `to:` | `Path` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |

## Flags

`.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Breadcrumb {
    Breadcrumb.Item(to: "/") { Text("Home") }
    Breadcrumb.Item(to: "/docs") { Text("Docs") }
    Breadcrumb.Item { Text("Components") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
