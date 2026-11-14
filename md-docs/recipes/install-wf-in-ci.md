---
kind: "recipe"
route: "/recipes/install-wf-in-ci"
group: "ship"
description: "Install the wf compiler on a CI runner or a host's build machine, pin its version, and build the site there before each deploy."
outputs: ["website", "app"]
from: ["/docs/guide/deploying#before-you-start", "/docs/guide/deploying#install-wf-on-a-build-machine"]
---
# Install wf on a build machine

This recipe installs `wf` on a machine that builds the site for you — a CI
runner, or the build step of a host such as Netlify — and runs the build
there. Every hosting recipe uses it.

You need a project that builds with `wf build` on your own machine, and a
build machine with `curl` and `bash`.

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | WF_VERSION=v6.0.0 bash
~/.webfluent/bin/wf build
```

## How it works

1. The install script downloads `wf` into `~/.webfluent/bin/`. `WF_VERSION`
   pins the release it installs, so a new release cannot change a deploy
   without you knowing.
   `-f` makes `curl` fail on an error from the server instead of piping
   the error page to `bash`; `--proto '=https' --tlsv1.2` refuses to fetch
   the script over anything but HTTPS.
2. `~/.webfluent/bin/wf build` builds the project in the current folder
   and writes the whole site to `build/`. The full path works even where
   the install has not put `wf` on the `PATH` yet.

A static build writes each page as its own `index.html` (`/contact` is
`contact/index.html`). A page whose `path` is `*` is written to the root as
`404.html`, which is the file most static hosts serve for a path they have
no file for.

## Variations

Without `WF_VERSION`, the script installs the latest release:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
```

To find the latest release, the script asks GitHub's API, which limits how
often one address may ask. A shared CI runner often hits that limit, and
the install stops with "could not determine the latest version". A pinned
`WF_VERSION` never asks, which is one more reason every hosting recipe
pins it.

You can also build on your own machine and upload `build/`. It is only
files.

## Prepare the site before the first deploy

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

After the deploy, [check it](check-a-deploy.md).
