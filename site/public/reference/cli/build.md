# wf build

> Build the project: a site or app (HTML, CSS, JS), a PDF, a slide deck or custom elements

The page: https://webfluent.monzeromer.dev/reference/cli/build

Build the project: a site or app (HTML, CSS, JS), a PDF, a slide deck or custom elements

```bash
wf build [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--stats` | Print what the build weighs: every file, gzipped, and the runtime modules it kept and left out |
| `--format FORMAT` | How findings are written: `human` (default), `json`, `sarif` (code scanning) or `github` (annotations on a pull request) |
| `--deny-warnings` | Let a warning stop the build as an error does |
| `--cache` | Reuse what earlier builds rendered, kept in `.wf-cache/build/`: only the pages an edit changed are rendered again |
| `--studio` | For an editor's preview: stamp every element with its node id (`data-wf-node`) and write `nodes.json`, from each id to its source |
| `--allow-output-outside` | Let `build.output` name a directory outside the project; without it a build writes only inside the project |

**Writes:** the site, in the output folder

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
