# Static or single-page

> Static pages or a single-page app: build.ssg, hydration, what the pre-rendered HTML holds, paths for dynamic routes, 404s and hosting needs.

The page: https://webfluent.monzeromer.dev/about/static-or-single-page

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

| | Static (`"ssg": true`) | Single-page (`"ssg": false`) |
|---|---|---|
| What is written | One HTML file per route — `index.html`, `about/index.html`, `posts/hello/index.html` — each the page's first paint | One `index.html` shell; the router draws every page in the browser |
| First paint | Immediate: the text is in the HTML | After the script loads and runs |
| Search engines and link previews | See the real page | See the shell (with its title and tags) until they run the script |
| Moving between pages | A real page load | Instant, with no reload; focus and scroll handled, the title announced |
| The host | Any static host, as it is | Must answer every path with `index.html` (the "SPA fallback") |
| Interactive | Yes, once the script *hydrates* the page in place | Yes |

The source is the same either way. Only the output changes.

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
  deploy ([Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md)).
- **[Responsive values](https://webfluent.monzeromer.dev/learn/13-styling.md#responsive-values) over `viewport`**
  for layout.

## Switch between static and single-page

To build a static site, set `ssg` in the config:

```json
{ "build": { "ssg": true } }
```

`"output_type": "static"` means the same as `"ssg": true`. Set `ssg` to
`false`, or leave it out, for a single-page build.

`wf init -t static` starts a project with `ssg` on; `wf init -t spa` starts
one with it off. You can change your mind at any time — the source does not
change.

## Pre-render a dynamic route with `paths:`

A page on a `:param` route, such as `/posts/:slug`, has no single file to
write. To pre-render it, list the values in `paths:`, and the build writes
one HTML file for each.

This blog writes one file per post in `posts.json`:

```wf
type Post { slug: String, title: String, date: String, body: String }

data posts: [Post] = "posts.json"

page PostPage(path: "/posts/:slug", slug: String, title: "Post", description: "A post.", paths: posts.map(p => p.slug)) {
    derived post = posts.find(p => p.slug == slug)
    head { meta(property: "og:title", content: post?.title ?? "Post") }
    if let p = post {
        Heading(p.title).h1
        Markdown(p.body)
    } else {
        Alert("No such post").warning
    }
}
```

For a post whose slug is `hello`, the build writes
`posts/hello/index.html`, with that post painted and listed in the sitemap.

The rules for `paths:`:

- It must be known at build time: a `data` file, a `const` or a literal
  list.
- For a route with several parameters, each value is a map that names
  them: `paths: pairs.map(x => { year: x.y, slug: x.s })`.
- Each value is seeded as the parameter (and as `params.slug`), painted,
  and listed in the sitemap.
- A value that is not in `paths:` has no file, so a static host answers
  with `404.html`.
- A single-page build does not need `paths:` at all.

### Give each file its own title

`title:` and `description:` may name the page's parameters —
`title: "{slug} — Blog"` — and each file gets its own. A title may also
look the entry up:
`title: "{posts.find(p => p.slug == slug)?.title ?? slug} — Blog"`. Tags
beyond the title and description go in `head { }`
([Content](https://webfluent.monzeromer.dev/recipes/search-and-link-previews.md#add-tags-for-one-page)).

## Add a 404 page

To show a page for a route no other page claims, declare a page on the
`*` route:

```wf
page NotFound(path: "*", title: "Not found", description: "There is no page at this address.") {
    Heading("Page not found").h1
    Link("Go to the home page", to: "/")
}
```

The router shows it for any address no page claims. In a static build it is
also written as `404.html`, which GitHub Pages, Netlify, Cloudflare Pages
and most static hosts serve for a path they have no file for.

## Serve a single-page build

A single-page build has one file, `index.html`, so every path a reader
might open — `/users/42` — has to reach it. Most hosts need one line for
this; [Deploying](https://webfluent.monzeromer.dev/recipes.md) has each in full:

```text
Netlify / Cloudflare Pages   public/_redirects:  /*  /index.html  200
nginx                        try_files $uri $uri/ /index.html;
Vercel                       "rewrites": [{ "source": "/(.*)", "destination": "/index.html" }]
```

> [!WARNING]
> Without this rule, the home page works, but a reload on any other address
> is a 404.

## Serve the site under a sub-path

To serve the site under a path such as `/my-site` — a GitHub Pages project
site — set `build.base_path`:

```json
{ "build": { "ssg": true, "base_path": "/my-site" } }
```

The build prefixes every link, every asset and the router's paths with it.
Write your paths from the site's root as if there were no prefix:
`Link("About", to: "/about")` links to `/my-site/about`.

## Keep addresses a host serves directly

A static build writes `/contact` as `contact/index.html`, and a host such
as GitHub Pages answers `/contact` with a `301` redirect to `/contact/`.
`build.clean_urls` makes the canonical address one the host serves without
the redirect:

```json
{ "build": { "ssg": true, "clean_urls": "file" } }
```

- `"file"` also writes `contact.html`, and addresses the page's assets
  from the root, under `base_path`.
- `"directory"` names every route with a trailing slash, `/contact/`: the
  canonical link, `og:url`, the sitemap, the breadcrumbs, and every
  `Link(to:)` and `navigate()` to one of the pages.

## Quick reference

| Goal | Setting or syntax |
|---|---|
| Pre-render every route | `"build": { "ssg": true }` or `"output_type": "static"` |
| One app shell | `"build": { "ssg": false }` (the default) |
| Pre-render a `:param` route | `paths:` on the page, known at build time |
| A title per pre-rendered file | `title: "{slug} — Blog"` |
| A 404 page | `page NotFound(path: "*", …)`, written as `404.html` when static |
| A single-page build on a host | Answer every path with `index.html` |
| A site under a sub-path | `"build": { "base_path": "/my-site" }` |
| No redirect to a trailing slash | `"build": { "clean_urls": "file" }` or `"directory"` |

## Next

- [Markdown pages](https://webfluent.monzeromer.dev/recipes/markdown-pages.md) and
  [search and link previews](https://webfluent.monzeromer.dev/recipes/search-and-link-previews.md): what
  the build writes for search engines.
- [Deploy to GitHub Pages](https://webfluent.monzeromer.dev/recipes/deploy-to-github-pages.md), or any host in the [recipes](https://webfluent.monzeromer.dev/recipes.md): put either shape on a host.
