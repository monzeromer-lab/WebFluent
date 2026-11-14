---
kind: "recipe"
route: "/recipes/serve-with-nginx"
group: "ship"
description: "Serve a WebFluent build with nginx: a server block with the build's gzip files, a 404 page, a single-page fallback, cache and security headers."
outputs: ["website", "app"]
from: ["/docs/guide/deploying#serve-with-nginx"]
---
# Serve a site with nginx

This recipe serves a WebFluent build from a server of your own with nginx:
the precompressed files the build wrote, its 404 page, and the cache and
security headers.

You need a server running nginx with a TLS certificate for the site, and a
`build/` folder from `wf build`
([Install wf on a build machine](install-wf-in-ci.md)).

Put the security headers in a file of their own, so every block that
needs them can include them:

```nginx
# /etc/nginx/snippets/site-headers.conf
add_header X-Content-Type-Options nosniff always;
add_header Referrer-Policy strict-origin-when-cross-origin always;
add_header X-Frame-Options DENY always;
# add_header Content-Security-Policy "…as in build/_headers…" always;
```

Then add this `server` block, and pick the `try_files` line for your kind
of build:

```nginx
server {
    listen 443 ssl http2;
    server_name example.com;
    ssl_certificate     /etc/letsencrypt/live/example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;
    root /var/www/site;

    gzip_static on;                          # serve the .gz files the build wrote
    include snippets/site-headers.conf;

    location / {
        try_files $uri $uri/ $uri.html =404; # a static build
        # try_files $uri $uri/ /index.html;  # a single-page build
    }
    error_page 404 /404.html;

    location = /sw.js {
        include snippets/site-headers.conf;
        add_header Cache-Control "no-cache";
    }

    # Pictures the build resized and renamed: hero.4a6178a1.960.webp, hero.4a6178a1.jpg
    location ~* "\.[0-9a-f]{8}(\.[0-9]+)?\.(webp|avif|jpe?g|png|gif)$" {
        include snippets/site-headers.conf;
        add_header Cache-Control "public, max-age=31536000, immutable";
    }
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
5. `sw.js` is never cached without asking. The pictures the build resized
   and renamed (`img/hero.<hash>.<width>.webp`, and the original as
   `img/hero.<hash>.jpg`) are cached for a year, since a changed picture
   gets a new name. A file of `public/` keeps the name you gave it, so it is
   left to the browser's usual checks: a new `/logo.png` reaches readers at
   once ([Set cache headers](cache-and-security-headers.md#set-cache-headers)).
6. The `add_header` lines are the security headers the build writes to
   `build/_headers`. Copy the `Content-Security-Policy` line from that file
   into the commented line to send the policy as a header too.
7. nginx drops the server's `add_header` lines in any `location` that has
   an `add_header` of its own. That is why the two locations that set
   `Cache-Control` include the headers file again: without it, `sw.js` and
   the pictures would be sent with no security headers.
8. `ssl_certificate` and `ssl_certificate_key` name the certificate the
   `listen 443 ssl` line needs; the paths above are where Certbot puts
   them.

`rsync -a --delete` makes the server's folder match `build/`, removing
files an older build left behind.

## Check it

`curl -I` a page and check the headers. Ask for a compressed copy and check
that nginx sends the build's `.gz` file:

```bash
curl -sI -H "Accept-Encoding: gzip" https://example.com/app.js
```

The response has `Content-Encoding: gzip`. Then go through
[Check a deploy](check-a-deploy.md).
