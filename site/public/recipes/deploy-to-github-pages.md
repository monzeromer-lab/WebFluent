# Deploy to GitHub Pages

> Deploy a WebFluent site to GitHub Pages with a GitHub Actions workflow, under the repository's sub-path and with no trailing-slash redirects.

The page: https://webfluent.monzeromer.dev/recipes/deploy-to-github-pages

This recipe publishes a WebFluent site to GitHub Pages from a GitHub
Actions workflow, which builds the site on every push to `main`. GitHub
serves a project site under `https://<you>.github.io/<repo>/`, so the build
must know that sub-path.

You need the project in a GitHub repository; the workflow installs `wf` as
[Install wf on a build machine](https://webfluent.monzeromer.dev/recipes/install-wf-in-ci.md) shows.

Set the sub-path, the site's address, `clean_urls` and `ssg` in
`webfluent.app.json`:

```json
{
    "build": { "ssg": true, "base_path": "/<repo>", "clean_urls": "file" },
    "meta": { "site_url": "https://<you>.github.io/<repo>" }
}
```

GitHub Pages serves files and nothing else: it has no rewrite that sends
every address to `index.html`. `"ssg": true` writes a file for every
page, so a reader who reloads `/<repo>/tasks`, or opens a link to it, gets
that page. A project made with `wf init -t spa` has `"ssg": false`; keep
it that way only if you also add the `404.html` step below.

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
      # Only for a single-page build ("ssg": false): GitHub serves
      # 404.html for an address it has no file for, so the app loads there
      # and its router draws the page.
      # - run: cp build/index.html build/404.html
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
   ([Keep addresses from redirecting](https://webfluent.monzeromer.dev/recipes/search-and-link-previews.md#keep-addresses-from-redirecting)).
3. The workflow checks out the repository, installs a pinned `wf`, builds
   the site, uploads `build/` as the Pages artifact and deploys it. The
   `pages: write` and `id-token: write` permissions are what
   `actions/deploy-pages` needs.

GitHub Pages serves `404.html` for a path it has no file for. A static
build writes one when the project has a page on `path: "*"`; a page on a
`:param` route needs its values listed in `paths:` to have files of its own
([Data files](https://webfluent.monzeromer.dev/learn/15-data.md)). GitHub Pages ignores
`_headers`, but the Content-Security-Policy still ships as a `<meta>` tag in
every page.

## Check it

Open the URL the workflow's deploy step prints, then open a page other
than the home page directly and reload it. With `clean_urls` set, the
address stays as you typed it, with no redirect. Then go through
[Check a deploy](https://webfluent.monzeromer.dev/recipes/check-a-deploy.md).
