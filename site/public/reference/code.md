# Code

> Code in a line of text; language: colours it. A block of code is a CodeBlock.

The page: https://webfluent.monzeromer.dev/reference/code

Code in a line of text; language: colours it. A block of code is a CodeBlock.

```wf
Code(_ content: String or Number or Bool, block: Bool, language: String)
```

**Deprecated since 8.0:** write `CodeBlock(…)` instead. It still draws as it always has through every 8.x release and is removed in 9.0; `wf fix --code V13` rewrites it where the rewrite is exact. `CodeBlock` draws the same code in a box of its own that a keyboard can scroll, and takes `title:`, `.copy` and `.lines`. `Code` stays for code in a line of text.

## Example

```wf
Stack(gap: .sm) {
    Text { Text("Start the dev server with ")  Code("wf serve")  Text(", then open the page.") }
    Text { Text("A page opens with ")  Code("page Home(path: \"/\")", language: "wf")  Text(".") }
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `content` | `String or Number or Bool (positional)` | The code, as text in a monospace font |
| `block:` | `Bool` | A block of several lines that keeps its line breaks and scrolls sideways, on the terminal colours, rather than a span in a line — deprecated since 8.0: a block of code is a `CodeBlock`. Written as a flag, `.block`, or `block: true`. |
| `language:` | `String` | Colours the code as this language: `wf` (or `wfx`), `json`, `bash` (`sh`, `shell`) or `css`; any other is shown plain |

## Flags

`.block`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
