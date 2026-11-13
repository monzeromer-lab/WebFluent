# Route

> Maps a path to a page (the original grammar; pages now declare their paths).

The page: https://webfluent.monzeromer.dev/reference/route

Maps a path to a page (the original grammar; pages now declare their paths).

```wf
Route(path: Path, page: Declaration)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `path:` | `Path` | URL pattern |
| `page:` | `Declaration` | The page to render |

## Flags

`.dropInside` `.trapFocus`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
