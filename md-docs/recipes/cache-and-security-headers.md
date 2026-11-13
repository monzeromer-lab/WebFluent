---
kind: "recipe"
route: "/recipes/cache-and-security-headers"
group: "ship"
description: "To let browsers keep files without serving a stale site, set `Cache-Control` by the kind of file:"
from: ["/docs/guide/deploying#set-cache-headers", "/docs/guide/deploying#set-security-headers"]
---
# Set cache and security headers

## Set cache headers

To let browsers keep files without serving a stale site, set
`Cache-Control` by the kind of file:

| Files | Cache-Control | Why |
|---|---|---|
| HTML, `app.js`, `pages/*`, `styles.css` | `no-cache` (revalidate every time) | their names do not change between builds |
| `sw.js` | `no-cache` | a stale worker holds a stale site ([Offline](index.md)) |
| resized images (`hero.<hash>.960.webp`) | `public, max-age=31536000, immutable` | the name changes when the file does |
| `public/` files | your choice | they keep the names you gave them |

`no-cache` still lets the browser keep a copy. The browser asks the server
each time, and a `304 Not Modified` answer means it does not download the
file again.

## Set security headers

With `build.csp` on, every page carries its Content-Security-Policy as a
`<meta>` tag. `build/_headers` holds the same policy, plus
`X-Content-Type-Options`, `Referrer-Policy`, `X-Frame-Options` and the
policy's `frame-ancestors`, which only work as real HTTP headers. This is
the file a build writes:

```text
/*
  Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin
  X-Frame-Options: DENY
```

Netlify and Cloudflare Pages read the file. On any other host, set the same
headers as the recipes above show. Serve the site over HTTPS, and turn on
HSTS once you are sure HTTPS works everywhere.
[Security](../about/security.md#before-you-deploy) has the full checklist.
