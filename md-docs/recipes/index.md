---
kind: "recipe"
route: "/recipes"
description: "Complete, runnable answers to “how do I…”: each recipe is one task, its whole code, and how it works."
---
# Recipes

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

- [Keep the active tab in the address](active-tab-in-the-address.md)
- [Keep filters in the address](filters-in-the-address.md)
- [Build a confirm dialog](confirm-dialog.md)
- [Copy to the clipboard](copy-to-the-clipboard.md)
- [Debounce a search](debounce-a-search.md)
- [Count down](count-down.md)
- [Make navigation responsive](responsive-navigation.md)
- [Pass a store's value to a component](store-value-as-a-prop.md)
- [Reorder a list by dragging](reorder-by-dragging.md)
- [Add keyboard shortcuts and a command palette](keyboard-shortcuts.md)

## Forms and data

Forms, asking a server, showing what it answers, and writing back.

- [Show validation messages on a form](validation-messages.md)
- [Sign in with a session cookie](sign-in-with-a-session-cookie.md)
- [Page through results](page-through-results.md)
- [Load more as the reader asks](load-more.md)
- [Update at once, and roll back on failure](optimistic-updates.md)
- [Show what the network is doing](show-the-network.md)
- [Send a message as the page unloads](send-on-unload.md)

## Real time

Sockets, streams, other tabs, other readers and workers.

- [Open a socket](open-a-socket.md)
- [Listen to server-sent events](server-sent-events.md)
- [Talk to the site's other tabs](talk-between-tabs.md)
- [Connect two readers' pages](connect-two-readers.md)
- [Run a script in a worker](run-a-worker.md)

## Content

Markdown, pages written as Markdown, code on the page, a page for unknown addresses, a PDF, and what search engines and link previews read.

- [Write a page as a Markdown file](markdown-pages.md)
- [Render Markdown](render-markdown.md)
- [Show code on a page](show-code.md)
- [Describe pages to search engines and link previews](search-and-link-previews.md)
- [Show your own page for an unknown address](not-found-page.md)
- [Make a PDF report](pdf-report.md)

## Media and themes

Pictures, video, icons, a carousel, other pages in a frame, and a dark theme.

- [Add a picture, sized for every screen](add-a-picture.md)
- [Add video and audio](video-and-audio.md)
- [Show an icon](show-an-icon.md)
- [Show a carousel](show-a-carousel.md)
- [Show another page in a frame](show-a-page-in-a-frame.md)
- [Add a dark-mode switch](dark-mode-switch.md)

## Motion

Animation, from one element appearing to a page carried into the next. Read them in order: each builds on the one before.

- [Animate an element in and out](animate-in-and-out.md)
- [Animate a list](animate-a-list.md)
- [Run a sequence of animations](animation-sequence.md)
- [Animate route changes](route-transitions.md)
- [Carry an element across a route change](shared-element.md)
- [Open a box to the height of its content](expand-to-content.md)
- [Count a number to its new value](count-a-number.md)
- [Tween a property when state changes](tween-on-change.md)
- [Declare your own keyframes](keyframes.md)
- [Drive an animation yourself](drive-an-animation.md)
- [Set motion defaults, and respect reduced motion](motion-defaults.md)

## Reach

More readers: other languages, and no network.

- [Translate a site](translate-a-site.md)
- [Translate by the words as written](translate-by-the-words.md)
- [Publish a page in each language](site-in-two-languages.md)
- [Make a site work offline](work-offline.md)
- [Keep writes made offline](keep-writes-made-offline.md)
- [Offer a new version](offer-a-new-version.md)

## Testing

Checking that a page does what it should before a reader finds out it does not.

- [Test a page](test-a-page.md)

## Ship

Putting a site on a host, in the order you need them: install `wf` where the site is built, set what changes per environment, deploy to a host or serve from your own server, then check the deploy.

- [Install wf on a build machine](install-wf-in-ci.md)
- [Use environment values, and keep secrets out](environments-and-secrets.md)
- [Deploy to GitHub Pages](deploy-to-github-pages.md)
- [Deploy to Netlify](deploy-to-netlify.md)
- [Deploy to Vercel](deploy-to-vercel.md)
- [Deploy to Cloudflare Pages](deploy-to-cloudflare-pages.md)
- [Build a Docker image](docker-image.md)
- [Serve a site with nginx](serve-with-nginx.md)
- [Serve a site with Apache](serve-with-apache.md)
- [Serve a site with Caddy](serve-with-caddy.md)
- [Serve an API on the same origin](api-on-the-same-origin.md)
- [Set cache and security headers](cache-and-security-headers.md)
- [Check a deploy](check-a-deploy.md)

## Desktop and phone apps

The same project in a window of its own, with Tauri or Electron.

- [Make a desktop or phone app with Tauri](tauri-app.md)
- [Make a desktop app with Electron](electron-app.md)

## Interop

Your own JavaScript, libraries, and custom elements, both ways.

- [Call your own JavaScript](call-your-own-script.md)
- [Call WF from a script](call-wf-from-a-script.md)
- [Hand an element to a script](hand-an-element-to-a-script.md)
- [Run a script after each page draws](listen-for-wf-render.md)
- [Draw into a node a script made](portal.md)
- [Load a library from a CDN](library-from-a-cdn.md)
- [Draw a chart from a library](chart-from-a-library.md)
- [Place a custom element from a library](place-a-custom-element.md)
- [Publish components as custom elements](publish-custom-elements.md)
- [Migrate from external](migrate-from-external.md)

## Server rendering

WebFluent as a template engine: HTML and PDF from data, from the command line, Node or Rust.

- [Render a template with data](render-a-template.md)
- [Render templates from Node](render-from-node.md)
- [Render templates from Rust](render-from-rust.md)

## Projects

Two complete projects, explained step by step. They are read along in the finished code under `examples/tutorials/`, not typed along; each says how to build it yourself if you want to.

- [Build a statically generated notebook](build-a-notebook-site.md)
- [Build a budget app that works offline](build-a-budget-app.md)
