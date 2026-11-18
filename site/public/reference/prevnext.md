# PrevNext

> The pages before and after this one in an ordered set — a guide's chapters: each a link that says which way it goes and the page's title.

The page: https://webfluent.monzeromer.dev/reference/prevnext

The pages before and after this one in an ordered set — a guide's chapters: each a link that says which way it goes and the page's title.

```wf
PrevNext(label: String) { … }
```

## Example

```wf
PrevNext {
    PrevNext.Prev("Forms", to: "/learn")
    PrevNext.Next("Data and requests", to: "/learn/15-data")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `label:` | `String` | Its accessible name (`aria-label` on its `<nav>`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `PrevNext`.

### PrevNext.Prev

The page before: its title, and where it is.

```wf
PrevNext.Prev(_ title: String, to: Path, label: String)
```

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String (positional)` | The previous page's title, shown bold under the word |
| `to:` | `Path (required)` | The route of the previous page, linked with `rel="prev"` and navigated in place |
| `label:` | `String` | The small word above the title |

### PrevNext.Next

The page after: its title, and where it is.

```wf
PrevNext.Next(_ title: String, to: Path, label: String)
```

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String (positional)` | The next page's title, shown bold under the word |
| `to:` | `Path (required)` | The route of the next page, linked with `rel="next"` and navigated in place |
| `label:` | `String` | The small word above the title |

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
