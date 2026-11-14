# Pagination

> Pages of a list: Previous, the first and last pages, those around the current one with a gap for the rest, and Next. The current page is aria-current="page"; bind: holds it, from 1.

The page: https://webfluent.monzeromer.dev/reference/pagination

Pages of a list: Previous, the first and last pages, those around the current one with a gap for the rest, and Next. The current page is aria-current="page"; bind: holds it, from 1.

```wf
Pagination(bind: State, pages: Number, siblings: Number, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The current page, from 1 |
| `pages:` | `Number  (required)` | How many pages there are |
| `siblings:` | `Number` | How many pages either side of the current one it shows (unset, 1) |
| `label:` | `String` | Its name for a screen reader; unset, `Pagination` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Parts

- `Pagination.Pagination.Prev`
- `Pagination.Pagination.Next`
- `Pagination.Pagination.Page`
- `Pagination.Pagination.Gap`

## Example

```wf
state page = 5
Pagination(bind: page, pages: 12, siblings: 1)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
