---
kind: "coming-from"
route: "/coming-from/latex"
framework: "LaTeX / Typst"
language: "latex"
description: "LaTeX and Typst ideas in WebFluent: page setup, headings, contents, page breaks, headers and footers, tables, figures, macros and slides."
---
# WebFluent for LaTeX and Typst authors

WebFluent prints PDFs and slide decks from the same language it builds
websites with, with no TeX installation and no browser. If you know LaTeX
or Typst, the shapes are familiar — a document, headings, a table of
contents, page headers and footers — but the layout is the web's: rows,
stacks and grids, styled with CSS.

## The map

| Concept | LaTeX / Typst | WebFluent |
|---|---|---|
| The paper | `\documentclass[a4paper]{article}` / `#set page(paper: "a4")` | `Document(size: "A4", margin: "20mm") { … }` |
| Title and author | `\title{…}\author{…}` / `#set document(title: …)` | `Document(title: "Q1 report", author: "Acme")` |
| A section | `\section{Results}` / `= Results` | `Heading("Results").h1` |
| A subsection | `\subsection{Method}` / `== Method` | `Heading("Method").h2` |
| A paragraph | a blank line between paragraphs | `Text("…")`, or `Markdown("…")` for several |
| Bold and italic | `\textbf{x}`, `\emph{x}` / `*x*`, `_x_` | `Text("x").bold`, `Text("x").italic`, or `**x**` in `Markdown` |
| A list | `itemize` / `- item` | `List { List.Item { Text("…") } }` |
| The contents | `\tableofcontents` / `#outline()` | `TableOfContents(levels: 2)` |
| A new page | `\newpage` / `#pagebreak()` | `PageBreak` |
| Keep together | `\begin{minipage}` / `block(breakable: false)` | `style { break-inside: avoid }` |
| Headers and footers | `fancyhdr` / `#set page(header: …)` | `Header { … }`, `Footer { Text("Page {page} of {pages}") }` |
| A table | `tabular` / `#table(…)` | `Table { Table.Head { … } Table.Body { … } }` |
| A figure | `\includegraphics{chart.png}` / `#image("chart.png")` | `Image(src: "/chart.png", alt: "…")` |
| A chart | `pgfplots` | `Chart(kind: .bar, data: SALES, x: "month", y: "web")` |
| A macro | `\newcommand{\price}[1]{…}` / `#let price(x) = …` | `component Price(_ amount: Number) { … }` |
| Data | `\input{data}`, `datatool` / `#let data = json("data.json")` | `data sales = "sales.json"`, or a `const` |
| Fonts | `fontspec` / `#set text(font: …)` | a `.ttf` or `.otf` under `fonts/`, named in `font-family` |
| Slides | Beamer frames / Touying | `Presentation { Slide { … } }`, with `"output_type": "slides"` |
| Building | `pdflatex`, `latexmk` / `typst compile` | `wf build`, with `"output_type": "pdf"` in the config |

## Things that work differently

**The layout is CSS.** A document is laid out from the same HTML and CSS a
web page would get: `Row`, `Stack` and `Grid` place things side by side,
and a `style { }` block or a `.css` file styles them. There are no TeX boxes
and glue to tune.

**Breaking across pages is a rule, not a command.** Paragraphs keep two
lines on each side of a break, a heading moves with what follows it, and a
table repeats its header row on every page it continues on. Ask for the
rest with `break-inside: avoid` and `PageBreak`.

**There is no math typesetting.** A formula is a picture or an SVG you
place with `Image`. There is no bibliography system either; references are
text you write.

**Arabic and right-to-left work out of the box.** A document whose `lang`
is `ar`, `he`, `fa` or `ur` is laid out right to left, and shaping comes
with it — no extra packages.

**One source, three outputs.** The page that prints as a PDF can also be a
web page, and a `Presentation` is a deck from the same components
([PDF and slides](../learn/17-pdf-and-slides.md)).

## A component, ported

A short report with a footer and a table, in LaTeX and in WebFluent.

```latex
\documentclass[a4paper]{article}
\usepackage{fancyhdr}
\pagestyle{fancy}
\cfoot{Page \thepage}
\title{Quarterly report}
\author{Acme Studio}
\begin{document}
\maketitle
\section{Sales}
Revenue grew in every region.
\begin{tabular}{lr}
Region & Revenue \\
North & \$2.4M \\
South & \$1.9M \\
\end{tabular}
\end{document}
```

```wf
page Report(path: "/", title: "Quarterly report", description: "Sales for the quarter.") {
    Document(size: "A4", title: "Quarterly report", author: "Acme Studio") {
        Footer { Text("Page {page} of {pages}").muted.sm.center }
        Heading("Quarterly report").h1
        Text("Acme Studio").muted
        Heading("Sales").h2
        Text("Revenue grew in every region.")
        Table(caption: "Revenue by region") {
            Table.Head { Table.Row { Table.Cell("Region")  Table.Cell("Revenue") } }
            Table.Body {
                Table.Row { Table.Cell("North")  Table.Cell("$2.4M") }
                Table.Row { Table.Cell("South")  Table.Cell("$1.9M") }
            }
        }
    }
}
```

- **The footer is an element**, drawn on every page, with `page` and
  `pages` in scope.
- **The table is the web's table**, broken between rows and with its header
  repeated when it spans pages.
