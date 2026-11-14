# Rating

> Stars, as radio buttons a screen reader hears as a choice of one to max:: the arrow keys move between them; .half takes halves; .readonly only shows.

The page: https://webfluent.monzeromer.dev/reference/rating

Stars, as radio buttons a screen reader hears as a choice of one to max:: the arrow keys move between them; .half takes halves; .readonly only shows.

```wf
Rating(bind: State, max: Number, label: String, …)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State  (required)` | The rating, a `Number` |
| `max:` | `Number` | How many stars (unset, 5) |
| `label:` | `String  (required)` | What is being rated, for a screen reader |
| `half:` | `Bool` | Half stars too |
| `readonly:` | `Bool` | Shown, not chosen |

## Flags

`.half` `.readonly`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state stars = 4
Rating(bind: stars, max: 5, label: "Your rating").half
Rating(bind: stars, label: "Average").readonly
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
