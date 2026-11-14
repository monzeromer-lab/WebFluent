# Unsafe.Html

> Markup, put in as markup. It is **not** sanitised — the name says so — and sanitize(…) is one call away for anything the project did not write itself.

The page: https://webfluent.monzeromer.dev/reference/unsafe-html

Markup, put in as markup. It is **not** sanitised — the name says so — and sanitize(…) is one call away for anything the project did not write itself.

```wf
Unsafe.Html(_ markup: String)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `markup` | `String  (positional)` | The HTML to insert |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state body = "<p>From the CMS</p>"
Unsafe.Html(sanitize(body))
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C03](https://webfluent.monzeromer.dev/errors/C03.md): A part outside the component it belongs to
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [V03](https://webfluent.monzeromer.dev/errors/V03.md): Markup put in as markup

Before WebFluent 5.0.
