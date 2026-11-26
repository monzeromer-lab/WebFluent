### Typography

| Component | Usage |
|-----------|-------|
| `Text` | `Text("Hello").bold.muted.center` |
| `Heading` | `Heading("Title").h1` — levels: `.h1` … `.h6` (`.h2` is the default) |
| `Code` | `Code("const x = 1")` — code in a line of text; `language:` colours it. `.block` is deprecated since 8.0 (`V13`): a block of code is a `CodeBlock` |
| `Blockquote` | `Blockquote { Text("Quote text") }` |
| `Unsafe.Html` | `Unsafe.Html(sanitize(body))` — markup, as markup. The one door for HTML a page did not write; every use draws a `V03` |
| `Markdown` | `Markdown(text)` — a small Markdown rendered as HTML, at build time and live: `#` headings, paragraphs (a line ending in two spaces or `\` breaks, and the paragraph's last line shows its `\`; any other newline is a space, as in CommonMark), fenced code, `>` quotes, one-level `-`/`1.` lists, `---`, GFM tables (a header row, a `|---|:-:|` row, then rows; `\|` is a pipe in a cell), `` `code` ``, `**strong**`, `*em*`, `[text](url)` (its words hold no unescaped `[`), `![alt](src)`; a backslash before ASCII punctuation is that character and nothing more, as in CommonMark (`\*not em\*`, `\[not a link\]`, `\#` starting a line is no heading, `\\` is one backslash; in a URL `javascript\:` is checked as `javascript:`), and a backslash before anything else, or in code or a fence, is shown; the text is escaped first, so HTML in it is shown, not run, and a link or picture whose URL a browser would run (`javascript:`, `vbscript:`, `data:`) is drawn as its words — a picture carried in its address (`![dot](data:image/png;base64,…)`) is drawn, a link to one is not; a site-relative one takes the `base_path`. `Markdown(text).inert` is for text somebody else wrote, such as a language server's or a `///` comment's documentation: entities are decoded before escaping (`&lt;T&gt;` shows `<T>`, a `<script>` stays text; in `` `code` `` and fences they stay as written), a link is drawn as its words, a picture is left out with the space after it, and the rest, backslash escapes too, is drawn as plain `Markdown` draws it (a language server's `margin\-top` shows `margin-top`) |
| `Highlight` | `Highlight(result.title, query: q)` — text with every match of `query:` marked (`<mark>`), its case ignored, in the static paint and live |

Text flags: `.bold`, `.italic`, `.underline`, `.uppercase`, `.lowercase`, `.left`, `.center`, `.right`, `.muted`, `.sm`, `.lg`, `.heading`, `.subtitle`, and the tones `.primary`, `.secondary`, `.danger`, `.success`, `.warning`, `.info`

