---
kind: "recipe"
route: "/recipes/deploy-to-netlify"
group: "ship"
description: "1."
from: ["/docs/guide/deploying#deploy-to-netlify"]
---
# Deploy to Netlify

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
