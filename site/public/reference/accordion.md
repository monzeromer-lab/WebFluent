# Accordion

> Sections a reader opens one by one, each a native <details>: Accordion.Item("Shipping") { … }. .single keeps one open at a time.

The page: https://webfluent.monzeromer.dev/reference/accordion

Sections a reader opens one by one, each a native <details>: Accordion.Item("Shipping") { … }. .single keeps one open at a time.

```wf
Accordion(single: Bool) { … }
```

## Example

```wf
Accordion.single {
    Accordion.Item("How long does shipping take?") {
        Text("Two to four working days in the EU, a week elsewhere.")
    }
    Accordion.Item("Can I return an order?", open: true) {
        Text("Within thirty days, unused and in its box.")
        Link("Start a return", to: "/learn")
    }
    Accordion.Item("Do you ship abroad?") {
        Text("To 40 countries; the checkout lists them.")
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `single:` | `Bool` | One open at a time: opening one closes the others (every item given one `name`). Written as a flag, `.single`, or `single: true`. |

## Flags

`.single`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Accordion`.

### Accordion.Item

One section: its title is what a reader presses, its block what opens.

```wf
Accordion.Item(_ title: String, open: Bool, name: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String (positional)` | What a reader presses: the `<summary>` |
| `open:` | `Bool` | Open when it is drawn; the reader opens and closes it after. Written as a flag, `.open`, or `open: true`. |
| `name:` | `String` | Items of one name open one at a time, as `<details name>` does: what `.single` writes for you |

Flags: `.open`

## Drawn for it

What `Accordion` draws itself, by its class; not written.

- `Accordion.Summary` (`.`): What a reader presses to open an item: its title.
- `Accordion.Panel` (`.`): What opens: the item's block.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
