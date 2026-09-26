# 27. Content

<!--
route: guide/content
group: shipping
blurb: Markdown, pages written as .md files, code on the page, and what the build writes for search engines and link previews.
description: The Markdown element, .md pages with front matter, search and sharing tags, canonical links, sitemaps, JSON-LD and code blocks.
-->

A site is also its words: pages of prose, a blog, docs. This chapter covers
writing content in Markdown, showing code, and what the build writes for
search engines and link previews. Pre-rendering is
[Static or single-page](26-static-and-spa.md); pictures and video are
[Media](28-media.md).

## The `Markdown` element

`Markdown(text)` renders a small, predictable Markdown — the same at build
time and in the browser, so a hydrated page repaints what was painted:

```wf
page Notes(path: "/") {
    state body = "# Release notes\n\nVersion **3.3** adds:\n\n- Markdown pages\n- `wf docs`\n\n> Ship it."
    Markdown(body)
    Markdown("Inline *emphasis*, a [link](/about) and an ![image](/img.png).")
}
```

Blocks: `#`–`######` headings, paragraphs, fenced code with a language
(`` ```js ``), `>` quotes, `-`/`*` and `1.` lists (one level), `---` rules.
Inline: `` `code` ``, `**strong**`, `*em*`/`_em_`, links, images. Raw HTML
in the text is shown as text, not run — Markdown from a user or an API is
safe to render.

## `.md` pages

A Markdown file in `src/` is a page. Its front matter names the page's
attributes; the rest is the body:

````md
---
path: /guide/install
title: Installing
description: Get wf on your machine in a minute.
layout: DocsShell
type: article
---

# Installing

Download a release, or build from source:

```sh
cargo install webfluent
```

Then `wf init my-site`.
````

The front matter keys are the page's attributes: `path` (default: `/` for
`index.md`, else `/<file-name>`), `title`, `description`, `layout`, `image`,
`type`, `noindex`. The page's name is the file's stem, capitalised.

`layout: DocsShell` frames the body in a component with a default slot —
the same one a `.wf` page names ([chapter 6](06-pages-and-routing.md#layouts)):

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

A docs site is a folder of `.md` files, one layout, and a `.wf` for the
`app`. (`wf docs` is something else: a gallery of every component the
project can use — [chapter 37](37-cli.md#wf-docs-wf-registry-wf-types).)

## Search and sharing

For every page the build writes the standard head from the page's
attributes and `meta.*` in `webfluent.app.json`:

```json
{
  "meta": {
    "title": "Acme",
    "description": "Tools for small teams.",
    "lang": "en",
    "site_url": "https://acme.example",
    "site_name": "Acme Inc.",
    "image": "/og-default.png",
    "favicon": "/favicon.svg",
    "sitemap": true,
    "fonts": ["https://fonts.googleapis.com/css2?family=Inter&display=swap"],
    "stylesheets": ["/print.css"]
  }
}
```

- `<title>` and `<meta name="description">` from the page's `title:` and
  `description:` — the snippet a search result shows and a link preview
  quotes. Keep the description under ~150 characters.
- A canonical link and `og:url`, when `site_url` is set (a relative
  canonical is worse than none, so without `site_url` the build emits none).
- Open Graph and Twitter card tags: `og:title`, `og:description`,
  `og:image` (the page's `image:` or the site's), `og:type` from `type:`
  (`website` or `article`), `og:site_name`.
- JSON-LD: an `Organization`, a `WebPage` or `Article`, and a
  `BreadcrumbList` derived from the route's segments.
- `hreflang` alternates for each locale when the project has several.
- `<meta name="robots" content="noindex">` and no sitemap entry for a
  `noindex: true` page.

A page adds its own tags with `head { }` ([chapter 6](06-pages-and-routing.md#per-page-head-tags)).

```wf
type Post { slug: String, title: String, summary: String, cover: String, date: String, body: String }

data posts: [Post] = "posts.json"

page PostPage(path: "/posts/:slug", slug: String, type: "article", paths: posts.map(p => p.slug)) {
    derived post = posts.find(p => p.slug == slug)
    head {
        meta(property: "og:title", content: post?.title ?? "Post")
        meta(property: "og:image", content: post?.cover ?? "/og-default.png")
        meta(property: "article:published_time", content: post?.date ?? "")
    }
    if let p = post {
        Image(src: p.cover, alt: p.title)
        Heading(p.title).h1
        Text(format(p.date, .date, "long")).muted
        Markdown(p.body)
    }
}
```

The `title:` and `description:` of a `:param` page may name its parameters
— `title: "{slug} — Blog"` — and each pre-rendered file, and the live page's
tab, has that route's value. The title may also splice an expression over
the parameters and the program's constants and data —
`title: "{posts.find(p => p.slug == slug)?.title ?? slug} — Blog"` — which the build
works out for each file and the router again on each visit. Anything else
the head should say about the entry goes in `head { }`, as above; the
runtime keeps those tags current as the parameter changes.

## A checklist for being found

- Set `meta.site_url` — without it there are no canonical links, no sitemap
  and no absolute sharing URLs.
- Give every page a `title:` and a `description:` of about 150 characters
  (`S01`–`S03` warn when they are missing or too long).
- Pre-render: `build.ssg: true`, so the text is in the HTML.
- One `h1` per page, headings in order (`A11`, `A12`).
- An `image:` per page that is shared, or `meta.image` for the site — 1200 ×
  630 pixels suits most previews.
- `noindex: true` on pages that should not be found: sign-in, thanks, drafts.
- Check the result: view a built page's source, and paste a URL into a
  link-preview debugger.

## Code on the page

`Code("…")` is inline code; `Code("…").block` a block. `language:` colours
it — `wf`, `json`, `bash` or `css` — at build time and in the browser
alike, with the theme's `syntax-*` tokens; a `Markdown` fence with one of
those languages is coloured the same way.

```wf
page Snippets(path: "/") {
    Text("Run ")  Code("wf serve")  Text(" to start.")
    Code("page Home(path: \"/\") {\n    Text(\"Hi\")\n}", language: "wf").block
    Code("$ wf build", language: "bash").block
}
```

## Next

[Media](28-media.md).
