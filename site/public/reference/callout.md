# Callout

> A note set apart from the text around it — .note, .tip, .important, .warning, .caution — with an icon and a title, as GitHub's alerts are. Its text is the positional, or its block.

The page: https://webfluent.monzeromer.dev/reference/callout

A note set apart from the text around it — .note, .tip, .important, .warning, .caution — with an icon and a title, as GitHub's alerts are. Its text is the positional, or its block.

```wf
Callout(_ text: String, kind: Enum, title: String) { … }
```

## Example

```wf
Callout("Every finding has a code, and a page that says how to fix it.", title: "Before you push").tip {
    Link("Read about the checks", to: "/errors")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | What it says |
| `kind:` | `.note .tip .important .warning .caution` | What kind of note it is |
| `title:` | `String` | Its title; unset, the kind's word (`Tip`) |

## Flags

`.note` `.tip` `.important` `.warning` `.caution`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Callout`.

### Callout.Title

Its title row: the icon and the words. Drawn for it; written by hand to draw something else there.

```wf
Callout.Title { … }
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Since WebFluent 6.2.0.
