# Show code on a page

> Show code on a page: inline in a sentence with Code, or as a block with CodeBlock, coloured by its language — wf, json, bash or css.

The page: https://webfluent.monzeromer.dev/recipes/show-code

`Code` shows code inline inside a sentence, and `CodeBlock` as a block, each coloured by its language. This page mixes inline code into a sentence and shows two coloured blocks:

**Needs:** nothing besides the page. It claims `/`: paste it into a new project, or replace the starter's `src/pages/Home.wf` with it.

```wf
page Snippets(path: "/", title: "Snippets", description: "Code on the page.") {
    Heading("Snippets").h1
    Text("Run ")  Code("wf serve")  Text(" to start.")
    CodeBlock("page Home(path: \"/\") {\n    Text(\"Hi\")\n}", language: "wf")
    CodeBlock("$ wf build", language: "bash")
}
```

## How it works

- **`Code("…")`** is inline code. Placed between two `Text`s, it reads as part of the sentence.
- **`CodeBlock("…")`** is a block, for code over several lines; `title:`, `.copy` and `.lines` add a bar with the file name, a copy button and line numbers.
- **`language:`** colours it — `wf`, `json`, `bash` or `css`. The colouring is done at build time and in the browser alike, with the theme's `syntax-*` tokens, and a `Markdown` fence in one of those languages is coloured the same way.
