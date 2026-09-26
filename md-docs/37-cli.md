# 37. Command line and editors

<!--
route: guide/cli
group: reference
blurb: Every wf command and flag, the dev server, and the language server in each editor.
description: The wf command line: init, build, serve, test, verify, fmt, generate, render, migrate, audit, docs, registry, types — and editor setup.
-->

One binary, `wf`, scaffolds, builds, serves, formats, tests, renders and
describes a project. A second, `wf-lsp`, brings the same checks into your
editor. `wf --help` lists the commands and `wf <command> --help` each one's
flags; `wf --version` prints the version.

Every command that takes a project runs in the current directory unless
told otherwise, and exits non-zero on failure, so each is a CI step as it is.

## The commands

| Command | Does |
|---|---|
| `wf init NAME [-t, --template spa\|static\|pdf\|slides]` | Creates a project from a template |
| `wf build [-d, --dir DIR] [--stats]` | Compiles the project; fails on errors, prints warnings |
| `wf serve [-d, --dir DIR]` | Builds, serves on `dev.port`, rebuilds and reloads on every save |
| `wf test [PATH] [--update]` | Runs the project's `test` declarations |
| `wf verify [PATH] [--json] [--budget MS]` | Loads every built page in headless Chrome |
| `wf fmt [PATH] [--check] [--stdout] [--to wf\|wfx]` | Formats sources, or switches their layout |
| `wf generate page\|component\|store NAME [-d, --dir DIR]` | Writes a starter file |
| `wf render FILE [--data JSON] [-f, --format FORMAT] [-o, --output OUT] [--theme NAME]` | Renders a template with data |
| `wf migrate [PATH] [--check] [--stdout] [--wfx]` | Carries a WebFluent 2 or 3 project forward |
| `wf audit [PATH] [--json]` | Lists what the project trusts |
| `wf docs [-d, --dir DIR] [-o, --out OUT]` | Writes a gallery of every component |
| `wf registry [--json]` | Describes every built-in |
| `wf types [PATH] [--json]` | Describes what the project declares |

## Creating and building

### `wf init`

```bash
wf init my-site -t static
```

Makes a directory `NAME` with a config, a `.gitignore` and a small working
application for the template: `spa` (the default — an interactive app),
`static` (a marketing site, pre-rendered), `pdf` (a document) or `slides` (a
deck). It refuses a directory that already exists.

### `wf build`

Runs the whole pipeline — parse, semantic checks, type checks, linters,
code generation — and writes `build.output` (`./build`). Errors stop the
build with the file, line, column, message and a hint; warnings print and
the build goes on. `-d DIR` builds another directory.

`--stats` prints what the build weighs: every text file with its gzipped
size and how it moved since the last build, the runtime modules it kept and
what reached each, and the design tokens it left out. [Performance](31-performance.md)
reads it.

### `wf serve`

The development loop. It builds once, serves the output on `dev.port`
(3000), watches `src/`, `public/` and the config, rebuilds on a change and
reloads every open tab. A build that fails leaves the last good page up and
draws the error over it — file, line, message and hint — until the next save
fixes it. The overlay is served at `/__wf/dev.js`, the build's state at
`/__wf/status` (JSON).

Every page it serves also carries a **stores** button: what each store
holds, a log of every action with the state on each side of it, and a click
to put a store back ([Stores](12-stores.md#looking-at-one-while-it-runs)). An
offline site's service worker is removed under `wf serve`, so an edit is
never hidden behind its cache.

### `wf generate`

```bash
wf generate page Pricing
wf generate component PriceCard
wf generate store Cart
```

Writes a starter declaration in `src/pages/`, `src/components/` or
`src/stores/` — in `.wfx` when the project is written that way — and never
overwrites a file. A page gets a path from its name (`/pricing`), a title, a
description and an `h1`.

## Checking

### `wf test`

```bash
wf test
wf test tests/cart.wf
wf test --update
```

Runs every `test "…" { }` under `tests/` (and in `src/`), or those in one
file; `--update` rewrites the snapshots. A test that clicks or types runs in
headless Chrome. [Testing](24-testing.md) covers writing them.

### `wf verify`

A build says what the compiler wrote. This says what a browser does with
it.

```bash
wf build && wf verify
```

It starts a headless Chrome, serves the output, and opens every route the
project has. A route fails on an uncaught exception, a `console.error`, a
file that did not arrive, an image that failed to load, or a page that
rendered no text — the things a compiler cannot see and a reader always
can.

```
  16 route(s) in http://127.0.0.1:39785
    ok   /                                    236ms     613 nodes    447.0 kB   11 req
    ok   /app/deployments                      68ms     657 nodes    360.6 kB   10 req

  16 page(s), 0 problem(s)
```

The numbers are the first contentful paint, the elements in the document
once it settled, the bytes the page fetched and the requests it made.
`--budget 400` fails a route that takes longer than that to paint.
`--json` is the same report for a pipeline.

It also says **which built-ins no page drew**. A component can be in the
registry, in the tests and in the documentation and still be broken in a
page; the classes the pages carried are the list of what actually ran, so
the difference is what nothing has exercised. One project is not expected
to draw them all — it is a fact about coverage, not a failure.

It needs a Chrome or Chromium on the machine, or `WF_CHROME` pointing at
one, and says so plainly when there is none.


### `wf audit`

```bash
wf audit
wf audit --json
```

Prints what a security review asks for: every `Unsafe.*` and whether it is
sanitised, every other origin the pages load from, everything kept on the
reader's machine, every `env` name and whether it is public, the policy the
build ships, and what the compiler depends on. It reports and never fails.
[Security](23-security.md#wf-audit).

## Formatting

### `wf fmt`

```bash
wf fmt
wf fmt --check
wf fmt src/pages/Home.wf --stdout
wf fmt --to wfx
```

Formats every `.wf` and `.wfx` under a path, or one file, in place: four
spaces a level, a line that closes a block one level out, trailing spaces
gone, tabs made spaces, runs of blank lines folded to one, a brace one space
from what it opens (`Row{` → `Row {`, `}else` → `} else`). A line carried on
by a trailing comma is left as written, and comments stay where they are.
The result is held to the file's own tokens, so formatting never changes what
a file says.

- `--check` writes nothing and fails when a file would change — for CI.
- `--stdout` prints one file instead of writing it.
- `--to wfx` rewrites a project's `.wf` files in the indented layout, and
  `--to wf` the other way. It is a change of layout and nothing else; it
  refuses a file whose indentation does not already follow its braces.

## Rendering

### `wf render`

```bash
wf render invoice.wf --data invoice.json -f pdf -o invoice.pdf
echo '{"name":"Ada"}' | wf render greeting.wf
```

Renders one template with JSON data: `-f` / `--format` is `html` (a whole
document, the default), `html-fragment` (the body only), `pdf` or `slides`;
`-o` writes a file instead of printing; `--theme` picks one of several
themes the template declares; without `--data` the JSON is read from stdin.
[Server rendering](34-server-rendering.md).

## Upgrading

### `wf migrate`

Two things, in one command.

**2 → 3** rewrites every `.wf` under `src/` in place, with a note for
anything that needed a decision. It is a change of spelling: the migrated
project builds to what it built before.

**3 → 4** is a change of what the compiler allows, so it runs over the
project rather than the files. It adds every `env` name your pages already
read to `public_env` — preserving what the project did, and printing the
list, because *public* means anyone who opens the site may read the value.
Then it names, with file and line, everything 4 refuses that 3 allowed: an
`on*` attribute, a `javascript:` or `data:` URL. Those have no automatic
rewrite; the old value was a string of JavaScript or a link that ran one.
Finally it states the changes that need no edit: a store is built on first
read rather than at boot, a `persist` value follows the site's other tabs,
and `WF.store`/`WF.host` were renamed.

`--check` writes nothing and tells you what it would do.


## Describing

### `wf docs`, `wf registry`, `wf types`

`wf docs` writes `docs/index.html` (or `-o OUT`): a self-contained gallery of
every built-in — props, cases, flags, events, slots, parts — and of every
component, enum, type, store and page the project declares, with their `///`
docs.

`wf registry` describes every built-in, and `wf types` what a project
declares, externals included; with `--json` each prints the data a tool
reads:

```bash
wf registry --json | jq '.components[] | select(.name == "Button") | .props[].name'
```

The registry is the compiler's own table: what it prints is exactly what
the checker accepts.

## Environment variables

| Variable | Read by | Does |
|---|---|---|
| `WF_INSTALL_DIR` | `install.sh`, `install.ps1` | Where the binary is installed (default `~/.webfluent/bin`) |
| `WF_VERSION` | `install.sh`, `install.ps1` | The release to install, `v4.0.1`, instead of the latest |
| `WF_CHROME` | `wf test`, `wf verify` | The Chrome or Chromium to drive, when it is not found on its own |
| `WF_BIN` | the Node binding | The `wf` binary to call |
| any `PUBLIC_…` name | `wf build` | A value for `env.NAME` ([Environments](30-environments.md)) |

## Editors

`wf-lsp` is the language server. Get the binary from a
[release](https://github.com/monzeromer-lab/WebFluent/releases)
(`wf-lsp-<version>-<arch>-<os>.tar.gz`) and put it on your `PATH`, or build
it from a clone with `cargo install --path crates/wf-lsp`. It gives:

- Diagnostics as you type — every check `wf build` runs.
- Completion: components and their props after `(`, flags and parts after
  `.`, enum cases after `:`, events after `on `, `$` tokens, `match` arms,
  slot names, fields of a record, methods of a string or a list.
- Hover: a component's doc and signature, a name's inferred type.
- Go to definition, find references, rename across the project.
- Document symbols and the outline.
- Quick fixes — "Change to `…`" on a misspelt name — and `Extract
  component` on a selection.

**Zed**: the extension is in `editors/zed`; install it with
`zed: install dev extension` pointing at that folder (it is not in Zed's
extension registry yet). It brings Tree-sitter highlighting, outline, brackets,
indentation, snippets and the language server (found on `PATH` or
downloaded from the latest release), for `.wf` and `.wfx`.

**VS Code**: `editors/vscode` — TextMate highlighting and the language
server. Until it is on the Marketplace, run `npm ci && npm run package` in
that folder and install the `.vsix` with *Extensions: Install from VSIX*.

**Neovim** (0.11 or later):

```lua
vim.filetype.add({ extension = { wf = "webfluent", wfx = "webfluent" } })
vim.lsp.config("wf_lsp", {
  cmd = { "wf-lsp" },
  filetypes = { "webfluent" },
  root_markers = { "webfluent.app.json" },
})
vim.lsp.enable("wf_lsp")
```

**Helix** — in `languages.toml`:

```toml
[[language]]
name = "webfluent"
scope = "source.webfluent"
file-types = ["wf", "wfx"]
roots = ["webfluent.app.json"]
comment-token = "//"
indent = { tab-width = 4, unit = "    " }
language-servers = ["wf-lsp"]

[language-server.wf-lsp]
command = "wf-lsp"
```

**Emacs** (with Eglot):

```elisp
(define-derived-mode webfluent-mode prog-mode "WebFluent")
(add-to-list 'auto-mode-alist '("\\.wfx?\\'" . webfluent-mode))
(with-eval-after-load 'eglot
  (add-to-list 'eglot-server-programs '(webfluent-mode . ("wf-lsp"))))
```

Any other editor with LSP support runs `wf-lsp` over stdio; the project
root is the folder holding `webfluent.app.json`. The Tree-sitter grammars in
`editors/tree-sitter-webfluent` (braces) and `editors/tree-sitter-webfluentx`
(indentation) give highlighting to an editor that reads Tree-sitter.

## Next

[Configuration](38-configuration.md).
