# Rating

> Stars, as radio buttons a screen reader hears as a choice of one to max:: the arrow keys move between them; .half takes halves; .readonly only shows.

The page: https://webfluent.monzeromer.dev/reference/rating

Stars, as radio buttons a screen reader hears as a choice of one to max:: the arrow keys move between them; .half takes halves; .readonly only shows.

```wf
Rating(bind: State, max: Number, label: String, …)
```

## Example

```wf
state stars = 4
Rating(bind: stars, max: 5, label: "Your rating")
Text("You gave it {stars} of 5").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State (required)` | The `Number` state it reads and writes: how many stars are chosen (halves with `.half`), 0 for none |
| `max:` | `Number` | How many stars it draws, and the highest rating |
| `label:` | `String (required)` | What is being rated: the group's name, read once; with `.readonly` it says `Quality: 4 out of 5` |
| `half:` | `Bool` | Half stars too: the rating moves in steps of 0.5 rather than 1. Written as a flag, `.half`, or `half: true`. |
| `readonly:` | `Bool` | Shown, not chosen: drawn as an image (`role="img"`) that says the rating, with no stops for Tab. Written as a flag, `.readonly`, or `readonly: true`. |

## Flags

`.half` `.readonly`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
