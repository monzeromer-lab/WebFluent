# Render Markdown

> Show Markdown text on a page with `Markdown(text)`: headings, lists, tables, code and links, drawn the same at build time and in the browser.

The page: https://webfluent.monzeromer.dev/recipes/render-markdown

`Markdown(text)` shows Markdown text on a page: release notes, a post's
body, a field a user wrote. It renders a small, predictable Markdown, the
same at build time and in the browser, so a hydrated page repaints exactly
what was painted. This page shows release notes held in a state, and one
line of inline Markdown:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
page Notes(path: "/", title: "Release notes", description: "What changed in this release.") {
    state body = "# Release notes\n\nVersion **3.3** adds:\n\n- Markdown pages\n- `wf docs`\n\n> Ship it."
    Markdown(body)
    Markdown("Inline *emphasis*, a [link](https://webfluent.monzeromer.dev/about.md) and an ![image](https://webfluent.monzeromer.dev/img.png).")
}
```

## How it works

The page shows a "Release notes" heading, a paragraph with **3.3** in bold,
a two-item list and a quote, then a line with emphasis, a link and a
picture.

What `Markdown` supports:

| Kind | Syntax |
|---|---|
| Blocks | `#`–`######` headings; paragraphs; fenced code with a language (`` ```js ``); `>` quotes; `-`/`*` and `1.` lists (one level); `---` rules; GFM tables |
| Inline | `` `code` ``, `**strong**`, `*em*` and `_em_`, links, images |

A paragraph's lines run together, as in CommonMark. To break a line, end it
with two spaces or a `\`.

Raw HTML in the text is shown as text, not run, so Markdown from a user or
an API is safe to render. A link or a picture goes through the scheme check
every other URL does ([Security](https://webfluent.monzeromer.dev/about/security.md#what-the-compiler-guarantees)):

- A link or picture whose URL a browser would run — `javascript:`,
  `vbscript:`, a `data:` URL — is drawn as its words, with no link and no
  picture.
- A picture carried in its address (`![dot](data:image/png;base64,…)`,
  `data:image/svg+xml,…`) is drawn, since an `<img>` runs nothing it
  shows. A link to one is still drawn as its words.
- A site-relative URL (`/about`, `/img.png`) is addressed from the site's
  `base_path`, as `Link(to:)`'s is.

### Write a table

To write a table, give a header row, a delimiter row with as many cells,
then the rows, as in GitHub's Markdown:

```md
| Function | Does | Where |
|---|:-:|--:|
| `log(x)` | Prints to the console | anywhere |
| `t("key")` | A translation \| a plural | anywhere |
```

The rules:

- A `:` at the left, at both ends or at the right of a delimiter cell
  aligns its column left, centre or right.
- The pipes at either end of a row may be left out, and `\|` is a pipe
  inside a cell.
- A row with fewer cells than the header has empty ones; cells past the
  header's are dropped.
- The rows run to a blank line or another block, and a table may start
  right after a paragraph's last line.
- A cell takes the inline Markdown a paragraph does.

### Escape a character with a backslash

To show a punctuation mark without its Markdown meaning, put a backslash
before it, as in CommonMark:

| You write | It shows |
|---|---|
| `\*not em\*` | `*not em*`, with no emphasis |
| `\[not a link\](https://webfluent.monzeromer.dev/x)` | the brackets and parentheses, with no link |
| `\#` at the start of a line | `#`, with no heading |
| `1\.` | `1.`, with no list |
| `\\` | one backslash |

Every ASCII punctuation mark can be escaped this way (`\-`, `\{`, `\(`,
`\_`, `\&` and the rest), and an escaped mark is only the character it is:
it makes no link, no emphasis and no entity.

How escapes behave in links:

- In a link's words, an escaped bracket is a bracket like any other
  (`[a\]b](/c)`). The words hold no unescaped `[`, so in `[a [b](https://webfluent.monzeromer.dev/c)` only
  `b` is the link.
- In a URL, an escape is read as the character it is, so
  `[x](javascript\:alert\(1\))` is checked as the `javascript:alert(1)` URL
  it says, and drawn as its words.

Where a backslash is shown as written:

- before anything that is not ASCII punctuation, as in `C:\Users`;
- in a code span or a fence, where nothing is Markdown;
- at the end of a paragraph's, a heading's or a list item's last line,
  which has no line after it to break to.

A fence's language takes escapes too: `c\+\+` after the backticks is `c++`.

## Variations

### Show text somebody else wrote with `.inert`

Documentation from a language server, a `///` comment or a package's
readme is text somebody else wrote. It often has HTML entities in it, and
links and pictures that should not be followed or fetched from where you
show it. To show it, add `.inert`:

```wf
page Hover(path: "/", title: "Hover", description: "Documentation from a language server.") {
    state docs = "Returns a list of &lt;T&gt; &mdash; see [the guide](https://example.com/guide).\n\n![build](https://ci.example.com/badge.svg)"
    Heading("Hover").h1
    Markdown(docs).inert
}
```

It draws one paragraph, `Returns a list of <T> — see the guide.`, with no
link in it, and leaves the badge out. Under `.inert`:

- **Entities are decoded, then escaped.** `&lt;color&gt;` shows as
  `<color>`, and a `<script>`, written out or encoded, is shown as text. In
  `` `code` `` and in a fence an entity stays as written, as CommonMark
  leaves it. The named entities are the ones documentation is written
  with — `&lt;` `&gt;` `&amp;` `&quot;` `&apos;` `&nbsp;`, the dashes,
  quotes and arrows, and a few signs — and any numeric entity works
  (`&#60;`, `&#x3C;`).
- **A link is drawn as its words**, with no `<a>`.
- **A picture is left out**, with the space after it. A paragraph of
  nothing but pictures is left out with them.
- **Everything else is drawn as plain `Markdown` draws it**: paragraphs,
  fences, `code`, `**strong**`, `*em*` and `_em_`, headings, lists, quotes,
  rules and backslash escapes. A language server escapes its documentation
  this way: the CSS one writes `margin\-top` and
  `&lt;length\-percentage&gt;\{1,4\}`, which `.inert` shows as `margin-top`
  and `<length-percentage>{1,4}`. An escaped `&` begins no entity:
  `\&lt;` shows the four characters `&lt;`.

`inert:` also takes a value, `Markdown(docs, inert: untrusted)`, and
follows it. Plain `Markdown` is unchanged: an entity in it shows as it was
written, so a page that wrote `&lt;` to mean those four characters still
shows them.
