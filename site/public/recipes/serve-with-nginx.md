# Serve a site with nginx

> Serve a WebFluent build with nginx: a server block with the build's gzip files, a 404 page, a single-page fallback, cache and security headers.

The page: https://webfluent.monzeromer.dev/recipes/serve-with-nginx

This recipe serves a WebFluent build from a server of your own with nginx:
the precompressed files the build wrote, its 404 page, and the cache and
security headers.

You need a server running nginx with a TLS certificate for the site, and a
`build/` folder from `wf build`
([Install wf on a build machine](https://webfluent.monzeromer.dev/recipes/install-wf-in-ci.md)).

Add this `server` block, and pick the `try_files` line for your kind of
build:

```nginx
server {
    listen 443 ssl http2;
    server_name example.com;
    root /var/www/site;

    gzip_static on;                          # serve the .gz files the build wrote

    location / {
        try_files $uri $uri/ $uri.html =404; # a static build
        # try_files $uri $uri/ /index.html;  # a single-page build
    }
    error_page 404 /404.html;

    location = /sw.js { add_header Cache-Control "no-cache"; }
    location ~* \.(webp|avif|jpg|jpeg|png)$ { add_header Cache-Control "public, max-age=31536000, immutable"; }

    add_header X-Content-Type-Options nosniff always;
    add_header Referrer-Policy strict-origin-when-cross-origin always;
    add_header X-Frame-Options DENY always;
    # add_header Content-Security-Policy "…as in build/_headers…" always;
}
```

Upload the build with any tool — for example, `rsync` — and reload nginx:

```bash
rsync -a --delete build/ server:/var/www/site/
```

## How it works

1. `root` is the folder the build is uploaded to.
2. `gzip_static on` sends the `.gz` file the build wrote beside a text
   file when the browser accepts gzip, so nginx compresses nothing itself.
3. `try_files` decides what a path reaches. In a static build, `/contact`
   is `contact/index.html` (`$uri/`) or, with `clean_urls: "file"`,
   `contact.html` (`$uri.html`); anything else is a 404. In a single-page
   build, every path falls back to `index.html`, where the router draws the
   page.
4. `error_page 404 /404.html` serves the page whose `path` is `*`.
5. `sw.js` is never cached without asking, and resized images, whose names
   change when the file does, are cached for a year
   ([Set cache headers](https://webfluent.monzeromer.dev/recipes/cache-and-security-headers.md#set-cache-headers)).
6. The `add_header` lines are the security headers the build writes to
   `build/_headers`. Copy the `Content-Security-Policy` line from that file
   into the commented line to send the policy as a header too.

`rsync -a --delete` makes the server's folder match `build/`, removing
files an older build left behind.

## Check it

`curl -I` a page and check the headers. Ask for a compressed copy and check
that nginx sends the build's `.gz` file:

```bash
curl -sI -H "Accept-Encoding: gzip" https://example.com/app.js
```

The response has `Content-Encoding: gzip`. Then go through
[Check a deploy](https://webfluent.monzeromer.dev/recipes/check-a-deploy.md).
