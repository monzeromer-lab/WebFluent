---
kind: "recipe"
route: "/recipes/check-a-deploy"
group: "ship"
description: "After each deploy, check these:"
from: ["/docs/guide/deploying#check-the-deploy"]
---
# Check a deploy

After each deploy, check these:

- Open a deep link directly and reload it. A 404 means the host does not
  fall back to `index.html` (a single-page build), or `base_path` is wrong.
- View a page's source. In a static build, the page's text is in the HTML.
- Check the response headers in the browser's network panel.
- Run a link-preview debugger on a page, and submit `sitemap.xml` to the
  search engines' consoles.
