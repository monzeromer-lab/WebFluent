---
kind: "recipe"
route: "/recipes/deploy-to-vercel"
group: "ship"
description: "1."
from: ["/docs/guide/deploying#deploy-to-vercel"]
---
# Deploy to Vercel

1. Add `vercel.json` at the project's root:

   ```json
   {
     "buildCommand": "curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash && ~/.webfluent/bin/wf build",
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
     ]
   }
   ```

2. Vercel does not read `_headers`. Copy every header the build wrote in
   `build/_headers` into `headers`, including the `Content-Security-Policy`
   line.
3. For a single-page build, add
   `"rewrites": [{ "source": "/(.*)", "destination": "/index.html" }]`.

**Verify it:** compare the response headers in the network panel with
`build/_headers`. Each header in the file should be there.
