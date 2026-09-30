# 29. Deploying

<!--
route: guide/deploying
group: shipping
blurb: The build is a folder of static files. Here is how to put it on GitHub Pages, Netlify, Vercel, Cloudflare Pages, a server of your own, or a container — or ship it as an Android app.
description: Deploying a build to GitHub Pages, Netlify, Vercel, Cloudflare Pages, nginx, Apache, Caddy or Docker, with cache and security headers, or as an Android app.
-->

`wf build` writes `build/`: HTML, CSS, JavaScript, images, `sitemap.xml`,
`robots.txt`, `_headers`, and a `.gz` beside every text file. That folder is
the whole site. Any host that serves static files can serve it; there is no
server process to run. The same build can also ship as
[an Android app](#an-android-app).

Before the first deploy:

1. Set `meta.site_url` to the address the site will have, and
   `build.base_path` if it lives under a sub-path (`/docs`).
2. Decide static or single-page ([chapter 26](26-static-and-spa.md)): a
   single-page build needs the host to answer every path with `index.html`.
3. Run `wf build` and `wf verify` — the second loads every page in a real
   browser and fails on errors.

## Installing `wf` on a build machine

Every recipe below runs the same two lines on the host's build machine:

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
~/.webfluent/bin/wf build
```

Pin a version so a release cannot change a deploy under you:
`WF_VERSION=v4.0.1`. Or build locally and upload `build/` — it is only files.

## GitHub Pages

A project site is served under `https://<you>.github.io/<repo>/`, so set
`"base_path": "/<repo>"` and `"site_url": "https://<you>.github.io/<repo>"`.
Then `.github/workflows/pages.yml`:

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
      - run: curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v4.0.1 bash
      - run: ~/.webfluent/bin/wf build
      - uses: actions/upload-pages-artifact@v3
        with:
          path: build
      - id: deployment
        uses: actions/deploy-pages@v4
```

In *Settings → Pages*, set the source to *GitHub Actions*. GitHub Pages
serves `404.html` for unknown paths, so a static build needs nothing else. It
ignores `_headers`; the Content-Security-Policy still ships as a `<meta>` tag
in every page.

## Netlify

`netlify.toml` at the project's root:

```toml
[build]
  command = "curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash && ~/.webfluent/bin/wf build"
  publish = "build"
```

Netlify reads `_headers` from the build, so the security headers apply. For a
single-page build, add `public/_redirects` holding `/*  /index.html  200`.

## Cloudflare Pages

Build command as for Netlify, output directory `build`. Cloudflare reads
`_headers` and `_redirects` the same way.

## Vercel

`vercel.json`:

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

Vercel does not read `_headers`: copy what the build wrote there into
`headers`, including the `Content-Security-Policy` line. For a single-page
build add `"rewrites": [{ "source": "/(.*)", "destination": "/index.html" }]`.

## nginx

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

Upload with anything: `rsync -a --delete build/ server:/var/www/site/`.

## Apache

`public/.htaccess` is copied to the build's root:

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

For the precompressed files, turn on `mod_deflate`, or serve `.gz` with
`MultiViews` and `AddEncoding gzip .gz`.

## Caddy

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

## Docker

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

Add the nginx settings above as `/etc/nginx/conf.d/default.conf`.

## Cache headers

| Files | Cache-Control | Why |
|---|---|---|
| HTML, `app.js`, `pages/*`, `styles.css` | `no-cache` (revalidate every time) | their names do not change between builds |
| `sw.js` | `no-cache` | a stale worker holds a stale site ([Offline](19-offline.md)) |
| resized images (`hero.<hash>.960.webp`) | `public, max-age=31536000, immutable` | the name changes when the file does |
| `public/` files | your choice | they keep the names you gave them |

`no-cache` still lets the browser keep a copy: it asks, and a `304` answers
without sending the file again.

## Security headers

With `build.csp` on, every page carries its Content-Security-Policy as a
`<meta>` tag, and `build/_headers` holds the same policy plus
`X-Content-Type-Options`, `Referrer-Policy`, `X-Frame-Options` and
`frame-ancestors` — which only work as real headers. Netlify and Cloudflare
Pages read the file; elsewhere set the headers as above. Serve over HTTPS,
with HSTS once you are sure. [Security](23-security.md#before-you-deploy) has
the checklist.

## An API on the same origin

The simplest way to call a backend from a WebFluent site is to serve both
from one origin — `/` the site, `/api` the backend — so there is no CORS and a
session cookie just works:

```nginx
location /api/ {
    proxy_pass http://127.0.0.1:8080/;
}
```

Then `api Backend(base: "/api")` needs nothing else. Netlify, Vercel and
Cloudflare have the same with a rewrite to an external URL.

## After deploying

- Open a deep link directly and reload it: a 404 means the host is not
  falling back to `index.html` (single-page) or the `base_path` is wrong.
- View a page's source: the text should be in the HTML (static builds).
- Check the response headers in the browser's network panel.
- Run a link-preview debugger on a page, and submit `sitemap.xml` to search
  consoles.

## An Android app

The same build can be an Android app: its pages, in a WebView, in a project
Android Studio opens. `wf init ledger -t android` starts one; in an existing
project it is two settings:

```json
{
  "build": {
    "output_type": "android",
    "android": {
      "application_id": "com.yourname.ledger",
      "app_name": "Ledger",
      "icon": "public/icon.png"
    }
  }
}
```

`application_id` is the app's identity on a phone and in Google Play, and it
can never change once the app is published, so the build refuses to guess
one. The other settings are under `build.android` in
[Configuration](38-configuration.md).

`wf build` then writes the site to `build/`, as always, and an Android
project to `android/` with the site inside it:

```text
android/
├── settings.gradle.kts, build.gradle.kts     written once
└── app/
    ├── build.gradle.kts                      written once: signing, dependencies
    ├── webfluent.properties                  every build: id, version, Android levels
    └── src/main/
        ├── AndroidManifest.xml               written once: permissions, deep links
        ├── java/…/MainActivity.java          written once: the WebView
        ├── assets/www/                       every build: the site
        └── res/                              every build: icons, name, colours
```

A file written once is yours: no build touches it again, so signing, a
permission or a screen of native code goes there. The rest follows the
project, and is written only where it changed, so Gradle has nothing to redo
after a build that changed nothing. `android/.gitignore` keeps it out of
version control, since `wf build` writes it again.

Open `android/` in Android Studio and run the app. From the command line,
with Gradle and the Android SDK installed, `gradle assembleDebug` in
`android/` builds it, and `gradle wrapper` once gives the project its own
`./gradlew`. A release for Google Play is `bundleRelease`, signed with your
upload key: Android Studio's *Build › Generate Signed App Bundle*, or a
`signingConfigs` block in `app/build.gradle.kts`.

What the app does:

- **Its pages are on the phone.** They are served from the app's own files
  on `https://appassets.androidplatform.net`, a host Android keeps for this,
  so storage, `fetch` and the Content-Security-Policy behave as on the web,
  and the app opens with no network. A request anywhere else goes to the
  network, as it would from a browser.
- **The WebView is Chromium**, the engine `wf verify` and `wf test` drive:
  what they check in `build/` is what the app shows. `wf serve` shows the
  pages in a browser while you work, and a debug build can be inspected from
  desktop Chrome at `chrome://inspect`.
- **Back goes back** through the pages the reader visited, then leaves the
  app. Android 13 and later preview that with the back gesture.
- **A link out of the app** — another site, `mailto:`, `tel:` — opens where
  the phone opens it. An upload field opens the phone's file picker.
- **The system bars take the page's `color-background`**, and the dark
  theme's at night, with icons that can be read on it. From Android 15 the
  app draws edge to edge, and the page is kept clear of the bars, a camera
  cutout and the keyboard.
- **The launcher icon** is made from `icon` at every screen density, and as
  an adaptive icon, whose shape the launcher chooses.

Inside an app the build leaves out what only a host needs: `base_path`, the
`offline` service worker, the `.gz` files, `_headers` and the sitemap. The
version the phone shows is `version`, and the version code Google Play orders
uploads by is worked out from it — `1.4.2` is `1004002` — so raising `version`
is the one step a release takes.

## Next

[Environments and configuration](30-environments.md).
