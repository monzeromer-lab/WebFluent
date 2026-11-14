---
chapter: 23
kind: "about"
route: "/about/static-or-single-page"
from: ["/docs/guide/static-and-spa"]
group: "about"
blurb: "The same source builds as pre-rendered pages or as one app shell. What each gives you, what the pre-rendered page holds, and what each asks of a host."
description: "Static pages or a single-page app: build.ssg, hydration, what the pre-rendered HTML holds, dynamic routes, and what each shape asks of a host."
---
# Static or single-page

This essay compares the two shapes a WebFluent build takes: a static
site, with a pre-rendered HTML file for every route, and a single-page
app, with one shell that draws every page in the browser.

The usual framing makes this a choice of framework: one for content
sites, another for apps, and a migration when a site turns out to be
both. WebFluent treats it as a setting. The same source builds either
shape, one key in the config, `build.ssg`, picks it, and you can change
your mind on any day without touching a page. What is worth deciding is
not which tool to use but what the reader should get first: the text, or
the app.

The sections below say what each shape writes, how a static page comes to
life, what the build can know when it pre-renders, and what each shape
asks of a host.

## Two shapes of build

`build.ssg` decides what `wf build` writes:

| What differs | Static (`"ssg": true`) | Single-page (`"ssg": false`) |
|---|---|---|
| What is written | One HTML file per route — `index.html`, `about/index.html`, `posts/hello/index.html` — each the page's first paint | One `index.html` shell; the router draws every page in the browser |
| First paint | Immediate: the text is in the HTML | After the script loads and runs |
| Search engines and link previews | See the real page | See the shell (with its title and tags) until they run the script |
| Moving between pages | A real page load | Instant, with no reload; focus and scroll handled, the title announced |
| The host | Any static host, as it is | Must answer every path with `index.html` (the "SPA fallback") |
| Interactive | Yes, once the script *hydrates* the page in place | Yes |

The source is the same either way. Only the output changes.
`"output_type": "static"` means the same as `"ssg": true`, and a build with
`ssg` set to `false`, or left out, is single-page. `wf init -t static`
starts a project with `ssg` on and `wf init -t spa` one with it off; either
can change its mind at any time without the source changing.

## When to choose each

**Choose static** for anything people find by searching or share by link:
marketing sites, docs, blogs, shops. The text is in the HTML, so search
engines and link previews see the real page.

**Choose single-page** for an app behind a sign-in, where every page is
personal and moving between pages fast matters more than the first paint.
Choose it too when your host cannot serve a file per route.

## Hydration

A static page is readable before any script runs. When `app.js` arrives,
it *hydrates* the page: it draws the page beside the one already there,
then takes the painted one over in place.

What happens to each part of the page:

- An element nothing holds on to stays exactly as it was painted. Its
  attributes are brought up to date; it is never moved or drawn again.
- An element that a handler, an effect, a `ref` or a widget holds is put
  where the painted one stood, inside the painted parent.
- Text that a signal keeps current changes inside the painted element.

The reader never sees the page flash or redraw, and keeps their scroll
position, their focus and anything they had selected. The page's largest
paint stays the first one, which is what a search engine's speed score
measures.

Under `wf serve`, the runtime compares the page it took over with the one
it drew, and says so in the console when they differ. A difference is a
WebFluent bug, never yours.

## What the static paint knows

The *static paint* is the HTML the build pre-renders for a page. To paint
it, the build evaluates the page as far as it can know the values:

| Source | In the pre-rendered HTML |
|---|---|
| Literals, `const`, `data`, `image`, `env` | Painted |
| `state` initial values, and `derived` values over them | Painted |
| Stores, from their initial values | Painted |
| Components | Expanded, with props substituted and slots filled |
| `t("…")` | In the default locale |
| `format`, `ago`, the list and string helpers | Computed, with a built-in table of locales |
| `resource` and `api` calls | The `loading` arm; the live page fetches and replaces it |
| `await fetch` in an action | Nothing, until the action runs |
| `viewport`, `query`, `hash`, `theme`, `network` | Unknown — a condition on them is drawn once the page is live |
| Anything read from `window`, `document`, `localStorage` | Unknown |

For the first paint to be right, prefer what the build can see:

- **Data files over fetches** for content that changes only when you
  deploy ([Fetching data](../learn/15-data.md)).
- **[Responsive values](../learn/13-styling.md#responsive-values) over `viewport`**
  for layout.

## Routes with no single file

A page on a `:param` route, such as `/posts/:slug`, is one page in the
source and any number of addresses on the site, so a static build has no
single file to write for it. It can only pre-render the values it is told
about, at build time: `paths:` on the page lists them — from a `data` file,
a `const` or a literal list — and the build writes one painted file per
value, each with its own title, and lists them in the sitemap
([Pre-render a page for each item](../recipes/pre-render-each-item.md)). A
value nobody listed has no file, so the host answers it with the site's
404 page. A single-page build has none of this to decide: its router draws
any value in the browser.

The same reasoning gives the 404 page its file. A page on the `*` route is
drawn by the router for any address no page claims, and a static build
also writes it as `404.html`, which most static hosts serve for a path they
have no file for
([Show your own page for an unknown address](../recipes/not-found-page.md)).

## What each shape asks of a host

A static build asks almost nothing: any host that serves files serves it.
A single-page build has one file, `index.html`, so every path a reader
might open — `/users/42` — has to reach it, and the host needs a rule that
answers every path with `index.html`. Without it the home page works, but
a reload on any other address is a 404. Each host's rule is one line, and
each deploy and server recipe in the [Ship recipes](../recipes/index.md#ship)
has it.

Two settings adapt a build to where it is served, whichever its shape.
`build.base_path` serves the site under a sub-path, such as a GitHub Pages
project site, and prefixes every link, asset and route with it, so the
source still writes paths from the site's root
([Deploy to GitHub Pages](../recipes/deploy-to-github-pages.md)).
`build.clean_urls` keeps every canonical address one the host serves
without redirecting `/contact` to `/contact/`
([Keep addresses from redirecting](../recipes/search-and-link-previews.md#keep-addresses-from-redirecting)).

## Next

- [Markdown pages](../recipes/markdown-pages.md) and
  [search and link previews](../recipes/search-and-link-previews.md): what
  the build writes for search engines.
- [Deploy to GitHub Pages](../recipes/deploy-to-github-pages.md), or any host in the [recipes](../recipes/index.md): put either shape on a host.
