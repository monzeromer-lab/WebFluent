# ClientOnly

> Its block drawn in the browser alone, never in the static paint, a template or a PDF: a clock, the reader's own data. A placeholder { … } block is what those show in its place.

The page: https://webfluent.monzeromer.dev/reference/clientonly

Its block drawn in the browser alone, never in the static paint, a template or a PDF: a clock, the reader's own data. A placeholder { … } block is what those show in its place.

```wf
ClientOnly { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Example

```wf
ClientOnly { Text("Drawn in your browser only") }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
