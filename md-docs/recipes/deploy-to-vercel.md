---
kind: "recipe"
route: "/recipes/deploy-to-vercel"
group: "ship"
description: "Deploy a WebFluent site to Vercel with a vercel.json that installs wf, builds, sets the security headers and rewrites a single-page build."
outputs: ["website", "app"]
from: ["/docs/guide/deploying#deploy-to-vercel"]
---
# Deploy to Vercel

This recipe has Vercel build a WebFluent site on every push and serve it.
Vercel does not read the `_headers` file the build writes, so the headers
go in `vercel.json`.

You need a Vercel account and the project in a repository Vercel can read;
the build command installs `wf` as
[Install wf on a build machine](install-wf-in-ci.md) shows.

Add `vercel.json` at the project's root. This one is for a single-page
build:

```json
{
  "buildCommand": "curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v6.0.0 bash && ~/.webfluent/bin/wf build",
  "outputDirectory": "build",
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "X-Content-Type-Options", "value": "nosniff" },
        { "key": "Referrer-Policy", "value": "strict-origin-when-cross-origin" },
        { "key": "X-Frame-Options", "value": "DENY" }
      ]
    }
  ],
  "rewrites": [{ "source": "/(.*)", "destination": "/index.html" }]
}
```

## How it works

1. `buildCommand` installs `wf` and builds the site; `outputDirectory`
   tells Vercel to serve `build/`.
2. `headers` sets the security headers on every path. Vercel does not read
   `_headers`, so copy every header the build wrote in `build/_headers`
   into this list, including the `Content-Security-Policy` line
   ([Set cache and security headers](cache-and-security-headers.md#set-security-headers)).
3. `rewrites` answers every path with `index.html`, so a reload on a deep
   link reaches the single-page build's router.

## Variations

For a static build, leave out `rewrites`: each page has its own file, and
an unknown path should be a 404.

## Check it

Compare the response headers in the browser's network panel with
`build/_headers`. Each header in the file should be there. Then go through
[Check a deploy](check-a-deploy.md).
