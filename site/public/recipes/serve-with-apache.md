# Serve a site with Apache

> Serve a WebFluent build with Apache: an .htaccess file with the 404 page, a single-page fallback, security headers and compressed files.

The page: https://webfluent.monzeromer.dev/recipes/serve-with-apache

This recipe serves a WebFluent build with Apache, configured by an
`.htaccess` file that ships with the build.

You need an Apache host with `mod_rewrite` and `mod_headers`, and a
`build/` folder from `wf build`
([Install wf on a build machine](https://webfluent.monzeromer.dev/recipes/install-wf-in-ci.md)).

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
Header always set Referrer-Policy "strict-origin-when-cross-origin"
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
   `build/_headers` ([Set cache and security headers](https://webfluent.monzeromer.dev/recipes/cache-and-security-headers.md#set-security-headers)).

## Variations

Compress the text files one of two ways:

- Turn on `mod_deflate`, which compresses on the way out. This is the
  simplest, and covers every text file.
- Serve the `.gz` files the build wrote, so Apache compresses nothing
  itself. `Options MultiViews` does not do it: content negotiation only
  runs when the file asked for does not exist, and `app.js` does. Add these
  lines after `RewriteEngine On`, before the single-page lines, as Apache's
  own `mod_deflate` documentation does it:

  ```apache
  # The build's app.js.gz for app.js, when the browser accepts gzip
  RewriteCond "%{HTTP:Accept-Encoding}" "gzip"
  RewriteCond "%{REQUEST_FILENAME}.gz" -s
  RewriteRule "^(.*)\.(css|js)$" "$1.$2.gz" [QSA]

  # Sent with the right type, and not compressed a second time
  RewriteRule "\.css\.gz$" "-" [T=text/css,E=no-gzip:1]
  RewriteRule "\.js\.gz$" "-" [T=text/javascript,E=no-gzip:1]

  <FilesMatch "\.(css|js)\.gz$">
      Header append Content-Encoding gzip
      Header append Vary Accept-Encoding
  </FilesMatch>
  ```

  This covers the scripts and stylesheets, the largest files. Pages are
  asked for by their folder (`/contact/`), which the rule cannot see as a
  file, so leave `mod_deflate` on for HTML.

## Check it

`curl -I` a page and check for `X-Content-Type-Options`,
`X-Frame-Options` and `Referrer-Policy`. With the `.gz` rules,
`curl -sI -H "Accept-Encoding: gzip" https://example.com/app.js` answers
`Content-Encoding: gzip` and `Content-Type: text/javascript`. Open a path that does not exist and check that you get
the site's `404.html`. Then go through [Check a deploy](https://webfluent.monzeromer.dev/recipes/check-a-deploy.md).
