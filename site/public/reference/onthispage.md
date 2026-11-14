# OnThisPage

> The page's headings as links, the one being read marked as the reader scrolls: a table of contents beside a long page.

The page: https://webfluent.monzeromer.dev/reference/onthispage

The page's headings as links, the one being read marked as the reader scrolls: a table of contents beside a long page.

```wf
OnThisPage(levels: Number, label: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `levels:` | `Number` | How many levels below the title it lists: `2` is `h2` and `h3` (the default 1, `h2` alone) |
| `label:` | `String` | Its title and name; unset, `On this page` |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Parts

- `OnThisPage.OnThisPage.Title`

## Example

```wf
OnThisPage(levels: 2)
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
