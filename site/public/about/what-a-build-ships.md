# What a build ships

> What a build does to be fast, wf build --stats, size budgets, paint timings from wf verify, and the few things that make a site slow.

The page: https://webfluent.monzeromer.dev/about/what-a-build-ships

This essay is about what `wf build` writes, and why a WebFluent site is
fast by default rather than by effort.

Most slow sites are not slow because someone chose to make them slow. They
are slow because every library, component and font arrived for a good
reason, and nothing ever weighed the total. A framework cannot weigh it
for you: it does not know which of its features your program uses, so it
ships them all and leaves the pruning to a bundler that reads JavaScript
it did not write.

A compiler that reads the whole program can do better. It knows which
runtime modules the code it generated calls, which built-ins' rules the
pages use, and which design tokens something names, so a build ships those
and nothing else. Your part is smaller: look at what it writes, set a
limit that warns you when it grows, and avoid the few patterns no compiler
can fix.

## What the build already does

You get these without setting anything:

- **Only the runtime you use.** The *runtime* is the JavaScript the
  compiled pages call. It is a set of modules, and a build keeps only the
  ones its code reaches, and of those only the functions its calls reach.
  A single-page build of one page of static text is an `app.js` of
  about 5 kB, 1.9 kB gzipped.
- **Short names.** A minified build shortens the runtime's own names, its
  top-level ones and each function's locals. What your pages call keeps
  its name.
- **A chunk per page.** `build.split` (on by default) writes each page's
  code and styles as their own files, loaded the first time its route is
  shown. A site of one page is not split: its code is all in
  `app.js`.
- **Pre-rendered HTML.** With `build.ssg`, each page's HTML is written at
  build time, so the first paint needs no script.
- **One pruned stylesheet.** It holds the rules of only the built-ins you
  use, and only the design tokens something names.
- **Precompressed files.** A `.gz` file sits beside every text file, for a
  host that serves it. `wf serve` sends the same bytes, so Lighthouse run
  against the dev server measures what a deploy sends.
- **Images at every width**, with `width` and `height` so nothing shifts, a
  placeholder while they load, the first image eager and the rest lazy
  ([Add a picture](https://webfluent.monzeromer.dev/recipes/add-a-picture.md)).
- **Minified output.** `build.minify` (on by default) strips comments and
  white space.

## See what a build weighs

To see the size of everything a build writes, run the build with
`--stats`:

```bash
wf build --stats
```

This is the output for the project `wf init -t spa` makes, trimmed (the
exact numbers move a little from release to release):

```text
  What it weighs
    app.js                               49.4 kB  (16.7 kB gzipped)
    styles.css                           26.5 kB  (5.0 kB gzipped)
    pages/Settings.js                     8.8 kB  (2.2 kB gzipped)
    pages/Profile.js                      5.6 kB  (1.5 kB gzipped)
    pages/Tasks.js                        4.2 kB  (1.3 kB gzipped)
    pages/Home.js                         3.5 kB  (1.1 kB gzipped)
    …
    total                                99.7 kB

  Runtime: 19 of 44 modules, 123.5 kB of 406.7 kB kept, 110.7 kB with its names shortened (before minifying)
    core          26.3 kB  always
    router        13.9 kB  WF.router
    form          12.7 kB  WF.form
    motion        12.6 kB  enter:
    each          10.6 kB  WF.each
    store          9.1 kB  WF.store
    keep           8.9 kB  WF.persist
    field          2.7 kB  WF.field
    …
    left out: keys format offline peer scalars text slot browser host …

  Tokens: 30 nothing in the output names, left out
```

Read it in three parts:

- **What it weighs** lists every text file with its gzipped size. The
  starter's form brings in `form` and `field`, its store brings in `store`
  and `keep`, its second page brings in `router`, and its entry animations
  bring in `motion`.
- **Runtime** lists each module with what the build kept of it — the
  functions the program reaches — and what reached it. When a module you
  did not expect is in the build, look here first.
- **Tokens** counts the design tokens the build left out because nothing
  names them.

Each build records its sizes in `.wf-sizes.json` beside the project, so the
next `--stats` shows how each file moved since the last build (for example,
`+20 B`).

The project's own scripts are listed by their path under `js/`, such as
`js/tilt.js` ([Call your own script](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)). The build
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
| Layout decided by `viewport` | `viewport` is unknown until the script runs, so the page paints one branch and then swaps it | Use [responsive values](https://webfluent.monzeromer.dev/learn/13-styling.md#responsive-values), which are CSS |
| `"runtime": "full"` | It ships every module | Use it only when a hand-written script calls `WF` in ways the build cannot see, or an icon's name arrives at run time |
| Web fonts | Each weight is a download | Use `display=swap` and few weights, or the baseline's system fonts |
| An unkeyed list that changes | `for item in items` rebuilds the list | Key it: `for item in items by item.id` moves the nodes that stayed |
| Pictures the build cannot process | An SVG, a URL or a path built at run time is used as written: one size, no placeholder | Use a literal `src:` into `public/`, or declare the picture with `image` ([Add a picture](https://webfluent.monzeromer.dev/recipes/add-a-picture.md)) |

## Next

[Static or single-page](https://webfluent.monzeromer.dev/about/static-or-single-page.md): the two shapes a build takes, and what each one gives a reader.
