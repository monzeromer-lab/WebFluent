# Item

> A row of a list, a settings page or a search's results: Item.Media at its start, its title: and description:, then its block, and Item.Actions at its end.

The page: https://webfluent.monzeromer.dev/reference/item

A row of a list, a settings page or a search's results: Item.Media at its start, its title: and description:, then its block, and Item.Actions at its end.

```wf
Item(title: String, description: String) { … }
```

## Example

```wf
state push = true
state digest = false
Stack(gap: .sm) {
    Item(title: "Push notifications", description: "Sent to your phone when a deploy fails") {
        Item.Media { Icon("bell") }
        Item.Actions { Switch(bind: push, aria-label: "Push notifications") }
    }
    Item(title: "Weekly digest", description: "A summary every Monday morning") {
        Item.Media { Icon("mail") }
        Item.Actions { Switch(bind: digest, aria-label: "Weekly digest") }
    }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `title:` | `String` | What it is: a bold line before the rest of its block |
| `description:` | `String` | A line under the title, small and muted |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Item`.

### Item.Media

What goes at its start: an icon, an avatar, a picture.

```wf
Item.Media { … }
```

### Item.Actions

What goes at its end: a switch, a button, a menu.

```wf
Item.Actions { … }
```

## Drawn for it

What `Item` draws itself, by its class; not written.

- `Item.Content` (`.`): Its title, description and block.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
