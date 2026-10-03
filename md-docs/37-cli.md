# 37. Command line and editors

<!--
route: guide/cli
group: reference
blurb: Every wf command and flag, the dev server, and the language server in each editor.
description: The wf command line: init, build, check, explain, serve, test, verify, fmt, generate, render, migrate, audit, docs, registry, types — and editor setup.
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
| `wf build [-d, --dir DIR] [--stats] [--format FORMAT] [--deny-warnings]` | Compiles the project; fails on errors, prints warnings |
| `wf check [-d, --dir DIR] [--format FORMAT] [--deny-warnings]` | Every finding a build would report, with nothing written |
| `wf explain [CODE]` | What a diagnostic code means, a program that draws it, and the fix |
| `wf serve [-d, --dir DIR]` | Builds, serves on `dev.port`, rebuilds and reloads on every save |
| `wf test [PATH] [--update]` | Runs the project's `test` declarations |
| `wf verify [PATH] [--json] [--budget MS] [--returning-visitor]` | Loads every built page in headless Chrome |
| `wf fmt [PATH] [--check] [--stdout] [--to wf\|wfx]` | Formats sources, or switches their layout |
| `wf generate page\|component\|store NAME [-d, --dir DIR]` | Writes a starter file |
| `wf render FILE\|DIR [--data JSON] [-f, --format FORMAT] [-o, --output OUT] [--page NAME] [--lang LANG] [--theme NAME] [--token NAME=VALUE]` | Renders a template with data |
| `wf migrate [PATH] [--check] [--stdout] [--wfx]` | Carries a WebFluent 2, 3 or 4.1 project forward |
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

It also writes `AGENTS.md`: the language reference, the file a coding agent
reads before it changes a project — so an assistant asked to add a page
writes the WebFluent of the `wf` that made it, flags and all, rather than
guessing. Its first line names the version it describes; a newer `wf`
brings a newer one, and an older project can take it from the
[repository](https://github.com/monzeromer-lab/WebFluent/blob/master/AGENTS.md).

### `wf build`

Runs the whole pipeline — parse, semantic checks, type checks, linters,
code generation — and writes `build.output` (`./build`). Errors stop the
build with the file, line, column, message and a hint; warnings print and
the build goes on. `-d DIR` builds another directory.

`--format` says how the findings are written: `human` (the default, on
standard error), `json` (one document on standard output — every finding
with its code, place, hint, related places and fixes, and the counts),
`sarif` (SARIF 2.1.0, for GitHub code scanning and every other SARIF
reader) or `github` (`::error file=…` annotations on standard output, which
a workflow shows on the pull request, with the rendering kept on standard
error). With `json` or `sarif` the build's progress goes to standard error,
so standard output is the document alone. `--deny-warnings` stops the build
on a warning, as on an error — the switch for CI.

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

A request that is not a page — a `fetch`, a script, an image — for a file
the build did not write gets a `404`, as a host would send, and a line in
the terminal saying which; only a navigation gets the single-page shell. A
missing API used to be answered with the shell's HTML, which then failed
to parse somewhere far from the cause.

A build under `wf serve` is a development build, and the page knows it:

- a response a type is declared for — `resource users: [User] = …`, an
  `api` endpoint's `-> User` — is held to that type as it arrives, and one
  that does not match takes the `error` arm as a `.parse` error naming the
  place: `$[1].name is missing`;
- a route no page answers (and no `path: "*"` page catches) shows a `404`
  box saying so, where a deployed page shows nothing;
- an item of a `for` that throws shows where it is, with what it threw,
  and the rest of the list draws — on a deployed page it leaves a gap and
  the console says why;
- an effect that changes what it reads every time it runs is stopped after
  a hundred runs, with a warning, instead of overflowing the stack (it is
  stopped on a deployed page too, silently).

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

### `wf check`

```bash
wf check
wf check --format sarif > wf.sarif
wf check --deny-warnings
```

Runs every check a build runs — the parser, the structure, the types, the
lints, the config — and writes nothing, so it is fast enough for a
pre-commit hook. It takes `--format` and `--deny-warnings` as `wf build`
does, exits `1` on a finding that stops a build and `0` otherwise, and says
`No problems in NAME.` when there are none. What only the output can show —
the security policy read back over the written pages — is the build's.

### `wf explain`

```bash
wf explain T05
wf explain
```

A code's entry in [Diagnostics](39-diagnostics.md), in the terminal: what it
means, a program that draws it, what the compiler says, the fix and the
link. With no code, every code with its severity and title.

### `wf test`

```bash
wf test
wf test tests/cart.wf
wf test --update
```

Runs every `test "…" { }` under `tests/` (and in `src/`), or those in one
file; `--update` rewrites the snapshots. A test that clicks or types runs in
headless Chrome, on a page that links the project's scripts. [Testing](24-testing.md) covers writing them.

### `wf verify`

A build says what the compiler wrote. This says what a browser does with
it.

```bash
wf build && wf verify
```

It starts a headless Chrome, serves the output, and opens every route the
project has — a `:param` route at each value its `paths:` names, or at a
placeholder (`/user/1`) when it names none. A route fails on an uncaught
exception, a `console.error`, a file that did not arrive (a `fetch` the
build has no file for is a `404`, not the page shell), an image that failed
to load, a router that drew nothing into `<main>`, or a page that rendered
no text — the things a compiler cannot see and a reader always can.

```
  16 route(s) in http://127.0.0.1:39785
    ok   /                                    236ms     613 nodes    447.0 kB   11 req
    ok   /app/deployments                      68ms     657 nodes    360.6 kB   10 req

  16 page(s), 0 problem(s)
```

The numbers are the first contentful paint, the elements in the document
once it settled, the bytes the page fetched and the requests it made.
`--budget 400` fails a route that takes longer than that to paint.
`--json` is the same report for a pipeline. `--returning-visitor` visits
every route a second time as a reader who was here before: their storage
holds what the previous build's pages kept — each `persist` value's
starting value as the build before this one wrote it, which a build saves
in `.wf-cache/` when they change — so a shape that changed without a
`version:` shows as the broken page it would be.

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
sanitised, the project's own scripts and what each declares, every other
origin the pages load from, everything kept on the
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
wf render templates/ --page Receipt --lang ar --data receipt.json
```

Renders one template with JSON data: `-f` / `--format` is `html` (a whole
document, the default), `html-fragment` (the body only), `pdf` or `slides`;
`-o` writes a file instead of printing; `--theme` picks one of several
themes the template declares; `--token color-primary=#8B5CF6` sets a design
token over the theme's, as often as it is given; `--lang` sets the
document's `<html lang>`; without `--data` the JSON is read from stdin. The
template may be a directory — every `.wf` and `.wfx` under it as one,
components shared — and `--page` names the page to render when there are
several. The template is held to what a build is — a component
nothing declares or a flag it does not take stops the render — except that
the names it reads are its data's. [Server rendering](34-server-rendering.md).

## Upgrading

### `wf migrate`

Three things, in one command.

**4.1 → 4.2** runs first, because 4.2 refuses `external`: a remote module
becomes a `meta.scripts` entry, a local one a plain script under `src/` with
its signatures as JSDoc (`X.f(…)` becomes `f(…)`), and an `external element`
call `Element("tag", …)`. It lists every `.js` already under `src/`, since
each now ships on every page.

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
declares — the names its scripts under `src/` declare included, each with
the parameters and return its JSDoc gives them; with `--json` each prints
the data a tool reads:

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
- Quick fixes, from the compiler's own findings: the nearest name for a
  misspelt field, member, method, variable, component or flag; `?.` for a
  value that may be null; the props a call leaves out; the arms a `match`
  misses; a bare word written as its flag; `alt: ""` for an image; an `_`
  before a name nothing reads. And `Extract component` on a selection.

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
