---
kind: "recipe"
route: "/recipes/deploy-to-netlify"
group: "ship"
description: "Deploy a WebFluent site to Netlify: a netlify.toml that installs wf and builds, a fallback for a single-page build, and the build's security headers."
outputs: ["website", "app"]
from: ["/docs/guide/deploying#deploy-to-netlify"]
---
# Deploy to Netlify

This recipe has Netlify build a WebFluent site on every push and serve it,
with the security headers the build writes.

You need a Netlify account and the project in a repository Netlify can
read; the build command installs `wf` as
[Install wf on a build machine](install-wf-in-ci.md) shows.

Add `netlify.toml` at the project's root:

```toml
[build]
  command = "curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v6.0.0 bash && ~/.webfluent/bin/wf build"
  publish = "build"
```

For a single-page build, also add `public/_redirects` with this line:

```text
/*  /index.html  200
```

Then connect the repository in Netlify, or push to the branch it deploys.

## How it works

1. `command` installs `wf` and builds the site; `publish` tells Netlify to
   serve the `build/` folder.
2. The build copies `public/_redirects` to the root of `build/`. Its line
   answers every path with `index.html` and a `200`, so a reload on
   `/users/42` reaches the single-page build's router.
3. Netlify reads `_headers` from the build, so the security headers apply
   with no further setting
   ([Set cache and security headers](cache-and-security-headers.md)).

## Check it

In the browser's network panel, select the page's document request. Its
response headers include `Content-Security-Policy` and
`X-Content-Type-Options: nosniff`, which came from `_headers`. Then go
through [Check a deploy](check-a-deploy.md).
