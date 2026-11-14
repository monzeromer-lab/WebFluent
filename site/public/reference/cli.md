# Command line and editors

> The wf command line — every command and flag, from init, build and serve to render, migrate and verify — and how to set up the language server in your editor.

The page: https://webfluent.monzeromer.dev/reference/cli

This chapter is the reference for `wf`, the one binary that creates, builds,
serves, checks, formats, tests, renders and describes a WebFluent project,
and for `wf-lsp`, the language server that brings the same checks into your
editor. Look up a command here when you need its exact flags, defaults and
behaviour.

`wf --help` lists the commands, `wf <command> --help` lists one command's
flags, and `wf --version` prints the version (`wf 5.11.0`).

```bash
wf init my-site -t static
cd my-site
wf serve
```

## Command summary

| Command | What it does |
|---|---|
| `wf init NAME [-t, --template spa\|static\|pdf\|slides] [-l, --locales LIST]` | Creates a project from a template, translated into `LIST` when given |
| `wf build [-d, --dir DIR] [--stats] [--format FORMAT] [--deny-warnings] [--cache] [--studio] [--allow-output-outside]` | Compiles the project; fails on errors, prints warnings |
| `wf serve [-d, --dir DIR] [--host ADDRESS] [--allow-output-outside]` | Builds, serves on `dev.port` to this machine, and rebuilds and reloads on every save |
| `wf generate page\|component\|store NAME [-d, --dir DIR]` | Writes a starter file |
| `wf check [-d, --dir DIR] [--format FORMAT] [--deny-warnings] [--allow-output-outside]` | Reports every finding a build would report, and writes nothing |
| `wf fix [-d, --dir DIR] [--code CODE] [--dry-run]` | Applies every finding's fix, or the fixes for one code |
| `wf config schema [--json]`, `wf config get\|set\|unset PATH [VALUE] [-d, --dir DIR]` | Lists every setting, and edits `webfluent.app.json` without changing its shape |
| `wf explain [CODE]` | Prints what a diagnostic code means, a program that draws it, and the fix |
| `wf test [PATH] [--update]` | Runs the project's `test` declarations |
| `wf verify [PATH] [--json] [--budget MS] [--returning-visitor] [--allow-output-outside]` | Loads every built page in headless Chrome |
| `wf audit [PATH] [--json]` | Lists what the project trusts |
| `wf fmt [PATH] [--check] [--stdout] [--to wf\|wfx]` | Formats sources, or switches their layout |
| `wf render TEMPLATE [--data JSON] [-f, --format FORMAT] [-o, --output OUT] [--page NAME] [--lang LANG] [--theme NAME] [--token NAME=VALUE]` | Renders a template with data |
| `wf migrate [PATH] [--check] [--stdout] [--wfx]` | Carries a WebFluent 2, 3 or 4.1 project forward |
| `wf commands [--json]` | Describes every command: its options, what it writes, and how to preview it |
| `wf assets [-d, --dir DIR] [--unused] [--plan PATH] [--json]` | Lists `public/` and where each file is used, or what the image pipeline makes of one picture |
| `wf theme [-d, --dir DIR] [--json]` | Shows the theme the build uses: every token, declared or inherited, where each is used, and its contrast |
| `wf motion [-d, --dir DIR] [--json]` | Lists the animations, eases, speeds, route transitions and the project's default |
| `wf context [-d, --dir DIR] [--open FILE[:START-END]] [--guide index\|full] [--json]` | Prints what a coding agent is given: the files, the problems, the components, the theme, the config with only its public `env`, the guide, and what was withheld |
| `wf translations [-d, --dir DIR] [--missing LOCALE] [--unused] [--plurals] [--json]` | Shows the translations as a table: each locale's plural forms and what it lacks, each key's messages and uses |
| `wf scope FILE:LINE[:COL] [-d, --dir DIR] [--data JSON] [--json]` | Lists the names that can be read at a place, each with its type and where it comes from |
| `wf rules [-d, --dir DIR] [--json]` | Lists the `validate` rules, the named patterns, when a form shows its messages, and every state held to a rule |
| `wf components [-d, --dir DIR] [--json]` | Lists every component a call can name — the built-ins and the project's — with what each takes |
| `wf docs [-d, --dir DIR] [-o, --out OUT]` | Writes a gallery of every component |
| `wf registry [--json]` | Describes every built-in |
| `wf types [PATH] [--json]` | Describes what the project declares |

## Conventions

These rules hold for every command unless its entry says otherwise:

- **The project directory.** A command that works on a project uses the
  current directory. `-d DIR` (or `--dir DIR`) names another one; a command
  that takes a `PATH` argument instead takes the directory, or sometimes
  one file, as that argument.
- **Exit status.** A command exits `0` when it succeeds and non-zero when it
  fails, so each command works as a CI step without a wrapper. A build that
  stops on its findings exits `1`; a build that cannot run at all — a file
  it cannot read, a config it cannot load — exits `2`.
- **`--json`.** A describing command prints a human-readable report by
  default and, with `--json`, the same information as JSON for a tool.
- **Previews.** Every command that writes files has a flag that shows what
  it would do and writes nothing: `wf fmt --check`, `wf migrate --check`,
  `wf fix --dry-run`. `wf commands` names that flag for each command.

## Scaffolding and building

### `wf init`

Creates a new project directory with a config, a `.gitignore`, an
`AGENTS.md` and a small working application.

**Usage:** `wf init [OPTIONS] NAME`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `NAME` | a directory name | required | The project's directory and name. `wf init` refuses a directory that already exists. |
| `-t`, `--template` | `spa`, `static`, `pdf`, `slides` | `spa` | The starter: `spa` is an interactive app, `static` a pre-rendered marketing site, `pdf` a document and `slides` a deck. |
| `-l`, `--locales` | comma-separated language tags | none | Starts the project translated, in the languages listed; the first is the default. `spa` and `static` only. |

```bash
wf init my-site -t static
wf init my-app --locales en,ar
```

With `--locales`, every visible string of the starter goes through
`t("…")`, the config's `i18n` names the locales (and `meta.lang` the
default), the language switch has one entry for each locale, and
`src/translations/` holds a file per locale:

- the default locale's file holds the starter's English, the text its pages
  and tests are written in;
- another locale's file holds the starter's own translation where it has
  one (Arabic), and the same English otherwise, for you to translate.

A locale is a language tag — `en`, `ar`, `pt-BR`, `zh-Hant` — written in any
case and stored in the usual one. A tag named twice, an empty list, or a
code that is not a tag (`pt_BR`) is refused with what to write instead. A
`pdf` document or a `slides` deck is made in one language and refuses
`--locales`.

`AGENTS.md` is the language reference — the file a coding agent reads before
it changes a project. With it, an assistant asked to add a page writes the
WebFluent of the `wf` that made the project, flags and all, instead of
guessing. Its first line names the version it describes. A newer `wf`
writes a newer one, and an older project can take the current one from the
[repository](https://github.com/monzeromer-lab/WebFluent/blob/master/AGENTS.md).

`wf init` writes a shorter config than the full set of defaults, and turns
`build.csp` on.

See also: [Translate a site](https://webfluent.monzeromer.dev/recipes/translate-a-site.md), [Configuration](https://webfluent.monzeromer.dev/reference/config.md).

### `wf build`

Runs the whole compiler — parse, semantic checks, type checks, linters and
code generation — and writes the site to `build.output` (`./build`).

**Usage:** `wf build [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project to build. |
| `--format` | `human`, `json`, `sarif`, `github` | `human` | How findings are written (below). |
| `--deny-warnings` | — | off | Stops the build on a warning, as on an error. Use it in CI. |
| `--cache` | — | off | Reuses what earlier builds pre-rendered, and renders again only the pages an edit changed. |
| `--studio` | — | off | Builds for an editor's preview: node ids on every element, and `nodes.json`. |
| `--stats` | — | off | Prints what the build weighs. |
| `--allow-output-outside` | — | off | Lets `build.output` name a directory outside the project (below). |

```bash
wf build
wf build --format github --deny-warnings
```

**A build stays inside its project.** It reads only the project's own
files: a `data`, `image`, `api … from` or `i18n.dir` names a
path inside the project (an absolute path or a `..` is an error), and a
symbolic link that leads out of the project — or to nothing, or back into a
directory above it — is left out of every walk, with a warning naming it. A
link that stays inside is followed. It writes only inside its project too:
`build.output` is refused when it climbs out (`../site`, `/var/www`) or is a
link to somewhere else, and a link already inside the output is refused
rather than written through. `--allow-output-outside` is how the person
running the build lets it write elsewhere; a project's config cannot ask for
it. A build removes only the `.gz` files an earlier build wrote (it keeps
their names in `.wf-cache/precompressed.json`); any other `.gz` in the
output is left alone.

An error stops the build and prints the file, line, column, message and a
hint. A warning prints and the build goes on.

`--format` takes four values:

| Value | Output |
|---|---|
| `human` | Rendered findings on standard error. |
| `json` | One document on standard output: every finding with its code, place, hint, related places and fixes, and the counts. |
| `sarif` | SARIF 2.1.0, for GitHub code scanning and every other SARIF reader. |
| `github` | `::error file=…` annotations on standard output, which a workflow shows on the pull request; the rendered findings stay on standard error. |

With `json` or `sarif`, the build's progress goes to standard error, so
standard output holds the document alone.

`--cache` keeps what it renders in `.wf-cache/build/`. A page is taken from
the cache when its inputs are unchanged: its own text, every component it
reaches, and what every page reads — the shell, the stores, constants and
data, every page's title and route, the config, the translations and the
compiler. The output is byte for byte what a build without `--cache`
writes. A build with nothing to reuse fills the cache, and each build keeps
only the entries it used.

`--studio` stamps every element's root with `data-wf-node`, its node id,
which is the same in the pre-rendered page and the live one. An instance of
one of the project's components also carries `data-wf-call`: the calls that
made it, innermost first. `nodes.json`, written beside the site, maps each
id to where it is written (file, bytes, line), what it calls, its parent,
the loops and branches it is inside, and what its arguments read. A build
without `--studio` stamps nothing and removes any `nodes.json` it finds.

`--stats` prints every text file with its gzipped size and how much it
changed since the last build, the runtime modules the build kept and what
reached each one, and the design tokens it left out.

See also: [Diagnostics](https://webfluent.monzeromer.dev/errors.md), [Performance](https://webfluent.monzeromer.dev/about/what-a-build-ships.md).

### `wf serve`

Runs the development server: it builds once, serves the output, and
rebuilds and reloads on every save.

**Usage:** `wf serve [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project to serve. |
| `--host` | an address | `127.0.0.1` | Where to listen: this machine only, unless told otherwise. `0.0.0.0` lets a phone on the same network reach it. |
| `--allow-output-outside` | — | off | Lets `build.output` name a directory outside the project. |

```bash
wf serve
wf serve --host 0.0.0.0     # a phone on this network, at the address it prints
```

The server listens on `dev.port` (3000), on `127.0.0.1`: nothing else on the
network can reach it unless `--host` says so. It watches `src/`, `public/`
and the config, rebuilds on a change, and reloads every open tab. Each
rebuild is a `wf build --cache`, so only the pages the change reached are
rendered again.

It serves the build and nothing beside it: an address that climbs out
(`/../`, written plainly or as `%2e%2e`), holds a backslash or a NUL, or
reaches a link out of the build is a `404`. It answers only a request whose
`Host` is a name it is reached by — `localhost`, an address written as
numbers, the `--host` it was given, and with `--host 0.0.0.0` a `.local`
name — so a page on another site cannot point a name of its own at this
machine and read the build (DNS rebinding); any other gets a `421`. It
answers several requests at once, streams files rather than reading them
whole, and compresses a file once until it changes.

When a build fails, the last good page stays up and the error is drawn over
it — file, line, message and hint — until a save fixes it. The overlay is
served at `/__wf/dev.js`, and the build's state, as JSON, at `/__wf/status`.

Every page the server sends also carries a **stores** button. It shows what
each store holds, a log of every action with the state on each side of it,
and lets you put a store back to before an action ran
([Stores](https://webfluent.monzeromer.dev/learn/11-stores.md#looking-at-one-while-it-runs)). The service worker of
an offline site is removed under `wf serve`, so an edit is never hidden
behind its cache.

A request that is not a navigation — a `fetch`, a script, an image — for a
file the build did not write gets a `404`, as a host would send, and a line
in the terminal naming the file. Only a navigation gets the single-page
shell. (Answering a missing API with the shell's HTML makes the page fail to
parse it, far from the cause.)

A build under `wf serve` is a *development build*, and it adds these checks
to the running page:

- A response with a declared type — `resource users: [User] = …`, an `api`
  endpoint's `-> User` — is held to that type as it arrives. One that does
  not match takes the `error` arm as a `.parse` error naming the place:
  `$[1].name is missing`.
- A route no page answers (and no `path: "*"` page catches) shows a `404`
  box saying so. A deployed page shows nothing.
- An item of a `for` that throws shows where it is and what it threw, and
  the rest of the list draws. On a deployed page the item leaves a gap and
  the console says why.
- An effect that changes what it reads every time it runs is stopped after
  a hundred runs, with a warning, instead of overflowing the stack. A
  deployed page stops it too, silently.

See also: [`dev` in Configuration](https://webfluent.monzeromer.dev/reference/config.md#dev).

### `wf generate`

Writes a starter page, component or store.

**Usage:** `wf generate [OPTIONS] KIND NAME`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `KIND` | `page`, `component`, `store` | required | What to write, in `src/pages/`, `src/components/` or `src/stores/`. |
| `NAME` | a name | required | The declaration's name. |
| `-d`, `--dir` | a directory | `.` | The project to write into. |

```bash
wf generate page Pricing
wf generate component PriceCard
wf generate store Cart
```

A page gets a path from its name (`/pricing`), a title, a description and an
`h1`. The file is `.wfx` when the project is written in that layout.
`wf generate` never overwrites a file: when one exists, it says so and
writes nothing.

## Checking and testing

### `wf check`

Runs every check a build runs — the parser, the structure, the types, the
lints and the config — and writes nothing.

**Usage:** `wf check [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project to check. |
| `--format` | `human`, `json`, `sarif`, `github` | `human` | How findings are written, as for [`wf build`](#wf-build). |
| `--deny-warnings` | — | off | Fails on a warning too. |
| `--allow-output-outside` | — | off | Accepts a `build.output` outside the project, as [`wf build`](#wf-build) does; without it the check reports what the build would refuse. |

```bash
wf check
wf check --format sarif > wf.sarif
wf check --deny-warnings
```

`wf check` is fast enough for a pre-commit hook. It exits `1` on a finding
that would stop a build and `0` otherwise, and prints `No problems in NAME.`
when there are none. A check that needs the written output — the security
policy read back over the pages — runs only in `wf build`.

See also: [Diagnostics](https://webfluent.monzeromer.dev/errors.md).

### `wf fix`

Applies every fix the compiler is certain of — the fix the editor offers
first — and writes the files.

**Usage:** `wf fix [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project to fix. |
| `--code` | a code, such as `V07` | every code | Fixes only that code's findings. |
| `--dry-run` | — | off | Writes nothing, and names the files that would change. |

```bash
wf fix
wf fix --code V07
wf fix --dry-run
```

A fix that overlaps another, or that the first fixes make way for, is taken
on a second pass, and passes repeat until nothing more changes. `wf fix`
then lists what is left: findings with no certain fix, which a person
decides. `wf check` lists them too.

See also: [Diagnostics](https://webfluent.monzeromer.dev/errors.md).

### `wf config`

Reads and changes `webfluent.app.json`, and lists every setting it can hold.

**Usage:** `wf config schema [--json]`, `wf config get PATH`,
`wf config set PATH VALUE`, `wf config unset PATH`

| Subcommand or option | Value | Meaning |
|---|---|---|
| `schema` | — | Every setting: its type, the values it takes, its default and one sentence. |
| `schema --json` | — | The same, as JSON for a tool (an editor's settings window is built from it). |
| `get PATH` | a dotted path | What the project sets at `PATH`, or the default. |
| `set PATH VALUE` | a dotted path and a value | Writes `VALUE`: JSON (`true`, `3`, `["en", "ar"]`), or a word as a string. |
| `unset PATH` | a dotted path | Removes the setting, so its default applies. |
| `-d`, `--dir` | a directory (default `.`) | The project, for `get`, `set` and `unset`. |

```bash
wf config schema
wf config schema --json
wf config get build.base_path
wf config set build.ssg true
wf config set meta.lang ar
wf config unset build.ssg
```

`set` checks the value against the setting's type before it writes
anything. `set` and `unset` keep the rest of the file as it was written.
Under a map, the rest of the path is one key: `build.budget.app.js` sets the
`app.js` entry of `build.budget`.

See also: [Configuration](https://webfluent.monzeromer.dev/reference/config.md).

### `wf explain`

Prints a diagnostic code's entry from [Diagnostics](https://webfluent.monzeromer.dev/errors.md) in
the terminal.

**Usage:** `wf explain [CODE] [--json]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `CODE` | a code, such as `T05` | none | The code to explain. Without one, `wf explain` lists every code with its severity and title. |
| `--json` | flag | off | Print JSON, for tools: the code's title, family, severity, summary, whether `lints` may lower it, its section of the guide as Markdown and its page. Without a code, every code. |

```bash
wf explain T05
wf explain
```

An entry says what the code means, shows a program that draws it, what the
compiler prints, the fix, and the link to the guide.

### `wf test`

Runs the project's `test "…" { }` declarations.

**Usage:** `wf test [OPTIONS] [PATH]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `PATH` | a directory, or one test file | `.` | The project, or one file of its tests. |
| `--update` | — | off | Rewrites every snapshot with the current render. |

```bash
wf test
wf test tests/cart.wf
wf test --update
```

`wf test` runs every test under `tests/` (and in `src/`), or the tests in
one file. A test that clicks or types runs in headless Chrome, on a page
that links the project's scripts.

See also: [Testing](https://webfluent.monzeromer.dev/learn/16-testing.md).

### `wf verify`

Loads every page of a built project in a real browser and reports what went
wrong there.

**Usage:** `wf verify [OPTIONS] [PATH]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `PATH` | a directory | `.` | The project whose build to load. |
| `--json` | — | off | The same report, as JSON for a pipeline. |
| `--budget` | milliseconds | none | Fails a route whose first paint is slower than this. |
| `--returning-visitor` | — | off | Visits every route again as a returning reader (below). |
| `--allow-output-outside` | — | off | Lets `build.output` name a directory outside the project. |

```bash
wf build && wf verify
```

Chrome runs with its sandbox. Where it cannot start with one (a container
that gives it none) it is started again without, and `wf verify` says so;
`WF_CHROME_NO_SANDBOX=1` skips the first try. Its profile is a private
directory with a name nobody can guess, and the browser and the profile go
when `wf verify` or `wf test` ends — however it ends, a signal included.

`wf build` reports what the compiler wrote; `wf verify` reports what a
browser does with it. It builds nothing: it starts a headless Chrome, serves
what `wf build` wrote, and opens every route the project has — a `:param`
route at each value its `paths:` names, or at a placeholder (`/user/1`)
when it names none.

A route fails on any of these:

- an uncaught exception or a `console.error`;
- a file that did not arrive (a `fetch` the build has no file for is a
  `404`, not the page shell);
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

The columns are the first contentful paint, the number of elements in the
document once it settled, the bytes the page fetched, and the requests it
made. `--budget 400` fails a route that takes longer than 400 ms to paint.

`--returning-visitor` visits every route a second time as a reader who was
here before. Their storage holds what the previous build's pages kept: each
`persist` value's starting value as the build before this one wrote it,
which a build saves in `.wf-cache/` when it changes. A persisted shape that
changed without a `version:` then shows as the broken page it would be.

The report also names **the built-ins no page drew**. A component can be in
the registry, the tests and the documentation and still be broken on a
page. The classes the pages carried list what actually ran, so the
difference is what nothing has exercised. One project is not expected to
draw them all: the list is a fact about coverage, not a failure.

`wf verify` needs Chrome or Chromium on the machine, or `WF_CHROME` pointing
at one, and says so when there is none.

See also: [Testing](https://webfluent.monzeromer.dev/learn/16-testing.md).

### `wf audit`

Prints what a security review asks about the project. It reports and never
fails.

**Usage:** `wf audit [OPTIONS] [PATH]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `PATH` | a directory | `.` | The project to audit. |
| `--json` | — | off | The same report, as JSON. |

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

See also: [Security](https://webfluent.monzeromer.dev/about/security.md#audit-the-project).

## Formatting

### `wf fmt`

Formats every `.wf` and `.wfx` file under a path, or one file, in place.

**Usage:** `wf fmt [OPTIONS] [PATH]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `PATH` | a directory, or one file | `.` | What to format. |
| `--check` | — | off | Writes nothing, and fails when a file would change. Use it in CI. |
| `--stdout` | — | off | Prints the formatted text instead of writing it. |
| `--to` | `wfx`, `wf` | none | Changes the layout instead of formatting (below). |

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
- a brace one space from what it opens (`Row{` → `Row {`, `}else` →
  `} else`).

A line carried on by a trailing comma stays as written, and comments stay
where they are. The result is checked against the file's own tokens, so
formatting never changes what a file says.

`--to wfx` rewrites the project's `.wf` files in the indented layout, and
`--to wf` goes the other way. It changes the layout and nothing else, and
refuses a file whose indentation does not already follow its braces.

See also: [Two layouts: `.wf` and `.wfx`](https://webfluent.monzeromer.dev/learn/04-language-basics.md#two-layouts-wf-and-wfx).

## Rendering

### `wf render`

Renders one template with JSON data, to HTML, a PDF or a slide deck.

**Usage:** `wf render [OPTIONS] TEMPLATE`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `TEMPLATE` | a `.wf` or `.wfx` file, or a directory | required | The template. A directory is every template under it as one, with components shared. |
| `--data` | a JSON file | standard input | The data the template reads. |
| `-f`, `--format` | `html`, `html-fragment`, `pdf`, `slides` | `html` | A whole HTML document, the body only, a PDF, or a slide deck. |
| `-o`, `--output` | a file | standard output | Where to write the result. |
| `--page` | a page name | none | The page to render, when the template has several. |
| `--theme` | a theme name | none | Which of several `theme` declarations to use. |
| `--token` | `NAME=VALUE` | none | A design token over the theme's, such as `color-primary=#8B5CF6`. Repeat it for more tokens. |
| `--lang` | a language tag | the page's `lang:`, else `en` | The document's `<html lang>`. |

```bash
wf render invoice.wf --data invoice.json -f pdf -o invoice.pdf
echo '{"name":"Ada"}' | wf render greeting.wf
wf render templates/ --page Receipt --lang ar --data receipt.json
```

A template is held to the same checks as a build: a component nothing
declares, or a flag a component does not take, stops the render. The one
difference is that the names a template reads come from its data.

See also: [Render a template](https://webfluent.monzeromer.dev/recipes/render-a-template.md).

## Upgrading

### `wf migrate`

Carries a project written for an older WebFluent forward.

**Usage:** `wf migrate [OPTIONS] [PATH]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `PATH` | a directory, or one `.wf` file | `.` | What to migrate. |
| `--check` | — | off | Writes nothing, and reports what would change. |
| `--stdout` | — | off | Prints the migrated text of one file instead of writing it. |
| `--wfx` | — | off | Writes the migrated files in the indented layout, as `.wfx`. |

```bash
wf migrate --check
wf migrate
wf migrate --wfx
```

`wf migrate` makes three passes, in this order:

1. **4.1 → 4.2**, first, because 4.2 refuses `external`. A remote module
   becomes a `meta.scripts` entry. A local one becomes a plain script under
   `src/`, with its signatures as JSDoc, and `X.f(…)` becomes `f(…)`. An
   `external element` call becomes `Element("tag", …)`. The pass lists
   every `.js` already under `src/`, since each now ships on every page.
2. **2 → 3** rewrites every `.wf` under `src/` in place, with a note for
   anything that needed a decision. It changes spelling only: the migrated
   project builds to what it built before.
3. **3 → 4** changes what the compiler allows, so it runs over the project
   rather than the files. It adds every `env` name your pages already read
   to `public_env`, which preserves what the project did, and prints the
   list, because *public* means anyone who opens the site may read the
   value. It then names, with file and line, everything 4 refuses that 3
   allowed: an `on*` attribute, a `javascript:` or `data:` URL. Those have
   no automatic rewrite, because the old value was a string of JavaScript
   or a link that ran one. Finally it lists the changes that need no edit:
   a store is built on first read rather than at boot, a `persist` value
   follows the site's other tabs, and `WF.store`/`WF.host` were renamed.

See also: [Upgrading](https://webfluent.monzeromer.dev/about/upgrading.md).

## Describing the project

These commands print what the project and the compiler hold. Each one is
also a function in the `webfluent` crate, named in its entry and in
[Building a tool on the compiler](#building-a-tool-on-the-compiler).

### `wf commands`

Describes every `wf` command as data.

**Usage:** `wf commands [--json]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf commands
wf commands --json
```

For each command it gives what the command does, each option — its flag,
what it takes (`nothing` for a flag, a `word` from the values it lists, a
`number`, a `path`, `text`) and its default — what the command writes, and
the flag that shows it first and writes nothing (`fmt --check`,
`migrate --check`, `fix --dry-run`). Build a tool page, a command palette or
a script from this output instead of from a copy of this chapter. The
library has the same as `webfluent::commands` (with the `internals` feature, outside semver).

### `wf assets`

Lists every file of `public/` and where the project uses it, or what the
image pipeline makes of one picture.

**Usage:** `wf assets [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project. |
| `--unused` | — | off | Lists only the files nothing names. |
| `--plan` | a path in `public/`, such as `/hero.png` | none | Shows what the image pipeline writes for that picture. |
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf assets
wf assets --unused
wf assets --plan /hero.png
wf assets --json
```

For each file it prints its kind (image, font, video, audio, document, code,
data), its size, a picture's dimensions, and every place the project names
it: a prop the registry types as a file (`Image`'s `src`, `Video`'s `poster`
and `captions`), a page's `image:`, a `const`, an `image` declaration, a
`data` file's strings, a value in `webfluent.app.json`, a stylesheet's
`url(…)`, a project script, or a manifest. A file a browser or a host asks
for by name — `favicon.ico`, `robots.txt`, `.well-known/…` — counts as used.
Anything else nothing names counts as unused.

`--plan PATH` applies the project's `build.media` to one picture: each width
it writes, in each format, with the file's name and size — for example
*3 widths × 1 format · 3 files · about 84 kB*. A size is exact once
`wf build` has written the file (it is read from `.wf-cache/media/`), or
when the picture is small enough to encode on the spot. Otherwise it is
estimated from strips of the picture encoded at that width, and says
`about`. The plan names a format the build cannot write, with the reason,
and a width it skips because the file would be no smaller than the
original. With `--json`, the same sentences are in the `said` field.

The library has the same as `webfluent::build::assets::{list, plan}`. It reads
through a `Vfs`, so an editor's unsaved buffers count.

See also: [Add a picture](https://webfluent.monzeromer.dev/recipes/add-a-picture.md).

### `wf theme`

Shows the theme the build uses, as an editor's theme panel shows it.

**Usage:** `wf theme [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project. |
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf theme
wf theme --json
```

It lists every token by group (colour, spacing, font size, line height,
radius, shadow, transition, font, motion, and the tokens the theme adds)
with its value, the baseline's value, whether the theme declares or
inherits it, the dark theme's value when `theme.dark` names one, the
properties that draw from it, and every place the project names it. It then
lists the colour pairings the stylesheet composes, with their contrast and
what WCAG AA asks of each — a pairing that falls short is the `A13` the
build reports — and the tokens one theme of the dark pair declares and the
other does not.

See also: [Design tokens](https://webfluent.monzeromer.dev/reference/tokens.md).

### `wf motion`

Lists what an element's motion props can name.

**Usage:** `wf motion [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project. |
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf motion
wf motion --json
```

It lists:

- every animation `animate:` and `exit:` take: the engine's, each with its
  keyframes, the rule its stylesheet plays it by, whether an element leaves
  with it and whether it repeats; then the project's own `animation`
  declarations;
- the named eases, with the timing function each stands for;
- the three speeds and their lengths;
- the `Router`'s transitions;
- the project's default length and pacing (`motion` in the config, or the
  engine's);
- `data-wf-motion="reduce"`, the attribute an editor sets on the page's
  root to show the page as a reader who asked for less motion sees it.

The library has it as `webfluent::motion::catalogue` (with the `internals` feature, outside semver).

See also: [Motion defaults](https://webfluent.monzeromer.dev/recipes/motion-defaults.md).

### `wf context`

Prints what a coding agent working on the project is given, so nothing it is
told is hidden from you.

**Usage:** `wf context [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project. |
| `--open` | `FILE` or `FILE:START-END` | none | The file open in the editor, and its selection as byte offsets: `src/pages/Home.wf:120-180`. |
| `--guide` | `index`, `full` | `index` | The guide as each chapter's sections, or every chapter whole. |
| `--json` | — | off | Prints JSON; otherwise one Markdown document. |

```bash
wf context
wf context --open src/pages/Home.wf:120-180
wf context --guide full --json
```

It includes:

- every file of the project — not the build's output, `.git/`,
  dependencies or a `.env` file;
- the open file and its selection;
- the problems the build reports;
- every component a call can name, with its props, cases, events and slots;
- the theme's tokens;
- `webfluent.app.json`, with every `env` value that `public_env` does not
  list withheld;
- the guide: each chapter's title, blurb and sections to cite
  (`16 Forms and validation › What a value must be: validate`), or with
  `--guide full` the chapters whole;
- what was withheld, and why.

The library has it as `webfluent::agent::context` (with the `internals` feature, outside semver). An agent's edits go
through `webfluent::agent::changeset::ChangeSet`: a layer over the editor's
buffers that the compiler builds before anyone reads it. It refuses paths
outside the project, in `.git/`, in the build's output and `.env` files;
reads the config with its private values withheld and keeps them through a
write; and is accepted whole, by file or by hunk. `ChangeSet::edit` makes an
agent's anchored edits (`webfluent::agent::edit`): each `old_text` must be
found once in a file the set has read — exactly, then ignoring what ends a
line, then the indent, then typographic quotes and dashes — and a file that
moved under the set since it was read is refused until it is read again.
Text found twice names its lines; text found nowhere names the nearest.

### `wf translations`

Shows the project's translations as a table.

**Usage:** `wf translations [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project. |
| `--missing` | a locale | none | Lists only the keys that locale has no message for. |
| `--unused` | — | off | Lists only the keys nothing names. |
| `--plurals` | — | off | Prints CLDR's plural rules instead. |
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf translations
wf translations --missing ar
wf translations --unused
wf translations --json
wf translations --plurals
```

For each locale it prints its file, whether it is the default and whether it
is right to left, the plural forms CLDR's rules give it, how many keys have
no message in it, how many lack a form it needs, and how many messages have
placeholders the default locale's do not.

A key is unused when nothing names it: no `t()`, no key built at run time
that may be it, and no `validate` rule that shows it.

`--json` prints the whole table: every key, plural keys with each form, each
message in each locale with its placeholders and line, every place a `t()`
names the key, and the calls that name a key no locale has (`I01`).

`--plurals` prints each locale's forms; with `--json`, also the form each of
a run of counts takes.

The library has it as `webfluent::i18n::table` (with the `internals` feature, outside semver), whose `set_message` writes
one message back and keeps every other byte of the file.

See also: [Translate a site](https://webfluent.monzeromer.dev/recipes/translate-a-site.md).

### `wf scope`

Lists the names that can be read at one place in a source file — what an
editor's path picker and completion offer there.

**Usage:** `wf scope [OPTIONS] PLACE`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `PLACE` | `FILE:LINE` or `FILE:LINE:COL` | required | The place: `src/pages/Signup.wf:24` or `src/pages/Signup.wf:24:9`. |
| `-d`, `--dir` | a directory | `.` | The project. |
| `--data` | a JSON file | none | A template's data, listed after the scope. |
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf scope src/pages/Deploys.wf:24
wf scope src/pages/Deploys.wf:24:9 --json
wf scope src/templates/invoice.wf:3 --data data/invoice.json
```

The names come in the order a path picker lists them:

1. the page's or component's own `state` and `derived` values, and the form
   it binds;
2. its props, and a page's route parameters;
3. what the statements around the place bind — a loop's item and index, an
   `if let`, a `match` arm, a slot's fill, an action's parameters and the
   `let`s before the place — innermost first;
4. the members of every store it `use`s;
5. with `--data`, the template's data.

Each row has its type as the checker infers it, its shape (`bool`,
`number`, `text`, `moment`, `list`, `record`, `enum` …), whether it may be
`null`, where it comes from (`for d in live`, `member of Deploys`), and a
record's fields to go on into (`d.region.code`). The library has it as
`webfluent::scope` (with the `internals` feature, outside semver).

### `wf rules`

Lists what a `validate` block can hold, and every state the project holds to
a rule.

**Usage:** `wf rules [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project. |
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf rules
wf rules --json
```

It lists:

- every rule: what it takes between its parentheses, when it holds, the
  message it shows when none is written, the translation key
  (`form.minLength`) that replaces that message in a translated project,
  whether an empty value passes it, and whether it is checked last;
- the bounds a refined type implies (`below`, `after`, `before`);
- the patterns people often write, by name, each with its expression and a
  message;
- `Form(show:)`'s cases, in the reader's terms;
- `data-wf-validation="shown"`, the attribute a preview sets on the page's
  root to show every message at once;
- every state the project holds to a rule, with its rules in order (those
  its type implies first) and whether the page checks them as the reader
  types.

The library has it as `webfluent::validation` (with the `internals` feature, outside semver).

See also: [Forms](https://webfluent.monzeromer.dev/learn/14-forms.md).

### `wf components`

Lists every component a call can name — the built-ins first, then the
project's own — in one vocabulary.

**Usage:** `wf components [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project. |
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf components
wf components --json
```

For each prop it gives the kind of value it takes (`text`, `number`, `enum`
and its cases, `file` and what kind, `state`, `path`, a declared `record`, a
`list`, `date`, `color`, `money` …), whether it is the positional one, its
default as written, a range where its type declares one
(`Number(0..=100)`), and whether `.name` alone sets it. Then it gives the
component's events, its slots and what each hands its fill, its parts and
their owner, the attribute families its root takes, and whether it takes
children.

This is the table the checker reads a call against. An inspector draws a
control per prop from it, and an Insert panel knows what may go where. The
library has it as `webfluent::registry::schema::of_project_via` (with the `internals` feature, outside semver).

### `wf docs`

Writes a self-contained component gallery.

**Usage:** `wf docs [OPTIONS]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `-d`, `--dir` | a directory | `.` | The project. |
| `-o`, `--out` | a directory | `docs` | Where to write `index.html`. |

```bash
wf docs
wf docs -o gallery
```

The gallery shows every built-in — props, cases, flags, events, slots,
parts — and every component, enum, type, store and page the project
declares, with their `///` documentation.

### `wf registry`

Describes every built-in component: props, cases, flags, events, slots and
parts.

**Usage:** `wf registry [--json]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf registry --json | jq '.components[] | select(.name == "Button") | .props[].name'
```

The registry is the compiler's own table: what it prints is exactly what the
checker accepts.

See also: [Built-in components](https://webfluent.monzeromer.dev/reference.md).

### `wf types`

Describes what a project declares: enums, types, components, stores and
pages.

**Usage:** `wf types [OPTIONS] [PATH]`

| Option | Value | Default | Meaning |
|---|---|---|---|
| `PATH` | a directory | `.` | The project. |
| `--json` | — | off | Prints JSON, for a tool. |

```bash
wf types
wf types --json
```

It includes the names the project's scripts under `src/` declare, each with
the parameters and return type its JSDoc gives it.

## Environment variables

| Variable | Read by | What it does |
|---|---|---|
| `WF_INSTALL_DIR` | `install.sh`, `install.ps1` | Where the binary is installed (default `~/.webfluent/bin`). |
| `WF_VERSION` | `install.sh`, `install.ps1` | The release to install, such as `v5.11.0`, instead of the latest. |
| `WF_CHROME` | `wf test`, `wf verify` | The Chrome or Chromium to drive, when it is not found on its own. |
| `WF_BIN` | the Node binding | The `wf` binary to call. |
| any `PUBLIC_…` name | `wf build` | A value for `env.NAME` ([Environments and secrets](https://webfluent.monzeromer.dev/recipes/environments-and-secrets.md)). |

## Editors

`wf-lsp` is the language server: it runs the compiler's checks as you type
and answers your editor's questions about the code. [Editors](https://webfluent.monzeromer.dev/reference/editors.md)
says how to install it, what it answers, and how to set up VS Code, Zed,
Neovim, Helix, Emacs and any other editor with LSP support.

## Building a tool on the compiler

Every command above is a function in the `webfluent` crate, so an editor, a
site builder or an agent can call it in process instead of running `wf`.
Each one reads the project through a `webfluent::vfs::Vfs`: `FsVfs` reads
the disk, and `OverlayVfs::over_disk()` lays the editor's unsaved buffers
over it, so a tool reports the project as it is being edited.

| Function | What it is |
|---|---|
| `build::{build, check}` with `BuildOptions` | `wf build` and `wf check`, returning every line and every finding as a `Diagnostic` with its fixes; `BuildOptions::studio` is `--studio` |
| `diagnostics::fixes::fix_all` | `wf fix`'s fix-all, for one family in one file |
| `print`, `reprint`, `edit::{rewrite_element, rewrite_statement, rewrite_body, rewrite_declaration, rewrite_program}` | An element, a statement or a declaration changed in the tree and written back as the smallest edit, every other byte as it was; refused when it would not read back the same |
| `codegen::node_id::{build_node_map_with, layers}` | Every element's id, file, place, parent and what it is inside; the node at a caret; the Layers tree |
| `registry::schema::of_project_via` | `wf components` |
| `style_value` | Each style property's shape, a value read into parts (`read_box`), and the narrowest declaration written back (`edit_box`) |
| `themes::view::{view_via, set_token, reset_token}` | `wf theme`, and one token written in its place |
| `motion::catalogue` | `wf motion` |
| `validation::{must_be_via, set_rule, remove_rule, move_rule}` | `wf rules`, and a state's `validate` block written back one rule at a time |
| `scope::paths::at_via` | `wf scope` |
| `i18n::{plurals, table}` | `wf translations`: CLDR's plural rules, the table, and `set_message` |
| `assets`, `media::plan` | `wf assets` |
| `config::{schema, json_edit}` | `wf config`, and a setting written keeping the file's shape |
| `commands` | `wf commands` |
| `agent::{changeset, context, diff, edit}` | `wf context`, and a change set an agent writes to instead of the working tree, built before anyone reads it, with anchored edits |

## Next

[Configuration](https://webfluent.monzeromer.dev/reference/config.md) — every key of `webfluent.app.json`.
