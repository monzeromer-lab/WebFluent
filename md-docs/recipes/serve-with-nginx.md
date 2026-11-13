---
kind: "recipe"
route: "/recipes/serve-with-nginx"
group: "ship"
description: "1."
from: ["/docs/guide/deploying#serve-with-nginx"]
---
# Serve a site with nginx

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
