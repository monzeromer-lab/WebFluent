# 48. Contributing

<!--
route: guide/contributing
group: help
blurb: How the compiler is put together, how to build and test it, and how to change the language, a built-in or this guide.
description: The repository, the compiler pipeline, building and testing, changing a built-in, the grammar or the runtime, and the guide's checks.
-->

WebFluent is one Rust crate — the compiler and the `wf` binary — with a
language server beside it, editor extensions, a small JavaScript runtime and
this guide, all in one repository:
[github.com/monzeromer-lab/WebFluent](https://github.com/monzeromer-lab/WebFluent).

## The repository

```text
src/
  lexer/, parser/, syntax/   source → tokens → tree; the .wfx layout in layout/
  sema/                      semantic checks, and the type checker (types.rs)
  linter/                    accessibility, contrast, unused, vocabulary, PDF and slide rules
  registry/                  every built-in's props, cases, flags, events, slots, parts
  codegen/                   JavaScript, HTML, CSS, the static paint, SEO, the service worker
  runtime/modules/           the browser runtime, one file per module
  themes/                    the baseline tokens and the built-ins' CSS
  template/                  the template engine behind `wf render`
  media/                     the image pipeline
  cli/                       each `wf` command
crates/wf-lsp/               the language server
editors/                     VS Code, Zed and the Tree-sitter grammars
md-docs/                     this guide
site/                        the documentation site, written in WebFluent
tests/                       Rust integration tests, tests/js (runtime), tests/browser (Chrome)
spec/                        design notes and plans
```

## The pipeline

```text
.wf / .wfx → lexer → parser → lower → semantic checks → type checker → linters → code generators
                                                                                 ├─ JavaScript + HTML + CSS
                                                                                 └─ PDF / slides
```

Every stage reads the **registry** for what a built-in takes, so a prop added
there is known to the checker, the linters, the language server, `wf docs`
and the [components reference](36-components-reference.md) at once.

## Building and testing

```bash
cargo build
cargo test --workspace
node --test tests/js/*.test.mjs
cargo fmt --check && cargo clippy --workspace -- -D warnings
```

The browser suites need Chrome: `cd tests/browser && npm ci && node offline.mjs`
(and `peer.mjs`). CI runs all of it on every push.

## Changing things

- **A built-in's props or flags**: edit `src/registry/builtins.rs`, its CSS in
  `src/themes/`, and its code generation; then run
  `python3 scripts/components-reference.py` to rewrite the reference.
- **The grammar**: the parser in `src/parser/v2.rs`, the formatter and the
  `.wfx` layout, and both Tree-sitter grammars in `editors/`. The guide's own
  code blocks are the grammar's test corpus.
- **The runtime**: a module in `src/runtime/modules/`; the manifest that says
  what reaches it is rewritten by a Rust test. Add a test in `tests/js/`.
- **A diagnostic**: give it a code, an entry with a failing example in
  [Diagnostics](39-diagnostics.md) — the test suite compiles every example and
  checks it draws its code.

## The guide

The guide lives in `md-docs/`, one file per chapter, and is the source of the
documentation site: `python3 scripts/site-from-guide.py` writes the site's
chapter pages, sidebar and search index from it. Each chapter starts with a
comment naming its route, its group in the sidebar, the line under its title
and its search description.

The test suite holds the guide to the compiler:

- every `wf` code block parses, type-checks and compiles without a warning;
- every `wf expect CODE` block draws exactly that code;
- every link between chapters, and every anchor, resolves;
- every declaration, keyword, built-in, browser value, built-in function,
  config key, command and diagnostic code appears in it;
- the tokens chapter is the baseline, and the configuration chapter's
  defaults are the compiler's;
- the site's pages are what the generator writes from the guide.

A code block that is deliberately incomplete says so with a `…`, and is only
checked for spelling.

## Releasing

[`RELEASING.md`](../RELEASING.md) in the repository has the steps: one tag
builds the binaries, the packages and the editor extensions.

## Next

[Introduction](01-introduction.md).
