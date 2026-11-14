# Markdown

> Markdown text rendered as HTML: headings, paragraphs, lists, quotes, code, links, emphasis. The text is escaped first, so HTML in it is shown, not run, and a link or picture whose URL a browser would run is drawn as its words. .inert is for text somebody else wrote.

The page: https://webfluent.monzeromer.dev/reference/markdown

Markdown text rendered as HTML: headings, paragraphs, lists, quotes, code, links, emphasis. The text is escaped first, so HTML in it is shown, not run, and a link or picture whose URL a browser would run is drawn as its words. .inert is for text somebody else wrote.

```wf
Markdown(_ text: String, inert: Bool)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The Markdown text |
| `inert:` | `Bool` | Text somebody else wrote, such as a language server's documentation: its entities are decoded, a link is drawn as its words and a picture is left out |

## Flags

`.inert`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Example

```wf
state docs = "Returns &lt;T&gt;, as [the guide](https://webfluent.monzeromer.dev/docs) says."
Markdown("Some **bold** text and a [link](https://webfluent.monzeromer.dev/docs).")
Markdown(docs).inert
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
