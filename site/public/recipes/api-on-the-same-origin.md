# Serve an API on the same origin

> Serve the site at / and its backend at /api from one origin, so pages call it without CORS and a session cookie works with no extra setting.

The page: https://webfluent.monzeromer.dev/recipes/api-on-the-same-origin

To call a backend without CORS, serve the site and the backend from one
origin: the site at `/`, the backend at `/api`. A session cookie then works
with no extra setting.

You need a server that serves the site, as in
[Serve a site with nginx](https://webfluent.monzeromer.dev/recipes/serve-with-nginx.md), and a backend it can reach.
In nginx, add this `location` to the site's `server` block:

```nginx
location /api/ {
    proxy_pass http://127.0.0.1:8080/;
}
```

## How it works

1. nginx sends every request under `/api/` to the backend on port 8080,
   with the `/api` prefix taken off (the trailing `/` of `proxy_pass` does
   that), and serves everything else from the build.
2. To the browser, the pages and the backend share one origin, so a
   request needs no CORS headers and carries the site's cookies.
3. A page then declares the service with `api Backend(base: "/api")`, and
   it needs nothing else.

## Variations

Netlify, Vercel and Cloudflare Pages do the same with a rewrite to an
external URL.

After the deploy, [check it](https://webfluent.monzeromer.dev/recipes/check-a-deploy.md).
