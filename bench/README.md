# Benchmarks

What the [benchmarks page](https://webfluent.monzeromer.dev/benchmarks) and
the home page show, and how to run each one again. Every number on the page
comes from a result file in this folder; `bench/report.mjs` gathers them
into `site/src/benchmarks.json`, and the page draws that file. Nothing on
the page is typed by hand.

| Folder | What it measures | Run it | Its results |
|---|---|---|---|
| [`toolchain/`](toolchain/run.mjs) | Each framework's starter, made by its own `create` command: install time, packages and `node_modules` on disk, production build, dev server ready, JavaScript shipped | `just bench-toolchain --runs=3` | `toolchain/results.json` |
| [`defaults/`](defaults/run.mjs) | Lighthouse on each starter, served by its own production server; and what the server sends with nothing configured | `just bench-defaults --runs=3` | `defaults/results.json` |
| [`defaults/capabilities/`](defaults/capabilities) | What each framework provides with no third-party package, for the twelve things a search engine, a link preview, a screen reader and the browser look for — by its own docs | edited by hand, one file a framework | the files themselves |
| [`runtime/`](runtime/run.sh) | [js-framework-benchmark](https://github.com/krausest/js-framework-benchmark), keyed, in headless Chrome | `just bench-runtime` | `runtime/results/` |
| [`pdf/`](pdf/RESULTS.md) | One invoice through `wf` and through Puppeteer, cold and warm | `just bench-pdf` | `pdf/results.json`, `pdf/RESULTS.md` |
| [`agents/`](agents/README.md) | What Claude Code spends to build the same app in WebFluent, against Nolan Lawson's published runs of twelve frameworks | `just bench-agents-run NAME` | `agents/runs/<batch>/results.json` |
| [`dataset/`](dataset/build.py) | Not a benchmark: the WebFluent training dataset published on Hugging Face | `python3 bench/dataset/build.py` | — |

Then `just bench-report` writes the page's data, and `cargo xtask site` the
page's Markdown twin.

## Fairness

- **One machine, one Node.** Every framework is measured on the same
  machine, Node and Chrome, which each result file records.
- **Each starter as its docs make it.** Nothing is tuned for WebFluent:
  the starters are what each framework's own `create` command writes,
  served by the server its docs name (`toolchain/run.mjs`,
  `defaults/run.mjs`).
- **Capabilities by the framework's own docs.** Each item of
  `defaults/capabilities/<framework>.json` has a level and the URL of the
  page that says so. The levels, from most to least done for you:
  `default` (there with nothing done; what the framework's own starter
  writes counts), `setting` (one setting), `api` (a feature made for that
  job, written on each page — Next.js's `metadata`, Nuxt's `useSeoMeta`),
  `official` (a package from the same team) and `manual` (written by hand,
  or someone else's package). A general way of writing any tag into the
  head (`<svelte:head>`, React 19's head tags, an Astro layout) is
  `manual`, and so is a header written into a general headers option
  (Next.js `headers()`, Nuxt route rules). Check a file again when its
  framework releases; `report.mjs` refuses an item with no level or source.
- **Agents against published runs.** WebFluent's agent runs use Lawson's
  harness unchanged but for WebFluent's skill, with the same model, effort
  and Claude Code version as his published medians; `report.mjs` reads the
  batch named by `--batch=` (the page shows `tools2-ab`).
- **What is behind is shown.** The runtime table and the agents' tokens,
  where WebFluent is behind, are on the page as measured.
