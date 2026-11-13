# Set cache and security headers

> Which Cache-Control each file of a WebFluent build should get, and the security headers the build writes to _headers for a host to send.

The page: https://webfluent.monzeromer.dev/recipes/cache-and-security-headers

This recipe sets the HTTP headers a host sends with a WebFluent build: how
long a browser may keep each file, and the security headers that only work
as real headers.

You need a host that lets you set headers, or one that reads `_headers`
(Netlify, Cloudflare Pages). The hosting recipes, such as
[Serve a site with nginx](https://webfluent.monzeromer.dev/recipes/serve-with-nginx.md), show where each host takes
them.

With `build.csp` on, this is the `build/_headers` file a build writes:

```text
/*
  Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  X-Frame-Options: DENY
```

## How it works

1. `/*` applies the headers under it to every path of the site.
2. `Content-Security-Policy` is the same policy every page already carries
   as a `<meta>` tag, plus `frame-ancestors`, which a browser ignores in a
   `<meta>` tag ([Security](https://webfluent.monzeromer.dev/about/security.md#ship-a-content-security-policy)).
3. `X-Content-Type-Options`, `Referrer-Policy` and `X-Frame-Options` only
   work as real HTTP headers, so they are in this file and nowhere else.

The build writes no cache headers: those are yours to set, by the kind of
file.

## Set cache headers

To let browsers keep files without serving a stale site, set
`Cache-Control` by the kind of file:

| Files | Cache-Control | Why |
|---|---|---|
| HTML, `app.js`, `pages/*`, `styles.css` | `no-cache` (revalidate every time) | their names do not change between builds |
| `sw.js` | `no-cache` | a stale worker holds a stale site ([Make a site work offline](https://webfluent.monzeromer.dev/recipes/work-offline.md)) |
| resized images (`hero.<hash>.960.webp`) | `public, max-age=31536000, immutable` | the name changes when the file does |
| `public/` files | your choice | they keep the names you gave them |

`no-cache` still lets the browser keep a copy. The browser asks the server
each time, and a `304 Not Modified` answer means it does not download the
file again.

## Set security headers

Netlify and Cloudflare Pages read `build/_headers` as it is. On any other
host, set the same headers in its configuration, as the hosting recipes
show: [nginx](https://webfluent.monzeromer.dev/recipes/serve-with-nginx.md), [Apache](https://webfluent.monzeromer.dev/recipes/serve-with-apache.md),
[Caddy](https://webfluent.monzeromer.dev/recipes/serve-with-caddy.md) and [Vercel](https://webfluent.monzeromer.dev/recipes/deploy-to-vercel.md). Serve the site over HTTPS, and turn
on HSTS once you are sure HTTPS works everywhere.
[Security](https://webfluent.monzeromer.dev/about/security.md#before-you-deploy) has the full checklist.

After the deploy, [check the headers it sends](https://webfluent.monzeromer.dev/recipes/check-a-deploy.md).
