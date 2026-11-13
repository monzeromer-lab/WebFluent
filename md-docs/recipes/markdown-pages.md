---
kind: "recipe"
route: "/recipes/markdown-pages"
group: "content"
description: "To write a page in Markdown, put a `.md` file under `src/`."
from: ["/docs/guide/content#write-a-page-as-a-md-file"]
---
# Write a page as a Markdown file

To write a page in Markdown, put a `.md` file under `src/`. Its *front
matter* — the lines between the two `---` lines at the top — names the
page's attributes; the rest is the body.

This file is a docs page at `/guide/install`:

````md
---
path: /guide/install
title: Installing
description: Get wf on your machine in a minute.
layout: DocsShell
type: article
image: /install.png
image_alt: A terminal running wf build
---

# Installing

Download a release, or build from source:

```sh
cargo install webfluent
```

Then `wf init my-site`.
````

The front matter keys are the page's attributes:

| Key | Meaning |
|---|---|
| `path` | The route. Default: `/` for `index.md`, otherwise `/<file-name>` |
| `title` | The page's title |
| `description` | The snippet a search result shows |
| `layout` | The component that frames the body |
| `image` · `image_alt` | The link-preview image, and what it shows |
| `type` | `website` (the default) or `article` |
| `noindex` | `true` keeps the page out of search results and the sitemap |
| `lang` | The language the page is written in, when it is not the site's: `en`, `ar`, `pt-BR` |
| `translates` | The page this one is a translation of, by name: `About` |

The page's name is the file's stem, capitalised: `install.md` is the page
`Install`. The body is placed as `Markdown(…)` in the page, inside the
layout's default slot when the front matter names a layout.

### Frame `.md` pages with a layout

`layout: DocsShell` frames the body in a component with a default slot —
the same kind of component a `.wf` page names as its `layout:`
([Pages and routing](../learn/05-pages-and-routing.md#layouts)):

```wf
component DocsShell {
    slot
    Row(gap: .lg) {
        Sidebar {
            Sidebar.Item(to: "/guide/install") { Text("Installing") }
            Sidebar.Item(to: "/guide/pages") { Text("Pages") }
        }
        Container { children }
    }
}
```

A docs site is then a folder of `.md` files, one layout, and a `.wf` file
for the `app`. (`wf docs` is something else: a gallery of every component
the project can use — [CLI](../reference/cli.md#wf-docs).)
