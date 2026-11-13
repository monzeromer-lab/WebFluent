---
kind: "recipe"
route: "/recipes/serve-with-apache"
group: "ship"
description: "Serve a WebFluent build with Apache: an .htaccess file with the 404 page, a single-page fallback, security headers and compressed files."
outputs: ["website", "app"]
from: ["/docs/guide/deploying#serve-with-apache"]
---
# Serve a site with Apache

This recipe serves a WebFluent build with Apache, configured by an
`.htaccess` file that ships with the build.

You need an Apache host with `mod_rewrite` and `mod_headers`, and a
`build/` folder from `wf build`
([Install wf on a build machine](install-wf-in-ci.md)).

Put this in `public/.htaccess`:

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

Then upload `build/` to the site's document root.

## How it works

1. The build copies `public/.htaccess` to the root of `build/`, so it
   lands in the document root with the site.
2. For a single-page build, uncomment the two rewrite lines: a path that
   is no file is answered with `index.html`, where the router draws the
   page.
3. `ErrorDocument 404 /404.html` serves the page whose `path` is `*`.
4. The `Header` lines send security headers the build writes to
   `build/_headers` ([Set cache and security headers](cache-and-security-headers.md#set-security-headers)).

## Variations

Compress the text files one of two ways:

- Turn on `mod_deflate`, which compresses on the way out.
- Serve the `.gz` files the build wrote: write `Options MultiViews` in
  place of `Options -MultiViews`, and add `AddEncoding gzip .gz`.

## Check it

`curl -I` a page and check for `X-Content-Type-Options` and
`X-Frame-Options`. Open a path that does not exist and check that you get
the site's `404.html`. Then go through [Check a deploy](check-a-deploy.md).
