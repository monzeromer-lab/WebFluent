# PrevNext.Prev

> The page before: its title, and where it is.

The page: https://webfluent.monzeromer.dev/reference/prevnext-prev

The page before: its title, and where it is.

```wf
PrevNext.Prev(_ title: String, to: Path, label: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String  (positional)` | The page's title |
| `to:` | `Path  (required)` | Route to navigate to; the link whose `to` matches the current route carries `.active` and `aria-current="page"` |
| `label:` | `String` | The word above the title (unset, `Previous`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
PrevNext {
    PrevNext.Prev("Forms", to: "/")
    PrevNext.Next("Stores", to: "/")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
