---
kind: "recipe"
route: "/recipes/serve-with-caddy"
group: "ship"
description: "1."
from: ["/docs/guide/deploying#serve-with-caddy"]
---
# Serve a site with Caddy

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
