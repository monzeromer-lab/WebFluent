# Recipes

> Complete, runnable answers to “how do I…”: each recipe is one task, its whole code, and how it works.

The page: https://webfluent.monzeromer.dev/recipes

Each recipe is one task: the complete code first, then how it works. Every
`wf` example on these pages parses and type-checks in the test suite; a
recipe that needs more than its page — a setting in `webfluent.app.json`,
a picture, a data file, a script, another recipe — says so in a
**Needs:** line above the code. To try one, paste the page into a new file
under `src/pages/` of a project. Most claim the address `/` or a name the
`wf init` starter already has (`Home`, `Tasks`, `Settings`, `User`), so
start a new project for it (`wf init try-it`), or replace the starter's
file of that name (`src/pages/Home.wf` for `/`).

## Interaction

Lists, dialogs, filters, the address bar and the keyboard: what a page does when the reader acts.

- [Keep the active tab in the address](https://webfluent.monzeromer.dev/recipes/active-tab-in-the-address.md)
- [Keep filters in the address](https://webfluent.monzeromer.dev/recipes/filters-in-the-address.md)
- [Build a confirm dialog](https://webfluent.monzeromer.dev/recipes/confirm-dialog.md)
- [Copy to the clipboard](https://webfluent.monzeromer.dev/recipes/copy-to-the-clipboard.md)
- [Debounce a search](https://webfluent.monzeromer.dev/recipes/debounce-a-search.md)
- [Count down](https://webfluent.monzeromer.dev/recipes/count-down.md)
- [Make navigation responsive](https://webfluent.monzeromer.dev/recipes/responsive-navigation.md)
- [Pass a store's value to a component](https://webfluent.monzeromer.dev/recipes/store-value-as-a-prop.md)
- [Reorder a list by dragging](https://webfluent.monzeromer.dev/recipes/reorder-by-dragging.md)
- [Add keyboard shortcuts and a command palette](https://webfluent.monzeromer.dev/recipes/keyboard-shortcuts.md)

## Forms and data

Forms, asking a server, showing what it answers, and writing back.

- [Show validation messages on a form](https://webfluent.monzeromer.dev/recipes/validation-messages.md)
- [Sign in with a session cookie](https://webfluent.monzeromer.dev/recipes/sign-in-with-a-session-cookie.md)
- [Page through results](https://webfluent.monzeromer.dev/recipes/page-through-results.md)
- [Load more as the reader asks](https://webfluent.monzeromer.dev/recipes/load-more.md)
- [Update at once, and roll back on failure](https://webfluent.monzeromer.dev/recipes/optimistic-updates.md)
- [Show what the network is doing](https://webfluent.monzeromer.dev/recipes/show-the-network.md)
- [Send a message as the page unloads](https://webfluent.monzeromer.dev/recipes/send-on-unload.md)

## Real time

Sockets, streams, other tabs, other readers and workers.

- [Open a socket](https://webfluent.monzeromer.dev/recipes/open-a-socket.md)
- [Listen to server-sent events](https://webfluent.monzeromer.dev/recipes/server-sent-events.md)
- [Talk to the site's other tabs](https://webfluent.monzeromer.dev/recipes/talk-between-tabs.md)
- [Connect two readers' pages](https://webfluent.monzeromer.dev/recipes/connect-two-readers.md)
- [Run a script in a worker](https://webfluent.monzeromer.dev/recipes/run-a-worker.md)

## Content

Markdown, pages written as Markdown, code on the page, a page for unknown addresses, a PDF, and what search engines and link previews read.

- [Write a page as a Markdown file](https://webfluent.monzeromer.dev/recipes/markdown-pages.md)
- [Render Markdown](https://webfluent.monzeromer.dev/recipes/render-markdown.md)
- [Show code on a page](https://webfluent.monzeromer.dev/recipes/show-code.md)
- [Describe pages to search engines and link previews](https://webfluent.monzeromer.dev/recipes/search-and-link-previews.md)
- [Show your own page for an unknown address](https://webfluent.monzeromer.dev/recipes/not-found-page.md)
- [Make a PDF report](https://webfluent.monzeromer.dev/recipes/pdf-report.md)

## Media and themes

Pictures, video, icons, a carousel, other pages in a frame, and a dark theme.

- [Add a picture, sized for every screen](https://webfluent.monzeromer.dev/recipes/add-a-picture.md)
- [Add video and audio](https://webfluent.monzeromer.dev/recipes/video-and-audio.md)
- [Show an icon](https://webfluent.monzeromer.dev/recipes/show-an-icon.md)
- [Show a carousel](https://webfluent.monzeromer.dev/recipes/show-a-carousel.md)
- [Show another page in a frame](https://webfluent.monzeromer.dev/recipes/show-a-page-in-a-frame.md)
- [Add a dark-mode switch](https://webfluent.monzeromer.dev/recipes/dark-mode-switch.md)

## Motion

Animation, from one element appearing to a page carried into the next. Read them in order: each builds on the one before.

- [Animate an element in and out](https://webfluent.monzeromer.dev/recipes/animate-in-and-out.md)
- [Animate a list](https://webfluent.monzeromer.dev/recipes/animate-a-list.md)
- [Run a sequence of animations](https://webfluent.monzeromer.dev/recipes/animation-sequence.md)
- [Animate route changes](https://webfluent.monzeromer.dev/recipes/route-transitions.md)
- [Carry an element across a route change](https://webfluent.monzeromer.dev/recipes/shared-element.md)
- [Open a box to the height of its content](https://webfluent.monzeromer.dev/recipes/expand-to-content.md)
- [Count a number to its new value](https://webfluent.monzeromer.dev/recipes/count-a-number.md)
- [Tween a property when state changes](https://webfluent.monzeromer.dev/recipes/tween-on-change.md)
- [Declare your own keyframes](https://webfluent.monzeromer.dev/recipes/keyframes.md)
- [Drive an animation yourself](https://webfluent.monzeromer.dev/recipes/drive-an-animation.md)
- [Set motion defaults, and respect reduced motion](https://webfluent.monzeromer.dev/recipes/motion-defaults.md)

## Reach

More readers: other languages, and no network.

- [Translate a site](https://webfluent.monzeromer.dev/recipes/translate-a-site.md)
- [Translate by the words as written](https://webfluent.monzeromer.dev/recipes/translate-by-the-words.md)
- [Publish a page in each language](https://webfluent.monzeromer.dev/recipes/site-in-two-languages.md)
- [Make a site work offline](https://webfluent.monzeromer.dev/recipes/work-offline.md)
- [Keep writes made offline](https://webfluent.monzeromer.dev/recipes/keep-writes-made-offline.md)
- [Offer a new version](https://webfluent.monzeromer.dev/recipes/offer-a-new-version.md)

## Testing

Checking that a page does what it should before a reader finds out it does not.

- [Test a page](https://webfluent.monzeromer.dev/recipes/test-a-page.md)

## Ship

Putting a site on a host, in the order you need them: install `wf` where the site is built, set what changes per environment, deploy to a host or serve from your own server, then check the deploy.

- [Install wf on a build machine](https://webfluent.monzeromer.dev/recipes/install-wf-in-ci.md)
- [Use environment values, and keep secrets out](https://webfluent.monzeromer.dev/recipes/environments-and-secrets.md)
- [Deploy to GitHub Pages](https://webfluent.monzeromer.dev/recipes/deploy-to-github-pages.md)
- [Deploy to Netlify](https://webfluent.monzeromer.dev/recipes/deploy-to-netlify.md)
- [Deploy to Vercel](https://webfluent.monzeromer.dev/recipes/deploy-to-vercel.md)
- [Deploy to Cloudflare Pages](https://webfluent.monzeromer.dev/recipes/deploy-to-cloudflare-pages.md)
- [Build a Docker image](https://webfluent.monzeromer.dev/recipes/docker-image.md)
- [Serve a site with nginx](https://webfluent.monzeromer.dev/recipes/serve-with-nginx.md)
- [Serve a site with Apache](https://webfluent.monzeromer.dev/recipes/serve-with-apache.md)
- [Serve a site with Caddy](https://webfluent.monzeromer.dev/recipes/serve-with-caddy.md)
- [Serve an API on the same origin](https://webfluent.monzeromer.dev/recipes/api-on-the-same-origin.md)
- [Set cache and security headers](https://webfluent.monzeromer.dev/recipes/cache-and-security-headers.md)
- [Check a deploy](https://webfluent.monzeromer.dev/recipes/check-a-deploy.md)

## Desktop and phone apps

The same project in a window of its own, with Tauri or Electron.

- [Make a desktop or phone app with Tauri](https://webfluent.monzeromer.dev/recipes/tauri-app.md)
- [Make a desktop app with Electron](https://webfluent.monzeromer.dev/recipes/electron-app.md)

## Interop

Your own JavaScript, libraries, and custom elements, both ways.

- [Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)
- [Call WF from a script](https://webfluent.monzeromer.dev/recipes/call-wf-from-a-script.md)
- [Hand an element to a script](https://webfluent.monzeromer.dev/recipes/hand-an-element-to-a-script.md)
- [Run a script after each page draws](https://webfluent.monzeromer.dev/recipes/listen-for-wf-render.md)
- [Draw into a node a script made](https://webfluent.monzeromer.dev/recipes/portal.md)
- [Load a library from a CDN](https://webfluent.monzeromer.dev/recipes/library-from-a-cdn.md)
- [Draw a chart from a library](https://webfluent.monzeromer.dev/recipes/chart-from-a-library.md)
- [Place a custom element from a library](https://webfluent.monzeromer.dev/recipes/place-a-custom-element.md)
- [Publish components as custom elements](https://webfluent.monzeromer.dev/recipes/publish-custom-elements.md)
- [Migrate from external](https://webfluent.monzeromer.dev/recipes/migrate-from-external.md)

## Server rendering

WebFluent as a template engine: HTML and PDF from data, from the command line, Node or Rust.

- [Render a template with data](https://webfluent.monzeromer.dev/recipes/render-a-template.md)
- [Render templates from Node](https://webfluent.monzeromer.dev/recipes/render-from-node.md)
- [Render templates from Rust](https://webfluent.monzeromer.dev/recipes/render-from-rust.md)

## Projects

Two complete projects, explained step by step. They are read along in the finished code under `examples/tutorials/`, not typed along; each says how to build it yourself if you want to.

- [Build a statically generated notebook](https://webfluent.monzeromer.dev/recipes/build-a-notebook-site.md)
- [Build a budget app that works offline](https://webfluent.monzeromer.dev/recipes/build-a-budget-app.md)
