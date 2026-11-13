---
kind: "recipe"
route: "/recipes/deploy-to-github-pages"
group: "ship"
description: "GitHub serves a project site under `https://<you>.github.io/<repo>/`, so the build must know that sub-path."
from: ["/docs/guide/deploying#deploy-to-github-pages"]
---
# Deploy to GitHub Pages

GitHub serves a project site under `https://<you>.github.io/<repo>/`, so the
build must know that sub-path.

1. In `webfluent.app.json`, set `"base_path": "/<repo>"` in `build` and
   `"site_url": "https://<you>.github.io/<repo>"` in `meta`.
2. GitHub Pages answers `/contact` with a `301` redirect to `/contact/`. To
   make the canonical address of every page one it serves without a
   redirect, also set `"clean_urls": "file"` (or `"directory"`) in `build`
   ([Content](index.md)).
3. Add `.github/workflows/pages.yml`:

   ```yaml
   name: Deploy
   on:
     push:
       branches: [main]
   permissions:
     contents: read
     pages: write
     id-token: write
   jobs:
     deploy:
       runs-on: ubuntu-latest
       environment:
         name: github-pages
         url: ${{ steps.deployment.outputs.page_url }}
       steps:
         - uses: actions/checkout@v4
         - run: curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v6.0.0 bash
         - run: ~/.webfluent/bin/wf build
         - uses: actions/upload-pages-artifact@v3
           with:
             path: build
         - id: deployment
           uses: actions/deploy-pages@v4
   ```

4. In the repository's **Settings → Pages**, set the source to
   **GitHub Actions**.
5. Push to `main`.

GitHub Pages serves `404.html` for a path it has no file for. It ignores
`_headers`, but the Content-Security-Policy still ships as a `<meta>` tag in
every page.

**Verify it:** open the URL the workflow's deploy step prints, then open a
page other than the home page directly and reload it. With `clean_urls` set,
the address stays as you typed it, with no redirect.
