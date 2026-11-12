# WebFluent docs redesign: the plan

Status: **planned, not started.** Nothing in `site/` has changed yet.
Last updated: 2026-11-12.

This file is the single source of truth for the redesign. It records what
was decided, what the site will look like, how the content moves, and the
order to build it in. Each phase ends in something that can ship on its
own.

---

## 1. What we decided, and where to see it

| Decision | Choice | Where to look |
|---|---|---|
| UX direction | **All ten options combined** into one experience (below) | Canvas page *Combined experience* |
| Visual system | **Option A: Blueprint**, Night first, Day as the alternate | Canvas page *Design system: pick one* |
| Colours | Unchanged: the palette in `site/src/theme.wf` (Day and Night) | §4.1 |
| Logo | Unchanged: the blue `wf` tile (`site/public/favicon.svg`) | — |
| Type | Instrument Sans (reading and interface), JetBrains Mono (code and measurements) | §4.2 |
| Live examples | Run the real compiler in the browser as WebAssembly | §6, workstream W1 |

**Design canvas:** https://claude.ai/artifact/Q7U2qAypKASeJ1oTYdV6ve. Its pages:

- *Blueprint: the site* — Home and the redesigned **Button reference** in Blueprint. These are the visual target.
- *Design system: pick one* — the three dark-first systems; A (Blueprint) is the chosen one.
- *Combined experience* — the full UX in the earlier light styling: Home, Learn, Recipe, Reference, Errors, Coming from, Search, Playground, phone views. It is the **structural** target; restyle it in Blueprint.
- *UX options (10)* — the ten directions, with the research behind each.
- *Desktop · dark / light, Phone* — the current site as shipped, for comparison.

**In this folder:**

- `blueprint-pages/` — all ten pages already drawn in Blueprint (only Home and Reference are published to the canvas). Use them as the visual spec for the pages not yet on the canvas.
- `generators/` — the Python scripts that produced the artboards (`combined/`, `blueprint/`, `design-system/`). Scratch tools, not part of the build.

**Research:** `research/Programming language documentation UX.md` (sourced report) and `research/notes/`.

**Rejected, so we don't revisit them:** light-first designs; the first three systems (Compiler, Workbench, Field Manual); a reference page built around a measuring diagram with leader labels everywhere. Measurements appear only when the reader turns on *Measure*.

---

## 2. The experience in one paragraph

The home page asks **what you are making** (website, app, PDF, slides, custom elements, desktop or phone) and shows a **live editor** you can break and fix. Every page says which of four kinds it is (**Tutorial, How-to, Reference, Error**). **Learn** is one path: the first three chapters run as an in-browser tour, then the book continues, with short quizzes at the hard chapters. **Recipes** answer "how do I…" with complete, runnable code. **Reference** is generated from the compiler's registry, with a live preview and controls. **Errors** has a page for every diagnostic code. **Coming from** pages map React, Vue, Svelte, HTML/CSS and LaTeX habits to WebFluent. Every code block can show what it compiles to. **Search** understands other frameworks' words. Every page has a Markdown twin for agents. The **Playground** is the compiler in the browser.

### Where each of the ten options lives

| Option | Becomes |
|---|---|
| 1 Four Rooms | The mode label on every page (Tutorial, How-to, Reference, Error), and the writing rule behind it. Not tabs. |
| 2 Live Tour | Learn chapters 1–3, run in the browser; the hero editor on Home |
| 3 Pick Your Output | Home's first question; the *Building* switch in the header; "works in" badges on pages |
| 4 Coming From | `/coming-from/<framework>` pages; callouts in chapters; search synonyms |
| 5 Registry Atlas | `/reference/*`, generated from `wf registry --json` |
| 6 Compiler as Guide | `/errors/<code>`, one per diagnostic; linked from the compiler's `docs:` URLs |
| 7 One Long Book | Learn: numbered chapters, progress, quizzes, print and one-page views |
| 8 Recipe-First | `/recipes/*`; showcase apps broken down into the recipes they use |
| 9 Source and Output | Tabs on every code block: `.wf`, Preview, HTML, CSS, JS, PDF |
| 10 Agent-Native | "Copy page", `.md` twin per page, `llms.txt`, `llms-full.txt`, an MCP endpoint later |

---

## 3. Information architecture

### 3.1 Top navigation

`Learn · Recipes · Reference · Errors · Playground · Showcase`, plus search (⌘K), the *Building* output switch, the Night/Day toggle and GitHub.

### 3.2 URL map

| Route | Page | Template |
|---|---|---|
| `/` | Home | T1 |
| `/learn` | Book contents, progress | T2 index |
| `/learn/<nn>-<slug>` | A chapter (1–3 are the tour) | T2 |
| `/learn/about/<slug>` | An explanation essay ("About …") | T2 variant |
| `/recipes`, `/recipes/<slug>` | Recipe finder, a recipe | T3 |
| `/reference` | Index of built-ins (by family) and of language/CLI/config/runtime | T4 index |
| `/reference/<component>` | A built-in | T4 |
| `/reference/{language,cli,config,runtime,tokens,grammar}` | Generated reference pages | T4 variant |
| `/errors`, `/errors/<CODE>` | Diagnostic index, one code | T5 |
| `/coming-from/<framework>` | Cheat sheet | T6 |
| `/playground` | The compiler in the browser | T7 |
| `/showcase` | Gallery of real outputs | T8 (to design) |
| `*.md`, `/llms.txt`, `/llms-full.txt` | Machine-readable twins | generated |

**Redirects:** every current URL (`/docs`, `/docs/guide/*`, `/docs/reference/*`, `/docs/cookbook/*`) gets a permanent redirect to its new home, using the table in §5. `site/src/pages/Redirects.wf` already exists for this.

---

## 4. The design system: Blueprint

### 4.1 Colour (Night first)

| Role | Night | Day | Used for |
|---|---|---|---|
| Ground | `#0E1012` | `#F9FAFB` | Page background, with the measuring grid where things are measured |
| Sheet | `#17191C` | `#FFFFFF` | Panels, the active tab |
| Raised | `#1F2225` | `#F4F6F7` | Inputs, hovered rows |
| Ink | `#EEF0F2` | `#17191C` | Text |
| Muted | `#A9AEB4` | `#5A5F64` | Secondary text |
| Line | `#2C3035` | `#E2E5E8` | Hairlines, borders |
| Action | `#3D6FA3` | `#2B5178` | Primary buttons |
| Link | `#8FBDE7` | `#2B5178` | Links, references |
| Construction (cyan) | `#6FD3E6` | `#0D6274` | **Only** focus rings, the active-tab rule, measurements |
| Error | `#D94F45` | `#B3261E` | Diagnostics, destructive actions |
| Console | `#101315` | `#1B1F22` | Code blocks (dark in both themes) |

Contrast, measured: ink on ground about 17:1, muted on ground 8.5:1 (Night) and 6.2:1 (Day), white on action 5.2:1 / 8.2:1. Everything passes WCAG AA.

### 4.2 Type

- **Instrument Sans** (self-host the woff2 in `site/public/fonts/`, like Inter today): display 56/1.02 at 650 and −0.035em; H1 40/1.08; H2 26/1.2; H3 19/1.3; body 16/1.65; small 13.
- **JetBrains Mono**: code 13/1.65, and every number that measures.
- Labels are sentence case, never all caps. No eyebrows over headings.

### 4.3 Shape, space, depth

- Radius 4px everywhere (3px for labels); the logo keeps its own.
- Spacing on a 4px step: 4, 8, 12, 16, 24, 32, 48, 64.
- Depth is a hairline, not a shadow. Only overlays (search, menus) float.
- **The grid:** 24px, `rgba(143,189,231,.07)`, only behind previews and stages.

### 4.4 Signature behaviours

- **Active tab:** a sheet pulled forward, with a 2px cyan top rule.
- **Sidebar current item:** a cyan inset rule on its left edge.
- **Measure:** on a reference preview, a toggle draws the real height, padding and radius in cyan.
- **Asides:** a cyan dot and leader line before the note; no boxed callouts with a coloured left border.

### 4.5 Components to build

Header (tabs, search trigger, output switch, theme toggle) · sidebar (book and index variants) · on-this-page rail · mode label and badges · button (four tones, three sizes, outlined, pill) · field (hint and error) · switch · segmented control · code block (tabs, copy, line numbers) · live editor (editor, preview, diagnostics, *Break it*, status bar) · diagnostic block · preview stage (toolbar, grid, Measure, snippet strip) · props table · example row · quiz · recipe card · output card · cheat-sheet row · search palette · footer.

Every component is a `.wf` component in `site/src/components/`, using `$tokens` from `site/src/theme.wf`. `theme Night` becomes the default, and `theme Day` the alternate.

---

## 5. Content: where each current page goes

The current guide has 52 chapters in `site/src/pages/guide/`. Each moves to exactly one kind of page, and its URL redirects.

| Current chapter | Becomes | Kind |
|---|---|---|
| 01 Introduction | Home and `/learn/about/why-webfluent` | About |
| 02 Getting started | Learn 1 (tour) | Tutorial |
| 03 Tutorial: Shelf | Learn 2 (tour) | Tutorial |
| 05 Tutorial: Todos | Learn 3 (tour) | Tutorial |
| 06 Tutorial: Field Notes | Recipe: a static site with Markdown content | How-to |
| 07 Tutorial: Pocket | Recipe: a desktop app with Tauri | How-to |
| 04 Coming from React, Vue or Svelte | `/coming-from/{react,vue,svelte}` | Reference |
| 08 Recipes | Split into `/recipes/*` | How-to |
| 09 Language basics | Learn 4 | Tutorial |
| 10 Pages and routing | Learn 5 | Tutorial |
| 11 Elements | Learn 6 | Tutorial |
| 12 State and reactivity | Learn 7, **quiz** | Tutorial |
| 13 Events | Learn 8 | Tutorial |
| 14 Control flow | Learn 9 | Tutorial |
| 15 Components | Learn 10, **quiz** | Tutorial |
| 16 Stores | Learn 11, **quiz** | Tutorial |
| 17 Types | Learn 12 | Tutorial |
| 19 Styling | Learn 13 | Tutorial |
| 23 Forms | Learn 14 | Tutorial |
| 24 Data | Learn 15, **quiz** | Tutorial |
| 29 Testing | Learn 16 | Tutorial |
| 37 PDF and slides | Learn 17, and the PDF/Slides quick starts | Tutorial |
| 18 Expressions | `/reference/language#expressions` | Reference |
| 20 Motion | Recipes (animate, route transitions) | How-to |
| 21 Media | Recipes (images, video) | How-to |
| 25 Realtime | Recipes (sockets, streams, peers) | How-to |
| 26 Offline | Recipe: work offline | How-to |
| 27 Content | Recipes (data files, Markdown pages) | How-to |
| 28 i18n | Recipe: translate (and RTL) | How-to |
| 34 Deploying | Recipes per host | How-to |
| 35 Environments | Recipe: env and secrets | How-to |
| 36 Desktop apps | Recipe: Tauri, Electron | How-to |
| 38 Server rendering | Recipe: templates from Rust and Node | How-to |
| 39 JavaScript interop | Recipes (scripts, libraries, custom elements) | How-to |
| 22 Accessibility | About: accessibility, plus A-code pages | About |
| 30 Security | About: the security model | About |
| 31 Performance | About: what a build ships | About |
| 33 Static and SPA | About: static and single-page output | About |
| 51 Upgrading | About: upgrading (4 → 5 and so on) | About |
| 52 Contributing | About: contributing | About |
| 32 Project structure | `/reference/project` | Reference |
| 41 CLI | `/reference/cli`, generated from `wf commands --json` | Reference |
| 42 Configuration | `/reference/config`, generated from `wf config schema --json` | Reference |
| 44 Built-ins | `/reference/*`, generated from `wf registry --json` | Reference |
| 45 Design tokens | `/reference/tokens`, generated from `wf theme --json` | Reference |
| 46 Runtime API | `/reference/runtime` | Reference |
| 47 Grammar | `/reference/grammar` | Reference |
| 48 Browser support | `/reference/browser-support` | Reference |
| 49 Glossary | `/reference/glossary` | Reference |
| 43 Diagnostics | `/errors` index and `/errors/<CODE>`, from the diagnostics registry | Error |
| 50 Troubleshooting | Folded into error pages and recipes | Error / How-to |

The old *cookbook* routes redirect to `/recipes`. The 77 reference pages under `/docs/reference/*` redirect to `/reference/*`.

**Writing rules** (from the research; apply to every page):
- One kind per page. A tutorial never pauses to list every prop; a reference page never teaches.
- Each page stands alone for someone who arrives from search: say what it is in the first sentence.
- Introduce every example with one sentence. Show it complete; no `...`. Mark anything omitted with a comment.
- Lead with a working example, then explain.
- Sentence case; plain verbs; no "simply" or "just".

---

## 6. Workstreams

Each workstream can move on its own; dependencies are noted.

**W1. Compiler in WebAssembly.** *Already in progress in a separate session (branch from the "Build the WebFluent compiler for WebAssembly" task).*
The only blocker found was `libwebp-sys` (C), used in `src/media/encode.rs`. The plan: gate WebP encoding behind a default feature, add a wasm-bindgen crate with `compile(files, config) → { files, diagnostics }`, and report the `.wasm` size. **Done when** a demo page compiles a counter in the browser and shows its diagnostics. Needed by W5.

**W2. The shared floor (verification).** No dependencies; start here.
- Every `.wf` block in the docs is compiled in CI (`wf check`), with a marker for blocks meant to fail (`// expect T05`). A failing sample fails the site build.
- `wf test` steps for the interactive examples.
- **Done when** a deliberately broken sample stops CI.

**W3. Generated reference and errors.** Depends on W4's templates; content needs no WASM.
- A small generator turns `wf registry --json`, `wf commands --json`, `wf config schema --json`, `wf theme --json` and the diagnostics registry (`diagnostics::codes`) into `data` JSON files under `site/src/`. Pages read them with `data` and `paths:`.
- Each component page: description, live preview (static until W5), props table, events, slots and parts, composition, the codes it can raise, "since", "works in".
- Each error page: meaning, the broken code, the real compiler output (captured at build time), the fix, why the rule exists, related codes and chapter.
- **Done when** all 77 built-ins and every diagnostic code have a page, and none is hand-written.

**W4. Theme, shell and templates.** No dependencies.
- `site/src/theme.wf`: make `Night` the default, `Day` the alternate; add the Blueprint tokens (§4); self-host Instrument Sans.
- Build the components in §4.5 and the templates T1–T7 from `blueprint-pages/`.
- Header, footer, search trigger, output switch (stored with `persist`, filtering via badges, never by forking content).
- **Done when** each template renders with placeholder content and passes `wf verify`, at 390px and 1440px, in Night and Day.

**W5. Live pieces.** Depends on W1.
- The live editor component: loads the WASM compiler on first edit (the static code and a pre-rendered preview paint first), with tabs (`.wf`, Preview, HTML, CSS, JS, PDF), *Break it*, and "Open in Playground".
- The Playground page: files, examples, preview (desktop/phone), output tabs, tests, Share (a URL holding the project), Download.
- The reference stage controls (tone, size, flags), generated from the registry.
- **Done when** the Home hero, every chapter's "Try it" and the Playground run the real compiler with nothing sent to a server.

**W6. Content migration and writing.** Uses W4's templates.
- Move every chapter per §5; add mode labels; split recipes; write the "About" essays; write `/coming-from/*` (React, Vue, Svelte, HTML & CSS, LaTeX/Typst); write quizzes for chapters 7, 10, 11 and 15.
- Keep `site/src/translations/ar.json` in step: every new string goes through `t()`, and RTL is checked from the first paint.
- **Done when** every old URL redirects to a new page and no new page is a stub.

**W7. Search and agent surfaces.**
- ⌘K palette over `search-index.json` (already generated): exact jumps for component names and codes, grouped results by kind, synonyms (`useState` → `state`, `computed` → `derived`, `v-for` → `for`).
- `.md` twin of every page, `llms.txt`, `llms-full.txt`, "Copy page".
- Later, only if wanted: an MCP endpoint, and an "Ask the docs" assistant that answers only from cited, version-scoped pages.

**W8. Quality and launch.**
- `wf verify` on every route, a performance budget (`build.budget`), accessibility (contrast, focus, keyboard, screen reader), RTL, Night and Day, 390/768/1440.
- Redirects tested from the old sitemap.
- Analytics for quizzes (counts per answer, no accounts), to find which paragraphs to rewrite.

---

## 7. Phases

| Phase | Ships | Workstreams | Exit check |
|---|---|---|---|
| 0. Foundations | Blueprint theme, shell, templates; CI compiles every sample | W2, W4 | Templates pass `wf verify`; a broken sample fails CI |
| 1. Generated reference | `/reference/*` and `/errors/*` live, old reference URLs redirect | W3 | 77 components and all codes have pages |
| 2. Do things | `/recipes/*`, `/coming-from/*`, the new Home (static hero) | W6 | Every old guide URL that maps to a recipe redirects |
| 3. The book | `/learn` restructured, About essays, quizzes | W6 | Learn 1–17 published; quizzes record answers |
| 4. Live | Hero editor, Try it, Playground, reference controls | W1, W5 | The compiler runs in the browser on every live block |
| 5. Search and agents | ⌘K palette with synonyms, `.md` twins, `llms.txt` | W7 | `useState` finds `state`; every page has a twin |
| Launch | Old site retired | W8 | Checklist in W8 passes |

Phases 0–3 need no WebAssembly, so the site can ship the new structure before the live pieces land. Until Phase 4, live blocks show the code, a static pre-rendered preview and the compiled output captured at build time.

---

## 8. Open questions

1. **Showcase and 404** are not designed yet. Do them in Blueprint before Phase 2.
2. **Arabic:** translate the whole new site at launch, or English first and Arabic by phase?
3. **Versions:** keep only the latest docs, or a version switcher (5.x) from the start?
4. **WASM size:** decide the budget once W1 reports the size; if it is large, the editor stays code-only until the reader presses Run.
5. **Ask the docs:** build it, or stop at `llms.txt` and the MCP endpoint?
6. **Quiz analytics:** which endpoint receives answers, given the strict CSP (`meta.connect`)?

---

## 9. Next step

Start Phase 0: W4 (theme and templates from `blueprint-pages/`) and W2 (compile every sample in CI) in parallel. Then publish the remaining Blueprint pages to the canvas from `blueprint-pages/`, if a visual review is wanted first.
