---
kind: "recipe"
route: "/recipes/install-wf-in-ci"
group: "ship"
description: "Do these once, before the first deploy:"
from: ["/docs/guide/deploying#before-you-start", "/docs/guide/deploying#install-wf-on-a-build-machine"]
---
# Install wf on a build machine

## Before you start

Do these once, before the first deploy:

1. Set `meta.site_url` in `webfluent.app.json` to the address the site will
   have. If the site lives under a sub-path (`/docs`), also set
   `build.base_path`.
2. Decide between a static build and a single-page build
   ([Static sites and SPAs](../about/static-and-spa.md)). A single-page build
   needs the host to answer every path with `index.html`.
3. Build the site and load it in a real browser:

   ```bash
   wf build
   wf verify
   ```

   `wf verify` loads every page in headless Chrome and fails on a script
   error, a failed request or a blank page.

A static build writes each page as its own `index.html`
(`/contact` is `contact/index.html`). A page whose `path` is `*` is written
to the root as `404.html`, which is the file most static hosts serve for a
path they have no file for.

## Install `wf` on a build machine

To build on the host's build machine, install `wf` and run the build. Every
recipe in this chapter uses these two lines:

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
~/.webfluent/bin/wf build
```

Pin a version so that a new release cannot change a deploy without you
knowing. Set `WF_VERSION` for the install script:

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v6.0.0 bash
```

You can also build on your own machine and upload `build/`. It is only
files.
