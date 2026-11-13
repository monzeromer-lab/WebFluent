---
kind: "recipe"
route: "/recipes/show-code"
group: "content"
description: "To show code, use `Code(\"…\")` for inline code and `Code(\"…\").block` for a block."
from: ["/docs/guide/content#show-code-on-the-page"]
---
# Show code on a page

To show code, use `Code("…")` for inline code and `Code("…").block` for a
block. To colour it, add `language:` — `wf`, `json`, `bash` or `css`. The
colouring is done at build time and in the browser alike, with the theme's
`syntax-*` tokens, and a `Markdown` fence in one of those languages is
coloured the same way.

This page mixes inline code into a sentence and shows two coloured blocks:

```wf
page Snippets(path: "/", title: "Snippets", description: "Code on the page.") {
    Heading("Snippets").h1
    Text("Run ")  Code("wf serve")  Text(" to start.")
    Code("page Home(path: \"/\") {\n    Text(\"Hi\")\n}", language: "wf").block
    Code("$ wf build", language: "bash").block
}
```
