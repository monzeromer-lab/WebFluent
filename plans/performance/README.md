# Performance: where the time goes

Measured 2026-11-13 on this machine (16 cores), release builds of `wf` and
`wf-lsp` at commit 15aa7f9a. The project is the documentation site
(`site/`): 77 `.wf` files, 22,754 lines, 63 pages plus 77 pages from one
`:slug` route, 732 files written.

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

To be added as part of the work, so that a regression shows:
- a `cargo xtask bench` that runs both and compares with the last
  recorded numbers;
- criterion benchmarks for parse, check and the evaluator.

## Found on the way

`wf check` refuses a project whose `build.output` points outside it,
although a check writes nothing. It should not need
`--allow-output-outside`.
