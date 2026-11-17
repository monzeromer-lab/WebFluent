# LoadingOverlay

> A region covered while it is busy: aria-busy on it, its content out of reach, and a spinner with label: over it — the region, not the page.

The page: https://webfluent.monzeromer.dev/reference/loadingoverlay

A region covered while it is busy: aria-busy on it, its content out of reach, and a spinner with label: over it — the region, not the page.

```wf
LoadingOverlay(visible: Bool, label: String) { … }
```

## Example

```wf
state saving = false
state name = "Ada Lovelace"
LoadingOverlay(visible: saving, label: "Saving your profile") {
    Card {
        Input(bind: name, label: "Name")
        Button("Save").primary { on click { saving = true } }
    }
}
Button("Finish saving") { on click { saving = false } }
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `visible:` | `Bool  (required)` | While it holds, the region is busy |
| `label:` | `String` | What it is busy with (unset, `Loading`) |

## Flags

`.visible`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Drawn for it

What `LoadingOverlay` draws itself, by its class; not written.

- `LoadingOverlay.Content` (`.`): The region.
- `LoadingOverlay.Cover` (`.`): What covers it while it is busy.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
