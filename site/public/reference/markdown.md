# Markdown

> Markdown text rendered as HTML: headings, paragraphs, lists, quotes, code, links, emphasis. The text is escaped first, so HTML in it is shown, not run, and a link or picture whose URL a browser would run is drawn as its words. .inert is for text somebody else wrote.

The page: https://webfluent.monzeromer.dev/reference/markdown

Markdown text rendered as HTML: headings, paragraphs, lists, quotes, code, links, emphasis. The text is escaped first, so HTML in it is shown, not run, and a link or picture whose URL a browser would run is drawn as its words. .inert is for text somebody else wrote.

```wf
Markdown(_ text: String, inert: Bool)
```

## Example

```wf
Markdown("""
    ## Getting started

    Install `wf`, then:

    - write a page in **src/pages**
    - run `wf serve` and open it
    - read [the guide](/learn) when you are stuck

    ```wf
    page Home(path: "/", title: "Home") { Heading("Hello").h1 }
    ```
    """)
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String (positional)` | The Markdown, drawn as HTML at build time and live; it is escaped first, so HTML in it is shown, not run |
| `inert:` | `Bool` | Text somebody else wrote, such as a language server's documentation: its entities are decoded, a link is drawn as its words and a picture is left out. Written as a flag, `.inert`, or `inert: true`. |

## Flags

`.inert`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare

Before WebFluent 5.0.
