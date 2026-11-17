# Background

> Drawn behind each PDF page's content, the size of the page: a cover, a band, a full-bleed picture.

The page: https://webfluent.monzeromer.dev/reference/background

Drawn behind each PDF page's content, the size of the page: a cover, a band, a full-bleed picture.

```wf
Background(on: Enum) { … }
```

## Example

```wf
Document(title: "Q3 report") {
    Background(on: .first) {
        Stack { style { height: 100%; background: linear-gradient(160deg, #0B1220, #1E3A8A) } }
    }
    Stack {
        style { color: #ffffff; padding-top: 240px }
        Heading("Q3 report").h1
        Text("Northwind Ltd · October 2026")
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `.all .first .rest .odd .even` | Which pages it is drawn on |

## Flags

`.all` `.first` `.rest` `.odd` `.even`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 5.2.0.
