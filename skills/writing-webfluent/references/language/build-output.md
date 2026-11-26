## Build output

`wf build` writes HTML, a runtime-and-app `app.js`, per-page chunks and stylesheets, and gzipped copies.

```
build/
├── index.html            # the shell (SPA) or the pre-rendered page (SSG)
├── app.js                # the runtime this program needs, stores, components
├── pages/<Name>.js       # one chunk per Page, loaded when its route shows (not for one page)
├── pages/<Name>.css      # the style rules only that page reaches
├── styles.css            # tokens, only the component rules the project uses,
│                         # the project's own .css files, and the compiled
│                         # style { } rules more than one page shares
├── sw.js                 # with "offline" in the config: the service worker
├── *.gz                  # every text file above, gzipped, beside itself
└── …                     # public/ copied to the root, sitemap.xml, robots.txt
```

`app.js` opens with the runtime — but only the part of it the program
reaches. The runtime is a set of feature modules (`each`, `router`, `net`,
`format`, `icons`, `carousel`, …), and a build reads what it needs from the
bundle it just wrote: every `WF.<name>` the code generator emitted, closed
over each module's dependencies, plus the glyphs the program names, which
are the only ones the icon table keeps. Then it goes further than the
module: of the modules it ships it keeps only the functions those calls
reach, and `WF` carries only the names the program calls — a `for` adds
`each`, a second page adds `router`, a `resource` adds `net`, and a
`Router` with no `transition:` carries no transition player. A minified
build (not `wf serve`'s) then shortens the runtime's own names, its
top-level ones and each function's locals; what the program calls on `WF`
keeps its name. A page that shows static text, with no `app` and so no
router, is an `app.js` of 1.8 kB gzipped.

Calls from outside the bundle count as calls: the project's own scripts,
an `elements` build's definitions. Nothing is hand-marked, so the set
cannot fall behind the compiler. A name a build cannot see — an icon
assembled at run time, a `WF.*` call built from a string — is the one case
it gets wrong, and `"build": { "runtime": "full" }` ships everything for
it.

`:root` carries only the design tokens something in the output names —
a stylesheet, a script, a page, a file copied from `public/` — and the
tokens those name in turn.

`wf build --stats` prints what the build weighs: every text file with its
gzipped size and how it moved since the last build, and each runtime module
with what the build kept of it and what reached it.

```
  What it weighs
    styles.css                             5.5 kB  (1.7 kB gzipped)
    app.js                                 4.9 kB  (1.8 kB gzipped)  −0.1 kB
    index.html                             0.8 kB  (0.4 kB gzipped)
    total                                 11.2 kB

  Runtime: 1 of 58 modules, 15.0 kB of 596.8 kB kept, 14.0 kB with its names shortened (before minifying)
    core          15.0 kB  always
    left out: theme keys nodes motion helpers format offline peer scalars form …
```

`build.budget` names a gzipped size an output must stay under. A build over
it prints a warning and goes on — a build that fails on size is a decision
for a project's own CI, which can read the same numbers:

```json
{ "build": { "budget": { "app.js": "40 kB", "styles.css": "30 kB" } } }
```

Each build records what it weighed in `.wf-sizes.json` beside the project
(not in the output, which is published), so the next `--stats` shows the
difference.

`build.compress` (on by default) writes `<file>.gz` beside every text
output over a kilobyte — HTML, JavaScript, CSS, SVG, JSON, XML — for a host
that serves a precompressed file when it has one (nginx `gzip_static on`,
Apache `MultiViews`, most CDNs and static hosts). The compressor is the
engine's own, within a percent of zlib's default level, so the build has no
dependency for it; `wf serve` sends the same files, and compresses on the
way out what has none, so what Lighthouse measures against the dev server is
what a deployed site sends. Hosts that ignore `.gz` files lose nothing.

`build.minify` (on by default) strips comments and whitespace from the
bundle and the sheets. `build.split` (on by default) writes each page as its
own chunk: a static build's page links `app.js` and its chunk side by side,
and a single-page build fetches a chunk the first time its route is shown.
The chunk also brings what drawing the page needs of its route — its title,
its language and its layout — so `app.js` holds only each route's path and
page name: a site of hundreds of pages does not send every page's title and
layout with every page.
A site of one page is not split: its code and its rules are in `app.js`
and `styles.css`.
A literal in a `style { }` block is compiled into a stylesheet rule under a
class named by the block's content, so a hundred identical cards share one
rule; only a value that reads state is set on the element. With `split`, a
rule only one page can reach — in its body, or through components no other
page uses — is written to `pages/<Name>.css`, which the page links after
`styles.css` and the router loads before drawing the page; rules the `App`
body or two pages reach stay in `styles.css`.

Images are given `decoding="async"`; the first on a page is eager with
`fetchpriority="high"` and every later one `loading="lazy"`, unless the
element sets `loading:` itself.

