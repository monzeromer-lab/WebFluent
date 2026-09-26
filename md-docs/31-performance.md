# 31. Performance

<!--
route: guide/performance
group: shipping
blurb: What a build already does to be fast, how to see what it weighs, budgets that warn, and the few things that make a site slow.
description: What a build does to be fast, wf build --stats, size budgets, paint timings from wf verify, and the few things that make a site slow.
-->

## What the build does already

- **Only the runtime you use.** The runtime is a set of modules; a build
  keeps the ones its code reaches. A page of static text carries 3 of its 37
  modules: an `app.js` of 11 kB, 3.7 kB gzipped.
- **A chunk per page.** `build.split` (on by default) writes each page's code
  and styles as their own files, loaded when its route is first shown.
- **Pre-rendered HTML** with `build.ssg`: the first paint needs no script.
- **One pruned stylesheet.** Only the built-ins you use, only the tokens
  something names.
- **Precompressed.** A `.gz` beside every text file, for a host that serves
  it; `wf serve` sends the same bytes, so Lighthouse against the dev server
  measures what a deploy sends.
- **Images at every width**, with `width`/`height` so nothing shifts, a
  placeholder while they load, the first one eager and the rest lazy
  ([Media](28-media.md)).
- **Minified** output (`build.minify`).

## Seeing what it weighs

```bash
wf build --stats
```

```text
  What it weighs
    app.js                               47.6 kB  (14.6 kB gzipped)  +20 B
    styles.css                           16.9 kB  (3.7 kB gzipped)
    index.html                            5.1 kB  (1.3 kB gzipped)
    pages/Suggest.js                      2.2 kB  (0.8 kB gzipped)
    pages/Home.js                         1.4 kB  (0.6 kB gzipped)
    …
    total                                90.7 kB

  Runtime: 14 of 37 modules, 89.3 kB of 220.3 kB (before minifying)
    core          19.6 kB  always
    motion        16.6 kB  needed by when, match, each, router
    form          12.1 kB  WF.form
    router        10.7 kB  WF.router
    store          6.5 kB  WF.store
    …
    left out: helpers format offline peer scalars text slot show browser …

  Tokens: 26 nothing in the output names, left out
```

That is the [tutorial](03-tutorial.md)'s site: a form brings `form` and
`field`, a store `store` and `keep`, a second page `router`.

Each build records its sizes in `.wf-sizes.json`, so the next `--stats`
shows what moved. Each runtime module is listed with what reached it — the
first place to look when a module you did not expect is in.

## Budgets

```json
{ "build": { "budget": { "app.js": "40 kB", "styles.css": "30 kB", "pages/Home.js": "10 kB" } } }
```

A gzipped size an output should stay under. Going over prints a warning and
the build goes on; a CI job that wants to fail on it can read the same
numbers.

## Timing it in a browser

```bash
wf build && wf verify --budget 800
```

`wf verify` loads every route in headless Chrome and prints its first
contentful paint, the nodes in the document and the bytes and requests it
took; `--budget` fails a route slower than that. Lighthouse and the
browser's performance panel work on the build as on any site.

## What makes a site slow

- **A large `data` file.** `data posts = "posts.json"` is inlined into the
  bundle, so a 2 MB file is 2 MB of JavaScript. Keep build-time data to what
  pages show; fetch the rest, or split it across pages that each read one.
- **A waterfall of fetches.** A resource whose URL depends on another's
  answer waits for it. Where you can, ask for both at once, or have the
  server send them together.
- **Layout decided by `viewport`.** It is unknown until the script runs, so
  the page paints one branch and swaps. Use responsive values, which are CSS.
- **`runtime: "full"`.** It ships every module. Only needed when a
  hand-written script calls `WF` in ways the build cannot see.
- **Web fonts.** Each weight is a download. Use `display=swap`, few weights,
  or the baseline's system fonts.
- **Unkeyed lists that change.** `for item in items` rebuilds the list;
  `for item in items by item.id` moves the nodes that stayed.
- **Pictures from `public/`.** They are copied as they are. Declare them with
  `image` to get every width and a placeholder.

## Next

[JavaScript interop](32-javascript-interop.md).
