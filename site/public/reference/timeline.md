# Timeline

> Events in order (<ol>), each a dot on a line, its title, its time and its block.

The page: https://webfluent.monzeromer.dev/reference/timeline

Events in order (<ol>), each a dot on a line, its title, its time and its block.

```wf
Timeline { … }
```

## Example

```wf
Timeline {
    Timeline.Item("7.0 released", at: @2026-11-10) {
        Text("Typed requests, results and error enums.")
    }
    Timeline.Item("6.1 released", at: @2026-09-02) {
        Text("Charts and QR codes on a web page.")
    }
    Timeline.Item("Public beta", at: @2026-06-15) {
        Text("The first hundred projects moved over.").muted
    }
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `Timeline`.

### Timeline.Item

One event: its title, at: its time, and its block.

```wf
Timeline.Item(_ title: String, at: Date or DateTime or String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `title` | `String  (positional)` | What happened |
| `at:` | `Date or DateTime or String` | When: a `Date`, a `DateTime` or an ISO string, written as a date |

## Drawn for it

What `Timeline` draws itself, by its class; not written.

- `Timeline.Time` (`.`): An event's time (<time>).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
