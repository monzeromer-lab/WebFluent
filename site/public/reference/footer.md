# Footer

> Repeated at the bottom of every PDF page (or those on: names), where page and pages are the page's number and the count; a <footer> on the web.

The page: https://webfluent.monzeromer.dev/reference/footer

Repeated at the bottom of every PDF page (or those on: names), where page and pages are the page's number and the count; a <footer> on the web.

```wf
Footer(on: Enum) { … }
```

## Example

```wf
Document(title: "Invoice INV-2041") {
    Footer {
        Row(justify: .between) {
            Text("Northwind Ltd · VAT GB123456789").muted.sm
            Text("Page {page} of {pages}").muted.sm
        }
    }
    Heading("Invoice INV-2041").h1
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `on:` | `one of its cases` | Which pages it is drawn on, in the bottom margin. Written as a flag, `.all`, or by name, `on: .all`.

- `.all` (the default): Every page
- `.first`: The first page only
- `.rest`: Every page but the first
- `.odd`: Odd pages (1, 3, 5 …)
- `.even`: Even pages |

## Flags

`.all` `.first` `.rest` `.odd` `.even`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
