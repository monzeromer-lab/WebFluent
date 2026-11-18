# Pagination

> Pages of a list: Previous, the first and last pages, those around the current one with a gap for the rest, and Next. The current page is aria-current="page"; bind: holds it, from 1.

The page: https://webfluent.monzeromer.dev/reference/pagination

Pages of a list: Previous, the first and last pages, those around the current one with a gap for the rest, and Next. The current page is aria-current="page"; bind: holds it, from 1.

```wf
Pagination(bind: State, pages: Number, siblings: Number, …)
```

## Example

```wf
state page = 4
Pagination(bind: page, pages: 12, siblings: 1, label: "Results")
Text("Showing page {page} of 12").muted.sm
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State (required)` | A Number state, the current page counted from 1: the buttons write it |
| `pages:` | `Number (required)` | How many pages there are; the last button and Next stop there |
| `siblings:` | `Number` | How many page buttons it shows on each side of the current one; the first and last always show, `…` stands for the rest |
| `label:` | `String` | Its accessible name (`aria-label` on its `<nav>`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Drawn for it

What `Pagination` draws itself, by its class; not written.

- `Pagination.Prev` (`.`): The previous page.
- `Pagination.Next` (`.`): The next page.
- `Pagination.Page` (`.`): A page.
- `Pagination.Gap` (`.`): The pages left out.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
