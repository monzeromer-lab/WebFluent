# WebFluent Studio — Mock → Product Plan

Turning WebFluent Studio from a UI mock into a working product: the user builds
websites by talking to an AI that generates and edits **WebFluent (`.wf`)** code;
the studio compiles it in-process and shows the result in an embedded webview;
clicking any element in the preview resolves straight to its code; and the AI can
test the site it built.

> Status: planning. This doc is the source of truth for the roadmap. Milestone 1
> (engine upgrades) is specced in detail; later milestones are scoped and will be
> re-detailed as we reach them.
>
> **Where we are:** **Milestone 1 (engine upgrades) is complete.** Slices 1–3 landed on
> branch `m1-engine-upgrades` as three green commits: **spans** → **node identity** →
> **edit engine + `WF.__debug`**. Given `.wf` + `studio:true`, WebFluent now returns
> compiled output with `node_id ↔ span ↔ path` maps, `apply_edits(source, ops)` performs
> reliable structured edits (11 ops, reparse-guarded, byte-preserving), and the runtime
> exposes `state`/`tree`/`dispatch`/`query*`. The headless test harness (§1.5) is deferred
> to M5. **Next: Milestone 2 — in-process compile & preview pipeline (Studio side)**,
> scoped below and to be re-detailed as we start it.

## Locked decisions

| Decision | Choice | Why |
|---|---|---|
| AI edit strategy | **Structured AST edits → span-based text patches** | Lowest AI-token cost and highest reliability; preserves human-authored source formatting (no full reformat). |
| Self-testing | **Unit + e2e**, driven by the AI | Unit for stores/actions/derived + component render; e2e for real user flows in the webview. |
| First milestone | **Engine upgrades first** | Land spans + node-identity + the apply-engine + debug hooks in WebFluent so all studio work sits on clean foundations. |
| Compiler integration | **In-process crate** | Studio depends on the `webfluent` crate and compiles in its own process — fastest talk→edit→see loop, no subprocess/IPC. |

## The core loop (the spine)

```
prompt ─▶ AI generates/edits .wf
             │  (generation = full file once; edits = structured ops)
             ▼
      in-process compile  ── SSG-render for instant paint ──▶ serve via wf:// ──▶ webview
             │                                                                     │
   node-id ⇄ AST ⇄ span maps                                        click / drive / assert
             ▲                                                                     │
             └───────────────── targeted edit / test result ◀──── IPC bridge ──────┘
```

Everything in the studio (inspector, outline, review, blocks, self-heal, testing)
is a consumer of this loop.

---

## Why change the engine? (goal → capability → gap)

Each engine change is *pulled* by a concrete product capability the mock only faked —
none of it is gold-plating. WebFluent already gives us the language and the compiler;
what's missing is everything that lets the AI and the studio **address, edit, and test**
the code precisely instead of shuffling whole files of text.

| Product goal (the vision) | Capability it requires | Engine change (the gap today) |
|---|---|---|
| Build by **talking** — AI writes real, runnable code | A real web language + compiler | ✅ WebFluent already provides this — we build on it |
| **Click a preview element → reach its code** | DOM → AST → source mapping | Source spans on the AST (§1.1) + node identity in output (§1.2) — neither exists |
| **Fewest possible tokens** | AI patches specific nodes, never regenerates files | Addressable AST + `apply_edits` engine (§1.3), built on spans — no such API |
| **AI tests the site it built** (unit + e2e) | Drive + introspect the running app | `WF.__debug` hook + headless harness (§1.4/§1.5) — no introspection today |
| **Fast talk → see loop** | Compile in-process, quickly | ✅ library API exists; add per-page/incremental caching later |

Without spans, "click this element and change its label" means re-sending the whole file
to the model and hoping it edits the right characters — slow, expensive, and error-prone.
With spans + node identity + an apply-engine, the same edit is: *click → node id → span →
one `SetText` op (tens of tokens) → patch → recompile*. That single difference is the
product.

**Why engine-first:** every real studio feature — selection, inspector, token-efficient
editing, self-testing — is a *consumer* of these primitives. Building the studio on the
static mock first would just be thrown away when the engine lands. So we pour the
foundation (M1), then build the house (M2+).

---

## What already exists (leverage)

**WebFluent engine**
- Full compiler: `lexer → parser → codegen(html/css/js/ssg/pdf/slides) → runtime`.
- Library API (`src/lib.rs` `Template`): `from_str/from_file`, `render_html`,
  `render_html_fragment`, `render_pdf`, `with_theme`. Studio can compile in-process.
- Runtime (`src/runtime/runtime.js`, ~700 LOC): `WF.h(tag, attrs, …children)`
  builds the DOM; signal reactivity; SPA router; `mount`/`hydrate` (SSG).
- 50+ components, design tokens + 4 themes, i18n + automatic RTL, animations,
  stores, control flow, data fetching — ~1:1 with the studio mock's surface.
- `wf-lsp` (hover, completion, document symbols) and a tree-sitter grammar.
- **The lexer already tracks `line`/`column` per token** — half of "spans" is done.

**WebFluent Studio (current mock)**
- Native GPUI shell with a wry webview embedded as a child of the window.
- Existing IPC bridge (`crates/studio/src/ipc.rs`): preview click → `data-wf-el`
  ancestor chain → backend selection; `page-loaded`; `runtime-error`;
  `window.__wfApply(...)` to push state back. Today keyed to a static mock site.

**The two load-bearing gaps**
1. **AST has no source spans** — the parser discards the lexer's positions.
2. **Codegen emits no node identity** — `WF.h(...)` output has no `data-wf-node`,
   so a DOM click can't resolve to a code node.

---

## Milestone 1 — Engine upgrades (WebFluent)

Goal: WebFluent can (a) tell you the source span of every AST node, (b) stamp every
rendered element with a stable node id, (c) apply structured edits as minimal text
patches, and (d) expose runtime state/tree for tests. No studio changes yet.

### 1.1 Source spans on the AST

- Add `Span { start: u32, end: u32, line: u32, col: u32 }` (byte offsets into
  source + 1-based line/col of `start`). Confirm lexer offsets are **byte** offsets
  (it currently collects `chars()` — switch to byte indexing or carry both).
- Attach spans where edits and selection need them, in priority order:
  1. `UIElement` — **whole-node span** + **args span** + **modifiers span** +
     **body/children span** (the `{ … }` interior) + **style-block span**. These
     sub-spans are what the edit engine patches.
  2. `Statement` (wrapper span for each stmt).
  3. `PageDecl` / `ComponentDecl` / `StoreDecl` (header + body spans).
  4. **Per-`Arg` and per-modifier spans** (each positional/named arg + each
     modifier keyword gets its own span) — surgical label/color/variant edits.
     **Deep sub-expression spans are deferred** (spans inside complex `Expr`
     trees like `a + b*c`); add later only if arg/interpolation editing needs it.
- Implementation: record `start` at the token before parsing a node, `end` after.
  A small `spanned!(self, { … })` helper in the parser keeps it tidy.
- Back-compat: spans are additive fields; html/css/js/pdf/slides/ssg ignore them.
  AST-literal tests need a `Span::dummy()` or `..Default::default()`.

**Acceptance:** `parse(src)` yields an AST where, for any UIElement, the source
slice `src[node.span]` round-trips to that element, and `src[node.body_span]` is
exactly the `{ … }` interior.

### 1.2 Node identity in codegen

- Deterministic **AST path id**: pre-order walk assigns each renderable UIElement a
  path like `Home:2.0.3` (page/component name → statement-index chain). Stable for a
  given source; recomputed each compile.
- Codegen (`src/codegen/js.rs`) stamps `data-wf-node="<id>"` on the **root element**
  of every UIElement's `WF.h(...)` output (one attribute; components that expand to
  wrapper+children get it on the wrapper). SSG (`ssg.rs`) stamps the same in
  pre-rendered HTML so it survives before hydration.
- Emit a **sidecar map** from the compile: `node_id → { span, path, component }`.
  With the in-process crate the studio gets this as a Rust struct (no JSON parse).
- **Ids are internal, studio-only.** Stamping happens *only* under the `studio: true`
  compile flag; **export/release builds never contain `data-wf-node`** or any debug
  attributes. The sidecar map lives in the studio process, not in shipped output.
- Durability: index paths shift when siblings are inserted/removed — fine for
  *live* selection (re-resolved after each edit). Cross-turn AI references are kept
  stable in the studio by re-resolving the selected logical node after each compile
  (no user-facing `id:` anchoring — decision C4).

**Acceptance:** compiling with `studio:true`, every rendered element carries a
`data-wf-node`, and the sidecar map resolves each id to the exact source span from 1.1.

### 1.3 Structured edit engine (ops → span patches)

The reliability/token core. The AST is for *addressing*; the mutation is a **text
patch on a span**, so human-authored formatting is preserved and we avoid a full
AST→source pretty-printer.

- Op set (typed, serde-serializable — this is the AI's tool schema):
  - `SetArg { node, arg: Positional(i) | Named(name), value }`
  - `SetText { node, value }` (first positional string arg)
  - `AddModifier { node, modifier }` / `RemoveModifier { node, modifier }`
  - `SetStyle { node, prop, value }` / `RemoveStyle { node, prop }`
  - `InsertChild { node, index, wf }` / `AppendChild { node, wf }`
  - `ReplaceNode { node, wf }` / `RemoveNode { node }`
  - `MoveNode { node, new_parent, index }`
- Apply algorithm: resolve `node` id → AST node → the relevant **sub-span**; turn
  the op into a `(range, replacement_text)` patch (e.g. `InsertChild` inserts before
  the body-span's closing `}`; `SetArg` replaces the arg's span; `SetStyle` edits or
  creates the `style { }` block). **Parse-validate** any `wf` snippet before patching.
  Apply patches right-to-left by offset, reparse, recompile, re-derive ids/spans.
- Expose as a library entry: `apply_edits(source, &[EditOp]) -> Result<String>`.

**Acceptance:** a batch of ops applied to a page produces valid `.wf` that recompiles;
malformed snippet ops are rejected without corrupting the file; unaffected regions of
the source are byte-identical (no reformat).

### 1.4 Runtime debug/introspection hook

- Add `WF.__debug` (behind the `studio`/dev flag) to `runtime.js`:
  - `state(scope?)` — snapshot of reactive signals (page/store state + derived).
  - `tree()` — the current rendered node tree keyed by `data-wf-node`.
  - `dispatch(node_id, event, payload?)` — synthesize a user event on a node.
  - `queryText(text) / queryRole(role)` — locate nodes for assertions.
- This is the ground truth for both e2e assertions and selection sync.

**Acceptance:** for a running app, `WF.__debug.state()` reflects live signal values
and updates after `dispatch(...)`.

### 1.5 Test-harness foundations

- Headless compile+run path for **unit tests**: compile a `.wf`, load runtime + app
  in a headless JS context (or an offscreen webview), instantiate stores/pages, call
  actions, assert `state()`/`derived`. Snapshot-render components with props → assert DOM.
- **e2e** primitives reuse 1.4 (`dispatch`, `query*`, `state`) over the webview.
- Deterministic seams: injectable clock/random so tests are repeatable.

**Milestone 1 exit:** WebFluent, given `.wf` + `studio:true`, returns compiled output
+ `node_id↔span↔path` maps; `apply_edits` performs reliable structured edits; the
runtime exposes state/tree/dispatch; a headless harness can run unit + e2e assertions.

### Milestone 1 — build sequence

Work on a branch off WebFluent `main`; keep `wf build`, PDF, and slides **green at every
commit** (they only gain additive fields they ignore).

1. **Slice 1 — spans (§1.1). ✅ Done.** Added `Span`; made lexer offsets byte-based
   (carry-both — a `byte_pos` tracked beside the `chars()` cursor, stamped centrally in
   `tokenize`). Attached spans to `UIElement` (whole / paren / body / style, per-`Arg`,
   per-modifier), to `Page`/`Component`/`Store` decls (whole / header / body), and to
   every `Statement` via a `Statement { kind, span }` wrapper stamped at the single
   `parse_statement` choke point. `Arg`, `modifiers: Vec<String>`, and the statement
   variants stayed data-identical (spans live on containers / parallel arrays), so all
   generators ignore spans with no behavior change. Verified by span round-trip unit
   tests (incl. a multibyte byte-offset case) with `wf build` / PDF / slides green.
   *Done:* `src[node.span]` round-trips for any node; all tests green.
2. **Slice 2 — node identity (§1.2). ✅ Done.** A shared `node_id::visit_nodes` walk assigns
   each `UIElement` a path id (`Home:2.0.3`) keyed by its span; JS + SSG stamp `data-wf-node`
   on element roots via the **same** map, so ids match by construction. Gated by a `studio`
   flag (`JsCodegen::set_studio`, `render_page_html_studio`) — export builds carry no
   `data-wf-node`. Cross-kind name collisions (`Name#page`) disambiguated deterministically.
   Verified by 10 unit tests + two adversarial review rounds.
   *Done:* a studio-compile stamps every element, the map resolves each id to its span, and
   an export-compile contains no `data-wf-node`.
3. **Slice 3 — apply engine + debug hook (§1.3/§1.4). ✅ Done.** `apply_edits(source, ops)`
   in `src/edit.rs` — 11 typed serde ops resolved via `visit_nodes` and applied as
   right-to-left span patches, parse-validated + reparse-guarded so a bad edit returns `Err`
   and never corrupts the file. `WF.__debug` (`state`/`tree`/`dispatch`/`query*`) in the
   runtime; studio-mode signal registration keeps export clean. 31 edit tests incl.
   multibyte, structural edges, overlap rejection, and MoveNode dup/loss.
   *Done:* op-batches round-trip to valid, recompiling `.wf` with untouched regions
   byte-identical, and `WF.__debug.state()` reflects live signals.

The headless test harness (§1.5) lands with M5 (self-testing). Slices 1–2 alone unblock
M2 (pipeline) and M3 (click-to-code), so we can start the studio side in parallel once
they're in.

---

## Milestone 2 — In-process compile & preview pipeline (Studio)

> **Status: re-detailed and ready to build** (M1 complete). Goal: replace the static
> `layali.html` mock with **real, in-process-compiled WebFluent output**, so the canvas
> preview shows the actual site the AI is building. Unblocks M3 (click-to-code) and M4.

**What exists to build on** (`crates/studio`):
- The preview is a wry webview built as a child of the gpui window (`build_preview`,
  `app.rs`), serving over `wf://` via `serve(request) → site::resource(path)` — today
  returning **static bytes** (`site.rs` → `layali.html`).
- `ipc::BRIDGE_JS` reports `data-wf-el` ancestor clicks; state is pushed via
  `window.__wfApply(...)` (`sync_preview`). The existing `Project` struct is a Home-
  dashboard *card*, not a compilable project.

**Locked decisions (2026-07):**
- **Compile facade in the engine.** Add `webfluent::compile_studio(...) -> CompiledSite`
  (html-per-route + css + js + `NodeMap`) wrapping the M1 studio primitives, so the studio
  stays thin. (M1 deferred assembling these.)
- **Seed a minimal real `.wf` page first** to prove compile→serve→reload→click end-to-end;
  port a richer demo later.
- **In-memory project first**; `webfluent.app.json` + `src/` disk load/save is M2's last step.

### Build sequence

1. **M2.1 — engine compile facade.** In `webfluent`: `compile_studio(program, opts) ->
   CompiledSite { pages: Vec<(route, html)>, css, js, node_map }` — builds the node map and
   runs `JsCodegen::set_studio` + `render_page_html_studio` per page + `generate_css`. Add
   `webfluent` as a **path dependency** of `crates/studio` (separate repos, path-dep during
   dev — Appendix C). *Done when:* studio compiles against the engine and a unit test
   round-trips a `.wf` string to a `CompiledSite` with a populated `node_map`.
2. **M2.2 — the `WfProject` model.** Sources (`HashMap<path, String>`) + parsed `Program`
   + `CompiledSite` + `NodeMap`, seeded from a small built-in `.wf` starter, held in app
   state; a `recompile()` reparses and re-runs the facade. *Done when:* the app boots with
   a seeded project whose `CompiledSite` is reachable from the serve path.
3. **M2.3 — dynamic `wf://` serving + recompile→reload.** The blocker: `serve` is a
   `'static` closure returning static bytes; give it a shared `Arc<RwLock<CompiledSite>>`
   the closure captures, so `serve(path)` returns the compiled page/css/js. On a `.wf`
   change → `recompile()` → swap the shared output → **SSG-paint** for instant repaint →
   `preview.raw().load_url(entry)` to reload → JS hydrates. Retire the static
   `site.rs`/`layali.html` path. *Done when:* editing the seeded `.wf` (e.g. via a debug
   hook) recompiles and the webview shows the new output within the perceptible-latency
   budget.

Persistence (`webfluent.app.json` + `src/`) and recompile caching/incremental land after
the loop is proven. Selection→inspector→edit (clicking a `data-wf-node` → `apply_edits`)
is **M3**.

## Milestone 3 — Selection → inspector → edit loop (Studio)

> **Status: re-detailed and ready to build** (M2 complete). The payoff — the first real
> "edit by selecting" path — validating the whole spine end to end.

**Locked decisions (2026-07):**
- **Multi-file node→source mapping now** — a node id must resolve to the source *file* it
  lives in, so an edit targets the right file.
- **Reuse the mock inspector controls** — keep the existing color/size/weight/align/bg/
  radius UI; rewire each control to emit an `EditOp` (the change is in what they *do*).

**M3.0 — fix multi-file node identity (prerequisite, surfaced by the decision above).**
`compile_sources` parses each file independently, so spans (byte offsets from 0) are **not
unique across files** — two nodes in different files can share a span, and the codegen's
span-keyed `id_for` then stamps the *wrong* id (the single-file seed never hit this). Fix:
the studio compiles from a single **merged source** (files concatenated in order, with
per-file offset ranges tracked), so all spans are unique, `apply_edits` runs on one offset
space, and the offset ranges map any node back to its file for editing/saving. *Done when:*
two structurally-identical files compile with no node-id collisions and a node resolves to
the correct file + source slice.

### Build sequence
1. **M3.1 — click → code.** Repoint `ipc::BRIDGE_JS` (`data-wf-el` → `data-wf-node`);
   `select_el` stores node ids; resolve via the node map → `{ span, component, file }`;
   highlight the element in the *real* preview (inject an outline on `[data-wf-node="id"]`)
   and surface its source slice. *Done when:* a preview click selects its node and the
   studio shows its source span.
2. **M3.2 — inspector → edit → reload.** Map the reused inspector controls to `EditOp`s
   (text→`SetText`, color/bg/radius→`SetStyle`, variant→`Add/RemoveModifier`,
   size/weight/align→`SetArg`/`SetStyle`); `apply_edits` on the node's file → `set_source`
   → `recompile_and_reload`. *Done when:* an inspector tweak edits the `.wf` and the preview
   shows it — the spine, end to end.
3. **M3.3 — outline + blocks.** Outline = the page's node tree from the map; blocks (add
   text/image/button) = `InsertChild`/`AppendChild` ops.

## Milestone 4 — AI codegen agent + token strategy (Studio)

> **Status: re-detailed and ready to build** (M3 complete). Re-detailed via a design
> panel + adversarial-verification pass (2026-07-24). The product owner expanded scope
> beyond the conservative first cut — **multi-node editing, multi-file sites, and all six
> providers are IN M4** — which roughly doubled the milestone, so it is split into
> **M4a (offline core)** and **M4b (live transport)**.

**Thesis.** Close the real AI loop **offline first**, behind one provider-neutral
`LlmClient` async trait, reusing the M3 `apply_ops → edit_node → recompile` path.
Every turn is **transactional** (pre-turn snapshot, success gated on *compile* not
reparse, roll back all touched files + derived state on failure while retaining the
last-good `CompiledSite`). `EditOp`s are the sole edit currency; a two-tier
addressable view (cheap whole-project outline for addressability + a verbatim slice
for only the selection) is the token lever — never the merged file.

**Locked decisions (baseline).**
- **One object-safe `LlmClient` async trait** is the only studio↔AI seam; a
  `ScriptedClient` fake is injected everywhere → the whole suite is offline/deterministic.
- **Editing = tool-use with the 11-variant `EditOp` schema**; decoded ops flow through
  the generalized apply path. Inspector + click-to-code stay untouched.
- **Transactional turns**: snapshot → apply → **compile-gate** → rollback-all on failure
  (the reparse-guard is *not* the compile gate — a batch can reparse yet fail to compile).
- **Bounded self-heal** on *content* errors only (parse/compile/decode); transport,
  rate-limit, and `max_tokens` errors are terminal and never heal.
- **Whole-batch permission** via a pure `classify_ops`; never split an atomically-validated batch.
- **Transport split**: pure `build_request`/`parse_response` **outside a `net` cargo
  feature** (default `cargo test` stays hermetic); thin `reqwest` glue behind `net` on a
  **dedicated tokio thread** bridged to GPUI's executor via a oneshot (resolves the
  reqwest-tokio vs GPUI-smol mismatch inside the seam). `KeyStore` trait, precedence
  keychain→env→none, key never logged or in a URL.
- **GPUI-free orchestration**: no `&mut WfProject` across an `await`; `app.rs` pumps the
  loop in `cx.spawn` and commits inside a synchronous `this.update`; app tests via `TestAppContext`.

**Locked decisions (expansions, product-owner choices 2026-07-24).**
- **Edit scope = multi-node, intent-driven.** The AI may edit the selected node *or* other
  nodes the user mentioned without selecting them (the engine already targets any node by
  id). Selection is a default context hint; **permission gating is the load-bearing guard**
  against off-target edits.
- **Generation = multi-file site.** One turn can emit a full multi-page/multi-file site;
  validated by `compile_merged` over `retained ∪ new` (the only gate that resolves
  cross-file references and catches breakage of retained files).
- **All six providers live in M4** (Anthropic, OpenAI, Gemini, DeepSeek, Moonshot/Kimi,
  Zhipu/GLM), collapsed to **3 wire families** (Anthropic Messages · OpenAI-compatible
  cluster · Gemini generateContent) + a data-driven `ProviderProfile` table.
- **Permission = auto-apply safe** (whole-batch): `SetText/SetStyle/RemoveStyle/SetArg`
  auto-apply; any structural or modifier op holds the whole batch; **destructive
  generation** (whole-file overwrite / replace-all on a populated project) also holds.
  All three modes ship; default auto-apply-safe.

**Pre-slice spikes — RESOLVED (2026-07-24).**
1. **Send spike → PASS.** `CompiledSite`, `FileRange`, `anyhow::Error`, and the
   `(CompiledSite, String, Vec<FileRange>)` tuple are all `Send + 'static` (verified by a
   compile-time assertion). M4.4's async `Compiler` runs on `background_executor()` — **no
   dedicated-thread fallback needed** for compilation (the transport bridge still needs its
   own tokio thread, for the reactor).
2. **Engine-shape spike → three findings:**
   - **`NodeInfo` carries no element kind/text/args/modifiers** — it is `{span, path,
     component}` where `component` is the *owner* name (`"Home"`), not the element type.
     `build_view` derives `{kind, text, args, modifiers}` from the node's **source slice**
     (`span.slice(&merged)`), exactly as `outline()` already does; `childIds` is derived in
     the studio from the id/span structure. *(Optional: capture kind + a text preview into
     `NodeInfo` during the existing `visit_nodes` walk — it already has the `&UIElement` —
     to make `build_view` cheaper. Deferred; derive-from-slice first.)*
   - **🔴 There is no semantic compile-gate.** `compile_studio` and all codegen return
     `String`/`CompiledSite` (never `Result`) and are **fully permissive**: undefined
     component refs, missing route targets, duplicate page names, and unresolved identifiers
     all compile `OK` and only break at *runtime* in the webview (verified empirically). The
     only real gates are lex + parse (+ `apply_edits`'s reparse-guard), whose diagnostics
     carry **line/column, not a byte span**. → **Decision (2026-07-24): add a semantic
     validation pass to the engine** (below). The re-slice's "reparse-OK/compile-FAIL"
     scenarios (M4.2/M4.4/M4.5) are *validation* failures, not `compile_studio` failures.
   - **Duplicate page/component names silently collide** (two `Page Home` → two pages with
     colliding node ids, last-wins). The engine raises nothing today; the explicit
     duplicate-name check moves into the new validation pass.

**M4.E — Engine semantic-validation pass (prerequisite; lands in `webfluent` before M4a's
gate-dependent slices).** Add `validate(program) -> Vec<Diagnostic>` (Diagnostics already
carry file + 1-based line/column): undefined component references, missing `Route` page
targets, duplicate page/component names (same-kind), and unresolved identifiers. This is the
real, static, deterministically-testable compile-gate; it also stops `wf build` from
silently shipping broken sites. The studio's compile path becomes **lex → parse → validate →
`compile_studio`**, and a merged **line/column → file** mapper (count each `FileRange`'s line
span) replaces the byte-span mapper the re-slice assumed. *Done when:* each broken-reference
class returns a `Diagnostic` with the right line/column, a clean program returns none, and
the studio surfaces the first diagnostic as its compile error.

**Spike-driven corrections to the slices below:** the compile-gate in **M4.2/M4.4** is
`recompile()` = parse + **`validate`** (not `compile_studio`, which never fails);
**M4.4**'s error→file mapping is **line/column → file**, not byte-span → file;
**M4.5**'s cross-file reference check and duplicate-name pre-check are *the `validate`
pass* run over `retained ∪ new` (a merged parse alone never catches them); **M4.6**'s
self-heal feeds back the `validate` `Diagnostic` (message + file-local line/column).

### M4a — Offline core (fully hermetic with `ScriptedClient`; a demoable talk-to-edit + talk-to-generate product)

1. **M4.0 — Neutral seam + `ScriptedClient` + `FlatEditOp` + object-root schemas + `KeyStore` + `LlmError`.**
   The `LlmClient` trait, provider-neutral `LlmRequest/Response`, the flat edit-tool decode
   path (`FlatEditOp → EditOp` with per-op required-field enforcement), an **object-root**
   tool schema (`{ops:[…]}`, no `oneOf/anyOf/$ref` — Gemini-safe), `KeyStore`
   (in-memory/env/chain) with `Secret<T>` redaction, the `LlmError` taxonomy, and the
   6-provider `ProviderProfile` table. *Done when:* a scripted turn round-trips a batch
   through the object-root schema into `Vec<EditOp>` offline; no schema contains
   `oneOf/anyOf/allOf/$ref`.
2. **M4.1 — `build_view` / `ProjectView`.** A cheap whole-project outline
   (`{id, component, text_preview, child_ids, page}` for every node) to address *any* node,
   plus a verbatim slice for *only* the selection neighborhood; ordered by document span,
   byte-deterministic. *Done when:* the outline covers every node across all files, the
   slice appears only when the selection resolves, and two calls are byte-identical.
3. **M4.2 — Transactional multi-node/multi-file edit substrate (`edit_nodes`).** Group ops
   by `resolve_node(op.node()).file`, stage per-file `apply_edits` **purely**, then
   snapshot sources **and derived state (`merged`+`ranges`)**, commit, compile-gate, and
   **roll back all** on any reparse-reject or compile failure — with `MoveNode`/overlap
   guards. *Done when:* a reparse-OK/compile-FAIL 2-file batch rolls back both sources *and*
   derived state, `resolve_node` still maps correctly, and a following edit commits.
   *(Fixes the atomicity leak the review caught: `recompile()` must retain last-good
   `{compiled, merged, ranges}` together.)*
4. **M4.3 — Offline multi-node edit loop + whole-batch permission.** `classify_ops` over
   multi-file batches; the three permission modes; held-batch approve/reject; `app.rs`
   pump (decode pure → `this.update{classify → edit_nodes → reload}`). *Done when:* a mixed
   safe+structural batch is held whole under auto-apply-safe, approve runs it atomically,
   and the pump reloads exactly once on Ok.
5. **M4.4 — Transactional merged-compile substrate for generation.** `compile_candidate`
   (pure, non-mutating merge+compile of an arbitrary source map, returns ranges even on
   error), atomic `commit`, and error→file mapping (route unique-id errors through
   `resolve_node`; reserve `map_merged_span` for duplicate-name/span-only cases, handling a
   separator-gap `span.start`). *Done when:* a valid 3-file cross-file set compiles and
   addresses all files, `commit` swaps atomically, and a later failed candidate leaves
   committed state intact. *(Send-spike gates async-Compiler vs dedicated-thread.)*
6. **M4.5 — Multi-file generation core (happy path).** `emit_wf` object-root tool
   (`{files:[{path,source,reason?}]}`); union blocks with last-wins dedup; wf-fenced
   fallback only when zero tool blocks decode; **truncation gates classification
   independently of decodability** (`MaxTokens` + no completeness signal → `Truncated`, not
   committed, not healed); assemble `retained ∪ new`; validate per-file parse → duplicate-name
   pre-check → `compile_candidate` over the whole set; deterministic generate-vs-edit router.
   *Done when:* a 3-file `emit_wf` on an empty project commits 3 sources; a reparse-OK/
   compile-FAIL set returns `Compile`; a truncated turn is `Truncated`; a dangling
   `site_meta.entry` is rejected.
7. **M4.6 — Cross-file bounded self-heal + destructive-generation permission + concurrency guard.**
   Heal feedback in the model's own **file-local** coordinates (never a merged offset),
   always carrying the proposed-path manifest; bounded by `heal_attempts`; a real
   non-healable branch (cross-file move, budget-exhausted). Destructive generation
   (`replace_all` on a populated project / overwriting AddFiles) is **held for review**. A
   source **epoch** captured at snapshot and checked at commit prevents a generation commit
   from clobbering a concurrent M3 edit. *Done when:* a missing-reference heal adds the file
   and commits; heal is bounded exactly; a `replace_all` on a populated project is held; a
   concurrent M3 edit during in-flight generation is not clobbered.

### M4b — Live transport (pure build/parse outside the `net` gate + thin net glue; `#[ignore]` live tests gate nothing)

8. **M4.7 — Family A: Anthropic** pure `build_request`/`parse_response` + recorded
   (redacted) fixtures. System-as-blocks with `cache_control`, tool-use round-trip,
   message-granularity mapping, structured-error-fields-only parsing, truncation-before-decode.
9. **M4.8 — Family B: OpenAI-compatible** (OpenAI/DeepSeek/Moonshot/Zhipu) one code path
   via data profiles: `arguments`-as-JSON-string, **model-aware `max_tokens` field**
   (`max_completion_tokens` for o-series/gpt-5+), truncation-before-decode, 200-with-error
   detection (GLM), split 429 (hard-quota terminal vs per-minute retryable).
10. **M4.9 — Family C: Gemini** with the flat schema **projected** (strip the full
    unsupported-key set, not just `oneOf`) and a guard over the *projected* bytes;
    `functionCall`/`functionResponse` mapping with synthetic ids and `is_error`; a recorded
    `oneOf`-rejection 400 as proof; key only in the `x-goog-api-key` header.
11. **M4.10 — Net glue.** `TokioBridge` (dedicated tokio thread, oneshot bridge, panic
    isolation), `LiveClient`, `RetryPolicy` (bridge retries only transport signals; a
    200-with-error is returned for `parse_response` to classify semantically),
    `KeyringStore` (manual/platform-verified; offline tests use in-memory/env), `make_client`
    factory, provider/model picker + key entry wired via `cx.spawn`, and the `app.rs` pump
    for both edit and generation turns — **no `&mut WfProject` across the `await`**.

**Testing.** ~10 deterministic offline tests per slice (~110 total) using `ScriptedClient`
+ in-memory `KeyStore` + an owned `WfProject`; default `cargo test` never touches the
network. Seam/transaction tests run the **real** `compile_studio` inline (the
reparse-OK/compile-FAIL branch and retained-file breakage are exercised against the real
engine). Wire tests are `build_request` goldens + **real recorded** `parse_response`
fixtures (provenance-tagged), including a real GLM 200-with-error and a real Gemini
`oneOf`-rejection 400, plus a no-key-in-error test per family. Live HTTP exists only in
`#[ignore]` tests (one per family, nightly) and gates nothing.

**Top risks** (each pinned to a slice + test): derived-state atomicity leak on rollback
(M4.2); truncation slip-through when tool calls stream per file (M4.5, M4.7–4.9);
destructive generation bypassing permission (M4.6); unverified engine assumptions
(engine-shape spike); `CompiledSite` `Send`-ness (send spike); Gemini schema drift
(projection); per-model `max_tokens` (M4.8); retry-seam contract for body-sourced
`retry_after`/200-with-error (M4.10); secret leakage (header-only auth + `Secret<T>` +
structured-error-only parsing); concurrent-edit clobber (epoch guard, M4.6).

## Milestone 5 — AI self-testing (unit + e2e)

- **Unit:** the AI derives tests from the AST — each `action`/`derived`/store → a unit
  test (call action, assert state/derived); component render snapshots. Run headless.
- **e2e:** the AI derives user scenarios from features (forms, buttons, `for` lists,
  routes) — e.g. a todo app: add → assert item + count; toggle → assert done; filter →
  assert visible set. Run in the webview via `WF.__debug.dispatch/query/state`.
- The studio orchestrates runs, collects pass/fail + diffs, and loops results back to
  the AI for self-repair. Surface as a "Tests" panel + status in the compile badge.

## Milestone 6 — Feature functionalization (breadth)

Wire the remaining mock surface to real behavior (see inventory). Highlights:
Publish/Export → real `build` (SSG/static/full) + a deploy target; Design-System
workspace → real WebFluent tokens/themes + component catalog (editing a token
recompiles dependents); RTL/i18n → `setLocale` + translation files; Review before/after
→ source/AST diff of a proposed op-batch; History/undo-redo → source snapshots (or an
op log); Share/collab, Settings, MCP servers → real integrations.

---

## Appendix A — Full studio feature inventory (mock → real → milestone)

**Auth & shell**
- Login (email/pw, Google/Apple SSO), sign-out → real auth/session. *(M6)*
- Home dashboard: project list, All/Website/System filters, status (published/draft/
  shared), open/new/profile/exit → real project store + persistence. *(M2/M6)*

**Onboarding**
- Provider pick (Claude/OpenAI/Gemini/DeepSeek/Kimi/GLM) → real provider registry. *(M4)*
- API key (BYO, test, OS keychain) / ACP agent connect → real key mgmt + agent conn. *(M4)*
- Starters (blank/venue/cafe/portfolio/product/import) → seed `.wf` scaffolds. *(M4)*

**Website workspace — chrome**
- Title bar: back, project name, **compile-status badge**, history, settings, share,
  publish, account → real compile status + real modals. *(M2–M6)*
- Toolbar: **RTL/LTR** → i18n locale/RTL; **device desktop/tablet/mobile** → responsive
  preview widths; **undo/redo** → source/op history; domain; publish. *(M3/M6)*

**Website workspace — assistant (left)**
- Messages + composer; attach; skills; **design-system picker**; **API-integration
  panel**; model/effort/permission menu; send → real agent loop + config. *(M4)*

**Website workspace — canvas (center)**
- States: empty / **compiling skeleton** / built / **before-after review wipe** /
  compiling overlay → real compile lifecycle + diff preview. *(M2/M6)*
- The preview itself → real compiled site in the webview. *(M2)*

**Website workspace — selection & context panel (right)**
- Click element(s), multi-select, selection chips → `data-wf-node` selection. *(M3)*
- **Inspector** (single element: color/size/weight/align/bg/radius, text, variant) →
  `EditOp`s on `style_block`/args. *(M3)*
- **Outline** (element tree) → AST tree from node maps. *(M3)*
- **Blocks** (add text/image/button) → `Insert/AppendChild` ops. *(M3)*
- **Review** (proposed changes, keep/discard, apply) → preview a pending op-batch as a
  diff; apply = commit ops. *(M3/M6)*
- Multi-select / Start / Working modes → panel state machine. *(M3)*

**Website workspace — modals**
- Publish (deploy; export static/full) → real `build` + deploy. *(M6)*
- Settings (providers/keys; MCP servers; advanced: context pruning, prompt cache,
  heal attempts) → real config + agent tuning. *(M4/M6)*
- Share (invite, roles, link access, collaborators) → real sharing/ACL. *(M6)*
- History (version restore) → source snapshot/op-log restore. *(M6)*
- Compile log; Profile; New project; Exit; Swap design system. *(M2/M6)*
- Toasts → real event notifications. *(M2)*

**Design-System workspace**
- Chat + **Foundations** (color tokens grouped, type scale, radii) → real WebFluent
  design tokens; edits recompile dependents. *(M6)*
- **Components** catalog by category (live specimens + "plan" schematics, ready/plan) →
  WebFluent's 50+ components; "generate" = author a component `.wf`. *(M6)*
- **Preview** (in-context composition, RTL) → compiled DS preview. *(M6)*
- **Inspector** (overview / color / type / component controls; add-token; generate). *(M6)*

## Appendix B — Cross-cutting concerns

- **Token budget telemetry** — measure tokens/edit to prove the AST-edit path pays off;
  keep the addressable-view compact (ids + shapes, not source).
- **Selection stability across recompiles** — re-resolve the selected logical node after
  each edit; consider explicit `id:` anchoring for AI cross-turn references.
- **Compile latency** — target an imperceptible recompile for single-page edits; add
  per-page incremental compile + output caching if needed.
- **Determinism for tests** — injectable clock/random; snapshot review.
- **Versioning** — in-process crate ties studio and engine versions; pin + CI both.

## Appendix C — Resolved decisions (2026-07)

1. **Repo shape** — **separate repos**; WebFluent added to the studio as a **path
   dependency** during development.
2. **Publish** — **export only** for now (static/full build artifacts); hosted deploy
   waits on the studio backend API.
3. **Persistence** — **local-first files** for now; cloud/backend deferred (this gates
   Share/collab, History, and multi-device — all M6+).
4. **Node ids** — **internal, studio-only**: present only under `studio:true`, stripped
   from every export/release build. No user-facing `id:` anchoring.
5. **Spans granularity** — **per-argument + per-modifier spans in M1** (surgical
   label/color/variant edits); **deep sub-expression spans deferred**.
