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
limits unauthenticated requests. Pin one: `WF_VERSION=v5.1.0`.

**Linux on ARM (a Raspberry Pi, an ARM server).** There is no prebuilt
binary yet: `cargo install webfluent`.

**`wf-lsp` not found by the editor.** It is a separate binary; see
[Getting started](02-getting-started.md#set-up-your-editor).

**The editor reports what `wf build` does not** — an element it calls
unknown (`Host`), a message `wf check` never prints. The editor is running
an older `wf-lsp` than the `wf` you build with: one on your `PATH` is used
before a downloaded release, and `cargo install` does not update it when
`wf` is updated. Compare `wf-lsp --version` with `wf --version` (the
editor's language server log names the version it started, too), install
the matching one, and restart the language server.

## Building

**"is not an element or a statement a render block can hold".** Code that
does something — `n = n + 1`, a call that is not a store's set-up — was
written loose in a page. Put it in `on click { }`, an `action` or an
`effect`.

**"Only the first argument may be positional".** An element takes one
unnamed value; name the rest: `Card("Laptop", price: 999)`.

**"a WebFluent 2 declaration".** The file uses the old grammar (`Page`,
`Component`). Run `wf migrate`.

**A warning I do not understand.** `wf explain CODE` prints its entry;
every code is in [Diagnostics](39-diagnostics.md) with an example and a
fix. A named argument written to the element "as an
attribute" is almost always a misspelled prop.

**A project that built under 4.x stops on errors.** 5.0 refuses what used
to compile and then fail in the browser. `wf check` lists every finding
without building; your editor's quick fixes apply the ones with a known
fix; `lints` lowers what cannot ship a broken page while you catch up
([Upgrading](45-upgrading.md#5-0)).

**A finding I mean to keep.** `// wf-allow(CODE)` on the line above it (or
at the end of its line) accepts that one; a warning a whole project does
not want goes in `lints`. An allow that covers nothing is itself reported
(`U07`).

**Failing CI on warnings, or annotating a pull request.** `wf check
--deny-warnings` fails on a warning; `--format github` writes annotations
a workflow shows on the pull request, and `--format sarif` a file for code
scanning ([Command line](37-cli.md#wf-check)).

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
has to be declared: a library in `meta.scripts`, fonts and stylesheets in
`meta.fonts` and `meta.stylesheets` — each widens the policy for exactly
that origin. An inline `<script>` in `public/` HTML is never
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

**A request is a 404 under `wf serve` (or `wf verify`).** Only a page
navigation gets the single-page shell; a `fetch` or a file the build did
not write gets a 404 and a line in the terminal. The API is not served by
the dev server: point the page at the server that has it (an `api`'s
`base:` from `env`), or put a fixture file in `public/`.

**"The response of … is not the type the program declares".** Under `wf
serve`, a response is held to the type its `resource` or `api` endpoint
declares, and one that differs takes the `error` arm as a `.parse` error
naming the first place it differs — `$[1].name is missing`. Fix the type,
or the server.

**"Item N could not be drawn".** One item of a `for` threw while it was
drawn — usually a field read on a value that is `null`. Under `wf serve`
the item shows the message; on a deployed page it leaves a gap and the
console says why. The rest of the list draws either way.

**"404 — no page has the route …".** No page's `path` matches the address
and no `page NotFound(path: "*")` catches what nothing else does. A
deployed page shows nothing there; declare a `*` page.

**"an effect changes something it reads, every time it runs".** The effect
writes a state it also reads, unconditionally, so it never settles; it was
stopped after 100 runs. Guard the write (`if x != wanted { x = wanted }`)
or move it to an action. `X02` reports the plain cases at build time.

**An `api` call stays loading.** A parameter its path names is empty —
`Backend.user(id: "")` for `users/:id` — so nothing was requested: `users/`
would be the whole collection. It loads when the value arrives.

**A layout that is right after the page loads jumps first.** Something depends
on `viewport`, which is unknown until the script runs. Use
[responsive values](15-styling.md#responsive-values) for layout.

## Styling

**My `style { }` does not apply.** Check the property name (a style block is
real CSS; a misspelt one is a `V05` warning) and the token (`$brand-colour`
that no theme defines is a `V06` error). Values end at a newline, a `;`, or
two spaces before the next declaration.

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

**Can I use npm packages?** A library, yes, by its CDN URL in `meta.scripts`
([JavaScript interop](32-javascript-interop.md)); your own JavaScript is a
plain `.js` file under `src/`.
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
