# Timeline

> Events in order (<ol>), each a dot on a line, its title, its time and its block.

The page: https://webfluent.monzeromer.dev/reference/timeline

Events in order (<ol>), each a dot on a line, its title, its time and its block.

```wf
Timeline { … }
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

- `Timeline.Timeline.Item`
- `Timeline.Timeline.Time`

## Example

```wf
Timeline {
    Timeline.Item("Released", at: "2026-11-14") { Text("Version 6.1") }
    Timeline.Item("Planned") { Text("Version 7") }
}
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
