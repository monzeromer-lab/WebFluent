---
kind: "recipe"
route: "/recipes/check-a-deploy"
group: "ship"
description: "Check a WebFluent site after each deploy: deep links that reload, text in the HTML, the response headers, link previews and the sitemap."
outputs: ["website", "app"]
from: ["/docs/guide/deploying#check-the-deploy"]
---
# Check a deploy

This recipe is the list to go through after each deploy, on any host: the
mistakes it catches pass every check on your own machine.

You need the deployed site's address.

- Open a deep link directly and reload it. A 404 means the host does not
  fall back to `index.html` (a single-page build), or `base_path` is wrong.
- View a page's source. In a static build, the page's text is in the HTML.
- Check the response headers in the browser's network panel.
- Run a link-preview debugger on a page, and submit `sitemap.xml` to the
  search engines' consoles.

## How it works

- A deep link loaded directly is the one request the host answers without
  the router: the home page can work while every other address is a 404.
- A static build pre-renders every page, so its text should be in the
  HTML the host sends ([Static sites and SPAs](../about/static-and-spa.md)).
- The headers show whether the host sends what `build/_headers` holds
  ([Set cache and security headers](cache-and-security-headers.md)).
- A link preview and the sitemap use the absolute addresses
  `meta.site_url` names
  ([Describe pages to search engines and link previews](search-and-link-previews.md)).

Before the next deploy, `wf verify` loads every page of the build in
headless Chrome and catches a script error, a failed request or a blank
page before it ships ([Install wf on a build machine](install-wf-in-ci.md)).
