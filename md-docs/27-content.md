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

Blocks: `#`–`######` headings, paragraphs (their lines run together, as
in CommonMark — end a line with two spaces or `\` to break it), fenced code with a language
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
    "owner": "organization",
    "same_as": ["https://github.com/acme"],
    "image": "/og-default.png",
    "image_alt": "The Acme logo on a blue field",
    "favicon": "/favicon.svg",
    "touch_icon": "/apple-touch-icon.png",
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
  (`website` or `article`), `og:site_name`, and `og:locale` in the form
  Open Graph reads — `meta.lang` `en` is `en_US`, `en-GB` is `en_GB`, `ar`
  is `ar_AR` — with an `og:locale:alternate` for each other locale.
- The image's size and description: `og:image:width` and
  `og:image:height` read from the file when the image is in `public/`
  (one on another origin is not fetched, so its card states no size), and
  `og:image:alt` and `twitter:image:alt` from the page's `image_alt:`, or
  `meta.image_alt` for the site's image.
- JSON-LD: the site's owner, a `WebSite` it publishes, a `WebPage` or
  `Article`, and a `BreadcrumbList` derived from the route's segments —
  each a page a reader can open; a level no route answers is left out.
- `hreflang` alternates for each locale when the project has several.
- The icons: `<link rel="icon">` for `favicon` (typed, so an SVG is read
  as one) and `<link rel="apple-touch-icon">` for `touch_icon`, each
  linked from the site's root under the `base_path`
  (`/my-site/favicon.svg`), so it stays right when the router moves the
  page to another address. A sharing image is best 1200×630.
- `<meta name="robots" content="noindex">` and no sitemap entry for a
  `noindex: true` page.

### Who the site belongs to

The owner is `meta.site_name`, published as an `Organization` unless
`meta.owner` says it is a person. A personal site names the person, what
they do and where else they are:

```json
{
  "meta": {
    "site_url": "https://ada.example",
    "site_name": "Ada Lovelace",
    "owner": "person",
    "job_title": "Analyst",
    "same_as": ["https://github.com/ada", "https://www.linkedin.com/in/ada/"]
  }
}
```

The JSON-LD then holds a `Person` (`@id` `…/#person`) with `jobTitle` and
`sameAs`; the `WebSite`'s `publisher` and every `WebPage`'s `about` point
at it. An organisation's `WebSite` names it as `publisher` too, and takes
`same_as` for its profiles.

### Addresses that do not redirect

A static build writes `/contact` as `contact/index.html`. Most static hosts
— GitHub Pages among them — answer `/contact` with a `301` to `/contact/`,
so a canonical link, a sitemap entry and every link that names `/contact`
name an address that redirects, which a search engine reads as "not this
page". `build.clean_urls` picks one of the two ways out:

```json
{ "build": { "ssg": true, "clean_urls": "file" } }
```

- `"file"` keeps `/contact` and writes `contact.html` beside
  `contact/index.html`, which the host serves for `/contact` directly. The
  page then addresses its scripts and sheets from the site's root
  (`/app.js`, under the `base_path`), so the one file reads the same from
  either address.
- `"directory"` names every route with its slash — the canonical link,
  `og:url`, the sitemap, the breadcrumbs, the `hreflang` alternates, and
  every `Link(to:)`, `Sidebar.Item(to:)` and `navigate()` whose target is
  one of the site's pages (`/contact` becomes `/contact/`; a file such as
  `/cv.pdf`, another origin, or an address worked out whole at run time is
  left as written).

Unset, the addresses are `/contact` and the files `contact/index.html`, as
before — right for a host that serves the directory without a redirect
(Netlify, Cloudflare Pages, Vercel with `cleanUrls`).

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
  630 pixels suits most previews — in `public/`, so its size is in the card,
  with an `image_alt:` (or `meta.image_alt`) saying what it shows.
- On GitHub Pages and hosts like it, `build.clean_urls`, so the canonical
  address is never one that redirects.
- A personal site: `meta.owner: "person"`, with `job_title` and `same_as`.
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
