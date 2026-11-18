# Code

> Inline code, or a code block with block; language: colours it.

The page: https://webfluent.monzeromer.dev/reference/code

Inline code, or a code block with block; language: colours it.

```wf
Code(_ content: String or Number or Bool, block: Bool, language: String)
```

## Example

```wf
Stack(gap: .sm) {
    Text { Text("Start the dev server with ")  Code("wf serve")  Text(", then open the page.") }
    Code("""
        page Home(path: "/", title: "Home") {
            state count = 0
            Button("+1") { on click { count = count + 1 } }
        }
        """, language: "wf").block
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `String or Number or Bool (positional)` | The code, as text in a monospace font |
| `block:` | `Bool` | A block of several lines that keeps its line breaks and scrolls sideways, on the terminal colours, rather than a span in a line. Written as a flag, `.block`, or `block: true`. |
| `language:` | `String` | Colours the code as this language: `wf` (or `wfx`), `json`, `bash` (`sh`, `shell`) or `css`; any other is shown plain |

## Flags

`.block`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
