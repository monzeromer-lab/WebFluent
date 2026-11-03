# 49. Contributing

<!--
route: guide/contributing
group: help
blurb: How the compiler is put together, how to build and test it, and how to change the language, a built-in or this guide.
description: Build and test the compiler, find your way around the repository and its pipeline, and change a built-in, the grammar, the runtime or this guide.
-->

This chapter is for working on WebFluent itself: the compiler, the
runtime, the editor support or this guide. It tells you how to build and
test the project, where each part lives, and what to update when you change
one.

WebFluent is one Rust crate — the compiler and the `wf` binary — with a
language server beside it, editor extensions, a small JavaScript runtime
and this guide, all in one repository:
[github.com/monzeromer-lab/WebFluent](https://github.com/monzeromer-lab/WebFluent).

## Before you start

You need:

- Rust 1.92 or later (the crate uses the 2024 edition).
- Node.js 20, for the runtime's tests and the browser suites.
- Chrome or Chromium, for the browser suites.

Clone the repository:

```bash
git clone https://github.com/monzeromer-lab/WebFluent
cd WebFluent
```

## Build and test

Build the compiler, then run the Rust tests, the runtime's tests and the
lints that CI runs:

```bash
cargo build
cargo test --workspace
node --test tests/js/*.test.mjs
cargo fmt --check && cargo clippy --workspace -- -D warnings
```

The browser suites check what only a real browser can — a service worker,
WebRTC, layers, keys, gestures and more. They drive the release build, so
build it first, then run them from `tests/browser`:

```bash
cargo build --release
cd tests/browser
npm ci
npm test
```

`npm test` runs twelve suites, starting with `offline.mjs` and `peer.mjs`;
each is also a script you can run alone, such as `node offline.mjs`. CI
runs all of it on every push, and a few browser suites more.

## The repository

Each part of the project has its own folder:

```text
src/
  lexer/, parser/, syntax/   source → tokens → tree; the .wfx layout in layout/
  sema/                      semantic checks, and the type checker (types.rs)
  linter/                    accessibility, contrast, unused, vocabulary, PDF and slide rules
  registry/                  every built-in's props, cases, flags, events, slots, parts
  codegen/                   JavaScript, HTML, CSS, the pre-rendered page, SEO, the service worker
  runtime/modules/           the browser runtime, one file per module
  themes/                    the baseline tokens and the built-ins' CSS
  template/                  the template engine behind `wf render`
  media/                     the image pipeline
  diagnostics/               every finding's code (codes.rs), checks, fixes and output
  print.rs, reprint.rs       the tree written back as source, and the smallest edit
  edit.rs                    an editor's edits: change the tree, write only what changed
  style_value.rs             a style value's shape, read into its parts and written narrowest
  cli/                       each `wf` command
crates/wf-lsp/               the language server
editors/                     VS Code, Zed and the Tree-sitter grammars
md-docs/                     this guide
site/                        the documentation site, written in WebFluent
tests/                       Rust integration tests, tests/js (runtime), tests/browser (Chrome)
spec/                        design notes and plans
```

## The pipeline

A source file passes through these stages, in order:

```text
.wf / .wfx → lexer → parser → lower → semantic checks → type checker → linters → code generators
                                                                                 ├─ JavaScript + HTML + CSS
                                                                                 └─ PDF / slides
```

Every stage reads the **registry** for what a built-in takes. So a prop
added there is known at once to the checker, the linters, the language
server, `wf docs` and the
[components reference](37-components-reference.md).

## Change a built-in's props or flags

1. Edit the built-in's entry in `src/registry/builtins.rs`.
2. Update its CSS in `src/themes/` and its code generation in
   `src/codegen/`.
3. Rewrite the [components reference](37-components-reference.md) from
   the registry, with the command below.

```bash
python3 scripts/components-reference.py
```

## Change the grammar

A change to the grammar touches every place that reads or writes source:

- the parser, in `src/parser/v2.rs`;
- the formatter and the `.wfx` layout;
- the printer, in `src/print.rs`;
- both Tree-sitter grammars, in `editors/`.

The guide's own code blocks are the grammar's test corpus.
`tests/printer_corpus.rs` prints every statement of every project in the
repository back in its place and holds it to the same tree, and
`tests/reprint_corpus.rs` holds every edit to changing only its own bytes.

## Change the runtime

Edit or add a module in `src/runtime/modules/`, and add a test for it in
`tests/js/`. The manifest that says what reaches each module,
`src/runtime/modules/manifest.json`, is written by a Rust test, so you do
not edit it by hand.

## Add a diagnostic

1. Give the finding a code, a severity and a title in
   `src/diagnostics/codes.rs`.
2. Add an entry for it to [Diagnostics](40-diagnostics.md), with a program
   that draws the code. The test suite compiles every example and checks
   that it draws its code, and a code with no entry in the guide fails a
   test.

## Change this guide

The guide lives in `md-docs/`, one file per chapter, and is the source of
the documentation site: `python3 scripts/site-from-guide.py` writes the
site's chapter pages, sidebar and search index from it. Each chapter
starts with a comment naming its route, its group in the sidebar, the line
under its title and its search description. `spec/DOCS_STYLE.md` is the
house style every chapter follows.

The test suite holds the guide to the compiler:

- every `wf` code block parses, type-checks and compiles without a
  warning;
- every `wf expect CODE` block draws exactly that code;
- every link between chapters, and every anchor, resolves;
- every declaration, keyword, built-in, browser value, built-in function,
  config key, command and diagnostic code appears in it;
- the tokens chapter is the baseline, and the configuration chapter's
  defaults are the compiler's;
- the site's pages are what the generator writes from the guide.

A code block that is deliberately incomplete says so with a `…`, and is
only checked for spelling.

## Release a version

[`RELEASING.md`](../RELEASING.md) in the repository lists the steps: one
tag builds the binaries, the packages and the editor extensions.

## Next

[Introduction](01-introduction.md).
