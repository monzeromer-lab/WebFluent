# 44. Troubleshooting

<!--
route: guide/troubleshooting
group: help
blurb: The problems people meet, what causes each, and the fix — from installing to deploying, plus the questions that come up most.
description: Install problems, build errors, blank pages after deploying, 404s on reload, CSP blocks, styles that do not apply, and a FAQ.
-->

Every diagnostic has its own entry in [Diagnostics](39-diagnostics.md);
search it for the code in brackets. This chapter covers what goes wrong
without one.

## Installing

**`wf: command not found` after installing.** Open a new terminal: the
installer added `~/.webfluent/bin` to your shell's startup file, which the
open session read before. Or run it by its full path, `~/.webfluent/bin/wf`.

**The installer says it cannot find the latest version.** GitHub's API
limits unauthenticated requests. Pin one: `WF_VERSION=v4.0.1`.

**Linux on ARM (a Raspberry Pi, an ARM server).** There is no prebuilt
binary yet: `cargo install webfluent`.

**`wf-lsp` not found by the editor.** It is a separate binary; see
[Getting started](02-getting-started.md#set-up-your-editor).

## Building

**"is not an element or a statement a render block can hold".** Code that
does something — `n = n + 1`, a call that is not a store's set-up — was
written loose in a page. Put it in `on click { }`, an `action` or an
`effect`.

**"Only the first argument may be positional".** An element takes one
unnamed value; name the rest: `Card("Laptop", price: 999)`.

**"a WebFluent 2 declaration".** The file uses the old grammar (`Page`,
`Component`). Run `wf migrate`.

**A warning I do not understand.** Every code is in [Diagnostics](39-diagnostics.md)
with an example and a fix. A named argument written to the element "as an
attribute" is almost always a misspelled prop.

**The build is slow.** The first build resizes every `image`; later builds
reuse `.wf-cache/`. Do not delete it in CI if you can cache it.

## In the browser

**The page is blank after deploying, but works locally.** The site is served
under a sub-path and the build does not know it. Set `build.base_path` to
the path (`"/my-repo"` on GitHub Pages) and rebuild.

**Reloading any page but the home page is a 404.** A single-page build needs
the host to answer every path with `index.html`
([Deploying](29-deploying.md)), or build static with `build.ssg: true`.

**"Refused to load the script" / "violates the following Content Security
Policy".** `build.csp` ships a strict policy. A script from another origin
has to be declared: load a module through `external`, fonts and stylesheets
through `meta.fonts` and `meta.stylesheets` — each widens the policy for
exactly that origin. An inline `<script>` in `public/` HTML is never
allowed; move it to a file.

**An icon shows as a word.** Its name is not one of the 32 built-in icons
([Media](28-media.md#icons)), or it is built at run time, which the build
cannot see — set `"build": { "runtime": "full" }` or name it literally.

**`WF.something is not a function` from my own script.** The build left that
runtime module out because nothing in your `.wf` code uses it. Set
`"build": { "runtime": "full" }`.

**The page shows old content after a deploy.** Either an HTTP cache held
`app.js` (serve it with `Cache-Control: no-cache`), or — on a site with
`offline` — the service worker is serving its store until the new version
is taken. Show an update button (`update.available`), or reload twice.

**My changes do not show under `wf serve`.** Check the terminal: a failed
build keeps the last good page and draws the error over it. An old service
worker from a production build on the same port can also hold pages;
`wf serve` removes it on the next load.

**A layout that is right after the page loads jumps first.** Something depends
on `viewport`, which is unknown until the script runs. Use
[responsive values](15-styling.md#responsive-values) for layout.

## Styling

**My `style { }` does not apply.** Check the property name (a style block is
real CSS) and the token (`$brand-colour` that no theme defines resolves to
nothing). Values end at a newline, a `;`, or two spaces before the next
declaration.

**My stylesheet does not override a built-in.** Your `.css` files come after
the built-ins' rules, so on equal specificity they win — but `.wf-btn--primary`
beats `.button`. Match the built-in's selector, or change the tokens it
reads, or use `style { }` on the element.

**Dark mode does not switch.** Name the dark theme in the config:
`"theme": { "name": "Light", "dark": "Night" }`. A colour written raw instead
of a token does not change with the theme.

## Data

**A request fails with a CORS error.** The API is on another origin and does
not allow this one. Allow it on the server, or serve the API from the same
origin ([Data](17-data.md#an-api-on-another-origin)).

**`env.MY_VALUE` is a compile error.** A page may only read public names:
rename it `PUBLIC_MY_VALUE` or list it in `public_env` — only if anyone may
see it ([Environments](30-environments.md)).

**A value from `.env` or the shell is not used.** Values are fixed at build
time: rebuild after changing them. The shell supplies only names the config
or `.env` declares, or `PUBLIC_` ones.

## Questions people ask

**Can I use npm packages?** A JavaScript module, yes, through `external`
from a CDN or your import map ([JavaScript interop](32-javascript-interop.md)).
There is no package manager for WebFluent code itself.

**Can I use it with React or Vue?** Publish components as custom elements
and place the tags ([chapter 32](32-javascript-interop.md#publishing-your-components-as-custom-elements)).

**Is there server-side rendering?** Pages are pre-rendered at build time
(`build.ssg`). Per-request HTML is [server rendering](34-server-rendering.md)
of templates, from your own server.

**Does it support TypeScript?** It has its own type checker, gradual and
built in ([Types](13-types.md)). There is nothing to configure.

**Can I write CSS the normal way?** Yes: any `.css` file under `src/` is
bundled, and `class:` adds its classes to an element.

**How big is it?** A page of static text ships about 4 kB of script,
gzipped; a full application a few tens of kilobytes. `wf build --stats`
shows yours.

**Which browsers?** The current versions of Chrome, Edge, Firefox and Safari
([Browser support](46-browser-support.md)).

**Where do I report a bug?** [GitHub issues](https://github.com/monzeromer-lab/WebFluent/issues),
with the smallest `.wf` file that shows it and the output of `wf --version`.

## Next

[Upgrading](45-upgrading.md).
