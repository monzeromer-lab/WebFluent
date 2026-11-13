# Serve a site with Caddy

> Serve a WebFluent build with Caddy: a Caddyfile site block that sends the build's gzip files and falls back to index.html for a single-page build.

The page: https://webfluent.monzeromer.dev/recipes/serve-with-caddy

This recipe serves a WebFluent build with Caddy, which also gets and
renews the site's TLS certificate.

You need a server running Caddy, and a `build/` folder from `wf build`
([Install wf on a build machine](https://webfluent.monzeromer.dev/recipes/install-wf-in-ci.md)).

Add a site block to the `Caddyfile`:

```text
example.com {
    root * /var/www/site
    encode gzip
    file_server {
        precompressed gzip
    }
    try_files {path} {path}/ {path}.html /index.html
    header X-Content-Type-Options nosniff
}
```

Then upload `build/` to `/var/www/site` and reload Caddy.

## How it works

1. `root` is the folder the build is uploaded to.
2. `precompressed gzip` serves the `.gz` files the build wrote; `encode
   gzip` compresses on the way out what has none.
3. `try_files` decides what a path reaches: the file, the folder's
   `index.html`, the `.html` file `clean_urls: "file"` writes, and last
   `/index.html`, the single-page fallback.
4. `header` sends a security header; add the others from
   `build/_headers` the same way
   ([Set cache and security headers](https://webfluent.monzeromer.dev/recipes/cache-and-security-headers.md#set-security-headers)).

## Variations

For a static build, remove `/index.html` from the end of `try_files`, so
an unknown path is a 404.

## Check it

Open a deep link directly and reload it, then check the headers with
`curl -I`. Then go through [Check a deploy](https://webfluent.monzeromer.dev/recipes/check-a-deploy.md).
