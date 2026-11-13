---
kind: "recipe"
route: "/recipes/deploy-to-cloudflare-pages"
group: "ship"
description: "1."
from: ["/docs/guide/deploying#deploy-to-cloudflare-pages"]
---
# Deploy to Cloudflare Pages

1. In the Cloudflare Pages project, set the build command to the one in the
   Netlify recipe above:
   `curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash && ~/.webfluent/bin/wf build`.
2. Set the build output directory to `build`.
3. For a single-page build, add `public/_redirects` as for Netlify.

Cloudflare Pages reads `_headers` and `_redirects` the same way Netlify
does.

**Verify it:** check the response headers in the network panel, as for
Netlify, and open a deep link directly.
