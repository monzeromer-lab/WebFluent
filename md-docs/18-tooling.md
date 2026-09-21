# 18. Tooling

One binary, `wf`, does everything: scaffolds, builds, serves, formats,
tests, renders and describes. A language server, `wf-lsp`, brings the same
checks into the editor.

## The commands

| Command | Does |
|---|---|
| `wf init NAME [-t spa\|static\|pdf\|slides]` | Creates a project from a template |
| `wf build [-d DIR]` | Compiles `src/` to the output directory; fails on errors, prints warnings |
| `wf serve [-d DIR]` | Builds, serves on `dev.port` (3000), rebuilds on every save and reloads the browser |
| `wf fmt [PATH] [--check] [--stdout] [--to wf\|wfx]` | Formats source files; converts layouts |
| `wf test [PATH] [--update]` | Runs `test "…" { }` declarations against their expectations and snapshots |
| `wf generate page\|component\|store NAME` | Adds a file with a starter declaration |
| `wf render FILE --data JSON [-f html\|html-fragment\|pdf\|slides] [-o OUT]` | Renders a template with data ([chapter 17](17-outputs.md#templates-rendering-with-data-on-a-server)) |
| `wf migrate [PATH] [--check] [--stdout] [--wfx]` | Rewrites WebFluent 2 sources to 3 |
| `wf docs [-o DIR]` | Writes a component gallery — every built-in and everything the project declares |
| `wf registry [--json]` | Describes every built-in: props, cases, flags, events, slots, parts |
| `wf types [--json]` | Describes what the project declares: enums, types, components, stores, pages |

## `wf serve`

The development loop. It builds once, serves the output, watches `src/`,
`public/` and the config, rebuilds on a change and reloads every open tab.
A build that fails leaves the last good page up and draws the error over
it — file, line, message and hint — until the next save fixes it. The
overlay is served at `/__wf/dev.js`, the build's state at `/__wf/status`
(JSON), for anything else that wants to know.

## `wf build`

Runs the whole pipeline: parse → semantic checks → type checks → linters →
codegen. Errors stop the build with the file, line, column, message and a
hint; warnings print and the build goes on. The exit code is non-zero on
an error, so a CI step is just `wf build`.

```bash
wf build && rsync -a dist/ user@host:/var/www/site/
```

## Diagnostics

Every finding carries a code. The families:

| Codes | Family | Examples |
|---|---|---|
| `T01`–`T11` | Types ([chapter 10](10-types.md#the-checks)) | Wrong type for a prop, unknown field, unwrapped optional, wrong arity |
| `A01`–`A15` | Accessibility | `A01` image without `alt`; `A03`/`A04` control without a label; `A05`–`A07` empty button/link/heading; `A10` table without a header row; `A11` heading level skipped; `A12` page without an `h1`; `A13` theme contrast below WCAG AA; `A14` a `role` whose children lack the role it requires; `A15` an `aria-label` that does not start with the visible text |
| `U01`–`U05` | Unused | `U01` state, `U02` derived, `U03` component, `U04` store member, `U05` action — declared and never read. A name starting with `_` is exempt |
| `V01`–`V02` | Vocabulary | `V01` a bare word in argument position that nothing declares (a flag is `.word`); `V02` a class the stylesheet does not define |
| Semantic | Structure | An unknown component, prop, flag or event; a fill for an undeclared slot; two positional arguments; a route claimed twice; an `emit` of an undeclared event |

Accessibility findings are warnings by default; the rules are in
[spec/ACCESSIBILITY_SPEC.md](../spec/ACCESSIBILITY_SPEC.md).

## `wf fmt`

Formats every `.wf` and `.wfx` under a path (or one file) to the canonical
layout: four-space indents, one statement per line, clause order `style`
→ `transition` → `on` → fills → children, spaces after commas and around
operators. `--check` fails without writing, for CI; `--stdout` prints one
file. `--to wfx` rewrites a `.wf` project in the indented layout (and
renames the files), `--to wf` the other way; both are lossless.

## `wf test`

A test declaration renders a body with the project's components, stores,
types and constants at hand, and holds the result to what it expects:

```wf
component PriceTag(_ amount: Number, sale: Bool = false) {
    Row(gap: .sm, align: .center) {
        Text(format(amount, .currency)).bold
        if sale { Badge("Sale").danger }
    }
}

test "a price tag shows the amount as currency" {
    PriceTag(1234.5)
    expect "$1,234.50"
    expect not "Sale"
}

test "a sale tag carries the badge" {
    PriceTag(10, sale: true)
    expect "Sale"
}

test "a list renders each row"(data: { rows: [{ name: "a" }, { name: "b" }] }) {
    for r in rows { Text(r.name) }
    expect "a"
    expect "b"
}
```

- Tests live in `tests/*.wf` (or beside the code in `src/`).
- `expect "text"` must appear in the rendered HTML; `expect not "text"`
  must not.
- `data: { … }` supplies values by name; a `state` in the body is seeded
  with its initial value.
- Each render is also compared to `tests/__snapshots__/<file>/<test>.html`,
  written when missing; `--update` rewrites them after an intended change.

`wf test` exits non-zero on a failure, with a diff.

## `wf generate`

```bash
wf generate page Pricing
```

writes `src/pages/Pricing.wf` with a page declaring `path: "/pricing"`, a
title and an `h1`; `component` and `store` likewise. It is
a starting point, not a framework — edit freely.

## `wf docs`, `wf registry`, `wf types`

`wf docs` writes `docs/index.html`: a gallery of every built-in with its
props, cases, flags, events, slots and parts, and every component, enum,
type, store and page the project declares, with their docs (`///`
comments). `wf registry --json` and `wf types --json` print the same data
for tools — a design tool, a generator, a script:

```bash
wf registry --json | jq '.components[] | select(.name == "Button") | .props[].name'
```

The registry is the compiler's own table: what it prints is exactly what
the checker accepts.

## Editors

`wf-lsp` is the language server, built from `crates/wf-lsp`
(`cargo install --path crates/wf-lsp`, or the binary from a release). It
gives:

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
`zed: install dev extension` pointing at that folder, or from the
extension registry. It brings Tree-sitter highlighting, outline, brackets,
indentation, snippets and the language server (found on `PATH` or
downloaded from the latest release), for `.wf` and `.wfx`.

**VS Code**: `editors/vscode` — TextMate highlighting and the language
server. Open the folder and run `Extensions: Install from VSIX` on the
packaged extension, or symlink it into `~/.vscode/extensions`.

Any other editor with LSP support runs `wf-lsp` over stdio.

## Project layout, as a reminder

```
webfluent.app.json    config
src/                  .wf / .wfx / .md pages, components, stores, themes, .css, translations/
public/               copied to the output root
tests/                wf test declarations and __snapshots__/
dist/                 the build (build.output)
```

## Next

[Components reference](19-components-reference.md), then the
[cookbook](20-cookbook.md).
