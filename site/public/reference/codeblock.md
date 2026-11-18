# CodeBlock

> A block of code with a bar above it: a file name, a copy button (.copy), and line numbers (.lines). language: colours it, as on Code; highlight: marks lines; files: makes it a tab for each file.

The page: https://webfluent.monzeromer.dev/reference/codeblock

A block of code with a bar above it: a file name, a copy button (.copy), and line numbers (.lines). language: colours it, as on Code; highlight: marks lines; files: makes it a tab for each file.

```wf
CodeBlock(_ code: String, highlight: [Number], files: [CodeFile], language: String, …)
```

## Example

```wf
CodeBlock("""
    page Home(path: "/", title: "Home") \{
        state count = 0
        Heading("Welcome").h1
        Button("+1").primary \{ on click \{ count = count + 1 \} \}
    \}
    """, language: "wf", title: "Home.wf", highlight: [2, 4]).copy.lines
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `code` | `String (positional)` | The code, shown as written in a block that scrolls sideways |
| `highlight:` | `[Number]` | The lines to mark, counted from 1: `[3, 4]`; each gets a tint and a bar in the primary colour |
| `files:` | `[CodeFile]` | A tab for each file, each its own code block: `[{ name: "App.wf", code: app, language: "wf" }, …]`; `.copy` and `.lines` apply to each |
| `language:` | `String` | Colours the code as this language: `wf` (or `wfx`), `json`, `bash` (`sh`, `shell`) or `css`; any other is shown plain |
| `title:` | `String` | The file name or the language, in the bar above the code; with `files:`, the tab list's accessible name |
| `copy:` | `Bool` | A button in the bar that copies the code and says `Copied`. Written as a flag, `.copy`, or `copy: true`. |
| `lines:` | `Bool` | Line numbers in a column beside the code, hidden from screen readers. Written as a flag, `.lines`, or `lines: true`. |

## Flags

`.copy` `.lines`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Parts

Written under its name, inside `CodeBlock`.

### CodeBlock.Bar

The bar above the code.

```wf
CodeBlock.Bar { … }
```

### CodeBlock.Lines

The line numbers.

```wf
CodeBlock.Lines(_ text: String)
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String (positional)` | The numbers, one a line |

## Drawn for it

What `CodeBlock` draws itself, by its class; not written.

- `CodeBlock.Marks` (`.`): The marks over the lines highlight: names.
- `CodeBlock.Mark` (`.`): One line's mark.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
