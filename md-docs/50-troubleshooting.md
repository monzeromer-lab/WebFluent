# 50. Troubleshooting

<!--
route: guide/troubleshooting
group: help
blurb: The problems people meet, what causes each, and the fix — from installing to deploying, plus the questions that come up most.
description: What you see when something goes wrong — install errors, build stops, blank pages after deploy, 404s on reload, CSP blocks, styles — the cause and the fix.
-->

This chapter lists the problems people meet with WebFluent, titled by what
you see. Each entry gives the cause, then the fix.

A problem that comes with a code in brackets, such as `[T05]`, has its own
entry in [Diagnostics](43-diagnostics.md): search that chapter for the
code, or run `wf explain T05` to print the entry in your terminal. This
chapter covers what goes wrong without one, and the messages that need
more context than the entry gives.

In this chapter:

- [Installing](#installing)
- [Building](#building)
- [In the browser](#in-the-browser)
- [Styling](#styling)
- [Data](#data)
- [Questions people ask](#questions-people-ask)

## Installing

### `wf: command not found` after installing

The installer added `~/.webfluent/bin` to your shell's startup file, but
the terminal you have open read that file before the line was there.

Open a new terminal. Or run `wf` by its full path, `~/.webfluent/bin/wf`.

### The installer cannot find the latest version

The installer asks GitHub's API for the latest release, and GitHub limits
requests that are not signed in.

Pin a version instead, and run the installer again with it set:
`WF_VERSION=v5.12.0`.

### There is no binary for Linux on ARM

There is no prebuilt binary yet for Linux on ARM, such as a Raspberry Pi
or an ARM server, so the installer stops.

Install from source with Rust:

```bash
cargo install webfluent
```

### On Windows, `wf` opens the Windows Firewall console

Windows already has a `wf`: `C:\Windows\System32\WF.msc`, the firewall
console, which a bare `wf` finds first whatever your `PATH` says. The
installer puts the same binary in place a second time, as `webfluent.exe`.

Type `webfluent` (or `wf.exe`) wherever this guide says `wf`.

### On Windows, an old version still runs after installing a new one

Each `.msi` before 5.10 had an `UpgradeCode` of its own, so a newer
installer cannot replace an older one, and both stay installed and on
`PATH`.

Uninstall the old version from *Apps*, then check `webfluent --version`.

### The editor cannot find `wf-lsp`

The language server is a separate binary from `wf`, and the editor looks
for it on its own.

Install it as [Getting started](02-getting-started.md#set-up-your-editor)
describes.

### The editor reports what `wf build` does not

The editor calls an element unknown (`Host`, for example), or shows a
message that `wf check` never prints. The editor is running an older
`wf-lsp` than the `wf` you build with: one on your `PATH` is used before a
downloaded release, and `cargo install` does not update it when you update
`wf`.

1. Compare `wf-lsp --version` with `wf --version`. The editor's language
   server log also names the version it started.
2. Install the `wf-lsp` that matches `wf`.
3. Restart the language server.

## Building

### "is not an element or a statement a render block can hold"

Code that does something — `n = n + 1`, or a call that is not a store's
set-up — is written loose in a page. A page's body describes what to show;
it does not run statements.

Put the code in an `on click { }` handler, an `action` or an `effect`.

### "Only the first argument may be positional"

An element takes one value without a name, and every other value must be
named.

Name the rest: `Card("Laptop", price: 999)`.

### "a WebFluent 2 declaration"

The file uses the old grammar, with capitalised declarations such as
`Page` and `Component`.

Run `wf migrate` to rewrite it ([Upgrading](51-upgrading.md#from-webfluent-2)).

### A warning you do not understand

Every warning has a code, and the code's entry explains it.

Run `wf explain CODE` to print the entry; every code is also in
[Diagnostics](43-diagnostics.md), with an example and a fix. A named
argument that the compiler writes to the element "as an attribute"
(`C04`) is almost always a misspelled prop.

### A project that built under 4.x stops on errors

5.0 refuses what used to compile and then fail in the browser, so a 4.x
project may stop on errors it always had.

1. Run `wf check`. It lists every finding and writes nothing.
2. Apply the fixes your editor offers as quick fixes, where the fix is
   known.
3. While you catch up, lower in `lints` the codes that cannot ship a
   broken page ([Upgrading](51-upgrading.md#5-0)).

### A finding you mean to keep

The compiler reports something you have decided is right for this line.

Put `// wf-allow(CODE)` on the line above it, or at the end of its line,
to accept that one finding. To silence a warning across a whole project,
set it in `lints` instead. An allow that covers nothing is itself reported
(`U07`).

### CI does not fail on warnings, or does not annotate pull requests

By default, warnings do not fail a build, and the output is written for a
terminal.

- `wf check --deny-warnings` fails on a warning.
- `--format github` writes annotations a GitHub workflow shows on the pull
  request.
- `--format sarif` writes a file for code scanning.

[Command line](41-cli.md#wf-check) lists every option.

### The build is slow

The first build resizes every `image` the project declares. Later builds
reuse the work kept in `.wf-cache/`.

Keep `.wf-cache/` between builds. In CI, cache it rather than deleting it.

## In the browser

### The page is blank after deploying, but works locally

The site is served under a sub-path, such as `/my-repo/` on GitHub Pages,
and the build does not know it, so the page asks for its files at the
wrong address.

Set `build.base_path` to the sub-path (`"/my-repo"`) and rebuild
([Static sites and SPAs](33-static-and-spa.md#serve-the-site-under-a-sub-path)).

### Reloading any page but the home page is a 404

A single-page build has one HTML file. When the reader reloads
`/settings`, the host looks for a file at that path and has none.

Configure the host to answer every path with `index.html`
([Deploying](34-deploying.md)), or build a static site with
`build.ssg: true`, which writes a file for every page.

### "Refused to load the script" or "violates the following Content Security Policy"

`build.csp` ships a strict policy, and the browser blocks a script,
font or stylesheet from an origin the policy does not name.

Declare what the page loads from elsewhere: a library in `meta.scripts`,
fonts in `meta.fonts`, stylesheets in `meta.stylesheets`. Each widens the
policy for exactly that origin. An inline `<script>` in an HTML file in
`public/` is never allowed; move its code to a `.js` file.

### An icon shows as a word

The name is not one of the 72 built-in icons
([Media](21-media.md#show-an-icon)), or the name is put together at run
time, which the build cannot see, so the build left its glyph out.

Fix the name, or write it as a literal. If it must be built at run time,
set `"build": { "runtime": "full" }`, which ships every icon.

### `WF.something is not a function` from your own script

The build ships only the runtime modules your `.wf` code uses. Your script
calls one that nothing in the `.wf` code reaches, so the build left it out.

Set `"build": { "runtime": "full" }` to ship the whole runtime.

### The page shows old content after a deploy

Either an HTTP cache is holding `app.js`, or — on a site with `offline` —
the service worker is serving what it stored until the reader takes the
new version.

- Serve `app.js` with `Cache-Control: no-cache`
  ([Deploying](34-deploying.md#set-cache-headers)).
- On an offline site, show an update button when `update.available` holds
  ([Offline](26-offline.md#offer-a-new-version)), or reload twice.

### Changes do not show under `wf serve`

Either the last build failed, or an old service worker is holding pages.

- Look at the terminal. A failed build keeps the last good page and draws
  the error over it until a build passes.
- A service worker from a production build served on the same port can
  hold pages. `wf serve` removes it on the next load.

### A request is a 404 under `wf serve` or `wf verify`

Only a page navigation gets the single-page shell. A `fetch`, or a file the
build did not write, gets a `404` and a line in the terminal. The dev
server does not serve your API.

Point the page at the server that has the API — an `api`'s `base:` read
from `env` — or put a fixture file in `public/`.

### "The response of … is not the type the program declares"

Under `wf serve`, a response is held to the type its `resource` or `api`
endpoint declares. One that differs takes the `error` arm, as a `.parse`
error that names the first place it differs, such as `$[1].name is
missing`.

Fix the declared type, or the server.

### "Item N could not be drawn"

One item of a `for` threw while it was drawn — usually a field read on a
value that is `null`. Under `wf serve` the item shows the message; on a
deployed page it leaves a gap, and the browser console says why. The rest
of the list draws either way.

Read the field through `?.`, or wrap the item's body in `if let`, so a
`null` value draws nothing instead of throwing.

### "404 — no page has the route …"

No page's `path` matches the address, and no `*` page catches what nothing
else does. A deployed page shows nothing there.

Declare a catch-all page, `page NotFound(path: "*", …)`
([Static sites and SPAs](33-static-and-spa.md#add-a-404-page)).

### "an effect changes something it reads, every time it runs"

The effect writes, every time it runs, a state it also reads, so it never
settles. The runtime stopped it after 100 runs.

Guard the write (`if x != wanted { x = wanted }`), or move it to an
action. `X02` reports the plain cases at build time.

### An `api` call stays loading

A parameter that the endpoint's path names is empty — `Backend.user(id:
"")` for `users/:id` — so nothing was requested: `users/` would ask for the
whole collection.

Nothing to fix if the value is still on its way: the call is sent when the
value arrives. Otherwise, pass the value.

### The layout jumps as the page loads

Something in the layout depends on `viewport`, which is unknown until the
script runs, so the first paint is drawn for the wrong width.

Use [responsive values](19-styling.md#responsive-values) for layout. They
compile to media queries, so the first paint is already right.

## Styling

### A `style { }` block does not apply

Either a property name is misspelt, or a token does not exist.

- A style block is real CSS. A misspelt property is a `V05` warning, with
  the nearest real one.
- A token no theme declares, such as `$brand-colour`, is a `V06` error.
- A value ends at a newline, a `;`, or two spaces before the next
  declaration. A `,` between two declarations is an `E002` error.

### Your stylesheet does not override a built-in

Your `.css` files come after the built-ins' rules, so on equal specificity
yours win. But a built-in's selector may be more specific:
`.wf-btn--primary` beats `.button`.

Match the built-in's selector, change the design tokens it reads, or use
`style { }` on the element.

### Dark mode does not switch

Either the dark theme is not named in the config, or a colour is written
raw instead of as a token.

Name the dark theme: `"theme": { "name": "Light", "dark": "Night" }`.
Write colours as tokens (`$surface`, `var(--color-surface)`): a raw colour
does not change with the theme.

## Data

### A request fails with a CORS error

The API is on another origin, and that server does not allow requests from
yours.

Allow your origin on the server, or serve the API from the same origin
([Fetching data](24-data.md#call-an-api-on-another-origin),
[Deploying](34-deploying.md#serve-an-api-on-the-same-origin)).

### `env.MY_VALUE` is a compile error

A page may only read public names, because every value it reads ends up in
the bundle, where anyone can read it (`E108`).

If anyone may see the value, rename it `PUBLIC_MY_VALUE` or list it in
`public_env` ([Environments](35-environments.md)). If not, keep it on the
server.

### A value from `.env` or the shell is not used

Values are fixed at build time. The shell also supplies only names that the
config or `.env` declares, or that begin `PUBLIC_`.

Rebuild after changing a value, and declare the name in the config or
`.env`.

## Questions people ask

### Can I use npm packages?

A library, yes: list its CDN address in `meta.scripts`
([JavaScript interop](39-javascript-interop.md)). Your own JavaScript is a
plain `.js` file under `src/`. There is no package manager for WebFluent
code itself.

### Can I use it with React or Vue?

Yes. Publish your components as custom elements, and place their tags from
React, Vue or any other page
([JavaScript interop](39-javascript-interop.md#publish-your-components-as-custom-elements)).

### Is there server-side rendering?

Pages are pre-rendered at build time with `build.ssg`. For HTML made per
request, render templates from your own server with the
[template engine](38-server-rendering.md).

### Does it support TypeScript?

WebFluent has its own type checker, gradual and built in
([Types](17-types.md)). There is nothing to configure.

### Can I write CSS the usual way?

Yes. Any `.css` file under `src/` is bundled, and `class:` adds its classes
to an element ([Styling](19-styling.md)).

### How big is the output?

A page of static text ships about 4 kB of script, gzipped; a full
application, a few tens of kilobytes. `wf build --stats` shows what yours
weighs.

### Which browsers does it support?

The current versions of Chrome, Edge, Firefox and Safari
([Browser support](48-browser-support.md)).

### Where do I report a bug?

Open an issue on [GitHub](https://github.com/monzeromer-lab/WebFluent/issues).
Include the smallest `.wf` file that shows the bug, and the output of
`wf --version`.

## Next

[Upgrading](51-upgrading.md).
