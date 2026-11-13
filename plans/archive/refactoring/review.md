# WebFluent Rust review

Date: 2026-11-12. Commit reviewed: `06336cae` (master).

This is the review the refactoring plan ([README.md](README.md)) builds on.
It was done with the Rust skills installed from
[morganlinton/BeautifulRust](https://github.com/morganlinton/BeautifulRust)
(`npx skills`): **`rust-review`** for the process and the report format, and
**`rust-best-practices`** with its reference digests (Rust API Guidelines,
Effective Rust, Microsoft's Pragmatic Rust Guidelines, the Google Rust Style
Guide, Rust Design Patterns) for the rules. Each finding cites its rule:
`C-*` are Rust API Guidelines codes, `M-*` Microsoft's, "Item N" Effective
Rust's, "§N" the `rust-best-practices` skill's sections.

`webfluent` is a published **library** (a template engine used by servers)
as well as the `wf` CLI, so the library rules apply. Edition 2024, MSRV 1.92.

## How it was done

1. `cargo clippy -W clippy::pedantic` over the workspace (step 2 of the
   `rust-review` process): 3,062 findings.
2. The same code under the skill's recommended lint config, BeautifulRust's
   `lints/lints.toml` and `lints/clippy.toml`: 7,251 unique findings in
   `src/` and `crates/` (tests and examples excluded). See §4.
3. Five areas reviewed in full against the skill: the front end (lexer,
   parser), the template engine and public API, the JS code generator, the
   type checker, and errors and global state across the crate.
4. **✅ means reproduced by running** a debug build of `06336cae` on a
   small input (the input is in the Proof column). **"read"** means
   confirmed by reading the cited code, not run.

## 1. Bugs found — fix these before refactoring

These are behaviour bugs, not style. They come first because the refactor
must not change behaviour (README §1), and each needs a failing test before
its fix (§11 of the skill).

| # | Bug | Where | Proof | Rule |
|---|---|---|---|---|
| B1 | **One character hangs the compiler until it runs out of memory.** `℘` (U+2118) or `℮` (U+212E) are `XID_Start` but not alphanumeric: the lexer dispatches them to `read_word`, which consumes nothing and returns `Identifier("")`, forever. Takes down `wf`, the LSP, the studio and `Template` | `lexer/v2.rs:286`, `:387-396`, `:118-134` | ✅ `page P(path: "/") { Text(℘) }` → `wf check` and `wf render` abort at a 2 GB cap | C-VALIDATE; Item 18 |
| B2 | **Deep nesting overflows the stack** (an abort, which `catch_unwind` cannot stop). No depth limit in the parser (`parse_unary`, `(`, string splices), the checker (`infer` ↔ `method_call_on` ↔ `property`) or the template engine's component expansion | `parser/v2.rs:3548-3558`, `:3769`, `:3370-3381`; `sema/types.rs:3070`, `3861`; `template/mod.rs:1545-1571` | ✅ `derived x = !!!…!true` (200k `!`) → abort | M-PANIC-IS-STOP; §11 (fuzz parsers) |
| B3 | **A state named in a non-Latin script never repaints.** `is_reactive_expr` scans the emitted JS byte by byte (`bytes[i] as char`), so `_عدد()` is not seen as a signal read. AGENTS.md promises names in any script | `codegen/js.rs:8853-8858`, `:8817` | ✅ `Text("{عدد}")` emits `` `${_عدد()}` `` (static); `Text("{count}")` emits `() => …` | Item 1; use `unicode_ident` as the lexer does |
| B4 | **`\` and `${` in a spliced string are not escaped for a JS template literal.** `"C:\\dir\\{name}"` becomes `` `C:\dir\${_name()}` ``: `\d` is lost and the splice is escaped away; `"$\{x\}"` becomes live `${x}` | `codegen/js.rs:7823`, `:1935`, `:4127` | ✅ emitted as shown | C-VALIDATE |
| B5 | **The last page's state leaks into `app`.** `emit_app` resets only some of the per-body fields; `validated`, `refs`, `form_handles`, `current_form`… keep the last page's values. An app `Checkbox(bind: x)` after a page that validates `x` emits `_x_check`, which only the page declares | `codegen/js.rs:2465-2486` vs `:2323-2332` | ✅ app script throws `ReferenceError` at boot: nothing draws | Item 1 (invalid states); group per-body state |
| B6 | **A template's data can inject CSS.** A `style { }` value from the data is HTML-escaped but not checked as CSS: `red; position:fixed; inset:0; background:url(…)` adds declarations (overlay, tracking request). The live page sets one property at a time and is not affected | `template/mod.rs:2033-2035`, `:2203` | ✅ `wf render` writes all four declarations into `style=""` | C-VALIDATE |
| B7 | **`Template` and `wf build` evaluate the same expression differently.** The template engine has its own interpreter: `name.length` on a string is `null` (build: char count), `name.length()` counts bytes (`"مرحبا"` → 10), `"12" < "9"` coerces to numbers (build refuses), `xs[1.5]` truncates | `template/mod.rs:964-1230`, `:2843-2909` vs `codegen/static_eval.rs:575`, `:1513` | read | single source of truth |
| B8 | **Render state leaks across requests after a panic.** `render_fragment` sets the `STUDIO` thread-local and clears it only on the normal path; a server that catches the panic keeps the thread, and later renders gain `data-wf-node` attributes from another template. `outfs::capture_from` and `memory.rs:167` have the same shape | `template/mod.rs:587-589`; `outfs.rs:52-54`; `memory.rs:167` | read (set/clear without a guard) | M-AVOID-STATICS; §6 drop guards |
| B9 | **A diagnostic names a store picked from `HashMap` order**, so `wf check --format json`/SARIF differ between runs when two stores share an action name | `sema/types.rs:5019-5024` | read (`HashMap::iter().find`) | determinism (§4) |
| B10 | **The editor misses a socket's handler scope.** `scope::child_bodies` re-implements `StatementKind::bodies` and omits `Connection`, so completion inside `socket s = ws(…) { on message(m) { … } }` doesn't offer `m` | `scope.rs:34-65` vs `parser/ast.rs:1360` | read | DRY |
| B11 | **T22's column is counted in bytes**, so on a line with Arabic names the squiggle lands in the wrong place | `sema/types.rs:1087` | read | correctness |
| B12 | **Numbers are clamped silently**: `errors { 70000 -> Map }` becomes status 65535, `404.5` becomes 404; a `Carousel`'s `interval: -5000` becomes a 0 ms autoplay | `parser/v2.rs:2245`, `:2067`; `codegen/js.rs:6177`, `:3136` | read | §6 (`as` casts) |
| B13 | **`.wfx` lexing is quadratic**: each implied brace re-walks the source from the start | `lexer/v2.rs:166-192` | read | §12 |
| B14 | **`ParserV2::parse` drops every error but the first**, though `WebFluentError::Diagnostics` exists; `ParserV2::new(vec![])` underflows | `parser/v2.rs:427-433`, `:241`, `:257` | read | C-GOOD-ERR; C-VALIDATE |

## 2. Findings by area

### 2.1 Errors (whole crate)

- **I/O errors lose their path and cause.** `From<io::Error>` stores
  `err.to_string()` and `WebFluentError` has no `source()`
  (`error/diagnostic.rs:87-92`). `Vfs::read_to_string` errors name no file,
  so `?` yields "IO Error: No such file or directory" with no path
  (`build.rs:1566`, `config/project.rs:1231`, `media/mod.rs:193`); 31
  hand-built `IoError(format!…)` sites. (C-GOOD-ERR, M-ERRORS-CANONICAL-STRUCTS)
- **`WebFluentError` is an exhaustive public enum** with five `String`
  variants, so every new failure kind is a breaking change; variants are
  misused (a serialisation failure in caller data is `ConfigError`, so
  `exit_code()` says "could not run"). (C-STRUCT-PRIVATE / §4 evolution)
- **16 `Result<_, String>`** outside `src/cli`, 5 public: `paged/mod.rs:246`,
  `:794`, `reprint.rs:46`, `patch.rs:259`, `registry/schema.rs:557`. A caller
  cannot tell "no PDF engine" from a failed write. (C-GOOD-ERR)
- **Errors swallowed as absence**: `let Ok(..) = vfs.read_to_string(..) else
  { continue }` (`i18n/mod.rs:39`) hides a locale that could not be read;
  `ProjectConfig::load(..).ok()` turns a malformed config into the defaults
  (`testing.rs:235`, `assets.rs:699`, `agent/context.rs:355`).
- **Docs promise the wrong variant**: `Template::from_str` says
  `CodegenError` "with every finding", the code returns `Diagnostics`
  (`template/mod.rs:110` vs `:272`). 151 public `Result` functions have no
  `# Errors`. (C-FAILURE)
- **Font registry locks** recover from poisoning four different ways
  (`paged/fonts.rs:189`, `:215`, `:221`, `:245`). (§5: poisoning is fatal)
- **Three diagnostic shapes**: `Diagnostic`, `StudioDiagnostic`
  (`studio.rs:79`, public, lossy) and `A11yWarning`/`VocabWarning`, whose
  conversion yields `code: ""` for an unknown rule. (M-ERRORS-CANONICAL-STRUCTS)
- Display text is capitalised and prefixed ("Codegen Error: …"); `exit_code()`
  is CLI policy living on the library error. (C-GOOD-ERR)

**Error design** (lands in the next major version, since `#[non_exhaustive]`
and payload changes are breaking together): one `thiserror` type per layer
(`IoError { path, #[source] io::Error }`, `ConfigError`, `PdfError`,
`RenderError`, `EditError`), and `WebFluentError` as the facade keeping its
variant names, each `#[error(transparent)]` over the layer error, so
`source()` walks the chain. Pin `Send + Sync + 'static` with a const
assertion. Non-breaking steps for 5.x: paths in messages, the doc fixes, the
drop guards (B8).

### 2.2 Front end (`lexer/`, `parser/`, `syntax/`, `layout/`)

- B1, B2, B12, B13, B14 above.
- **Error positions**: an error in a style value points at the next line;
  `parse_sub_expression` replaces the inner error's line, column, code and
  hint with the outer token's (`parser/v2.rs:3275-3288`, `:3371-3395`).
- **The public AST is not `#[non_exhaustive]`** and is re-exported with a
  glob (`parser/mod.rs: pub use ast::*`); 74 types with public fields, so each
  grammar feature is a semver break. (C-STRUCT-PRIVATE, M-NO-GLOB-REEXPORTS)
- `LexerV2`/`ParserV2` lack `Debug` and docs; comments are a public
  `(usize, usize, String)` tuple (`lexer/v2.rs:77`). (C-DEBUG, C-CUSTOM-TYPE)
- Good: the lexer works on `Vec<char>`, so no slice splits a UTF-8
  character; `parse_recovering` always makes progress; `.wf` ↔ `.wfx`
  conversion verifies its own output token for token.

### 2.3 Template engine and public API (`template/`, `lib.rs`)

- B6, B7, B8 above, and its share of B2.
- **Per-render cost is O(rows × data)**: `eval_static` converts the whole
  data object on every `format()`/method call; `child()` clones the
  components map per lambda (`template/mod.rs:1233-1257`, `:916-930`).
  (M-HOTPATH)
- **The public surface is the whole compiler**: every module is `pub`
  (`lexer`, `sema`, `agent`, `platform`, `memory`…), so every refactor is a
  semver event; `#![allow(dead_code)]` crate-wide hides dead code.
  (Item 22, M-SINGLE-ITEM-PATH)
- `Template` has no `Debug`; `with_*` builders lack `#[must_use]`
  (`tpl.with_lang("ar");` does nothing); `PdfReport` has public fields.
  (C-DEBUG, C-STRUCT-PRIVATE)
- Misplaced docs (`with_theme`'s doc on `with_studio`; `wf fmt`'s on
  `pub mod data`); feature docs still say `version = "4"` (`lib.rs:124`).
- Doc examples use `unwrap()` instead of `?`. (C-QUESTION-MARK)
- Good: `Template` is `Arc`-backed, `Send + Sync`, tested from four threads;
  URLs from data go through `url::guard`; templates are checked once at
  construction.

### 2.4 JS code generator (`codegen/js.rs`)

- B3, B4, B5 above.
- **JS string literals are escaped by hand, inconsistently**: titles escape
  `\` and `"` but not newlines, `redirect` only `"`, route `path`, `base_path`,
  OpenAPI endpoint paths and locale names not at all (`:906`, `:938`, `:950`,
  `:1124`, `:1385`, `:2834`); 11 other sites already use
  `serde_json::to_string`. One `js_str` helper. (C-VALIDATE)
- **`emit_ui_element` is 1,182 lines** dispatching on strings after a generic
  pass most special built-ins ignore (`:2880-4061`); a table of special
  emitters looked up first. (`too_many_lines`; Command pattern)
- **Two expression emitters and two statement emitters**, already drifted
  (lambda parameters split on `", "` vs `','`) (`:1684-1973` vs
  `:7804-8286`). One emitter with an `ExprCx` argument.
- **Scope stacks pushed and truncated by hand through `RefCell`**, 30+
  borrows; an early return would leak names (`:1561-2141`). A `with_locals`
  closure helper. Six bool fields for build modes. (§6 guards, C-CUSTOM-TYPE)
- **Public `JsCodegen` API**: four `set_*(bool)` setters, `serde_json::Value`
  in `set_env`, no `Debug`, docs on the wrong setters (`:615-775`). A
  `JsOptions` struct, or make the module `pub(crate)`.
- **Borrow-checker clones**: the translations map (`:2183`), every page's AST
  into `current_body` (`:2332`, `:2409`), a whole `UIElement` to swap its
  args (`:3989`), and a quadratic rebuild of `else if` chains (`:6687`).
- **Output building**: 303 `emit_line(&format!(…))` (two allocations a line),
  55 hand-paired `indent += 1/-= 1`. `line(format_args!(…))` and a
  `block(open, close, f)` helper. (`format_push_string`)
- `camel_to_kebab` is copied byte for byte in `ssg.rs:2959` and
  `template/mod.rs:2931`; `use crate::parser::ast::*` at line 6.
- Good: name mangling (`sig`) is deliberate and backed by a generated list;
  one `method_to_js` table; deterministic output; comments explain past bugs.

### 2.5 Type checker (`sema/`, `scope.rs`)

- B2 (its share), B9, B10, B11 above.
- **Each declaration's checker gets its own copy of the source file**
  (`source_of: &dyn Fn(usize) -> Option<String>`, `types.rs:576`), and
  `own_actions` deep-clones every action's AST (`:839`): about 40 copies of a
  40-declaration file per check, on every LSP edit. (C-CALLER-CONTROL)
- **`Type` is cloned at almost every step**: `unwrapped()` and `lookup()`
  return owned values, 42 call sites (`types.rs:205`, `:989`). Return `&Type`.
- **The non-empty-list proof compares a lossy text rendering** of the
  expression, so `xs.filter(a)` and `xs.filter(b)` look the same and T04 is
  missed (`types.rs:828`, `:5796-5828`). Compare `Expr`s.
- **Diagnostic codes and severities are strings chosen at each call site**
  (113 `"T01"` literals; severity from which helper is called, not from the
  code registry). (C-CUSTOM-TYPE)
- **Three passes build the same index and report E101 in two wordings**
  (`sema/mod.rs:31-79`, `types.rs:329-395`, `linter/semantic.rs:546`).
- **Desugared forms dispatched on magic method names** (`"__if"`,
  `"__case"`, …) across seven files; a typo becomes a T16 at a place the
  author never wrote. Own `Expr` variants. (§1 enums over magic strings)
- `Type` lacks `#[non_exhaustive]`, `Eq`, `Hash`; `TypeInfo` lacks `Debug`;
  `Member { method: bool }` should be an enum. Context bools (`in_args`,
  `async_ok`, …) set and reset by hand, one reset to `false` instead of its
  previous value (`types.rs:748`).
- Good: deterministic suggestions; defensive slicing; tables pinned to the
  checker by tests.

## 3. File splits proposed by the reviews

For README Phase 5. Both keep one type with `impl` blocks spread over files.

**`src/codegen/js.rs` → `src/web/js/`**: `mod.rs` (struct, setters,
`generate`), `names.rs` (`sig`, `bare`, …), `expr.rs`, `statement.rs`,
`store.rs`, `body.rs` (page, component, app), `dom/flow.rs` (if, for, show,
match), `dom/element.rs`, `widgets/{overlay,control,nav,display,host}.rs`,
`net.rs` (resources, connections), `reactive.rs`, and a shared `case.rs` in
`codegen/`. Tests move beside the code they test.

**`src/sema/types.rs` → `src/check/types/`**: `mod.rs` (`check`,
`check_in`), `ty.rs` (`Type`, `Scalar`), `world.rs`, `checker.rs`,
`report.rs`, `stmt.rs`, `element.rs`, `reactivity.rs`, `expr.rs`
(`infer`, `property`), `method.rs`, `call.rs`, `walk.rs`, `tables.rs`,
`names.rs`, `jsdoc.rs`, `handles.rs`, `refine.rs`. `scope.rs`'s inline
`paths` module moves to `scope/paths.rs`.

The line ranges for each are in the area reviews' notes; they move with the
code, so the split PRs re-derive them.

## 4. The lint config

The skill says a project with no `[lints]` table should adopt BeautifulRust's
`lints/lints.toml` and `lints/clippy.toml`. Under it, today:

| Lint | Findings | Where | Notes |
|---|---|---|---|
| `unreachable_pub` | 1,629 | everywhere | Falls as modules become `pub(crate)` (README Phase 4) |
| `missing_docs` | 1,571 | `paged` 870, `parser` 604, `lexer` 318 | Falls the same way: most are items that should not be public |
| `clippy::as_conversions` | 882 | layout, PDF, codegen | Many are `usize`↔`u32` spans; one checked conversion at the source boundary (finding B12) removes most |
| `clippy::uninlined_format_args` | 427 | | `cargo clippy --fix` |
| `clippy::format_push_string` | 282 | `codegen` | The `line(format_args!)` helper |
| `clippy::cast_possible_truncation` | 229 | | With `as_conversions` |
| `clippy::print_stdout` | 177 | `cli` 171 | Allowed in the CLI layer only |
| `clippy::missing_errors_doc` | 151 | | Facade and template first |
| `clippy::too_many_lines` | 111 | | The Phase 5 splits |
| `clippy::exhaustive_enums` | 103 | `paged` 27, `parser` 13 | The 6.0 API pass |
| `clippy::fn_params_excessive_bools` | 94 | `cli` 24, `codegen` 14 | `max-fn-params-bools = 0`: every bool parameter |
| `clippy::unwrap_used` | 85 | `wf-lsp` 34, `cli` 20, `codegen` 18 | Few; most are invariants that want `expect("why")` |
| `missing_debug_implementations` | 58 | | Derive |
| `clippy::allow_attributes(_without_reason)` | 64 | | `#[expect(…, reason)]` |
| `clippy::wildcard_imports` | 46 | | |
| **Total** | **7,251** | | (tests and examples excluded) |

Turned on all at once this would bury real findings, so README §5.1 adopts
it whole with a staged allow-list that each phase removes entries from.

## 5. What is already good

Credit where it is due, because it says what to keep:

- `Diagnostic` is a strong error record: registry code, span, end, hint,
  related places, machine-applicable fixes. The rest of the error model
  should grow towards it.
- Library code never prints or exits; the CLI owns stdout and exit codes.
- The lexer works on chars; parsing recovers and always progresses; the
  `.wf` ↔ `.wfx` converter verifies its own output.
- `Template` is cheap to clone, `Send + Sync`, checked once, and tested from
  several threads.
- Output is deterministic in most places, by design: sorted locales, keys,
  routes and suggestions.
- Comments explain *why* a past bug happened, and regression tests are named
  for the behaviour they keep.
