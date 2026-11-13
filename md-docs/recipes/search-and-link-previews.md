---
kind: "recipe"
route: "/recipes/search-and-link-previews"
group: "content"
description: "For every page, the build writes the standard head from the page's attributes and the `meta` settings in `webfluent.app.json`."
from: ["/docs/guide/content#describe-each-page-to-search-engines-and-link-previews", "/docs/guide/content#a-checklist-for-being-found"]
---
# Describe pages to search engines and link previews

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
  ([A page in its own language](site-in-two-languages.md)).
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
