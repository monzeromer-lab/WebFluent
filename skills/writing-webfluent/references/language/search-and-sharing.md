## Search and Sharing

Set `meta.site_url` and the compiler writes everything a search engine and a link
preview need. Without it, the tags that require an absolute URL are omitted
rather than guessed at — a relative canonical causes problems later, so none is
better than a wrong one.

```json
{
  "meta": {
    "site_url": "https://example.com",
    "site_name": "Ledger",
    "description": "Fallback for pages that set none",
    "image": "/card.png",
    "image_alt": "The ledger, open on a desk",
    "owner": "organization",
    "same_as": ["https://github.com/ledger"],
    "sitemap": true
  }
}
```

Every page then carries:

- a self-referencing absolute `<link rel="canonical">`
- Open Graph and Twitter card tags, built from the page's title, description and image;
  `og:image:width`/`og:image:height` read from the file when the image is in `public/`,
  `og:image:alt`/`twitter:image:alt` from the page's `image_alt:` or `meta.image_alt`,
  and `og:locale` as Open Graph spells it (`en` → `en_US`, `en-GB` → `en_GB`) with an
  `og:locale:alternate` per other locale — in the page's own `lang:` when it has one
- `<link rel="alternate" hreflang="…">` for the page and each of its translations
  (`translates:`), each at its own address, plus `x-default` for the one in the
  site's language. A locale the browser switches to on one address
  (`setLocale`, `?lang=`) is not an alternate: that address's HTML is the
  default locale's
- JSON-LD (`WebSite`, the owner, `WebPage`/`Article`, and a `BreadcrumbList`
  derived from the route). The owner is `site_name` as an `Organization`, or a
  `Person` with `"owner": "person"` (and `job_title`); `same_as` lists its other
  profiles; `owner_details` adds any other schema.org property as written
  (`alternateName`, `worksFor`, `alumniOf`, `knowsAbout`, … — not the node's
  `@type`, `@id`, `name`, `url`, `jobTitle` or `sameAs`, which are `E111`);
  `WebSite.publisher` names it, and on a personal site every page is `about` it

`meta.preload` lists files on the site to fetch with the page — a font a
project `.css` file's `@font-face` names, the first screen's picture — as a
`<link rel="preload">` in every head, ahead of the stylesheets, `as` taken
from the extension (a font is also `crossorigin`). One on another origin is
`E111`.

A static build writes `/contact` as `contact/index.html`, and a host such as
GitHub Pages answers `/contact` with a `301` to `/contact/`. `build.clean_urls`
keeps the canonical address one the host serves directly: `"file"` also writes
`contact.html` (and addresses the page's assets from the root under
`base_path`); `"directory"` names every route `/contact/` — canonical, `og:url`,
sitemap, breadcrumbs, and every `Link(to:)`/`navigate()` to one of the pages.

A build also writes `sitemap.xml` and `robots.txt`. Dynamic routes, catch-alls and
`noindex` pages are left out of the sitemap. `priority` and `changefreq` are not
emitted: Google ignores both.

