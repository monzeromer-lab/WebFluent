# Unsafe

> The one door for markup a page did not write. It has a single part, Unsafe.Html.

The page: https://webfluent.monzeromer.dev/reference/unsafe

The one door for markup a page did not write. It has a single part, Unsafe.Html.

```wf
Unsafe
```

## Flags

`.dropInside` `.trapFocus`

## Parts

- `Unsafe.Unsafe.Html`

## Example

```wf
state body = "<p>From the CMS</p>"
Unsafe.Html(sanitize(body))
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
