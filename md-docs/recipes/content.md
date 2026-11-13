---
chapter: 27
kind: "recipe"
route: "guide/content"
group: "content"
blurb: "Markdown, pages written as .md files, code on the page, and what the build writes for search engines and link previews."
description: "How to render Markdown, write .md pages with front matter, set search and sharing tags, canonical links, sitemaps, JSON-LD, and show code."
---
# 27. Content

A site is also its words: pages of prose, a blog, docs. This chapter shows
you how to write that content in Markdown, show code on a page, and control
what the build writes for search engines and link previews.

In this chapter:

- render Markdown with the `Markdown` element, including tables, escapes
  and text somebody else wrote;
- write whole pages as `.md` files;
- describe each page to search engines and link previews;
- show code with syntax colouring.

Pre-rendering is covered in [Static or single-page](../about/static-and-spa.md);
pictures and video in [Media](media.md).

## Render Markdown with the `Markdown` element

To show Markdown text on a page, pass it to `Markdown(text)`. It renders a
small, predictable Markdown, the same at build time and in the browser, so
a hydrated page repaints exactly what was painted.

This page shows release notes held in a state, and one line of inline
Markdown:

```wf
page Notes(path: "/", title: "Release notes", description: "What changed in this release.") {
    state body = "# Release notes\n\nVersion **3.3** adds:\n\n- Markdown pages\n- `wf docs`\n\n> Ship it."
    Markdown(body)
    Markdown("Inline *emphasis*, a [link](/about) and an ![image](/img.png).")
}
```

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
every other URL does ([Security](../about/security.md#what-the-compiler-guarantees)):

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
| `\[not a link\](/x)` | the brackets and parentheses, with no link |
| `\#` at the start of a line | `#`, with no heading |
| `1\.` | `1.`, with no list |
| `\\` | one backslash |

Every ASCII punctuation mark can be escaped this way (`\-`, `\{`, `\(`,
`\_`, `\&` and the rest), and an escaped mark is only the character it is:
it makes no link, no emphasis and no entity.

How escapes behave in links:

- In a link's words, an escaped bracket is a bracket like any other
  (`[a\]b](/c)`). The words hold no unescaped `[`, so in `[a [b](/c)` only
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

## Write a page as a `.md` file

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

## Describe each page to search engines and link previews

For every page, the build writes the standard head from the page's
attributes and the `meta` settings in `webfluent.app.json`. A config that
fills in every site-wide setting:

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

What the build writes into each page's head:

- **`<title>` and `<meta name="description">`**, from the page's `title:`
  and `description:`. The description is the snippet a search result shows
  and a link preview quotes; keep it under about 150 characters.
- **A canonical link and `og:url`**, when `site_url` is set. A relative
  canonical is worse than none, so without `site_url` the build writes
  none.
- **Open Graph and Twitter card tags**: `og:title`, `og:description`,
  `og:image` (the page's `image:` or the site's), `og:type` from `type:`
  (`website` or `article`), `og:site_name`, and `og:locale` in the form
  Open Graph reads — `meta.lang` `en` is `en_US`, `en-GB` is `en_GB`, `ar`
  is `ar_AR` — with an `og:locale:alternate` for each other locale.
- **The image's size and description**: `og:image:width` and
  `og:image:height`, read from the file when the image is in `public/` (one
  on another origin is not fetched, so its card states no size), and
  `og:image:alt` and `twitter:image:alt` from the page's `image_alt:`, or
  `meta.image_alt` for the site's image. A sharing image is best at
  1200×630.
- **JSON-LD**: the site's owner, a `WebSite` it publishes, a `WebPage` or
  `Article`, and a `BreadcrumbList` derived from the route's segments. Each
  breadcrumb is a page a reader can open; a level no route answers is left
  out.
- **`hreflang` alternates** for the page and each of its translations
  (`translates:`), each at its own address and in its own `lang:`, with
  `x-default` for the one in the site's language
  ([A page in its own language](i18n.md#a-page-in-its-own-language)).
- **The icons**: `<link rel="icon">` for `favicon` (typed, so an SVG is
  read as one) and `<link rel="apple-touch-icon">` for `touch_icon`. Each
  is linked from the site's root under the `base_path`
  (`/my-site/favicon.svg`), so it stays right when the router moves the
  page to another address.
- **`<meta name="robots" content="noindex">`**, and no sitemap entry, for a
  `noindex: true` page.

### Add tags for one page

To add tags of a page's own — an article's cover, its publication date —
write them in `head { }` ([Pages and routing](../learn/05-pages-and-routing.md#per-page-head-tags)).
This blog post page sets its own Open Graph tags from the post:

```wf
type Post { slug: String, title: String, summary: String, cover: String, date: String, body: String }

data posts: [Post] = "posts.json"

page PostPage(path: "/posts/:slug", slug: String, title: "Post", description: "A post from the blog.", type: "article", paths: posts.map(p => p.slug)) {
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

The `title:` and `description:` of a `:param` page may name its
parameters — `title: "{slug} — Blog"` — and each pre-rendered file, and the
live page's tab, has that route's value. The title may also splice an
expression over the parameters and the program's constants and data:
`title: "{posts.find(p => p.slug == slug)?.title ?? slug} — Blog"`. The
build works it out for each file, and the router again on each visit.
Anything else the head should say about the entry goes in `head { }`, as
above; the runtime keeps those tags current as the parameter changes.

### Say who the site belongs to

The JSON-LD names the site's owner: `meta.site_name`, published as an
`Organization` unless `meta.owner` says it is a person. To publish a
personal site, name the person, what they do and where else they are:

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
`sameAs`. The `WebSite`'s `publisher` and every `WebPage`'s `about` point
at it. An organisation's `WebSite` names it as `publisher` too, and takes
`same_as` for its profiles.

To say anything else schema.org allows about a person or an organisation,
add it to `meta.owner_details`. It is merged into the node as written:

```json
{
  "meta": {
    "owner_details": {
      "alternateName": "Augusta Ada King",
      "worksFor": [{ "@type": "Organization", "name": "Analytical Engines" }],
      "alumniOf": { "@type": "CollegeOrUniversity", "name": "University of London" },
      "knowsAbout": ["Mathematics", "Computing"],
      "knowsLanguage": ["en", "fr"]
    }
  }
}
```

What the node is — `@type`, `@id`, `name`, `url` — and what `job_title` and
`same_as` write come from those settings; setting one in `owner_details`
is `E111`.

> [!WARNING]
> Structured data has to agree with what the pages show, and the build
> cannot check that. State in `owner_details` only what the site says too.

### Preload fonts served from the site

A font named by a `@font-face` in a project `.css` file is found only once
`styles.css` has arrived and been read. To fetch it with the page instead,
list it in `meta.preload`:

```json
{ "meta": { "preload": ["/fonts/inter-latin.woff2"] } }
```

```css
/* src/fonts.css */
@font-face {
  font-family: "Inter";
  font-weight: 400 800;
  font-display: swap;
  src: url("/fonts/inter-latin.woff2") format("woff2");
}
```

Each path becomes a `<link rel="preload">` in every page's head, ahead of
the stylesheets, with `as` taken from its extension: `font`, `style`,
`script` or `image`. A font preload also carries `crossorigin`, without
which the browser fetches the font twice.

Serving the fonts from `public/` instead of a font service also takes a
third-party stylesheet and two connections out of the way of the first
paint. A path on another origin is `E111`: the policy the build ships
would refuse it.

### Keep addresses from redirecting

A static build writes `/contact` as `contact/index.html`. Most static
hosts — GitHub Pages among them — answer `/contact` with a `301` redirect
to `/contact/`. A canonical link, a sitemap entry and every link that names
`/contact` then name an address that redirects, which a search engine reads
as "not this page". To fix it, set `build.clean_urls` to one of two values:

```json
{ "build": { "ssg": true, "clean_urls": "file" } }
```

- **`"file"`** keeps `/contact` and writes `contact.html` beside
  `contact/index.html`, which the host serves for `/contact` directly. The
  page then addresses its scripts and sheets from the site's root
  (`/app.js`, under the `base_path`), so the one file reads the same from
  either address.
- **`"directory"`** names every route with its slash: the canonical link,
  `og:url`, the sitemap, the breadcrumbs, the `hreflang` alternates, and
  every `Link(to:)`, `Sidebar.Item(to:)` and `navigate()` whose target is
  one of the site's pages (`/contact` becomes `/contact/`). A file such as
  `/cv.pdf`, another origin, or an address worked out whole at run time is
  left as written.

Unset, the addresses are `/contact` and the files `contact/index.html`.
That is right for a host that serves the directory without a redirect:
Netlify, Cloudflare Pages, and Vercel with `cleanUrls`.

## A checklist for being found

- Set `meta.site_url`. Without it there are no canonical links, no sitemap
  and no absolute sharing URLs.
- Give every page a `title:` and a `description:` of about 150 characters.
  `S01`–`S03` warn when one is missing or too long.
- Pre-render with `build.ssg: true`, so the text is in the HTML.
- Use one `h1` per page, with headings in order (`A11`, `A12`).
- Give every shared page an `image:`, or set `meta.image` for the site — at
  1200 × 630 pixels, which suits most previews — in `public/`, so its size
  is in the card, with an `image_alt:` (or `meta.image_alt`) that says what
  it shows.
- On GitHub Pages and hosts like it, set `build.clean_urls`, so the
  canonical address never redirects.
- For a personal site, set `meta.owner: "person"`, with `job_title`,
  `same_as` and, for the rest of who they are, `owner_details`.
- Serve web fonts from `public/`, named in `meta.preload`, instead of a
  font service's stylesheet.
- Set `noindex: true` on pages that should not be found: sign-in, thanks,
  drafts.
- Check the result: view a built page's source, and paste a URL into a
  link-preview debugger.

## Show code on the page

To show code, use `Code("…")` for inline code and `Code("…").block` for a
block. To colour it, add `language:` — `wf`, `json`, `bash` or `css`. The
colouring is done at build time and in the browser alike, with the theme's
`syntax-*` tokens, and a `Markdown` fence in one of those languages is
coloured the same way.

This page mixes inline code into a sentence and shows two coloured blocks:

```wf
page Snippets(path: "/", title: "Snippets", description: "Code on the page.") {
    Heading("Snippets").h1
    Text("Run ")  Code("wf serve")  Text(" to start.")
    Code("page Home(path: \"/\") {\n    Text(\"Hi\")\n}", language: "wf").block
    Code("$ wf build", language: "bash").block
}
```

## Next

[Internationalisation](i18n.md): keep one JSON file per locale, read it with t(), and switch the whole site's language — and its direction — at once.
