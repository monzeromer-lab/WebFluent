---
kind: "recipe"
route: "/recipes/deploy-to-cloudflare-pages"
group: "ship"
description: "Deploy a WebFluent site to Cloudflare Pages: the build command and output folder to set, and a fallback for a single-page build."
outputs: ["website", "app"]
from: ["/docs/guide/deploying#deploy-to-cloudflare-pages"]
---
# Deploy to Cloudflare Pages

This recipe has Cloudflare Pages build a WebFluent site on every push and
serve it, with the security headers the build writes.

You need a Cloudflare account and a Pages project connected to the
repository; the build command installs `wf` as
[Install wf on a build machine](install-wf-in-ci.md) shows.

In the Pages project's build settings, set the build command:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v6.0.0 bash && ~/.webfluent/bin/wf build
```

and set the build output directory to `build`.

For a single-page build, also add `public/_redirects` with this line:

```text
/*  /index.html  200
```

## How it works

1. The build command installs `wf` and builds the site, the same command
   [Deploy to Netlify](deploy-to-netlify.md) uses. Cloudflare Pages serves
   the `build` folder.
2. The build copies `public/_redirects` to the root of `build/`. Its line
   answers every path with `index.html`, so a reload on a deep link
   reaches the single-page build's router.
3. Cloudflare Pages reads `_headers` and `_redirects` the same way Netlify
   does, so the security headers apply with no further setting.

## Check it

Check the response headers in the browser's network panel, as for
Netlify, and open a deep link directly. Then go through
[Check a deploy](check-a-deploy.md).
