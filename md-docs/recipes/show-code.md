---
kind: "recipe"
route: "/recipes/show-code"
group: "content"
description: "Show code on a page with `Code`: inline in a sentence, or as a block coloured by language — wf, json, bash or css — at build time and in the browser."
from: ["/docs/guide/content#show-code-on-the-page"]
outputs: ["website", "app", "pdf", "slides", "elements", "desktop"]
---
# Show code on a page

`Code` shows code: inline inside a sentence, or as a block, coloured by its
language. This page mixes inline code into a sentence and shows two
coloured blocks:

```wf
page Snippets(path: "/", title: "Snippets", description: "Code on the page.") {
    Heading("Snippets").h1
    Text("Run ")  Code("wf serve")  Text(" to start.")
    Code("page Home(path: \"/\") {\n    Text(\"Hi\")\n}", language: "wf").block
    Code("$ wf build", language: "bash").block
}
```

## How it works

- **`Code("…")`** is inline code. Placed between two `Text`s, it reads as
  part of the sentence.
- **`Code("…").block`** is a block, for code over several lines.
- **`language:`** colours it — `wf`, `json`, `bash` or `css`. The colouring
  is done at build time and in the browser alike, with the theme's
  `syntax-*` tokens, and a `Markdown` fence in one of those languages is
  coloured the same way.
