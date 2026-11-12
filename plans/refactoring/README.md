# WebFluent refactoring: the plan

Status: **Phase 1 done** (the editors are their own repositories, linked
as submodules). Phase 0 is next. No compiler code has moved yet.
Last updated: 2026-11-12.

This file is the single source of truth for the refactor. It records what
the code looks like today, what it should look like, and the order to get
there. Every phase ends in a state that builds, passes `cargo test`, and
can be released.

The rules it applies come from the `rust-best-practices` and `rust-review`
skills (Rust API Guidelines, Effective Rust, Microsoft's Pragmatic Rust
Guidelines, the Google Rust Style Guide) and from how comparable Rust
toolchains are laid out: rust-analyzer, Ruff/ty, Typst, oxc, swc, Biome
and Gleam. Sources are in §12.

---

## 1. The one rule: behaviour does not change

A refactor that changes output is a bug. Before anything moves, Phase 0
records what the compiler produces, and every later PR is held to it:

- the same bytes for every project the repository builds (`site/`,
  `examples/`, `tests/fixtures/*`, `tests/migrate/corpus/*`);
- the same diagnostics (code, span, message, fix) for every fixture;
- the same public API of the `webfluent` crate (checked with
  `cargo public-api` / `cargo semver-checks`);
- the same `wf` commands and flags (`wf commands --json` is the contract).

Quirks that look wrong are flagged in an issue and left alone. Fixing them
is separate work.

---

## 2. What we found

### 2.1 Size and shape

| | |
|---|---|
| Crate | one package, `webfluent` (lib + `wf` bin), plus `crates/wf-lsp` |
| Source | 165 `.rs` files, **123,885 lines** in `src/` |
| Public surface | ~50 `pub mod`s at the crate root; nearly every internal is public |
| Integration tests | **78 test binaries**, 30,844 lines in `tests/` |
| Largest file | `src/codegen/js.rs`: 10,873 lines. One `impl JsCodegen` of **7,671 lines**; `emit_ui_element` alone is **1,177** |
| Next largest | `sema/types.rs` 7,547 (`impl Checker` 4,446), `parser/v2.rs` 5,140, `codegen/ssg.rs` 3,246 (`render_builtin` 898), `parser/parser.rs` 3,216, `template/mod.rs` 2,921 |
| Lint config | none: no `[lints]` table, no `clippy.toml`. `#![allow(dead_code)]` crate-wide in `lib.rs` |
| Toolchain verdict | `cargo fmt` clean, default `clippy` clean. `clippy::pedantic` gives ~3,500 warnings: 440 `uninlined_format_args`, 633 `must_use_candidate`, 292 `format_push_string`, ~400 lossy `as` casts, 151 missing `# Errors`, 89 identical match arms, 48 wildcard imports |
| Panics on paths | 616 `.unwrap()`, 404 `.expect(` outside tests (not all reachable; needs an audit, not a sweep) |
| Clones | 1,241 `.clone()` (audit hot paths only) |
| Editors | `editors/` holds the VS Code extension, the Zed extension and two tree-sitter grammars. Generated `parser.c` files are 7.4 MB of history; the working tree is ~2.5 GB with `node_modules` and the Zed build |

### 2.2 The module graph has cycles

The modules cannot be split into crates as they stand. Pairs that import
each other in production code (one example edge each way):

| Cycle | Edge → | Edge ← |
|---|---|---|
| codegen ↔ sema | `codegen/ssg.rs:2991` calls `sema::lower` | `sema/mod.rs:558` uses `codegen::url` |
| linter ↔ diagnostics | `linter/styles.rs:5` imports `diagnostics::Diagnostic` | `diagnostics/check.rs:123` calls `linter::structure` |
| diagnostics ↔ sema | `diagnostics/check.rs:55` | `sema/mod.rs:165` |
| codegen ↔ diagnostics | `codegen/csp.rs:265` | (check runs codegen checks) |
| codegen ↔ registry | `codegen/builtin.rs:536` | `registry/mod.rs:530` |
| registry ↔ sema | `sema/mod.rs:20` | `registry/json.rs:210` |
| registry → build → codegen → registry | `registry/schema.rs:581` calls `build::read_project_collecting_via` | |
| codegen ↔ i18n, codegen ↔ themes | `static_eval.rs:723`, `scoped_css.rs:487` | `i18n/mod.rs:18`, `themes/view.rs:124` |
| config ↔ linter / themes / i18n | `config/project.rs:1358` uses `linter::vocabulary::levenshtein` | `linter/secrets.rs:14`, `themes/resolve.rs:29`, `i18n/mod.rs:24` |
| build ↔ i18n, build ↔ themes | `build.rs:561`, `build.rs:575` | `i18n/table.rs:524`, `themes/view.rs:394` |
| i18n ↔ linter, linter ↔ themes | `i18n/table.rs:179`, `themes/view.rs:18` | `linter/project.rs:250`, `linter/contrast.rs:382` |
| lexer ↔ parser | `lexer/v2.rs:862` calls `parser::v2::splice_end` | |
| print ↔ reprint | `print.rs:2082` | `reprint.rs:24` |
| project_js ↔ sema, parser → project_js → sema → parser | `project_js/mod.rs:26`, `parser/ast.rs:100` | `sema/types.rs:341` |

Most of these are one helper in the wrong place (`levenshtein`, `url`,
`splice_end`) or a "run everything" function living in a low layer
(`diagnostics::check`, `registry::schema`). They are cheap to fix, and
fixing them is the real work of the split.

### 2.3 Duplication

- **Two expression interpreters.** `codegen/static_eval.rs` evaluates over
  its own `Static` value; `template/mod.rs` has a second one over
  `serde_json::Value` (`eval_expr` :945, `eval_binary_op` :2824) that falls
  back to the first for regex and ranges.
- **Two HTML renderers for built-ins.** `render_builtin`, `render_ui_element`
  and `render_labelled_input` exist in both `codegen/ssg.rs` and
  `template/mod.rs`.
- **Two semantic checkers.** `linter/semantic.rs` (undefined components,
  route targets, duplicates) runs beside `sema::check`, which checks the same
  things.
- **Three diagnostic shapes.** `diagnostics::Diagnostic`, the legacy
  `error::A11yWarning` / `VocabWarning` (80 uses), and a second `Diagnostic`
  struct in `studio.rs:79`.
- **Two grammars.** `lexer/lexer.rs` + `parser/parser.rs` are WebFluent 2.
  Only `migrate` uses them, but they sit in the compiler's front end.

### 2.4 Errors and global state

- `WebFluentError` mixes structured variants with five `String` variants
  (`CodegenError`, `ConfigError`, `IoError`, `EditError`, `NotFound`), and
  10 functions return `Result<_, String>` (`paged/mod.rs:246,794`,
  `patch.rs:259,298`, `reprint.rs:46`, `registry/schema.rs:557`,
  `linter/structure.rs:47`, …). `IoError(String)` drops the `io::Error`
  and its path.
- Hidden state: `thread_local!`s for the clock (`platform.rs:72`), output
  capture (`outfs.rs:30`), the media cache (`memory.rs:27`) and studio node
  ids (`template/mod.rs:1468`). These make the library's output depend on
  what the thread did before.

### 2.5 The editors are tied to the compiler's source

The editors cannot simply be moved out, because these read each other's
files directly:

| Coupling | Where |
|---|---|
| Zed's vocabulary check reads `src/lexer/token.rs` and `grammar.js` | `editors/zed/scripts/check-vocabulary.mjs` |
| A compiler test **writes** the grammar's test corpus | `tests/docs_parse.rs:858` (into `editors/tree-sitter-*/test/guide`) |
| Compiler tests read editor files | `tests/printer_corpus.rs:40,138`, `tests/reprint_corpus.rs:564`, `tests/stale_version.rs:46` |
| The grammar test parses the compiler repo's `.wf` files | `justfile` `grammar-test` |
| Zed pins a commit **of this repository** for its grammar | `editors/zed/extension.toml` (`path = "editors/tree-sitter-webfluent"`) |
| CI and release build and publish the extensions | `.github/workflows/ci.yml:49,277`, `release.yml:136` |
| Release scripts read the Zed pin | `scripts/ship.sh:116`, `scripts/release-preflight.sh:48` |
| The guide tells contributors where the editors are | `site/src/pages/guide/02-getting-started.wf`, `41-cli.wf`, `52-contributing.wf` |

---

## 3. Where we are going

### 3.1 Principles

1. **One published crate, built in layers.** `webfluent` is on crates.io
   (5.14.0) and people depend on it as a template engine, so it stays one
   package. Inside it, the modules are arranged in layers (§3.2), and a test
   fails the build when a module reaches up a layer. That gives the clean
   dependency graph a workspace split would give, without publishing a
   dozen internal crates (§3.3 explains why a split would force that).
2. **`webfluent`'s public API stays where it is.** Today's paths
   (`webfluent::Template`, `webfluent::parse_source`, …) keep working;
   modules that are internal become `pub(crate)` only where nothing outside
   uses them, checked by `cargo semver-checks`.
   `webfluent = { version = "5", default-features = false }` keeps giving
   the engine without the CLI, and `cargo install webfluent` keeps giving `wf`.
3. **The core does no I/O and holds no global state.** File access goes
   through `Vfs`/`OutFs` and the clock through a context value. This makes
   output reproducible, makes the LSP cheap, and keeps salsa possible later
   without designing for it now.
4. **One of each thing:** one diagnostic type, one expression evaluator,
   one built-in renderer, one grammar in the compiler.
5. **Library rules where there are outside users** (the facade and the
   template engine): typed errors, `#[non_exhaustive]`, docs with
   `# Errors`. **Application rules for the CLI**: `anyhow` with context,
   concrete types.

### 3.2 The layers

Layers go bottom to top. A module may use modules in its own layer's
lower rows and in lower layers, never above.

```
L7  lib.rs facade + main.rs (`wf`)                         [crates/wf-lsp: separate, unpublished]
L6  cli/            the `wf` commands, the dev server
L5  tools/          migrate (+ the WebFluent 2 grammar), testing, verify, browser,
                    audit, studio, agent, patch, init, generate, gallery
L4  build/          the pipeline, cache, media, assets, offline, openapi
L3  web/            JS/HTML/CSS/SSG codegen, the runtime JS, the one evaluator
    paged/          PDF and slides
    template/       data → web's evaluator and renderer, or paged
L2  check/          sema (scope, types, lower), linters, validate rules, check_project
L1  registry/       built-ins, icons
    project/        config, themes, i18n tables, project_js scan, data, style_value
L0  syntax/         lexer, parser, AST, .wfx layout, printer, fmt, edit
    diagnostics/    Diagnostic, Fix, codes, rendering
    base/           spans, line index, Vfs/OutFs, clock, text utilities (levenshtein)
```

| Layer module | Takes from today's `src/` | Notes |
|---|---|---|
| `base` | `vfs`, `outfs`, `platform`, `memory`, text helpers (`levenshtein`), span and line-index types | Uses nothing else in the crate |
| `diagnostics` | `diagnostics/{codes,format,render,fixes,allow}`, `error/` | `Diagnostic { code, severity, span, labels, help, fixes }`, `Fix { edits, applicability }`. The orchestrator `check.rs` moves **up** to `check` |
| `syntax` | `lexer/{v2,token}`, `parser/{v2,ast}`, `syntax`, `layout`, `print` + `reprint` (merged into `printer`), `fmt`, `edit` | The WebFluent 2 lexer and parser leave (→ `tools::migrate::legacy`) |
| `registry` | `registry/`, icon tables | Pure data plus lookups. `registry/schema.rs` (which reads a project) moves to `cli` |
| `project` | `config/`, `themes/` tokens and resolution, `i18n/` tables and plurals, `project_js/`, `data`, `style_value` | Loads and describes a project. Validation moves out to the linters |
| `check` | `sema/`, `scope`, `linter/`, `validation`, `diagnostics/check.rs` | The single `check_project` lives here. `linter/semantic.rs` merges into `sema` |
| `web` | `codegen/` except PDF, `runtime/`, `motion` | `static_eval` becomes the one evaluator, `ssg` the one built-in renderer |
| `paged` | `paged/`, charts and QR shared with web | Keeps the `pdf` and `bundled-fonts` cfgs |
| `template` | `template/` | A thin layer over `web` and `paged` |
| `build` | `build.rs`, `build/`, `media/`, `assets`, `codegen/offline`, `openapi` | |
| `tools` | `migrate/` (+ legacy grammar), `testing`, `verify`, `browser/`, `audit`, `studio`, `agent/`, `patch`, `init/`, `generate`, `gallery` | |
| `cli` | `cli/`, `commands`, dev server | Behind the `cli` feature, as now |

`wf-lsp` stays its own crate in `crates/` (it is not published, so it may
depend on `webfluent` by path), and so does a new `xtask` crate.

### 3.3 Why not a workspace of crates

A workspace split (`webfluent-syntax`, `webfluent-check`, … under
`crates/`) is what rust-analyzer, Ruff and Typst do, and it would turn a
layering mistake into a compile error and let the layers build in
parallel. But crates.io refuses to publish a crate whose dependencies are
not on crates.io. So if `webfluent` is published and depends on
`webfluent-syntax`, then `webfluent-syntax` must be published too, and so
on down. That is why Typst ships `typst-syntax`, `typst-layout` and the
rest. Since we do not want to publish internal crates, the split is off.

What we lose by staying one crate, and how we make up for it:

| A split would give | One crate gets it from |
|---|---|
| Cycles as compile errors | The layer test (Phase 3), run with `cargo test` |
| Narrow public surfaces | `pub(crate)` per layer, `unreachable_pub` lint, semver checks |
| Parallel, incremental builds | Smaller files and functions (Phase 5) help incremental builds; measure with `--timings` |

If `webfluent` ever stops being published to crates.io (shipped only as
the `wf` binary through GitHub releases and `install.sh`), the layers can
become crates with one `git mv` per layer, because Phase 3 already made
the graph acyclic.

### 3.4 Not doing now, on purpose

| Idea | Why not yet |
|---|---|
| A lossless CST (rowan) | Large rewrite of the parser, printer and LSP. `wf fmt`, `.wfx` conversion and `reprint` already work over the AST. Revisit only if trivia loss keeps causing bugs |
| salsa (query-based incremental compilation) | Still 0.x in 2026. Files here are checked mostly independently, so a per-file cache keyed by content hash covers the LSP. Principle 3 keeps it possible |
| Arena-allocated AST | Worth it for speed, not for clarity. Measure first |
| Rewriting the runtime JS | Out of scope. It moves into the `web` layer unchanged |

---

## 4. Phases

Each phase is a series of PRs that are each small enough to review in one
sitting. The phases are in dependency order. Phase 1 (editors) is
independent of Phases 2–6 and can run alongside them.

### Phase 0 — The safety net (first, 2–3 PRs)

1. **Output golden files.** A test (`tests/it/golden.rs`) builds every
   project in the repository and compares a manifest of output paths and
   SHA-256 hashes against `tests/golden/*.json`. `UPDATE_GOLDEN=1` rewrites
   them. This pins §1's first rule.
2. **Diagnostic snapshots** of every fixture with `insta`
   (`insta::glob!("fixtures/**/*.wf")`), recording code, span, message and
   fixes.
3. **API baseline.** Record `cargo public-api` for `webfluent` and run
   `cargo semver-checks` in CI.
4. **Workspace lints, at `warn`.** Add the table in §5.1 with
   `[lints] workspace = true` in every member. Nothing fails yet; this only
   makes the debt visible.
5. **Build-time baseline.** Record `cargo build --timings` (clean and
   incremental after touching `codegen/js.rs`) so Phase 4 can show it helped.

Done when: CI runs the golden, snapshot and semver checks on every PR.

### Phase 1 — The editors move to their own repositories (independent)

**Done (2026-11-12).** The three repositories exist, public, with the full
history of their folders (`git subtree split`), their own CI, and (VS Code)
their own release workflow:

- [tree-sitter-webfluent](https://github.com/monzeromer-lab/tree-sitter-webfluent):
  both grammars as `webfluent/` and `webfluentx/` under one
  `tree-sitter.json`; both parsers regenerate byte for byte as before.
- [webfluent-zed](https://github.com/monzeromer-lab/webfluent-zed): takes
  its grammars from tree-sitter-webfluent at a pinned `rev`.
- [webfluent-vscode](https://github.com/monzeromer-lab/webfluent-vscode):
  tags `v<version>` to release the `.vsix` and publish to the marketplaces.
  Its `VSCE_PAT` / `OVSX_PAT` secrets must be added to that repository.

In this repository `editors/` is three shallow HTTPS submodules. A plain
clone builds and tests without them (the grammar-corpus test skips);
CI checks them out. The vocabulary check moved to
`scripts/check-vocabulary.mjs`; the release preflight and `ship.sh` check
that the Zed extension pins the grammar commit linked here. One step was
done differently from the list below: the guide corpus is still compared
with the grammar submodule by `tests/docs_parse.rs`, not exported by an
`xtask`, since the submodule makes the files reachable.

**The repositories:**

| Repository | Holds | Ships to |
|---|---|---|
| `monzeromer-lab/tree-sitter-webfluent` | Both grammars (`webfluent` and `webfluentx`) in one repo, using `tree-sitter.json`'s `grammars` array; committed `src/parser.c`; corpus tests; bindings | npm and crates.io (`tree-sitter-webfluent`) |
| `monzeromer-lab/webfluent-zed` | The Zed extension: `extension.toml`, `languages/`, `snippets/`, the Rust crate | Zed's extension registry (`zed-industries/extensions`) |
| `monzeromer-lab/webfluent-vscode` | The VS Code extension | VS Code Marketplace / Open VSX |

The LSP (`wf-lsp`) **stays** in this repository: it depends on the
compiler's internals, and both extensions already download it from this
repository's releases.

**Steps:**

1. **Cut the couplings first, in this repository** (§2.5):
   - The compiler publishes its vocabulary as data: `wf registry --json`
     already has the components; add the keywords and write
     `vocabulary.json` as a release asset. The Zed check and the grammar
     read that file (pinned to a compiler version) instead of
     `src/lexer/token.rs`.
   - `tests/docs_parse.rs` stops writing into `editors/`. The guide corpus
     becomes an exported artifact: `cargo xtask export-corpus <dir>` writes
     every `.wf`/`.wfx` block of the guide plus the `site/`, `examples/` and
     `tests/` sources. The grammar repository's CI downloads it from the
     compiler's release (or a pinned commit) and parses it.
   - `tests/printer_corpus.rs`, `reprint_corpus.rs` and `stale_version.rs`
     drop their `editors/` paths. A version check for the extensions moves
     into each extension's own CI.
2. **Extract with history.** For each repository,
   `git filter-repo --subdirectory-filter editors/<dir>` on a fresh clone
   keeps every commit that touched it. The two grammars go into one
   repository as `webfluent/` and `webfluentx/`.
3. **Give each repository its own CI and release**, moved from
   `ci.yml` (`editors` job) and `release.yml` (VSIX job). Zed's
   `extension.toml` points `[grammars.*]` at the new grammar repository
   with its own `rev`. `just zed-pin-grammar` moves there too.
4. **Link them back as submodules**, under `editors/`:

   ```ini
   [submodule "editors/tree-sitter-webfluent"]
       path = editors/tree-sitter-webfluent
       url = https://github.com/monzeromer-lab/tree-sitter-webfluent.git
       shallow = true
   [submodule "editors/zed"]
       path = editors/zed
       url = https://github.com/monzeromer-lab/webfluent-zed.git
       shallow = true
   [submodule "editors/vscode"]
       path = editors/vscode
       url = https://github.com/monzeromer-lab/webfluent-vscode.git
       shallow = true
   ```

   HTTPS URLs (Zed's registry requires them, and they need no SSH key).
   `shallow = true` keeps clones small. Nothing in the compiler's build or
   tests needs the submodules checked out: `cargo build` and `cargo test`
   work on a plain `git clone`. Only an opt-in CI job
   (`submodules: recursive`) checks that the pinned editor commits still
   accept the current corpus.
5. **Delete `editors/*` from this repository** in the same PR that adds the
   submodules. Keep `exclude = ["editors/*"]` in the workspace and in
   `[package] exclude`.
6. **Update the docs and scripts:** `RELEASING.md`, `scripts/ship.sh`,
   `scripts/release-preflight.sh`, `justfile` (the `grammar-*` and `zed-*`
   recipes become `git submodule update --remote` plus a pointer), and the
   guide pages `02-getting-started`, `41-cli` and `52-contributing`.
   Add a `CONTRIBUTING` note: `git clone --recurse-submodules` to work on
   editors, `git submodule update --remote editors/zed` to bump one.

Done when: a plain clone builds and tests; each extension repository
builds, tests and releases on its own; `editors/` holds only the three
submodule pointers.

### Phase 2 — Repository hygiene (1–2 PRs)

- Move `spec/`, `reports/` and `research_notes/` under `plans/` or
  `docs/design/`. Delete the empty `examples_probe/`. Keep `md-docs/`
  (the compiler embeds it) but name its role in the README.
- Clear stale `.claude/worktrees/*` checkouts (local only, not tracked).
- Remove `#![allow(dead_code)]` from `lib.rs` and delete what the compiler
  then reports as dead, one module per commit. Items kept on purpose get
  `#[expect(dead_code, reason = "…")]`.
- Replace the 30 `#[allow(…)]`s with `#[expect(…, reason = "…")]` or a fix
  (11 `unused_imports`, 7 `too_many_arguments`, 3 `deprecated`, …).

### Phase 3 — Break the cycles (5–8 PRs)

Moving files and untangling them at the same time makes diffs
unreviewable. So first fix the graph **where the files are now**, then
move whole folders into their layers in Phase 4.

| Cycle | Resolution |
|---|---|
| `levenshtein` used by config, linter, sema | Move to `base::text` |
| `codegen::url` used by sema | Move to `base::url` (or `syntax`) |
| `lexer` → `parser::v2::splice_end` | Move `splice_end` into the lexer |
| `print` ↔ `reprint` | Merge into one `printer` module with `print` and `reprint` submodules |
| `diagnostics::check` calls linter/sema/themes/i18n | Move `check.rs` up into a new `check` module that sits above `sema` and `linter`. `diagnostics` keeps only the types, codes and rendering |
| linter/codegen import `diagnostics::Diagnostic` | Fine once the above is done: `diagnostics` is a leaf |
| `codegen` ↔ `sema` (`sema::lower` called by `ssg`, `scoped_css`) | Allowed direction (codegen above sema). Remove the other direction: the `url` helper above |
| `codegen` ↔ `registry` | `registry` becomes pure data. `codegen/builtin.rs` tables the registry reads (tags, roles) move into `registry` |
| `registry` → `build` (`schema.rs:581`) | Move `registry/schema.rs`'s project reading to the CLI's `describe`. The registry stops reading projects |
| `registry` ↔ `sema` (`registry/json.rs:210`) | The JSON export of the registry moves up to the CLI or `tools` |
| `config` ↔ `themes` / `i18n` / `linter` | `config` holds data and loading only. Its checks (unknown keys, secrets, theme names) move into `linter` as project lints |
| `codegen` ↔ `i18n` / `themes` | `i18n` and `themes` stop importing codegen (the helpers they use move to `base` or `project`) |
| `build` ↔ `i18n` / `themes` | The project-reading helpers they call move from `build` into `project` (one level down) |
| `i18n` ↔ `linter`, `linter` ↔ `themes` | The linter imports them; they stop importing the linter (`contrast` helpers move to `themes`) |
| `project_js` ↔ `sema`, `parser` → `project_js` | `project_js` returns its own `JsType`. `sema` converts it to `Type`. The parser stops knowing about project scripts |
| WebFluent 2 lexer and parser in the front end | Move to `migrate::legacy` |

**Enforcement:** a test (`tests/it/layers.rs`) reads every `use crate::X`
and `crate::X::` path and fails on an edge the layer table in §3.2
forbids. It starts with today's cycles as named exceptions, each with a
reason, and each PR in this phase deletes some. It stays for good: it is
what keeps the one crate layered.

Done when: the layer test passes with no exceptions.

### Phase 4 — Move the folders into their layers (1 PR per layer, bottom up)

1. Create the layer modules (`src/base/`, `src/syntax/`, …) and move
   today's folders into them with `git mv`, bottom layer first, one PR per
   layer. Each PR changes paths, not logic.
2. `lib.rs` re-exports every public item at its old path (`pub use`), so
   users see no change; `cargo semver-checks` proves it.
3. Narrow visibility as each layer lands: anything only its own layer
   uses becomes `pub(crate)` or `pub(super)`. Turn on `unreachable_pub`.
4. Point `wf-lsp` at the new paths (it may use internals the facade does
   not re-export; that is fine for an unpublished crate in this workspace).
5. Make the root `Cargo.toml` hold `[workspace.package]`,
   `[workspace.dependencies]` and `[workspace.lints]` for `webfluent`,
   `wf-lsp` and `xtask`.
6. Compare `cargo build --timings` with the Phase 0 baseline.

Done when: `src/` holds the layer folders and nothing else besides
`lib.rs` and `main.rs`, the layer test has no exceptions, and the golden and
semver checks still pass.

### Phase 5 — Inside the layers (parallel, one concern per PR)

These can run in any order once their layer exists. Each PR is
behaviour-preserving, held by Phase 0.

**`web`: break up `codegen/js.rs`.**

```
src/web/js/
  mod.rs          JsCodegen struct, `generate` (the entry point)
  expr.rs         emit_expr and friends
  statement.rs    emit_statement, emit_statement_dom
  store.rs        emit_store, emit_store_expr, emit_store_statement
  element.rs      emit_ui_element: a dispatch on the built-in's name
  builtins/       one file per built-in family (forms, overlays, navigation, media, layout)
  tables.rs       BROWSER_GLOBALS, BROWSER_VALUES and other constant tables
```

`emit_ui_element` (1,177 lines) becomes a `match` that calls one function
per built-in, each next to its siblings. The same treatment applies to
`ssg::render_builtin` (898 lines).

**One evaluator.** `static_eval` becomes `web::eval`. The template engine
converts its `serde_json::Value` data into `eval`'s value type once at the
boundary and stops interpreting expressions itself. Snapshot both paths
over the same expression corpus **before** merging, so any difference is
seen, not guessed.

**One built-in renderer.** `template` calls `ssg`'s renderer with a
"no hydration markers" option instead of carrying its own
`render_builtin`, `render_ui_element` and `render_labelled_input`.

**`check`: split `sema/types.rs`.** `Checker` becomes
`types/{mod, infer, methods, statement, element, match_, property}.rs`.
`linter/semantic.rs` merges into `sema` (it re-checks what `sema::check`
already checks); its tests move with it.

**Errors.**

- Each layer gets its own error type with `thiserror`, one variant
  per cause, `#[non_exhaustive]`, `source()` for wrapped causes.
  `IoError(String)` becomes `Io { path: PathBuf, source: io::Error }`.
- The facade's `WebFluentError` keeps its public variants (semver) and
  converts from the layer errors with `From`.
- The 10 `Result<_, String>` sites get real error types.
- `A11yWarning`, `VocabWarning` and studio's second `Diagnostic` are
  replaced by `diagnostics::Diagnostic`.
- `cli` uses `anyhow` with `.context(…)` and maps diagnostics to
  exit codes as today (1 = findings, 2 = could not run).

**State.** The four `thread_local!`s become fields of a context passed in:
`Clock` (for `now`), the output sink (`OutFs` already exists), the media
cache, and the studio id counter. The font database stays a `OnceLock`
(read-only after load; that is fine).

**Panics.** Audit `unwrap`/`expect` per layer, starting with the parser
and the template engine (they take outside input). A reachable one becomes
a diagnostic or an `Err`; an invariant gets `expect("why it holds")`.
Turn on `clippy::unwrap_used` per layer (`#![warn]` in its `mod.rs`) once its audit is done.

**API shape** (from the skill's checklist, applied per layer): `bool`
parameters in shared signatures become two-variant enums; `&String` /
`&Vec<T>` parameters become `&str` / `&[T]`; `get_` prefixes go; public
types derive `Debug` and the applicable std traits; magic numbers become
named `const`s; `format!` replaces `+` string building; clones that only
quiet the borrow checker go (audit the 1,241 in hot paths only: codegen,
eval, layout).

### Phase 6 — Tests and tooling

- **One integration test binary** (`tests/it/main.rs` with modules). 78 binaries link the whole compiler 78 times; one is about three
  times faster to build (matklad).
- **Fixture tests via `datatest-stable`**, one test per fixture file, so
  `cargo nextest` can filter and parallelise them. Run CI with nextest.
- **Snapshots with `insta`** where tests now compare long strings by hand.
- **Fuzzing** (`cargo fuzz`): lexer, parser, `fmt` round-trip
  (`parse(print(parse(x))) == parse(x)`), `.wf` ↔ `.wfx` round-trip,
  Markdown, sanitize.
- **Dependency hygiene in CI:** `cargo deny` (advisories, licenses, bans,
  sources) and `cargo machete`.
- **`cargo xtask`** for repository automation that is Rust-shaped
  (corpus export, golden updates, codegen of tables). The `justfile` keeps
  the short recipes and calls into it.
- **Dev profile** as Ruff uses: `opt-level = 1`,
  `debug = "line-tables-only"`, `opt-level = 3` for `insta` and `similar`.

### Phase 7 — Pedantic lints to `deny` in CI

Raise the §5.1 table from `warn` to CI-enforced (`cargo clippy -- -D
warnings`) one lint group at a time, mechanical ones first:
`uninlined_format_args` and `redundant_closure` (`cargo clippy --fix`),
then casts (`cast_possible_truncation` etc.: each one is a real question),
then docs (`missing_errors_doc`, `missing_panics_doc`) on the facade and
the template engine. Never `#![deny(warnings)]` in source.

---

## 5. Settings

### 5.1 Workspace lints

```toml
[workspace.lints.rust]
unsafe_code = "warn"            # paged/ and fonts may need it; each use gets // SAFETY:
unreachable_pub = "warn"
missing_debug_implementations = "warn"
unused_qualifications = "warn"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
# Pedantic lints that cost more than they give here:
module_name_repetitions = "allow"
too_many_lines = "allow"        # until Phase 5 splits the giants; then remove
must_use_candidate = "allow"    # 633 hits; add #[must_use] by hand where discarding is a bug
missing_errors_doc = "allow"    # turned on for the facade and template in Phase 7
similar_names = "allow"
# Restriction lints worth having:
dbg_macro = "warn"
print_stdout = "warn"           # allowed in the cli layer only
print_stderr = "warn"
exit = "warn"                   # allowed in the cli layer only
get_unwrap = "warn"
allow_attributes_without_reason = "warn"
```

Every member: `[lints] workspace = true`. The `cli` module allows
`print_stdout`/`exit` with `#![expect(…, reason = …)]`. Suppressions use
`#[expect(lint, reason = "…")]` on the narrowest item.

### 5.2 MSRV and edition

Keep `edition = "2024"` and `rust-version = "1.92"` in
`[workspace.package]`. `#[expect]` (1.81) and `cargo publish --workspace`
are both available.

### 5.3 Dependencies

Move every version into `[workspace.dependencies]`. Add `thiserror`
(library layers), `anyhow` (CLI), `insta`, `datatest-stable` and
`expect-test` (dev only). Run `cargo tree --duplicates` after Phase 4.

### 5.4 The source fingerprint

The root `build.rs` hashes all of `src/` into `WF_SOURCE_FINGERPRINT` so
the build cache never reuses another compiler's output. Staying one crate
keeps this working as it is; it only needs to keep walking `src/`
recursively after the moves (it does).

---

## 6. Order and size

| Phase | PRs (estimate) | Can start | Blocks |
|---|---|---|---|
| 0 Safety net | 2–3 | now | everything after it |
| 1 Editors out | 5–6 here + set-up of 3 repos | now, alongside 0 | nothing |
| 2 Hygiene | 1–2 | after 0 | 3 |
| 3 Break cycles | 5–8 | after 2 | 4 |
| 4 Layer folders | ~12 (one per layer) | after 3 | 5 |
| 5 Inside the layers | ~15, parallel | per layer, after 4 | 7 |
| 6 Tests and tooling | 4–6 | after 4 | — |
| 7 Lints to deny | 5–10, small | after 5 per layer | — |

Releases can go out between any two PRs.

**Phases 3 and 4 move files.** A branch that edits `src/codegen/js.rs`
while another moves it to `src/web/js/` will conflict, and git does not
always follow a file that moved and changed in the same merge. So while a
move PR is open, keep other branches short: merge or rebase them before
the move, not across it. This includes parallel Claude sessions in
`.claude/worktrees/`. Each move PR is small and merges the same day, so
this is a few hours per PR, not a freeze.

---

## 7. Risks

| Risk | Mitigation |
|---|---|
| A move changes output by accident | Phase 0 golden hashes and snapshots, on every PR |
| The facade breaks a user's `use webfluent::…` path | `cargo semver-checks` and the `public-api` baseline in CI |
| Merging the two evaluators changes template output | Snapshot both over one corpus first; differences become issues, decided one at a time |
| Long-running branches conflict with the moves | One PR per layer, merged the same day; rebase other branches before it, not across it |
| Submodules confuse contributors | Nothing in the compiler needs them. `CONTRIBUTING` explains `--recurse-submodules` for editor work |
| Zed's registry rejects the new extension repo | Follow its prerequisites: HTTPS, public, allowed licence (MPL-2.0) at the root, id without "zed" |
| A layer test is easier to ignore than a compiler error | It runs in `cargo test` and CI; exceptions need a `reason` in the test's table |

---

## 8. What "done" looks like

- `src/` arranged in layers; no cycles, held by the layer test.
- No file over ~1,500 lines; no function over ~150 except declarative
  tables.
- One `Diagnostic`, one evaluator, one built-in renderer, one grammar in
  the compiler.
- `cargo clippy --workspace --all-targets -- -D warnings` clean with the
  §5.1 table.
- No `Result<_, String>`, no `thread_local!` below the `cli` layer.
- One integration test binary, fixtures under nextest, fuzz targets running
  nightly.
- `editors/` is three submodules; each extension builds and releases from
  its own repository.
- Same output bytes as before, proven by the golden files.

---

## 9. Where to start

Phase 0, item 1 (the golden-file test), and in parallel Phase 1, step 1
(cutting the editor couplings in this repository). Neither changes the
compiler, and both make every later step safe.

---

## 10. Checklist per PR

- [ ] `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test` clean
- [ ] Golden hashes and diagnostic snapshots unchanged (or the change is the point and explained)
- [ ] `cargo semver-checks` clean for `webfluent`
- [ ] No new `unwrap()` on input-reachable paths, no new `#[allow]` without `reason`
- [ ] Moves use `git mv`; logic changes and moves are separate commits

---

## 11. Decisions (settled 2026-11-12)

| Question | Decision |
|---|---|
| Names of the editor repositories | `tree-sitter-webfluent`, `webfluent-zed`, `webfluent-vscode` under `monzeromer-lab` (created) |
| One grammar repository or two | One: the `.wfx` grammar is generated from the `.wf` one's `grammar.js` and includes its scanner, so they cannot live apart |
| Publish internal crates | No. `webfluent` stays one published crate, built in layers (§3.3) |
| Keep `wf-lsp` here | Yes |
| Pause feature work | No pause; keep other branches short around each move PR (§6) |

---

## 12. Sources

- Rust skills used: `rust-best-practices` and `rust-review` (beautiful-rust 1.5.0), digests of the Rust API Guidelines, Effective Rust, Rust Design Patterns, Microsoft's Pragmatic Rust Guidelines and the Google Rust Style Guide.
- matklad, *Large Rust Workspaces* — https://matklad.github.io/2021/08/22/large-rust-workspaces.html
- rust-analyzer architecture — https://rust-analyzer.github.io/book/contributing/architecture.html
- matklad, *Delete Cargo Integration Tests* — https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html
- matklad, *How to Test* — https://matklad.github.io/2021/05/31/how-to-test.html
- matklad, *Fast Rust Builds* — https://matklad.github.io/2021/09/04/fast-rust-builds.html
- matklad, *Resilient LL Parsing* — https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html
- Ruff workspace and lints — https://raw.githubusercontent.com/astral-sh/ruff/main/Cargo.toml
- Ruff diagnostics and fixes — https://docs.rs/ruff_diagnostics/
- oxc linter rules and fixes — https://oxc.rs/docs/contribute/linter/adding-rules.html
- swc_core facade features — https://lib.rs/crates/swc_core/features
- Biome architecture — https://biomejs.dev/internals/architecture
- Gleam compiler layout — https://github.com/gleam-lang/gleam
- salsa — https://docs.rs/crate/salsa/latest
- Cargo `[lints]` — https://doc.rust-lang.org/cargo/reference/lints.html
- insta `glob!` — https://insta.rs/docs/advanced/
- datatest-stable — https://github.com/nextest-rs/datatest-stable
- cargo-deny — https://embarkstudios.github.io/cargo-deny/
- tree-sitter `init` and `tree-sitter.json` — https://tree-sitter.github.io/tree-sitter/cli/init
- Zed language extensions — https://zed.dev/docs/extensions/languages
- Zed publishing prerequisites — https://zed.dev/docs/extensions/publishing/prerequisites
- `.gitmodules` (`shallow`, `update`) — https://git-scm.com/docs/gitmodules
