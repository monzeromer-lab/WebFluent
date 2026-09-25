# 16. Content

A site is also its words: pages of prose, a blog, docs. This chapter covers
writing content in Markdown, pre-rendering the site to static HTML, and
what the build does for search engines and link previews.

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
the same one a `.wf` page names ([chapter 3](03-pages-and-routing.md#layouts)):

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
project can use — [chapter 18](18-tooling.md#wf-docs-wf-registry-wf-types).)

## Static builds

With `"build": { "ssg": true }` every page is pre-rendered to HTML at build
time: `index.html`, `about/index.html`, `posts/hello/index.html`, one per
route (and one per value in `paths:` for a `:param` page). Each file is the
page's first paint — text, lists over seeded state, components expanded,
the `loading` arm of every resource — with the app script loading after
it. The page is readable before any script runs, and interactive once the
script hydrates the paint in place, without repainting it.

What the static paint knows: literals, `state` initial values, `derived`
over them, `data`, `const`, `env`, `t` in the default locale, `format` and
`ago` with a built-in locale table. What it does not: fetched data, the
browser's values (`viewport`, `query`, `hash`, `theme`), anything read from
`window`. A condition on an unknown renders its live branch once hydrated.

The output also holds:

| File | |
|---|---|
| `404.html` | The `path: "*"` page, served by a static host for a route it has no file for |
| `sitemap.xml`, `robots.txt` | Every indexable route (`meta.sitemap`, on by default; needs `meta.site_url`) |
| `styles.css`, `pages/*.css` | Shared and per-page stylesheets |
| `app.js`, `pages/*.js` | The runtime the program reaches for, and each page's script, loaded when its route shows |
| `*.gz` | Each text file, precompressed (`build.compress`) |
| `_headers` | A strict Content-Security-Policy for hosts that read one (`build.csp`) |
| `public/*` | Copied as they are to the output root |

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

A page adds its own tags with `head { }` ([chapter 3](03-pages-and-routing.md#per-page-head-tags)).

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

`title:` and `description:` of a `:param` page are one string for every
value, so name the section there ("Blog") and let the `og:title` in
`head { }` carry each post's own; the runtime keeps the head tags current
as the parameter changes.

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

## Images and media

```wf
page Media(path: "/") {
    Image(src: "/hero.jpg", alt: "The team at the launch", width: 1200, height: 630)
    Image(src: "/deco.png", alt: "")
    Video(src: "/demo.mp4", captions: "/demo.en.vtt").controls
    Audio(src: "/talk.mp3", transcript: "/talk.txt").controls
}
```

Give every `Image` an `alt:` — empty for a decorative one — and a `width:`
and `height:` where you know them so the layout does not shift; the first
image on a page loads eagerly and later ones lazily. Files in `public/`
are served from the root.

A `Video` without `captions:` and an `Audio` without `transcript:` each
draw an `A09`: a video nobody can hear is a video nobody can follow.
`captions:` becomes a `<track>`; `transcript:` a link beneath the player.

### `image`: a picture the build processes

Naming a picture with `image` hands the build the file itself:

```wf
image hero = "hero.jpg"

page Home(path: "/", title: "Home", description: "The front page.") {
    Image(hero, alt: "The team at the launch",
          sizes: "(max-width: 768px) 100vw, 1200px",
          placeholder: .blur)
    Text("It is {hero.width} by {hero.height}, and mostly {hero.color}.")
}
```

The build reads its real size and its average colour, writes it again at
every width in `media.widths` smaller than the original, and gives the page
a `<picture>`:

- a `<source>` per format, each with a `srcset` of the widths, so the
  browser picks the one it needs for the space `sizes:` describes;
- `width` and `height` from the file, so the box is the right shape before
  a byte of the image has arrived and **nothing on the page moves**;
- a placeholder in the meantime — `.blur` inlines a sixteen-pixel-wide copy
  as a data URI, `.color` fills the box with the average colour, `.none`
  does neither;
- a content hash in every file's name, so a host can cache them forever.

The name is a value: `hero.src`, `.width`, `.height`, `.color`,
`.placeholder`, `.srcset`.

```json
{ "build": { "media": {
    "formats": ["webp"],
    "widths": [480, 960, 1440, 1920],
    "quality": 78,
    "pipeline": true
} } }
```

`pipeline: false` copies the file as `public/` always did. The work is
cached in `.wf-cache/media/`, so a build that changes no image does no
image work. A PDF or a slide deck embeds the real picture rather than
drawing a box where one should be.

## Next

[Outputs](17-outputs.md).
