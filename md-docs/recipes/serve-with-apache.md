---
kind: "recipe"
route: "/recipes/serve-with-apache"
group: "ship"
description: "1."
from: ["/docs/guide/deploying#serve-with-apache"]
---
# Serve a site with Apache

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
