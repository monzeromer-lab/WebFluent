---
kind: "recipe"
route: "/recipes/api-on-the-same-origin"
group: "ship"
description: "To call a backend without CORS, serve the site and the backend from one origin: the site at `/`, the backend at `/api`."
from: ["/docs/guide/deploying#serve-an-api-on-the-same-origin"]
---
# Serve an API on the same origin

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
