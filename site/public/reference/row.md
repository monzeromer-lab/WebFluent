# Row

> Horizontal flex layout. It stays a row at every width unless .stacks or a responsive direction says otherwise.

The page: https://webfluent.monzeromer.dev/reference/row

Horizontal flex layout. It stays a row at every width unless .stacks or a responsive direction says otherwise.

```wf
Row(gap: Enum, align: Enum, justify: Enum, …) { … }
```

## Example

```wf
Row(gap: .md, align: .center, justify: .between) {
    style { padding: 0.75rem 1rem; border: 1px solid $color-border; border-radius: $radius-md }
    Row(gap: .sm, align: .center) {
        Avatar(initials: "AL").primary
        Stack(gap: .xs) {
            Text("Ada Lovelace").bold
            Text("Owner").muted.sm
        }
    }
    Badge("Active").success
    Button("Manage").sm
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `gap:` | `one of its cases` | Space between its children, from the theme's spacing scale (`wf-gap--*`, `gap:`); a flag (`.lg`) or a value per breakpoint (`{ base: .sm, lg: .lg }`). Written as a flag, `.xs`, or by name, `gap: .xs`.

- `.xs`: Extra small: `$spacing-xs`, 0.25rem
- `.sm`: Small: `$spacing-sm`, 0.5rem
- `.md` (the default): Medium: `$spacing-md`, 1rem (the default)
- `.lg`: Large: `$spacing-lg`, 1.5rem
- `.xl`: Extra large: `$spacing-xl`, 1.5–2rem, growing with the screen |
| `align:` | `one of its cases` | Where its children sit across the row (vertically) (`align-items`); a flag for `.stretch`/`.baseline`, or a value per breakpoint. Written as a flag, `.start`, or by name, `align: .start`.

- `.start`: `align-items: flex-start`
- `.center`: `align-items: center`
- `.end`: `align-items: flex-end`
- `.stretch` (the default): `align-items: stretch`: each child fills that axis
- `.baseline`: `align-items: baseline`: text baselines line up |
| `justify:` | `one of its cases` | How its children share the free space along the row (`justify-content`); a flag for `.between`/`.around`/`.evenly`, or a value per breakpoint. Written as a flag, `.start`, or by name, `justify: .start`.

- `.start` (the default): `justify-content: flex-start`: packed at the start
- `.center`: `justify-content: center`
- `.end`: `justify-content: flex-end`: packed at the end
- `.between`: `justify-content: space-between`: the first and last at the edges
- `.around`: `justify-content: space-around`: equal space around each
- `.evenly`: `justify-content: space-evenly`: equal space everywhere |
| `direction:` | `one of its cases` | Which way its children run (`flex-direction`): a case, `direction: .column`, or a value per breakpoint, `{ base: .column, md: .row }`. Written as a flag, `.row`, or by name, `direction: .row`.

- `.row` (the default): Side by side (`flex-direction: row`)
- `.column`: One above another (`flex-direction: column`) |
| `stacks:` | `Bool` | Below 768px it turns into a column (`flex-direction: column`); without it the row stays a wrapping row at every width. Written as a flag, `.stacks`, or `stacks: true`. |

## Flags

`.xs` `.sm` `.md` `.lg` `.xl` `.stretch` `.baseline` `.between` `.around` `.evenly` `.row` `.column` `.stacks`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Before WebFluent 5.0.
