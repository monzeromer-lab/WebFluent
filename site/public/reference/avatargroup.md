# AvatarGroup

> Avatars overlapping in a row: past max: the rest are one +3.

The page: https://webfluent.monzeromer.dev/reference/avatargroup

Avatars overlapping in a row: past max: the rest are one +3.

```wf
AvatarGroup(max: Number, label: String) { … }
```

## Example

```wf
AvatarGroup(max: 4, label: "Reviewers") {
    Avatar(src: "/icon-512.png", alt: "Ada Lovelace")
    Avatar(initials: "GH").primary
    Avatar(initials: "AT")
    Avatar(initials: "ED")
    Avatar(initials: "KJ")
    Avatar(initials: "MH")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `max:` | `Number` | How many avatars show; the rest are hidden and counted in one `+N` avatar (read "N more"). `0` shows them all |
| `label:` | `String` | Who they are: the accessible name of the group (`role="group"`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
