### Images the build processes

`image hero = "media/hero.jpg"` reads the picture at build time and writes
it again at every width a page will ask for:

```wf
image hero = "hero.jpg"

page Home(path: "/", title: "Home") {
    Image(hero, alt: "The team at the launch",
          sizes: "(max-width: 768px) 100vw, 1200px",
          placeholder: .blur)
    Text("It is {hero.width} by {hero.height}, and mostly {hero.color}.")
}
```

What the build writes: `hero.<hash>.<width>.webp` at each width in
`media.widths` that is smaller than the original (never up-scaled), plus
the original under its own hashed name — so a host can cache every one of
them forever. What the page gets is a `<picture>` with a `<source>` per
format, `width` and `height` from the real file (so nothing shifts when it
arrives), and a placeholder: `.blur` inlines a 16-pixel-wide copy as a data
URI, `.color` fills the box with the image's average colour, `.none` does
neither.

The name is a value too: `hero.src`, `.width`, `.height`, `.color`,
`.placeholder`, `.srcset`.

```json
{ "build": { "media": {
    "formats": ["webp"], "widths": [480, 960, 1440, 1920],
    "quality": 78, "pipeline": true
} } }
```

`Image(src: "/hero.jpg", alt: "…")` with a literal path of `public/` to a
PNG, JPEG, GIF or WebP is processed the same way, without a declaration:
the same files, the same `<picture>`, `sizes:` and `placeholder:` too. An
SVG, a URL or a path computed at run time is used as written, and so is a
picture carried in its address (`data:image/png;base64,…`).

WebP and JPEG are written lossily at `quality`. A file that would be no
smaller than the original is not written: the original serves there, and
`wf assets --plan` says so.

`media.pipeline: false` copies the file as `public/` always did. The work
is cached in `.wf-cache/media/` by content hash, so a build that changes no
image does no image work. A PDF or a deck embeds the real picture.

`data posts = "posts.json"` (or `data posts: [Post] = "content/posts.json"`)
is a constant whose value is a JSON file's, read at build time from the
project directory or its `src/`; it is inlined into the bundle and seeded
into the static paint. A declared type is held to the file when it is read:
a record that does not fit stops the build with `T33`, naming the file and
the place (`` `posts.json` … $[3].date is "March 5", not a Date ``). A page on a `:param` route names the values a
static build renders it for with `paths:`, one value per parameter (a map
by name for several):

```wf
type Post { slug: String, title: String }
data posts: [Post] = "posts.json"
page Post(path: "/p/:slug", title: "Post", slug: String, paths: posts.map(p => p.slug)) {
    derived post = posts.find(p => p.slug == slug)
    Heading(post?.title ?? "?").h1
}
```

The static build writes `p/<slug>/index.html` for each, with `slug` and
`params.slug` seeded, and lists them in the sitemap.

A `.md` file under `src/` is a page: front matter between `---` lines
names `path` (default `/<stem>`, `/` for `index`), `title`, `description`,
`layout`, `image`, `type`, `noindex`, `lang` and `translates`; the rest is its Markdown, placed
as `Markdown(…)` in the page's body (inside the layout's default slot when
one is named).

```md
---
title: About us
description: Who we are.
layout: Shell
---
# About

We make *things*.
```

