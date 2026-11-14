# Show your own page for an unknown address

> Show your own page for an address no page answers: a page whose path is *, kept out of search, and written to 404.html by a static build.

The page: https://webfluent.monzeromer.dev/recipes/not-found-page

A mistyped link or a page that moved should land on a page that says so
and offers a way on, not on a blank screen or the host's own error. A page
whose `path` is `*` answers every address no other page does:

**Needs:** nothing besides the page. A static build (`"ssg": true`) also
writes it to `build/404.html`, which the host must serve for an unknown
path: the host recipes below show how.

```wf
page NotFound(path: "*", title: "Page not found", description: "There is no page at this address.", noindex: true) {
    Heading("We couldn't find that page").h1
    Text("The address may be mistyped, or the page may have moved.")
    Link("Go to the front page", to: "/")
}

page Front(path: "/", title: "Front", description: "The front page.") {
    Heading("Front").h1
    Link("A link to nowhere", to: "/no-such-page")
}
```

`Front` is there so the example has somewhere to go back to; in your
project, keep your own `/` page and add only `NotFound` (in
`src/pages/NotFound.wf`). Open `/no-such-page` and the router draws
`NotFound`.

## How it works

- `path: "*"` matches any address. The router tries the other pages first
  — static paths, then paths with a `:param` — and `*` last, so it never
  hides a real page, wherever it is declared.
- `noindex: true` keeps the page out of search results and out of the
  sitemap.
- In a **single-page build** (the default), the router draws `NotFound`
  for any unknown address once the app has loaded. The host has to send
  `index.html` for an address it has no file for, so the app loads at all:
  that is the single-page fallback in
  [Serve a site with nginx](https://webfluent.monzeromer.dev/recipes/serve-with-nginx.md),
  [Apache](https://webfluent.monzeromer.dev/recipes/serve-with-apache.md) and [Caddy](https://webfluent.monzeromer.dev/recipes/serve-with-caddy.md).
- In a **static build** (`"build": { "ssg": true }`), the build also writes
  the page as `build/404.html`, with its title and text already in the
  HTML. GitHub Pages, Netlify and Cloudflare Pages serve `404.html` for a
  path they have no file for, with the status 404; nginx and Apache do once
  their config names it (`error_page 404 /404.html;`,
  `ErrorDocument 404 /404.html`).
- `wf serve` draws the page for an unknown address too, so you can see it
  while you work.

Check the site's own links before you add the `*` page. `wf check`
reports a `Link(to:)` or `navigate()` whose path no page's route matches as
`R01`, but once a page answers `*`, every path matches it, and that check
has nothing left to report.

## Variations

- **Say which address was asked for**: a static `404.html` is served at
  every unknown address, so read the address in the browser, from a
  project script (`function currentPath() { return location.pathname; }`
  in `src/path.js`), and show it with `Text("No page at {currentPath()}.")`.
  The text is filled in when the page's script runs.
