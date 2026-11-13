---
kind: "recipe"
route: "/recipes/deploy-to-github-pages"
group: "ship"
description: "Deploy a WebFluent site to GitHub Pages with a GitHub Actions workflow, under the repository's sub-path and with no trailing-slash redirects."
outputs: ["website", "app"]
from: ["/docs/guide/deploying#deploy-to-github-pages"]
---
# Deploy to GitHub Pages

This recipe publishes a WebFluent site to GitHub Pages from a GitHub
Actions workflow, which builds the site on every push to `main`. GitHub
serves a project site under `https://<you>.github.io/<repo>/`, so the build
must know that sub-path.

You need the project in a GitHub repository; the workflow installs `wf` as
[Install wf on a build machine](install-wf-in-ci.md) shows.

Set the sub-path, the site's address and `clean_urls` in
`webfluent.app.json`:

```json
{
    "build": { "base_path": "/<repo>", "clean_urls": "file" },
    "meta": { "site_url": "https://<you>.github.io/<repo>" }
}
```

Then add `.github/workflows/pages.yml`:

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

Finally, in the repository's **Settings → Pages**, set the source to
**GitHub Actions**, and push to `main`.

## How it works

1. `build.base_path` prefixes every link, every asset and the router's
   paths with `/<repo>`, and `meta.site_url` is the address the canonical
   links and the sitemap name.
2. GitHub Pages answers `/contact` with a `301` redirect to `/contact/`.
   `"clean_urls": "file"` (or `"directory"`) makes the canonical address of
   every page one it serves without a redirect
   ([Keep addresses from redirecting](search-and-link-previews.md#keep-addresses-from-redirecting)).
3. The workflow checks out the repository, installs a pinned `wf`, builds
   the site, uploads `build/` as the Pages artifact and deploys it. The
   `pages: write` and `id-token: write` permissions are what
   `actions/deploy-pages` needs.

GitHub Pages serves `404.html` for a path it has no file for. It ignores
`_headers`, but the Content-Security-Policy still ships as a `<meta>` tag in
every page.

## Check it

Open the URL the workflow's deploy step prints, then open a page other
than the home page directly and reload it. With `clean_urls` set, the
address stays as you typed it, with no redirect. Then go through
[Check a deploy](check-a-deploy.md).
