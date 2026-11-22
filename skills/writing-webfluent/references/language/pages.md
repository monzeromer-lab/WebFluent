### Pages

```wf
page Home(path: "/", title: "Home", description: "The front page.") {
    state count = 0

    Container {
        Heading("Welcome").h1
        Text("Count: {count}")
        Button("+1").primary { on click { count = count + 1 } }
    }
}
```

- `path` — URL route. A dynamic segment is a typed parameter of the page:
  `page User(path: "/user/:id", id: String) { Text(id) }`
- `title` — Browser tab title, and the heading of a search result; may name
  the page's parameters, `title: "{slug} — Blog"`, or splice an expression
  over them and the constants: `"{posts.find(p => p.slug == slug)?.title ?? slug} — Blog"`
- `description` — The snippet a search result and a link preview show
- `image` — Link-preview image, site-relative or absolute
- `image_alt` — What that image shows (`og:image:alt`, `twitter:image:alt`)
- `type` — `"website"` (default) or `"article"`
- `noindex: true` — keeps the page out of search results and out of the sitemap
- `lang: "en"` — the language the page is written in, when it is not the
  site's: its `<html lang dir>`, `og:locale` and JSON-LD `inLanguage`, in the
  static paint and on the live page (the router sets them on a route change)
- `translates: Home` — the page this one is a translation of; pages joined
  this way list each other as `hreflang` alternates, each at its own address
  and in its own `lang` (`R05` when it names no page, or two share a language)
- `layout: Shell(crumb: "Home")` — the component that frames the page; the
  page renders in its default slot, and the skip link jumps past the
  layout's own chrome to the page's first element
- `guard` — Expression that must hold for the route to render
- `redirect` — Where to send the visitor when the guard fails

```wf
page Post(path: "/blog/slow-roasting", title: "Slow roasting, explained",
          description: "Why we take eleven minutes over it.",
          type: "article", image: "/roast.png") { Container { Text("…") } }
```

Pages own their routes: `app` places a bare `Router`, and the table is built
from every page's `path`, ordered by specificity (static segments before
`:params`, `*` last), so declaration order cannot shadow a route. A path
ending in `*` (`/docs/*`) answers that path and everything beneath it; a
static build paints it at the path (`docs/index.html`) unless a page of its
own has that path, and `path: "*"` alone is the site's `404.html`.

A page may add tags of its own to the document's head — painted at build
time where the value is known, kept current on the live page, and gone
when the page is:

```wf
page Post(path: "/p/:slug", title: "Post", slug: String) {
    head {
        meta(property: "og:image", content: post.image)
        link(rel: "canonical", href: "https://example.com/p/{slug}")
        script(src: "/analytics.js", defer: true)
    }
    …
}
```

A tag's `src:` and `href:` go through the scheme check at build time and on
the live page; a `script(src:)` is written out, never read from a value, and
a `meta(http-equiv: "refresh")` takes no address from one (`E107`).

<!-- core -->
