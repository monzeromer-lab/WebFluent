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
> **Where we are:** all decisions locked. **Active milestone: M1 (engine).**
> **Slice 1 (spans) is complete** — byte-accurate source spans on `UIElement`
> (whole/paren/body/style, per-arg, per-modifier), on `Page`/`Component`/`Store`
> decls, and on every `Statement`; round-tripping and green across build/PDF/slides.
> **Immediate next step: Slice 2 — node identity** (see *Milestone 1 — build sequence*).

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
2. **Slice 2 — node identity (§1.2).** Add the `studio:true` flag; stamp `data-wf-node`
   in the JS + SSG codegen; build the in-process `node_id → {span, path, component}` map.
   *Done when:* a studio-compile stamps every element and the map resolves each id to its
   span, and an export-compile contains **no** `data-wf-node`.
3. **Slice 3 — apply engine + debug hook (§1.3/§1.4).** `apply_edits(source, ops)` and the
   `WF.__debug` runtime hook. *Done when:* an op-batch round-trips to valid, recompiling
   `.wf` with untouched regions byte-identical, and `WF.__debug.state()` reflects live signals.

The headless test harness (§1.5) lands with M5 (self-testing). Slices 1–2 alone unblock
M2 (pipeline) and M3 (click-to-code), so we can start the studio side in parallel once
they're in.

---

## Milestone 2 — In-process compile & preview pipeline (Studio)

- Add `webfluent` as a crate dependency of `crates/studio`.
- Replace the static-mock serving with a `Project` model: `.wf` sources in memory +
  parsed ASTs + compiled output + the node maps.
- Compile on edit → **SSG-render** for instant paint (no blank screen) → hydrate →
  serve HTML/CSS/JS over the existing `wf://` custom protocol → (re)load in the webview.
- Recompile strategy: recompile the touched page; keep it under the perceptible-latency
  budget for the talk→see loop; measure, then add caching/incremental as needed.
- Persist projects to disk (`webfluent.app.json` + `src/`), matching the CLI layout.

## Milestone 3 — Selection → inspector → edit loop (Studio)

- Repoint the IPC bridge from `data-wf-el` to **`data-wf-node`**; click → node id →
  (via the map) AST node + span → highlight + populate the inspector.
- Inspector edits (color/size/weight/align/bg/radius, text, variant/modifier) emit
  **`EditOp`s** (Milestone 1.3) → `apply_edits` → recompile → re-highlight. This is the
  first real "edit by selecting" path and validates the whole spine.
- Outline panel = the page's AST tree (from the maps). Blocks (add text/image/button) =
  `InsertChild`/`AppendChild` ops.

## Milestone 4 — AI codegen agent + token strategy (Studio)

- Generation (new page/site/component): AI emits full `.wf` once; validated by parse.
- Editing: the AI is handed a **compact addressable view** — a node tree of
  `{id, component, args, modifiers, text, childIds}` + the selected node's source
  slice — never the whole file; it replies with a small batch of `EditOp`s.
  This is the primary AI-token lever (tens of tokens/edit vs. whole-file regen).
- Self-heal: on compile error, feed the error + offending span back to the AI for a
  scoped fix (bounded by `heal_attempts`, already in the settings model).
- Wire model/effort/permission/skills/MCP/ACP config to real provider calls; keys in
  the OS keychain (already the mock's stated behavior).

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
