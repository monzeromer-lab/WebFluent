# 31. Performance

<!--
route: guide/performance
group: quality
blurb: Learn what a build already does to be fast, see what it weighs, set budgets that warn, time pages in a browser, and avoid what makes a site slow.
description: What a build does to be fast, wf build --stats, size budgets, paint timings from wf verify, and the few things that make a site slow.
-->

This chapter shows you how to keep a WebFluent site fast. Most of the work
is already done by the build. Your part is to measure what it writes, set a
limit that warns you when it grows, and avoid the few patterns that make a
page slow.

In this chapter:

- What the build already does for you.
- See what a build weighs, and why each runtime module is in it.
- Set a size budget, and time each page in a real browser.
- Avoid the patterns that make a site slow.

## What the build already does

You get these without setting anything:

- **Only the runtime you use.** The *runtime* is the JavaScript the
  compiled pages call. It is a set of modules, and a build keeps only the
  ones its code reaches. A single-page build of one page of static text
  carries 2 of the 41 modules: an `app.js` of 18.9 kB, 6.5 kB gzipped.
- **A chunk per page.** `build.split` (on by default) writes each page's
  code and styles as their own files, loaded the first time its route is
  shown.
- **Pre-rendered HTML.** With `build.ssg`, each page's HTML is written at
  build time, so the first paint needs no script.
- **One pruned stylesheet.** It holds the rules of only the built-ins you
  use, and only the design tokens something names.
- **Precompressed files.** A `.gz` file sits beside every text file, for a
  host that serves it. `wf serve` sends the same bytes, so Lighthouse run
  against the dev server measures what a deploy sends.
- **Images at every width**, with `width` and `height` so nothing shifts, a
  placeholder while they load, the first image eager and the rest lazy
  ([Media](21-media.md)).
- **Minified output.** `build.minify` (on by default) strips comments and
  white space.

## See what a build weighs

To see the size of everything a build writes, run the build with
`--stats`:

```bash
wf build --stats
```

This is the output for the [tutorial](03-tutorial.md)'s site, trimmed:

```text
  What it weighs
    app.js                               70.6 kB  (21.7 kB gzipped)
    styles.css                           21.7 kB  (4.5 kB gzipped)
    index.html                            5.2 kB  (1.3 kB gzipped)
    suggest/index.html                    2.8 kB  (0.9 kB gzipped)
    pages/Suggest.js                      2.2 kB  (0.9 kB gzipped)
    pages/Home.js                         1.4 kB  (0.7 kB gzipped)
    …
    total                               118.7 kB

  Runtime: 15 of 41 modules, 142.6 kB of 377.6 kB (before minifying)
    core          38.1 kB  always
    motion        19.5 kB  needed by when, match, each, router
    router        15.4 kB  WF.router
    form          13.4 kB  WF.form
    hydrate       12.0 kB  WF.hydrating
    each          10.3 kB  WF.each
    store          9.0 kB  WF.store
    keep           7.3 kB  needed by store
    field          5.2 kB  WF.field
    …
    left out: helpers format offline peer scalars text slot show browser …

  Tokens: 26 nothing in the output names, left out
```

Read it in three parts:

- **What it weighs** lists every text file with its gzipped size. The
  tutorial's form brings in `form` and `field`, its store brings in `store`
  and `keep`, and its second page brings in `router`.
- **Runtime** lists each module with what reached it. When a module you did
  not expect is in the build, look here first.
- **Tokens** counts the design tokens the build left out because nothing
  names them.

Each build records its sizes in `.wf-sizes.json` beside the project, so the
next `--stats` shows how each file moved since the last build (for example,
`+20 B`).

The project's own scripts are listed by their path under `js/`, such as
`js/tilt.js` ([JavaScript interop](39-javascript-interop.md)). The build
copies them as written: it does not minify a file it promises to ship byte
for byte.

## Set a size budget

To be warned when a file grows past a size, name its gzipped limit in
`build.budget`:

```json
{ "build": { "budget": { "app.js": "40 kB", "styles.css": "30 kB", "pages/Home.js": "10 kB" } } }
```

Any output may have a budget, including a page's chunk and one of your own
scripts under `js/`. A build over budget prints a warning and goes on. A CI
job that should fail on it can read the same numbers.

## Time pages in a browser

To measure how fast each page paints, build the site and run `wf verify`
with a budget in milliseconds:

```bash
wf build && wf verify --budget 800
```

`wf verify` loads every route in headless Chrome. For each route it prints
the first contentful paint, the number of nodes in the document, and the
bytes and requests it took. `--budget 800` fails a route whose first paint
is slower than 800 ms. Lighthouse and the browser's performance panel work
on the build as on any other site.

## Avoid what makes a site slow

These are the patterns that slow a WebFluent site down, and what to do
instead:

| Pattern | Why it is slow | Do this instead |
|---|---|---|
| A large `data` file | `data posts = "posts.json"` is inlined into the bundle, so a 2 MB file is 2 MB of JavaScript | Keep build-time data to what pages show. Fetch the rest, or split it across pages that each read one file |
| A waterfall of fetches | A resource whose URL depends on another's answer waits for that answer | Ask for both at once, or have the server send them together |
| Layout decided by `viewport` | `viewport` is unknown until the script runs, so the page paints one branch and then swaps it | Use [responsive values](19-styling.md#responsive-values), which are CSS |
| `"runtime": "full"` | It ships every module | Use it only when a hand-written script calls `WF` in ways the build cannot see, or an icon's name arrives at run time |
| Web fonts | Each weight is a download | Use `display=swap` and few weights, or the baseline's system fonts |
| An unkeyed list that changes | `for item in items` rebuilds the list | Key it: `for item in items by item.id` moves the nodes that stayed |
| Pictures the build cannot process | An SVG, a URL or a path built at run time is used as written: one size, no placeholder | Use a literal `src:` into `public/`, or declare the picture with `image` ([Media](21-media.md)) |

## Next

[Project structure](32-project-structure.md): where files go, how the compiler reads them, how names work across a project, and how a project grows without getting lost.
