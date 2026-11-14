# Breadcrumb

> Breadcrumb trail. An item without to is the current page.

The page: https://webfluent.monzeromer.dev/reference/breadcrumb

Breadcrumb trail. An item without to is the current page.

```wf
Breadcrumb { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Breadcrumb.Breadcrumb.Item`

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
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
