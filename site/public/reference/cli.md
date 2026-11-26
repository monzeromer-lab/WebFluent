# Command line

> Every wf command and its options, from init, build and serve to render, migrate and verify, and the language server in each editor.

The page: https://webfluent.monzeromer.dev/reference/cli

This chapter is the reference for `wf`, the one binary that creates, builds, serves, checks, formats, tests, renders and describes a WebFluent project, and for `wf-lsp`, the language server that brings the same checks into your editor. Look up a command here when you need its exact flags, defaults and behaviour.

`wf --help` lists the commands, `wf <command> --help` lists one command's flags, and `wf --version` prints the version (`wf X.Y.Z`).

```bash
wf init my-site -t static
cd my-site
wf serve
```

## Conventions

These rules hold for every command unless its entry says otherwise:

- **The project directory.** A command that works on a project uses the current directory. `-d DIR` (or `--dir DIR`) names another one; a command that takes a `PATH` argument instead takes the directory, or sometimes one file, as that argument.
- **Exit status.** A command exits `0` when it succeeds and non-zero when it fails, so each command works as a CI step without a wrapper. A build that stops on its findings exits `1`; a build that cannot run at all — a file it cannot read, a config it cannot load — exits `2`.
- **`--json`.** A describing command prints a human-readable report by default and, with `--json`, the same information as JSON for a tool.
- **Previews.** Every command that writes files has a flag that shows what it would do and writes nothing: `wf fmt --check`, `wf migrate --check`, `wf fix --dry-run`. `wf commands` names that flag for each command.

## Describing the project

These commands print what the project and the compiler hold. Each one is also a function in the `webfluent` crate, named in its entry and in [Building a tool on the compiler](#building-a-tool-on-the-compiler).

## Environment variables

| Variable | Read by | What it does |
|---|---|---|
| `WF_INSTALL_DIR` | `install.sh`, `install.ps1` | Where the binary is installed (default `~/.webfluent/bin`). |
| `WF_VERSION` | `install.sh`, `install.ps1` | The release to install, such as `vX.Y.Z`, instead of the latest. |
| `WF_CHROME` | `wf test`, `wf verify` | The Chrome or Chromium to drive, when it is not found on its own. |
| `WF_CHROME_NO_SANDBOX` | `wf test`, `wf verify` | `1` starts Chrome without its sandbox from the outset, instead of trying with one first (a container that gives it none). |
| `WF_BIN` | the Node binding | The `wf` binary to call. |
| any `PUBLIC_…` name | `wf build` | A value for `env.NAME` ([Environments and secrets](https://webfluent.monzeromer.dev/recipes/environments-and-secrets.md)). |
| `WF_ALLOW_SHELL_ENV` | `wf build`, `wf serve` | Names, comma-separated, that `public_env` lists without the `PUBLIC_` prefix and whose value the shell may supply (`WF_ALLOW_SHELL_ENV=API_URL`). Without it such a name takes its value from the config or `.env` only, so a cloned project cannot bundle a shell secret. |

## Editors

`wf-lsp` is the language server: it runs the compiler's checks as you type and answers your editor's questions about the code. [Editors](https://webfluent.monzeromer.dev/reference/editors.md) says how to install it, what it answers, and how to set up VS Code, Zed, Neovim, Helix, Emacs and any other editor with LSP support.

## Building a tool on the compiler

Every command above is a function in the `webfluent` crate, so an editor, a site builder or an agent can call it in process instead of running `wf`. Each one reads the project through a `webfluent::vfs::Vfs`: `FsVfs` reads the disk, and `OverlayVfs::over_disk()` lays the editor's unsaved buffers over it, so a tool reports the project as it is being edited.

Most of these are the compiler's own parts: they are reached with the crate's `internals` feature (`webfluent = { version = "7", features = ["internals"] }`), are not documented, and a release may change them without a major version. The ones marked "no" are the documented API.

| Function | Needs `internals` | What it is |
|---|---|---|
| `webfluent::build::{build, check}` with `BuildOptions` | no | `wf build` and `wf check`, returning every line and every finding as a `Diagnostic` with its fixes; `BuildOptions::studio` is `--studio` |
| `webfluent::diagnostics::fixes::fix_all` | no | `wf fix`'s fix-all, for one family in one file |
| `webfluent::syntax::{print, reprint}`, `webfluent::edit::{rewrite_element, rewrite_statement, rewrite_body, rewrite_declaration, rewrite_program}` | yes | An element, a statement or a declaration changed in the tree and written back as the smallest edit, every other byte as it was; refused when it would not read back the same |
| `webfluent::syntax::node_id::{build_node_map_with, layers}` | no | Every element's id, file, place, parent and what it is inside; the node at a caret; the Layers tree |
| `webfluent::registry::schema::of_project_via` | yes | `wf components` |
| `webfluent::paged::style_value` | yes | Each style property's shape, a value read into parts (`read_box`), and the narrowest declaration written back (`edit_box`) |
| `webfluent::themes::view::{view_via, set_token, reset_token}` | yes | `wf theme`, and one token written in its place |
| `webfluent::motion::catalogue` | yes | `wf motion` |
| `webfluent::validation::{must_be_via, set_rule, remove_rule, move_rule}` | yes | `wf rules`, and a state's `validate` block written back one rule at a time |
| `webfluent::scope::paths::at_via` | yes | `wf scope` |
| `webfluent::i18n::{plurals, table}` | yes | `wf translations`: CLDR's plural rules, the table, and `set_message` |
| `webfluent::build::assets::{list, plan}`, `webfluent::media::plan` | yes | `wf assets` |
| `webfluent::config::{schema, json_edit}` | yes | `wf config`, and a setting written keeping the file's shape |
| `webfluent::commands` | yes | `wf commands` |
| `webfluent::agent::{changeset, context, diff, edit}` | yes | `wf context`, and a change set an agent writes to instead of the working tree, built before anyone reads it, with anchored edits |

## Commands

### Scaffolding and building

#### `wf init`

Create a new WebFluent project: [its page](https://webfluent.monzeromer.dev/reference/cli/init.md).

```bash
wf init NAME [options]
```

Creates a new project directory with a config, a `.gitignore`, an `AGENTS.md`, the `writing-webfluent` agent skill (in `.agents/skills/` and `.claude/skills/`) and a small working application.

```bash
wf init my-site -t static
wf init my-app --locales en,ar
```

With `--locales`, every visible string of the starter goes through `t("…")`, the config's `i18n` names the locales (and `meta.lang` the default), the language switch has one entry for each locale, and `src/translations/` holds a file per locale:

- the default locale's file holds the starter's English, the text its pages and tests are written in;
- another locale's file holds the starter's own translation where it has one (Arabic), and the same English otherwise, for you to translate.

A locale is a language tag — `en`, `ar`, `pt-BR`, `zh-Hant` — written in any case and stored in the usual one. A tag named twice, an empty list, or a code that is not a tag (`pt_BR`) is refused with what to write instead. A `pdf` document or a `slides` deck is made in one language and refuses `--locales`.

`AGENTS.md` is the language reference — the file a coding agent reads before it changes a project. With it, an assistant asked to add a page writes the WebFluent of the `wf` that made the project, flags and all, instead of guessing. Its first line names the version it describes. The skill points an agent at `AGENTS.md` when the work is WebFluent, in the folders agents look in for one. A newer `wf` writes a newer one, and an older project can take the current one from the [repository](https://github.com/monzeromer-lab/WebFluent/blob/master/AGENTS.md).

`wf init` writes a shorter config than the full set of defaults, and turns `build.csp` on.

See also: [Translate a site](https://webfluent.monzeromer.dev/recipes/translate-a-site.md), [Configuration](https://webfluent.monzeromer.dev/reference/config.md).

#### `wf build`

Build the project: a site or app (HTML, CSS, JS), a PDF, a slide deck or custom elements: [its page](https://webfluent.monzeromer.dev/reference/cli/build.md).

```bash
wf build [options]
```

Runs the whole compiler — parse, semantic checks, type checks, linters and code generation — and writes the site to `build.output` (`./build`).

```bash
wf build
wf build --format github --deny-warnings
```

**A build stays inside its project.** It reads only the project's own files: a `data`, `image`, `api … from` or `i18n.dir` names a path inside the project (an absolute path or a `..` is an error), and a symbolic link that leads out of the project — or to nothing, or back into a directory above it — is left out of every walk, with a warning naming it. A link that stays inside is followed. It writes only inside its project too: `build.output` is refused when it climbs out (`../site`, `/var/www`) or is a link to somewhere else, and a link already inside the output is refused rather than written through. `--allow-output-outside` is how the person running the build lets it write elsewhere; a project's config cannot ask for it. A build removes only the `.gz` files an earlier build wrote (it keeps their names in `.wf-cache/precompressed.json`); any other `.gz` in the output is left alone.

An error stops the build and prints the file, line, column, message and a hint. A warning prints and the build goes on.

`--format` takes four values:

| Value | Output |
|---|---|
| `human` | Rendered findings on standard error. |
| `json` | One document on standard output: every finding with its code, place, hint, related places and fixes, and the counts. |
| `sarif` | SARIF 2.1.0, for GitHub code scanning and every other SARIF reader. |
| `github` | `::error file=…` annotations on standard output, which a workflow shows on the pull request; the rendered findings stay on standard error. |

With `json` or `sarif`, the build's progress goes to standard error, so standard output holds the document alone.

`--cache` keeps what it renders in `.wf-cache/build/`. A page is taken from the cache when its inputs are unchanged: its own text, every component it reaches, and what every page reads — the shell, the stores, constants and data, every page's title and route, the config, the translations and the compiler. The output is byte for byte what a build without `--cache` writes. A build with nothing to reuse fills the cache, and each build keeps only the entries it used.

`--studio` stamps every element's root with `data-wf-node`, its node id, which is the same in the pre-rendered page and the live one. An instance of one of the project's components also carries `data-wf-call`: the calls that made it, innermost first. `nodes.json`, written beside the site, maps each id to where it is written (file, bytes, line), what it calls, its parent, the loops and branches it is inside, and what its arguments read. A build without `--studio` stamps nothing and removes any `nodes.json` it finds.

`--stats` prints every text file with its gzipped size and how much it changed since the last build, the runtime modules the build kept and what reached each one, and the design tokens it left out.

See also: [Diagnostics](https://webfluent.monzeromer.dev/errors.md), [Performance](https://webfluent.monzeromer.dev/about/what-a-build-ships.md).

#### `wf serve`

Start the development server: [its page](https://webfluent.monzeromer.dev/reference/cli/serve.md).

```bash
wf serve [options]
```

Runs the development server: it builds once, serves the output, and rebuilds and reloads on every save.

```bash
wf serve
wf serve --host 0.0.0.0     # a phone on this network, at the address it prints
```

The server listens on `dev.port` (3000), on `127.0.0.1`: nothing else on the network can reach it unless `--host` says so. It watches `src/`, `public/` and the config, rebuilds on a change, and reloads every open tab. Each rebuild is a `wf build --cache`, so only the pages the change reached are rendered again.

It serves the build and nothing beside it: an address that climbs out (`/../`, written plainly or as `%2e%2e`), holds a backslash or a NUL, or reaches a link out of the build is a `404`. It answers only a request whose `Host` is a name it is reached by — `localhost`, an address written as numbers, the `--host` it was given, and with `--host 0.0.0.0` a `.local` name — so a page on another site cannot point a name of its own at this machine and read the build (DNS rebinding); any other gets a `421`. It answers several requests at once, streams files rather than reading them whole, and compresses a file once until it changes.

When a build fails, the last good page stays up and the error is drawn over it — file, line, message and hint — until a save fixes it. The overlay is served at `/__wf/dev.js`, and the build's state, as JSON, at `/__wf/status`.

Every page the server sends also carries a **stores** button. It shows what each store holds, a log of every action with the state on each side of it, and lets you put a store back to before an action ran ([Stores](https://webfluent.monzeromer.dev/learn/11-stores.md#debugging-stores)). The service worker of an offline site is removed under `wf serve`, so an edit is never hidden behind its cache.

A request that is not a navigation — an `http` or `api` call, a script, an image — for a file the build did not write gets a `404`, as a host would send, and a line in the terminal naming the file. Only a navigation gets the single-page shell. (Answering a missing API with the shell's HTML makes the page fail to parse it, far from the cause.)

A build under `wf serve` is a *development build*, and it adds these checks to the running page:

- The body of each status an endpoint's error enum names is held to the type its case carries. (A response's declared type — `resource users: [User] = …`, an endpoint's `-> User!` — is held in every build, served or deployed: one that does not match fails as `.decode(at, message)`, naming the place, `$[1].name is missing`.)
- A route no page answers (and no `path: "*"` page catches) shows a `404` box saying so. A deployed page shows nothing.
- An item of a `for` that throws shows where it is and what it threw, and the rest of the list draws. On a deployed page the item leaves a gap and the console says why.
- An effect that changes what it reads every time it runs is stopped after a hundred runs, with a warning, instead of overflowing the stack. A deployed page stops it too, silently.

See also: [`dev` in Configuration](https://webfluent.monzeromer.dev/reference/config.md#dev).

#### `wf start`

Serve a built project as a host would: `$PORT`, the build's headers and compressed files, a single-page build's shell for every route, and `--proxy` for an API elsewhere: [its page](https://webfluent.monzeromer.dev/reference/cli/start.md).

```bash
wf start [options]
```

Serves what `wf build` wrote, as a host would. It builds nothing: run `wf build` first.

```bash
wf build && wf start
PORT=8080 wf start --proxy /api=http://localhost:9000/api
```

What it sends:

- **The files of the build,** each with the `.gz` beside it when the client takes gzip, and the headers the build's `_headers` file names for its path (the content security policy, the frame rule). A page is `no-cache`; a file whose name carries its hash is cached for a year.
- **A navigation the build wrote no file for** gets a static build's `404.html` with a `404`, or a single-page build's shell, which draws the route itself. A script, a picture or an API call for a file the build did not write gets a `404`, never the shell.
- **A request under a `--proxy` prefix** goes to the server behind it, its method, headers and body as sent, and its answer comes back as that server gave it. A page calls its own origin (`api Backend(base: "/api")`), so the policy needs nothing added and there is no CORS, and the API's address is chosen where the site starts, not when it was built:

  ```json
  { "scripts": { "build": "wf build", "start": "wf start --proxy /api=$API_BASE_URL/api" } }
  ```

  It speaks HTTP/1.1 to the server behind it, over TLS for an `https://` URL, and trusts what this machine trusts: its certificate store, and the certificates `SSL_CERT_FILE` or `NODE_EXTRA_CA_CERTS` names — a company's own authority, or a development one such as Caddy's or mkcert's. A certificate nothing vouches for is refused, with a `502` that says so.

It is a small server for a container, a preview or an app that serves itself. Behind a CDN or a static host, upload `build/` instead ([Deploy to GitHub Pages](https://webfluent.monzeromer.dev/recipes/deploy-to-github-pages.md)).

#### `wf inspect`

One page as a reader meets it, as text: landmarks, headings, links, buttons, fields, tables, and anything that failed: [its page](https://webfluent.monzeromer.dev/reference/cli/inspect.md).

```bash
wf inspect TARGET [options]
```

Opens one page in a headless Chrome and prints it as a reader meets it, in text.

```bash
wf inspect /
wf inspect /pokemon/pikachu --mobile --screenshot
wf inspect http://localhost:3000/settings
wf inspect /tasks --do 'type "Milk" into "New task"' --do 'click "Add"'
```

```text
/ — "Dashboard" (en) at 1280×800
  link "Skip to content" → #wf-main
  navigation "Main"
    link "Dashboard" → / [current=page]
    link "Tasks" → /tasks
    button "Log in"
  main
    h1 Dashboard
    "Welcome to your dashboard."
    checkbox "Learn WebFluent"
no problems: nothing threw, logged an error or failed to arrive
```

A route is the project's build, served for the moment; a URL is a server already running (`wf serve`, `wf start`, anything). Each line is one thing a reader meets, indented under the landmark or list it is in: a heading (`h1 …`), a link and where it goes, a button or other control with its name and states (`[pressed]`, `[expanded]`, `[disabled]`, `[current=page]`), a field with its label and value, a table's caption and first rows, a list's first items, a picture's `alt` (or `(no alt)`, or `[broken]`), and text in quotes. What is hidden — `display: none`, `hidden`, `inert`, a closed dialog — is left out, as a reader never meets it.

With `--do`, the steps are played first, in order, finding what they name as a test does (a button's text, a field's label), and the outline is the page as it stands after them. A step that cannot be played — nothing called that, an `expect` whose text is not there — is the first problem, and the steps after it are not played.

Last come the problems: what threw, what was logged as an error, and what failed to arrive. It exits `0` either way: it reports, `wf verify` judges.

#### `wf generate`

Generate a new page, component, or store — or a block, a whole page to own: sign-in, pricing, dashboard, settings: [its page](https://webfluent.monzeromer.dev/reference/cli/generate.md).

```bash
wf generate KIND NAME [options]
```

Writes a starter page, component or store.

```bash
wf generate page Pricing
wf generate component PriceCard
wf generate store Cart
```

A page gets a path from its name (`/pricing`), a title, a description and an `h1`. The file is `.wfx` when the project is written in that layout.

`wf generate block NAME` writes a whole page instead — `sign-in` (a form that checks what is typed), `pricing` (three plans, monthly or yearly), `dashboard` (figures, a trend and a table) or `settings` (a profile, notifications and the account, in tabs) — into `src/pages/`. It is ordinary source, the project's own to change; it builds with nothing to report. `wf generate` never overwrites a file: when one exists, it says so and writes nothing.

### Checking and testing

#### `wf check`

Check the project — every finding a build would report — and write nothing: [its page](https://webfluent.monzeromer.dev/reference/cli/check.md).

```bash
wf check [options]
```

Runs every check a build runs — the parser, the structure, the types, the lints and the config — and writes nothing.

```bash
wf check
wf check --format sarif > wf.sarif
wf check --deny-warnings
```

`wf check` is fast enough for a pre-commit hook. It exits `1` on a finding that would stop a build and `0` otherwise, and prints `No problems in NAME.` when there are none. A check that needs the written output — the security policy read back over the pages — runs only in `wf build`.

See also: [Diagnostics](https://webfluent.monzeromer.dev/errors.md).

#### `wf config`

Read and change `webfluent.app.json`: every setting, what the project sets, and an edit that keeps the file's shape: [its page](https://webfluent.monzeromer.dev/reference/cli/config.md).

```bash
wf config <schema|get|set|unset>
```

Reads and changes `webfluent.app.json`, and lists every setting it can hold.

```bash
wf config schema
wf config schema --json
wf config get build.base_path
wf config set build.ssg true
wf config set meta.lang ar
wf config unset build.ssg
```

`set` checks the value against the setting's type before it writes anything. `set` and `unset` keep the rest of the file as it was written. Under a map, the rest of the path is one key: `build.budget.app.js` sets the `app.js` entry of `build.budget`.

See also: [Configuration](https://webfluent.monzeromer.dev/reference/config.md).

#### `wf api`

A service's contract as files, from its OpenAPI spec: add one, sync it, check it is current, turn `api … from` into files, list them: [its page](https://webfluent.monzeromer.dev/reference/cli/api.md).

```bash
wf api <add|sync|check|eject|list>
```

Writes a service's contract as files from its OpenAPI spec (3.0 or 3.1, JSON or YAML), and keeps them current.

```bash
wf api add openapi.json --name Backend
wf api sync
wf api sync Backend --skip uploadXml
wf api check
wf api eject Backend
wf api list --json
```

Every generated file opens with a `// Code generated by …` comment that names `wf api sync` and its spec and says DO NOT EDIT, and `sync` never overwrites or removes a file in `generated/` without it. The output is the printer's, so it is the same for the same spec, and a change to the spec is the smallest diff. `wf check` warns (`N07`) when a spec has changed since its files were written. The compiler never fetches: download the spec first (`curl -o openapi.json …`). See [Data and APIs](https://webfluent.monzeromer.dev/learn/15-data.md#openapi).

#### `wf fix`

Fix every finding that has a fix — or every one of a code — as `wf check` and the editor offer it, and say what is left: [its page](https://webfluent.monzeromer.dev/reference/cli/fix.md).

```bash
wf fix [options]
```

Applies every fix the compiler is certain of — the fix the editor offers first — and writes the files.

```bash
wf fix
wf fix --code V07
wf fix --dry-run
```

A fix that overlaps another, or that the first fixes make way for, is taken on a second pass, and passes repeat until nothing more changes. `wf fix` then lists what is left: findings with no certain fix, which a person decides. `wf check` lists them too.

See also: [Diagnostics](https://webfluent.monzeromer.dev/errors.md).

#### `wf explain`

What a diagnostic code means, a program that draws it, and the fix; with no code, every code: [its page](https://webfluent.monzeromer.dev/reference/cli/explain.md).

```bash
wf explain [CODE] [options]
```

Prints a diagnostic code's entry from [Diagnostics](https://webfluent.monzeromer.dev/errors.md) in the terminal.

```bash
wf explain T05
wf explain
```

An entry says what the code means, shows a program that draws it, what the compiler prints, the fix, and the link to the guide.

#### `wf test`

Run the project's `test "…" { … }` declarations under tests/: [its page](https://webfluent.monzeromer.dev/reference/cli/test.md).

```bash
wf test [PATH] [options]
```

Runs the project's `test "…" { }` declarations.

```bash
wf test
wf test tests/cart.wf
wf test --update
```

`wf test` runs every test under `tests/` (and in `src/`), or the tests in one file. A test that clicks or types runs in headless Chrome, on a page that links the project's scripts.

See also: [Testing](https://webfluent.monzeromer.dev/learn/16-testing.md).

#### `wf audit`

What this project trusts: markup it did not write, other origins, what it keeps on the reader's machine, its `env` names, its policy and its dependencies: [its page](https://webfluent.monzeromer.dev/reference/cli/audit.md).

```bash
wf audit [PATH] [options]
```

Prints what a security review asks about the project. It reports and never fails.

```bash
wf audit
wf audit --json
```

The report lists:

- every `Unsafe.*` and whether its input is sanitised;
- the project's own scripts and what each declares;
- every other origin the pages load from;
- everything kept on the reader's machine;
- every `env` name and whether it is public;
- the policy the build ships;
- what the compiler depends on.

See also: [Review a site's security](https://webfluent.monzeromer.dev/recipes/review-security.md) and [Security](https://webfluent.monzeromer.dev/about/security.md).

#### `wf verify`

Load every page of a built project in a real browser: errors, failed requests, paint timing, and which built-ins it drew: [its page](https://webfluent.monzeromer.dev/reference/cli/verify.md).

```bash
wf verify [PATH] [options]
```

Loads every page of a built project in a real browser and reports what went wrong there.

```bash
wf build && wf verify
```

Chrome runs with its sandbox. Where it cannot start with one (a container that gives it none) it is started again without, and `wf verify` says so; `WF_CHROME_NO_SANDBOX=1` skips the first try. Its profile is a private directory with a name nobody can guess, and the browser and the profile go when `wf verify` or `wf test` ends — however it ends, a signal included.

`wf build` reports what the compiler wrote; `wf verify` reports what a browser does with it. It builds nothing: it starts a headless Chrome, serves what `wf build` wrote, and opens every route the project has — a `:param` route at each value its `paths:` names, or at a placeholder (`/user/1`) when it names none.

A route fails on any of these:

- an uncaught exception or a `console.error`;
- a file that did not arrive (a request the build has no file for is a `404`, not the page shell);
- an image that failed to load;
- a router that drew nothing into `<main>`;
- a page that rendered no text.

The report prints one line per route:

```text
  16 route(s) in http://127.0.0.1:39785
    ok   /                                    236ms     613 nodes    447.0 kB   11 req
    ok   /app/deployments                      68ms     657 nodes    360.6 kB   10 req

  16 page(s), 0 problem(s)
```

The columns are the first contentful paint, the number of elements in the document once it settled, the bytes the page fetched, and the requests it made. `--budget 400` fails a route that takes longer than 400 ms to paint.

`--returning-visitor` visits every route a second time as a reader who was here before. Their storage holds what the previous build's pages kept: each `persist` value's starting value as the build before this one wrote it, which a build saves in `.wf-cache/` when it changes. A persisted shape that changed without a `version:` then shows as the broken page it would be.

`--fakes tests/fakes.wf` is for a site whose API is elsewhere, or not running. It builds the project again, aside from `build/`, with the fakes that file declares answering every endpoint — the same `fake` declarations a test names ([Testing](https://webfluent.monzeromer.dev/learn/16-testing.md#fakes)) — and loads every page of that build, which sends no request to a service. A fake that does not fit its contract stops the build (`N06`). The aside build is removed when the visits are done.

The report also names **the built-ins no page drew**. A component can be in the registry, the tests and the documentation and still be broken on a page. The classes the pages carried list what actually ran, so the difference is what nothing has exercised. One project is not expected to draw them all: the list is a fact about coverage, not a failure.

`wf verify` needs Chrome or Chromium on the machine, or `WF_CHROME` pointing at one, and says so when there is none.

See also: [Testing](https://webfluent.monzeromer.dev/learn/16-testing.md).

### Describing the project

#### `wf motion`

The animations an element can play, the eases and speeds, the route transitions and the project's default: [its page](https://webfluent.monzeromer.dev/reference/cli/motion.md).

```bash
wf motion [options]
```

Lists what an element's motion props can name.

```bash
wf motion
wf motion --json
```

It lists:

- every animation `animate:` and `exit:` take: the engine's, each with its keyframes, the rule its stylesheet plays it by, whether an element leaves with it and whether it repeats; then the project's own `animation` declarations;
- the named eases, with the timing function each stands for;
- the three speeds and their lengths;
- the `Router`'s transitions;
- the project's default length and pacing (`motion` in the config, or the engine's);
- `data-wf-motion="reduce"`, the attribute an editor sets on the page's root to show the page as a reader who asked for less motion sees it.

The library has it as `webfluent::motion::catalogue` (with the `internals` feature, outside semver).

See also: [Motion defaults](https://webfluent.monzeromer.dev/recipes/motion-defaults.md).

#### `wf context`

What an agent is given: the files, the open file, the problems, the components, the theme, the config with only its public env, the language reference, and what was withheld: [its page](https://webfluent.monzeromer.dev/reference/cli/context.md).

```bash
wf context [options]
```

Prints what a coding agent working on the project is given, so nothing it is told is hidden from you.

```bash
wf context
wf context --open src/pages/Home.wf:120-180
wf context --guide full --json
```

It includes:

- every file of the project — not the build's output, `.git/`, dependencies or a `.env` file;
- the open file and its selection;
- the problems the build reports;
- every component a call can name, with its props, cases, events and slots;
- the theme's tokens;
- `webfluent.app.json`, with every `env` value that `public_env` does not list withheld;
- the guide: each chapter's title, blurb and sections to cite (`16 Forms and validation › What a value must be: validate`), or with `--guide full` the chapters whole;
- what was withheld, and why.

The library has it as `webfluent::agent::context` (with the `internals` feature, outside semver). An agent's edits go through `webfluent::agent::changeset::ChangeSet`: a layer over the editor's buffers that the compiler builds before anyone reads it. It refuses paths outside the project, in `.git/`, in the build's output and `.env` files; reads the config with its private values withheld and keeps them through a write; and is accepted whole, by file or by hunk. `ChangeSet::edit` makes an agent's anchored edits (`webfluent::agent::edit`): each `old_text` must be found once in a file the set has read — exactly, then ignoring what ends a line, then the indent, then typographic quotes and dashes — and a file that moved under the set since it was read is refused until it is read again. Text found twice names its lines; text found nowhere names the nearest.

#### `wf translations`

The translations as a table: each locale's plural forms and what it lacks, each key's messages, and where a `t()` names it: [its page](https://webfluent.monzeromer.dev/reference/cli/translations.md).

```bash
wf translations [options]
```

Shows the project's translations as a table.

```bash
wf translations
wf translations --missing ar
wf translations --unused
wf translations --json
wf translations --plurals
```

For each locale it prints its file, whether it is the default and whether it is right to left, the plural forms CLDR's rules give it, how many keys have no message in it, how many lack a form it needs, and how many messages have placeholders the default locale's do not.

A key is unused when nothing names it: no `t()`, no key built at run time that may be it, and no `validate` rule that shows it.

`--json` prints the whole table: every key, plural keys with each form, each message in each locale with its placeholders and line, every place a `t()` names the key, and the calls that name a key no locale has (`I01`).

`--plurals` prints each locale's forms; with `--json`, also the form each of a run of counts takes.

The library has it as `webfluent::i18n::table` (with the `internals` feature, outside semver), whose `set_message` writes one message back and keeps every other byte of the file.

See also: [Translate a site](https://webfluent.monzeromer.dev/recipes/translate-a-site.md).

#### `wf scope`

What a path picker lists at a place: the names in scope there, each with its type and where it comes from, and the fields to go on into: [its page](https://webfluent.monzeromer.dev/reference/cli/scope.md).

```bash
wf scope PLACE [options]
```

Lists the names that can be read at one place in a source file — what an editor's path picker and completion offer there.

```bash
wf scope src/pages/Deploys.wf:24
wf scope src/pages/Deploys.wf:24:9 --json
wf scope src/templates/invoice.wf:3 --data data/invoice.json
```

The names come in the order a path picker lists them:

1. the page's or component's own `state` and `derived` values, and the form it binds;
2. its props, and a page's route parameters;
3. what the statements around the place bind — a loop's item and index, an `if let`, a `match` arm, a slot's fill, an action's parameters and the `let`s before the place — innermost first;
4. the members of every store it `use`s;
5. with `--data`, the template's data.

Each row has its type as the checker infers it, its shape (`bool`, `number`, `text`, `moment`, `list`, `record`, `enum` …), whether it may be `null`, where it comes from (`for d in live`, `member of Deploys`), and a record's fields to go on into (`d.region.code`). The library has it as `webfluent::scope` (with the `internals` feature, outside semver).

#### `wf rules`

The rules a `validate` block may hold, the named patterns, when a form shows its messages, and every state the project holds to a rule: [its page](https://webfluent.monzeromer.dev/reference/cli/rules.md).

```bash
wf rules [options]
```

Lists what a `validate` block can hold, and every state the project holds to a rule.

```bash
wf rules
wf rules --json
```

It lists:

- every rule: what it takes between its parentheses, when it holds, the message it shows when none is written, the translation key (`form.minLength`) that replaces that message in a translated project, whether an empty value passes it, and whether it is checked last;
- the bounds a refined type implies (`below`, `after`, `before`);
- the patterns people often write, by name, each with its expression and a message;
- `Form(show:)`'s cases, in the reader's terms;
- `data-wf-validation="shown"`, the attribute a preview sets on the page's root to show every message at once;
- every state the project holds to a rule, with its rules in order (those its type implies first) and whether the page checks them as the reader types.

The library has it as `webfluent::validation` (with the `internals` feature, outside semver).

See also: [Forms](https://webfluent.monzeromer.dev/learn/14-forms.md).

#### `wf theme`

The theme the build uses: every token, declared or inherited, where each is used, and its contrast pairs: [its page](https://webfluent.monzeromer.dev/reference/cli/theme.md).

```bash
wf theme [options]
```

Shows the theme the build uses, as an editor's theme panel shows it.

```bash
wf theme
wf theme --json
```

It lists every token by group (colour, spacing, font size, line height, radius, shadow, transition, font, motion, and the tokens the theme adds) with its value, the baseline's value, whether the theme declares or inherits it, the dark theme's value when `theme.dark` names one, the properties that draw from it, and every place the project names it. It then lists the colour pairings the stylesheet composes, with their contrast and what WCAG AA asks of each — a pairing that falls short is the `A13` the build reports — and the tokens one theme of the dark pair declares and the other does not.

See also: [Design tokens](https://webfluent.monzeromer.dev/reference/tokens.md).

#### `wf components`

Every component a call can name — the built-ins and the project's — with what each takes: its props and their kinds, events, slots, parts and children: [its page](https://webfluent.monzeromer.dev/reference/cli/components.md).

```bash
wf components [options]
```

Lists every component a call can name — the built-ins first, then the project's own — in one vocabulary.

```bash
wf components
wf components --json
```

For each prop it gives the kind of value it takes (`text`, `number`, `enum` and its cases, `file` and what kind, `state`, `path`, a declared `record`, a `list`, `date`, `color`, `money` …), whether it is the positional one, its default as written, a range where its type declares one (`Number(0..=100)`), and whether `.name` alone sets it. Then it gives the component's events, its slots and what each hands its fill, its parts and their owner, the attribute families its root takes, and whether it takes children.

This is the table the checker reads a call against. An inspector draws a control per prop from it, and an Insert panel knows what may go where. The library has it as `webfluent::registry::schema::of_project_via` (with the `internals` feature, outside semver).

#### `wf assets`

What `public/` holds — each file's kind, size and dimensions, and where the project names it — or what the media pipeline will make of one image: [its page](https://webfluent.monzeromer.dev/reference/cli/assets.md).

```bash
wf assets [options]
```

Lists every file of `public/` and where the project uses it, or what the image pipeline makes of one picture.

```bash
wf assets
wf assets --unused
wf assets --plan /hero.png
wf assets --json
```

For each file it prints its kind (image, font, video, audio, document, code, data), its size, a picture's dimensions, and every place the project names it: a prop the registry types as a file (`Image`'s `src`, `Video`'s `poster` and `captions`), a page's `image:`, a `const`, an `image` declaration, a `data` file's strings, a value in `webfluent.app.json`, a stylesheet's `url(…)`, a project script, or a manifest. A file a browser or a host asks for by name — `favicon.ico`, `robots.txt`, `.well-known/…` — counts as used. Anything else nothing names counts as unused.

`--plan PATH` applies the project's `build.media` to one picture: each width it writes, in each format, with the file's name and size — for example *3 widths × 1 format · 3 files · about 84 kB*. A size is exact once `wf build` has written the file (it is read from `.wf-cache/media/`), or when the picture is small enough to encode on the spot. Otherwise it is estimated from strips of the picture encoded at that width, and says `about`. The plan names a format the build cannot write, with the reason, and a width it skips because the file would be no smaller than the original. With `--json`, the same sentences are in the `said` field.

The library has the same as `webfluent::build::assets::{list, plan}`. It reads through a `Vfs`, so an editor's unsaved buffers count.

See also: [Add a picture](https://webfluent.monzeromer.dev/recipes/add-a-picture.md).

#### `wf commands`

Every `wf` command, its options and what it writes, as data: [its page](https://webfluent.monzeromer.dev/reference/cli/commands.md).

```bash
wf commands [options]
```

Describes every `wf` command as data.

```bash
wf commands
wf commands --json
```

For each command it gives what the command does, each option — its flag, what it takes (`nothing` for a flag, a `word` from the values it lists, a `number`, a `path`, `text`) and its default — what the command writes, and the flag that shows it first and writes nothing (`fmt --check`, `migrate --check`, `fix --dry-run`). Build a tool page, a command palette or a script from this output instead of from a copy of this chapter. The library has the same as `webfluent::commands` (with the `internals` feature, outside semver).

#### `wf docs`

Document the project's components as a site, each `example` drawn live: [its page](https://webfluent.monzeromer.dev/reference/cli/docs.md).

```bash
wf docs [options]
```

Documents the project's components as a site, in the design of this reference.

```bash
wf docs
wf docs -o component-docs
```

A page a component: its `///` comment and header, every [`example`](https://webfluent.monzeromer.dev/learn/10-components.md#documentation-comments) of it drawn live beside its source — in a frame of the project's own build, with its theme, stylesheets and scripts — then its props (type, what it is when left out, each prop's `///`), events, slots, parts and what places it. A component with no example is drawn from its props. The types, stores and pages each have a page.

The project is checked first, and an error stops it, as a build does. Serve the folder to read it. A second run clears what the first wrote; a folder it did not write is left alone.

#### `wf registry`

Describe every built-in component: props, cases, flags, events, slots, parts: [its page](https://webfluent.monzeromer.dev/reference/cli/registry.md).

```bash
wf registry [options]
```

Describes every built-in component: props, cases, flags, events, slots and parts.

```bash
wf registry --json | jq '.components[] | select(.name == "Button") | .props[].name'
```

The registry is the compiler's own table: what it prints is exactly what the checker accepts.

See also: [Built-in components](https://webfluent.monzeromer.dev/reference.md).

#### `wf types`

Describe what a project declares: enums, types, components, stores, pages: [its page](https://webfluent.monzeromer.dev/reference/cli/types.md).

```bash
wf types [PATH] [options]
```

Describes what a project declares: enums, types, components, stores and pages.

```bash
wf types
wf types --json
wf types --holes
```

It includes the names the project's scripts under `src/` declare, each with the parameters and return type its JSDoc gives it.

With `--holes` it lists every place the checker cannot say what a value is, each with its file, line and kind, then the count of each kind. A hole is counted where the unknown value is made, not at every place it goes afterwards. Since 7.0 a program that checks has none: `Any` and `Map` are not types (`T23`, `T24`), every `state`, parameter and return says what it holds, and the browser, events and built-ins are typed. The list is for a project on its way there — one `wf migrate` has not finished — and for a tool that wants to prove it. `--json` gives `total`, the count of each kind, and the list.

```text
  0 place(s) where a type is unknown, in 0 file(s)
```

### Rendering

#### `wf render`

Render a .wf template with JSON data: [its page](https://webfluent.monzeromer.dev/reference/cli/render.md).

```bash
wf render TEMPLATE [options]
```

Renders one template with JSON data, to HTML, a PDF or a slide deck.

```bash
wf render invoice.wf --data invoice.json -f pdf -o invoice.pdf
echo '{"name":"Ada"}' | wf render greeting.wf
wf render templates/ --page Receipt --lang ar --data receipt.json
```

A template is held to the same checks as a build: a component nothing declares, or a flag a component does not take, stops the render. The one difference is that the names a template reads come from its data.

See also: [Render a template](https://webfluent.monzeromer.dev/recipes/render-a-template.md).

### Upgrading

#### `wf migrate`

Carry a project forward: WebFluent 3 → 4, 4 → 4.2, 6 → 7 and 7 → 8, each step that applies (a WebFluent 2 project: migrate with 7.x first): [its page](https://webfluent.monzeromer.dev/reference/cli/migrate.md).

```bash
wf migrate [PATH] [options]
```

Carries a project written for an older WebFluent forward.

```bash
wf migrate --check
wf migrate
wf migrate --wfx
```

`wf migrate` makes four passes, in this order:

1. **4.1 → 4.2**, first, because 4.2 refuses `external`. A remote module becomes a `meta.scripts` entry. A local one becomes a plain script under `src/`, with its signatures as JSDoc, and `X.f(…)` becomes `f(…)`. An `external element` call becomes `Element("tag", …)`. The pass lists every `.js` already under `src/`, since each now ships on every page.
2. **2 → 3** rewrites every `.wf` under `src/` in place, with a note for anything that needed a decision. It changes spelling only: the migrated project builds to what it built before.
3. **3 → 4** changes what the compiler allows, so it runs over the project rather than the files. It adds every `env` name your pages already read to `public_env`, which preserves what the project did, and prints the list, because *public* means anyone who opens the site may read the value. It then names, with file and line, everything 4 refuses that 3 allowed: an `on*` attribute, a `javascript:` or `data:` URL. Those have no automatic rewrite, because the old value was a string of JavaScript or a link that ran one. Finally it lists the changes that need no edit: a store is built on first read rather than at boot, a `persist` value follows the site's other tabs, and `WF.store`/`WF.host` were renamed.
4. **6 → 7** applies every fix the findings 7.0 brings carry — the request forms 6.x had (`N08`), what each declaration holds, returns and fails with (`T25`, `T26`, `N03`, `T32`, `T31`), the comparison a condition meant (`T29`), a splice or a unit (`T34`) — until nothing is left to fix, then lists every error a person decides, by place, and the changes that need no edit. With `--check` it says what it would fix and writes nothing.

See also: [Upgrading](https://webfluent.monzeromer.dev/errors/upgrading.md).

### Formatting

#### `wf fmt`

Format a project's source files; with `--to`, rewrite them in the other layout: `.wfx` (blocks by indentation) or `.wf` (blocks in braces): [its page](https://webfluent.monzeromer.dev/reference/cli/fmt.md).

```bash
wf fmt [PATH] [options]
```

Formats every `.wf` and `.wfx` file under a path, or one file, in place.

```bash
wf fmt
wf fmt --check
wf fmt src/pages/Home.wf --stdout
wf fmt --to wfx
```

The formatter applies these rules:

- four spaces for each level, and a line that closes a block one level out;
- trailing spaces removed, and tabs turned into spaces;
- runs of blank lines folded to one;
- a brace one space from what it opens (`Row{` → `Row {`, `}else` → `} else`).

A line carried on by a trailing comma stays as written, and comments stay where they are. The result is checked against the file's own tokens, so formatting never changes what a file says.

`--to wfx` rewrites the project's `.wf` files in the indented layout, and `--to wf` goes the other way. It changes the layout and nothing else, and refuses a file whose indentation does not already follow its braces.

See also: [Two layouts: `.wf` and `.wfx`](https://webfluent.monzeromer.dev/learn/04-language-basics.md#two-layouts-wf-and-wfx).
