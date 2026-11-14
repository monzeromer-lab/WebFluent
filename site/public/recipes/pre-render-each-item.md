# Pre-render a page for each item

> Pre-render a page on a :param route once per item with paths:, each file with its own title, painted and listed in the sitemap.

The page: https://webfluent.monzeromer.dev/recipes/pre-render-each-item

A page on a `:param` route, such as `/posts/:slug`, has no single file a
static build can write. List the values in `paths:`, and the build writes
one HTML file for each. This blog writes one file per post in `posts.json`:

**Needs:** `"build": { "ssg": true }` in `webfluent.app.json`, and
`src/posts.json`, a list of posts with a `slug`, a `title`, a `date` and a
`body`.

```wf
type Post { slug: String, title: String, date: String, body: String }

data posts: [Post] = "posts.json"

page PostPage(path: "/posts/:slug", slug: String, title: "{posts.find(p => p.slug == slug)?.title ?? slug} — Blog", description: "A post.", paths: posts.map(p => p.slug)) {
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

## How it works

- `paths:` must be known at build time: a `data` file, a `const` or a
  literal list.
- Each value is seeded as the parameter (and as `params.slug`), painted,
  and listed in the sitemap.
- For a route with several parameters, each value is a map that names
  them: `paths: pairs.map(x => { year: x.y, slug: x.s })`.
- `title:` and `description:` may name the page's parameters —
  `title: "{slug} — Blog"` — and each file gets its own. A title may also
  look the entry up, as this one does. Tags beyond the title and
  description go in `head { }`
  ([Add tags for one page](https://webfluent.monzeromer.dev/recipes/search-and-link-previews.md#add-tags-for-one-page)).
- A value that is not in `paths:` has no file, so a static host answers
  with `404.html` ([Show your own page for an unknown address](https://webfluent.monzeromer.dev/recipes/not-found-page.md)).
- A single-page build does not need `paths:` at all: the router draws the
  page for any value in the browser.

[Static or single-page](https://webfluent.monzeromer.dev/about/static-or-single-page.md#what-the-static-paint-knows)
says what else the build can paint into each file.
