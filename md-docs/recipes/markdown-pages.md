---
kind: "recipe"
route: "/recipes/markdown-pages"
group: "content"
description: "Write a page as a Markdown file under `src/`: front matter names its route, title and layout, and the rest is the body, framed by a component."
from: ["/docs/guide/content#write-a-page-as-a-md-file"]
outputs: ["website", "app"]
---
# Write a page as a Markdown file

A `.md` file under `src/` is a page: the lines between the two `---` at its
top — its *front matter* — name the page's route, title and layout, and the
rest is its body. It suits docs, notes and articles, where most of the page
is prose. A docs site is a folder of `.md` files, one layout component, and
an `app`:

**Needs:** the two `.md` pages shown under [How it works](#how-it-works),
in `src/guide/`.

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

app {
    Router
}
```

## How it works

Each page of the site is a `.md` file. This one, `src/guide/install.md`, is
the docs page at `/guide/install`:

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

The sidebar's second link is `src/guide/pages.md`, the page at
`/guide/pages`:

````md
---
path: /guide/pages
title: Pages
description: One file per page, and the route it answers.
layout: DocsShell
---

# Pages

Each `.md` file under `src/` is a page. Its front matter names the route.
````

With the two files and `DocsShell` in place, `wf serve` shows both pages,
each framed by the sidebar.

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
([Pages and routing](../learn/05-pages-and-routing.md#layouts)). In
`DocsShell`, `slot` declares the default slot and `children` is where the
page's body lands, beside the sidebar.

(`wf docs` is something else: a gallery of every component the project can
use — [CLI](../reference/cli.md#wf-docs).)
