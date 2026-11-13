# Performance: where the time goes

Measured 2026-11-13 on this machine (16 cores), release builds of `wf` and
`wf-lsp` at commit 15aa7f9a. The project is the documentation site
(`site/`): 77 `.wf` files, 22,754 lines, 63 pages plus 77 pages from one
`:slug` route, 732 files written.

## Done so far (2026-11-13)

Measured with `cargo xtask bench`, which prints each figure beside the
last run on the same machine.

| | Before | After |
|---|---|---|
| `wf build`, documentation site | 4.4 s | 0.36 s |
| `wf build`, the site at 4× its size | not measured | 0.91 s |
| `wf check`, site | 82 ms | 62 ms |
| `wf check`, at 4× | 260 ms | 163 ms |
| A hover, completion or outline, site | 46 ms | 0.1–0.2 ms |
| The same, at 4× | 155 ms | 0.1–0.3 ms |
| An edit's findings, site | 145 ms | 74 ms |
| The same, at 4× | 305 ms | 126 ms |
| The wf-lsp test over the whole site (debug) | 167 s | 10 s |

What changed, in the order it was done:

**The language server**
1. It answers from the project as last analysed, and analyses again only
   after:
   - an edit, an open, a close or a save;
   - a file changed on disk (or after 500 ms when the editor does not
     report that).
2. A project analysed again reuses each file's parse while its text is the
   same. The pause before an edit is checked is 50 ms, not 75.
3. Each file keeps its tokens, made once per edit rather than once per
   request.
4. The project keeps the checker's type table:
   - the one the check after an edit made;
   - or one made the first time a hover asks for it.
   
   A hover no longer type-checks the whole program.

**Data and parallel work**
5. A `Static` list or map shares its contents (`Arc`), so reading a `data`
   constant no longer copies it.
6. Pages are painted on every core, and the `.gz` files are compressed on
   every core. Both are written in the order they always were.

**The allocator and the compressor**
7. `wf` and `wf-lsp` allocate with mimalloc. The system allocator made the
   threads wait on its lock.
8. The compressor compares eight bytes at a time and keeps its tables per
   thread. It finds the same matches.

**Serial passes**
9. A build weighs its output from the `.gz` files it has just written,
   instead of compressing everything again, one file at a time.
10. The tokens a build reads are found in one pass rather than one pass
    per token.
11. The pages a policy is held to are read back on every core.

Every change was held to the golden output, byte for byte.

**Left:**
- An edit's findings still come from checking the whole program:
  - about 20 ms on the site and 75 ms at 4×, after the 50 ms pause;
  - a check per file is the next step there.
- A page still clones the translation table and the component table.
- On the site at 4×, the serial stages left are these, each worth a look:
  - analysis (about 180 ms);
  - JavaScript (about 160 ms);
  - painting (about 230 ms, across cores).

## What a build ships (2026-11-13)

Built output, gzipped, before → after:

| Project | `app.js` | `styles.css` |
|---|---|---|
| counter example | 9.0 kB → 9.0 kB | 3.7 kB → 1.9 kB |
| `wf init -t static` | 23.3 kB → 21.3 kB | 5.4 kB → 4.2 kB |
| `wf init -t spa` (it animates) | 23.5 kB → 23.5 kB | 6.1 kB → 5.0 kB |

**What changed:**
- **Motion.** The runtime's animations (`motion`, 20 kB before minifying)
  ship only to a program that animates. The node helpers that branches,
  lists and routes need are a module of their own.
- **The built-in stylesheet** keeps a component's section only when the
  program draws that component. That covers the grid, columns, spacers,
  dividers, code and its highlighting, blockquotes, Markdown, drag and
  drop, and every document and slide element.
- **Rule-level pruning.** The sections every page keeps (responsive
  reflow, focus, reduced motion) lose the selectors that name a component
  whose section went.

**Next: prune the runtime by function, not by module.** `core` (44 kB
before minifying) ships whole to every project. A small project reaches
about half of it:

| Project | Functions reached | Share of `core`'s source |
|---|---|---|
| counter | 41 of 107 | 44% |
| static starter | 45 of 107 | 49% |
| SPA starter | 55 of 107 | 58% |

That count follows calls from the generated code through `core`'s own
functions, but not from the other modules shipped, so it is slightly low.

The runtime is plain declarations in one closure, so the compiler can
follow calls exactly and emit only the functions reached. It gets "only
the code a page needs" without writing helpers out at every use.

## Is it the tests, or the tools?

Both, for different reasons.

- **The tests are slow because of how they run**, not only because of what
  they test. Most run a debug build, which is 10–30× slower than a release
  build at this kind of work, and many build or load whole projects. One
  wf-lsp test file took 171 s.
- **The tools are slow in two places that a user meets every day:**
  - every language-server request reloads the whole project;
  - a site build pre-renders pages by deep-copying data.
  
  `wf check` itself is fast.

## The compiler

| Command (docs site) | Time |
|---|---|
| `wf check` | 82 ms |
| `wf build` | 4.4 s (one core) |
| `wf build`, `ssg: false` | 0.41 s |
| `wf build`, `compress: false` | 4.0 s |
| `wf build`, `minify: false` / `csp: false` / `split: false` | no real change |

The build's stages:

| Stage | Time |
|---|---|
| Parse, check, CSS | 81 ms |
| JavaScript | 70 ms |
| Pre-rendering | 3.2 s |
| Writing, the security-policy read-back, sitemap, `public/`, service worker | 160 ms |
| `.gz` files | 430 ms |

**Pre-rendering is 73% of a build.** Inside it:
- 62 static pages take 0.54 s;
- 77 pages of `/docs/reference/:slug` take 2.6 s, about 34 ms each for
  about 30 KB of HTML each.

A sampling profile (pprof, since `perf` is not permitted here) of the
whole build:

- **`Static` values are deep-copied.** About 23% of the time is spent
  cloning and freeing `Static`. Reading a `data` constant (the site's
  115 KB `registry.json`) copies the whole tree every time an expression
  names it, so a page that reads `registry.components` copies the registry.
- **The hand-written gzip** (`longest_match`) takes about 25% of the
  samples. It runs on other threads, so it adds about 0.35 s of wall time.
- **Hashing takes 9%.** Scope frames and maps use std's SipHash.
- **Each page rebuilds the translation table** from `Static` (3.5%) and
  clones the component table and the node map.
- **Component expansion copies statement trees** (`substitute_ui`, 3.5%).

## The language server

| Request on the docs site (77 files) | Median |
|---|---|
| open → diagnostics | 81 ms |
| completion, hover, definition, outline, semantic tokens | 45–50 ms each |
| formatting (1,490-line file) | 75 ms |
| an edit → diagnostics | 145 ms (75 ms of it is the deliberate pause for more keystrokes) |

The same file in bigger copies of the project (`scripts/bench/grow.py`):

| Files | Hover | Edit → diagnostics | `wf check` |
|---|---|---|---|
| 77 | 46 ms | 145 ms | 0.08 s |
| 128 | 79 ms | 198 ms | 0.15 s |
| 230 | 155 ms | 305 ms | 0.26 s |

**Every request reloads the whole project** (`Backend::project` →
`Project::load` in `crates/wf-lsp/src/backend.rs`): it walks the folder,
reads and merges every file, and builds the type information again, even
for a hover or an outline. So every answer costs as much as checking the
project, and grows with it. The analysis itself is not the problem;
repeating it is.

## What to do, in order of what it buys

1. **wf-lsp: keep the analysed project.**
   - Build it once.
   - Rebuild it only when a buffer changes, a watched file changes or
     the config changes.
   - Hand requests a shared snapshot (`Arc<Project>`).
   
   Requests that read no new text (hover, completion, definition, outline,
   tokens) drop from 45–155 ms to the cost of the lookup itself.
2. **wf-lsp: re-check only what an edit touches.**
   - Re-parse the edited file alone; the file cache already exists.
   - Re-run the checks over the merged program.
   - Report the edited file first.
3. **Static values share their contents.**
   - `Static::List` and `Static::Map` hold `Arc<[…]>`, or `Rc`, so a clone
     is a reference count, not a copy of the tree.
   - A scope lookup hands back a reference where it can.
   
   This takes most of the 2.6 s off the `:slug` pages and helps every page
   that reads data.
4. **Pre-render pages in parallel.** Pages are independent once the scope
   for the program exists. With `rayon`, or scoped threads as the `.gz`
   pass already uses, the 3.2 s pass should drop to roughly a quarter on
   this machine.
5. **Per build, not per page.**
   - The translation table is converted once.
   - The component table and the node map are borrowed, not cloned.
6. **A faster hasher** (`rustc-hash`/`ahash`) for the evaluator's frames
   and the checker's tables.
7. **gzip:** a faster match finder or a lower level for big files, held to
   the same output size within a percent; or `flate2` with the pure-Rust
   backend, which a WebAssembly build also accepts.
8. **`wf serve`:** rebuild only the pages an edit can reach. `--cache`
   exists for `wf build`; check that `wf serve` uses it.

## How to measure

- **The compiler:**

  ```bash
  hyperfine -N "wf check -d site" "wf build -d site"
  ```

  The site writes to `../docs`, so a copy with `build.output` inside it is
  easier to time.
- **The language server:**

  ```bash
  scripts/bench/lsp.py target/release/wf-lsp site src/pages/Reference.wf
  ```

- **Scaling:**

  ```bash
  scripts/bench/grow.py site /tmp/site-x4 4
  ```

  Then run either command above on the copy.
- **A profile:** `perf` needs `perf_event_paranoid` ≤ 1. Without that, use
  a temporary `pprof` guard around `webfluent::cli::main` in a scratch
  build, which is how the profile above was taken.

`cargo xtask bench` runs both, and compares with the last recorded
numbers. Not added yet: criterion benchmarks for parse, check and the
evaluator.

## Found on the way

`wf check` refused a project whose `build.output` points outside it,
although a check writes nothing. It now takes `--allow-output-outside`,
as `wf build` does.
