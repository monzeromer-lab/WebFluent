# 26. Static or single-page

<!--
route: guide/static-and-spa
group: shipping
blurb: The same source builds as pre-rendered pages or as one app shell. What each gives you, what the static paint knows, and how dynamic routes are rendered.
description: Static pages or a single-page app: build.ssg, hydration, what the static paint knows, paths for dynamic routes, and hosting needs.
-->

A web build comes in two shapes, chosen by one switch, `build.ssg`:

| | Static (`"ssg": true`) | Single-page (`"ssg": false`) |
|---|---|---|
| What is written | One HTML file per route — `index.html`, `about/index.html`, `posts/hello/index.html` — each the page's first paint | One `index.html` shell; the router draws every page in the browser |
| First paint | Immediate: the text is in the HTML | After the script loads and runs |
| Search engines and link previews | See the real page | See the shell (with its title and tags) until they run the script |
| Moving between pages | A real page load | Instant, no reload; focus and scroll handled, title announced |
| The host | Any static host, as it is | Must answer every path with `index.html` (the "SPA fallback") |
| Interactive | Yes, once the script *hydrates* the paint in place | Yes |

`wf init -t static` starts with `ssg` on; `-t spa` with it off. Change your
mind at any time — the source is the same.

**Choose static** for anything people find by searching or share by link:
marketing sites, docs, blogs, shops. **Choose single-page** for an app
behind a login, where every page is personal and navigation speed matters
more than a first paint — or when your host cannot serve a file per route.

## Hydration

A static page is readable before any script runs. When `app.js` arrives it
**hydrates** the page: it walks the HTML that is already there, attaches
handlers and subscriptions to it, and only replaces what the live values say
differently. The reader never sees the page flash or re-draw.

## What the static paint knows

The build evaluates the page to paint it, as far as it can know the values:

| Source | In the pre-rendered HTML |
|---|---|
| Literals, `const`, `data`, `image`, `env` | Painted |
| `state` initial values, and `derived` over them | Painted |
| Stores, from their initial values | Painted |
| Components | Expanded, props substituted, slots filled |
| `t("…")` | In the default locale |
| `format`, `ago`, the list and string helpers | Computed, with a built-in table of locales |
| `resource` and `api` calls | The `loading` arm; the live page fetches and replaces it |
| `await fetch` in an action | Nothing, until the action runs |
| `viewport`, `query`, `hash`, `theme`, `network` | Unknown — a condition on them is drawn once live |
| anything read from `window`, `document`, `localStorage` | Unknown |

So for the first paint to be right, prefer what the build can see: data
files over fetches for content that changes at deploy time, and
[responsive values](15-styling.md#responsive-values) over `viewport` for
layout.

## Static paths

A page on a `:param` route is written once per value its `paths:` names —
one HTML file per post:

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

- `paths:` must be knowable at build time: a `data` file, a `const`, a
  literal list.
- For a route with several parameters, each value is a map naming them:
  `paths: pairs.map(x => { year: x.y, slug: x.s })`.
- Each value is seeded as the parameter (and `params.slug`), painted, and
  listed in the sitemap.
- `title:` and `description:` may name the page's parameters —
  `title: "{slug} — Blog"` — and each file gets its own. A title may also
  look the entry up —
  `title: "{posts.find(p => p.slug == slug)?.title ?? slug} — Blog"`
  ([Content](27-content.md#search-and-sharing)).
- A value that is not in `paths:` has no file, so a static host answers with
  `404.html`. In a single-page build `paths:` is not needed at all.

## The 404 page

`page NotFound(path: "*")` is what the router shows for a route no page
claims, and in a static build it is also written as `404.html`, which
GitHub Pages, Netlify, Cloudflare Pages and most static hosts serve for a
path they have no file for.

## Serving a single-page build

Every path a reader might open — `/users/42` — has to reach `index.html`.
Most hosts need one line for it; [Deploying](29-deploying.md) has each:

```text
Netlify / Cloudflare Pages   public/_redirects:  /*  /index.html  200
nginx                        try_files $uri $uri/ /index.html;
Vercel                       "rewrites": [{ "source": "/(.*)", "destination": "/index.html" }]
```

Without it, the home page works and a reload anywhere else is a 404.

## Under a sub-path

`build.base_path` — `"/my-site"` for a GitHub Pages project site — prefixes
every link, every asset and the router's paths. Write your paths from the
site's root as if there were no prefix: `Link("About", to: "/about")`.

## Next

[Content](27-content.md).
