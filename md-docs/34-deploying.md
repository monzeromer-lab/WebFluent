# 34. Deploying

<!--
route: guide/deploying
group: ship
blurb: The build is a folder of static files. Put it on GitHub Pages, Netlify, Vercel, Cloudflare Pages, a server of your own, or a container.
description: Deploy a build to GitHub Pages, Netlify, Vercel, Cloudflare Pages, nginx, Apache, Caddy or Docker, with cache and security headers.
-->

This chapter shows you how to put a WebFluent site on the web. `wf build`
writes one folder, `build/`, and that folder is the whole site: any host
that serves static files can serve it, and there is no server process to
run.

The folder holds HTML, CSS, JavaScript, images, `sitemap.xml`,
`robots.txt`, `_headers` (the security headers, for hosts that read the
file), and a `.gz` file beside every text file. The same folder is also what
a desktop shell shows: [Desktop apps](36-desktop-apps.md) makes a Tauri or
an Electron app of it.

## Before you start

Do these once, before the first deploy:

1. Set `meta.site_url` in `webfluent.app.json` to the address the site will
   have. If the site lives under a sub-path (`/docs`), also set
   `build.base_path`.
2. Decide between a static build and a single-page build
   ([Static sites and SPAs](33-static-and-spa.md)). A single-page build
   needs the host to answer every path with `index.html`.
3. Build the site and load it in a real browser:

   ```bash
   wf build
   wf verify
   ```

   `wf verify` loads every page in headless Chrome and fails on a script
   error, a failed request or a blank page.

A static build writes each page as its own `index.html`
(`/contact` is `contact/index.html`). A page whose `path` is `*` is written
to the root as `404.html`, which is the file most static hosts serve for a
path they have no file for.

## Install `wf` on a build machine

To build on the host's build machine, install `wf` and run the build. Every
recipe in this chapter uses these two lines:

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
~/.webfluent/bin/wf build
```

Pin a version so that a new release cannot change a deploy without you
knowing. Set `WF_VERSION` for the install script:

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v5.13.0 bash
```

You can also build on your own machine and upload `build/`. It is only
files.

## Deploy to GitHub Pages

GitHub serves a project site under `https://<you>.github.io/<repo>/`, so the
build must know that sub-path.

1. In `webfluent.app.json`, set `"base_path": "/<repo>"` in `build` and
   `"site_url": "https://<you>.github.io/<repo>"` in `meta`.
2. GitHub Pages answers `/contact` with a `301` redirect to `/contact/`. To
   make the canonical address of every page one it serves without a
   redirect, also set `"clean_urls": "file"` (or `"directory"`) in `build`
   ([Content](27-content.md#keep-addresses-from-redirecting)).
3. Add `.github/workflows/pages.yml`:

   ```yaml
   name: Deploy
   on:
     push:
       branches: [main]
   permissions:
     contents: read
     pages: write
     id-token: write
   jobs:
     deploy:
       runs-on: ubuntu-latest
       environment:
         name: github-pages
         url: ${{ steps.deployment.outputs.page_url }}
       steps:
         - uses: actions/checkout@v4
         - run: curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v5.13.0 bash
         - run: ~/.webfluent/bin/wf build
         - uses: actions/upload-pages-artifact@v3
           with:
             path: build
         - id: deployment
           uses: actions/deploy-pages@v4
   ```

4. In the repository's **Settings → Pages**, set the source to
   **GitHub Actions**.
5. Push to `main`.

GitHub Pages serves `404.html` for a path it has no file for. It ignores
`_headers`, but the Content-Security-Policy still ships as a `<meta>` tag in
every page.

**Verify it:** open the URL the workflow's deploy step prints, then open a
page other than the home page directly and reload it. With `clean_urls` set,
the address stays as you typed it, with no redirect.

## Deploy to Netlify

1. Add `netlify.toml` at the project's root:

   ```toml
   [build]
     command = "curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash && ~/.webfluent/bin/wf build"
     publish = "build"
   ```

2. For a single-page build, add `public/_redirects` with this line. The
   build copies it to the root of `build/`:

   ```text
   /*  /index.html  200
   ```

3. Connect the repository in Netlify, or push to the branch it deploys.

Netlify reads `_headers` from the build, so the security headers apply with
no further setting.

**Verify it:** in the browser's network panel, select the page's document
request. Its response headers include `Content-Security-Policy` and
`X-Content-Type-Options: nosniff`, which came from `_headers`.

## Deploy to Cloudflare Pages

1. In the Cloudflare Pages project, set the build command to the one in the
   Netlify recipe above:
   `curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash && ~/.webfluent/bin/wf build`.
2. Set the build output directory to `build`.
3. For a single-page build, add `public/_redirects` as for Netlify.

Cloudflare Pages reads `_headers` and `_redirects` the same way Netlify
does.

**Verify it:** check the response headers in the network panel, as for
Netlify, and open a deep link directly.

## Deploy to Vercel

1. Add `vercel.json` at the project's root:

   ```json
   {
     "buildCommand": "curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash && ~/.webfluent/bin/wf build",
     "outputDirectory": "build",
     "headers": [
       {
         "source": "/(.*)",
         "headers": [
           { "key": "X-Content-Type-Options", "value": "nosniff" },
           { "key": "Referrer-Policy", "value": "strict-origin-when-cross-origin" },
           { "key": "X-Frame-Options", "value": "DENY" }
         ]
       }
     ]
   }
   ```

2. Vercel does not read `_headers`. Copy every header the build wrote in
   `build/_headers` into `headers`, including the `Content-Security-Policy`
   line.
3. For a single-page build, add
   `"rewrites": [{ "source": "/(.*)", "destination": "/index.html" }]`.

**Verify it:** compare the response headers in the network panel with
`build/_headers`. Each header in the file should be there.

## Serve with nginx

1. Add a `server` block. Pick the `try_files` line for your kind of build:

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

2. Upload the build with any tool. For example:

   ```bash
   rsync -a --delete build/ server:/var/www/site/
   ```

3. Reload nginx.

**Verify it:** `curl -I` a page and check the headers. Ask for a compressed
copy and check that nginx sends the build's `.gz` file:

```bash
curl -sI -H "Accept-Encoding: gzip" https://example.com/app.js
```

The response has `Content-Encoding: gzip`.

## Serve with Apache

1. Put this in `public/.htaccess`. The build copies it to the root of
   `build/`:

   ```apache
   Options -MultiViews
   RewriteEngine On
   # A single-page build:
   # RewriteCond %{REQUEST_FILENAME} !-f
   # RewriteRule ^ /index.html [L]
   ErrorDocument 404 /404.html
   Header always set X-Content-Type-Options "nosniff"
   Header always set X-Frame-Options "DENY"
   ```

2. For a single-page build, uncomment the two rewrite lines.
3. Compress the text files. Either turn on `mod_deflate`, which compresses
   on the way out, or serve the build's `.gz` files with `MultiViews` (in
   place of `-MultiViews`) and `AddEncoding gzip .gz`.
4. Upload `build/` to the site's document root.

**Verify it:** `curl -I` a page and check for `X-Content-Type-Options` and
`X-Frame-Options`. Open a path that does not exist and check that you get
the site's `404.html`.

## Serve with Caddy

1. Add a site block to the `Caddyfile`:

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

   `precompressed gzip` serves the build's `.gz` files. The last entry of
   `try_files`, `/index.html`, is the single-page fallback; remove it for a
   static build, so an unknown path is a 404.
2. Upload `build/` to `/var/www/site` and reload Caddy.

**Verify it:** open a deep link directly and reload it, then check the
headers with `curl -I`.

## Build a Docker image

1. Add a `Dockerfile`. The first stage installs `wf` and builds the site;
   the second serves it with nginx:

   ```dockerfile
   FROM debian:stable-slim AS build
   RUN apt-get update && apt-get install -y curl ca-certificates
   RUN curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
   WORKDIR /site
   COPY . .
   RUN /root/.webfluent/bin/wf build

   FROM nginx:alpine
   COPY --from=build /site/build /usr/share/nginx/html
   ```

2. To use the settings from [Serve with nginx](#serve-with-nginx), copy
   them into the second stage as `/etc/nginx/conf.d/default.conf`. When TLS
   ends in front of the container, write `listen 80;` in place of the
   `listen 443 ssl http2;` line.
3. Build and run the image:

   ```bash
   docker build -t site .
   docker run -p 8080:80 site
   ```

**Verify it:** open `http://localhost:8080` and a deep link, and check the
headers with `curl -I http://localhost:8080/`.

## Set cache headers

To let browsers keep files without serving a stale site, set
`Cache-Control` by the kind of file:

| Files | Cache-Control | Why |
|---|---|---|
| HTML, `app.js`, `pages/*`, `styles.css` | `no-cache` (revalidate every time) | their names do not change between builds |
| `sw.js` | `no-cache` | a stale worker holds a stale site ([Offline](26-offline.md)) |
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
[Security](30-security.md#before-you-deploy) has the full checklist.

## Serve an API on the same origin

To call a backend without CORS, serve the site and the backend from one
origin: the site at `/`, the backend at `/api`. A session cookie then works
with no extra setting. In nginx:

```nginx
location /api/ {
    proxy_pass http://127.0.0.1:8080/;
}
```

Then `api Backend(base: "/api")` needs nothing else. Netlify, Vercel and
Cloudflare Pages do the same with a rewrite to an external URL.

## Check the deploy

After each deploy, check these:

- Open a deep link directly and reload it. A 404 means the host does not
  fall back to `index.html` (a single-page build), or `base_path` is wrong.
- View a page's source. In a static build, the page's text is in the HTML.
- Check the response headers in the browser's network panel.
- Run a link-preview debugger on a page, and submit `sitemap.xml` to the
  search engines' consoles.

## Next

[Environments](35-environments.md).
