# PrevNext

> The pages before and after this one in an ordered set — a guide's chapters: each a link that says which way it goes and the page's title.

The page: https://webfluent.monzeromer.dev/reference/prevnext

The pages before and after this one in an ordered set — a guide's chapters: each a link that says which way it goes and the page's title.

```wf
PrevNext(label: String) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Its name for a screen reader (unset, `Pages`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `PrevNext.PrevNext.Prev`
- `PrevNext.PrevNext.Next`

## Example

```wf
PrevNext {
    PrevNext.Prev("Forms", to: "/")
    PrevNext.Next("Stores", to: "/")
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
