# Footer

> Repeated at the bottom of every PDF page (or those on: names), where page and pages are the page's number and the count; a <footer> on the web.

The page: https://webfluent.monzeromer.dev/reference/footer

Repeated at the bottom of every PDF page (or those on: names), where page and pages are the page's number and the count; a <footer> on the web.

```wf
Footer(on: Enum) { … }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `.all .first .rest .odd .even` | Which pages it is drawn on |

## Flags

`.all` `.first` `.rest` `.odd` `.even` `.dropInside` `.trapFocus`

## Slots

- `children`

## Example

```wf
Footer { Text("© Company Inc.").muted.sm }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.0.
