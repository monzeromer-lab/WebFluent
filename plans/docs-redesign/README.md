# WebFluent docs rewrite: the plan

Status: **Phase 0 done** (2026-11-13). Next is Phase 1, the generated reference and errors (§9 W3). Progress is in §14.
Version 2, 2026-11-13. It replaces the first plan (2026-11-12). The design decisions in that plan stand, and are folded in here. The first plan was written before 6.0 and before the compiler ran in a browser, so the parts it left open are now settled with numbers.

This file is the single source of truth for the rewrite. Every phase ends in a site that can ship.

---

## 0. In one page

We are rewriting the WebFluent documentation as a new site with five kinds of page:

- **Learn:** a 17-chapter book. The first three chapters are a tour that runs in the browser, and the hard chapters have quizzes.
- **Recipes:** "how do I…" answers with complete, runnable code.
- **Reference:** generated from the compiler's own tables, never written by hand.
- **Errors:** one page per diagnostic code, linked from every finding the compiler prints.
- **About:** short essays that explain why things work the way they do.

Around those: **Coming from** pages for React, Vue, Svelte, plain HTML and CSS, and LaTeX. A **Playground** that runs the real compiler as WebAssembly. A **Markdown twin of every page** for agents.

The look is **Blueprint**: Night first, with the current palette and the `wf` logo.

Three things make this plan different from a fresh redesign:

1. **The guide is code.** `md-docs/` is compiled into `wf` itself (`wf context`, `wf explain`, the `docs:` link on every diagnostic), and the test suite parses, checks and runs every example in it. The rewrite keeps that, and extends it to every page.
2. **The compiler now runs in the browser.** `crates/wf-wasm` and `wf.js` shipped in 5.14. Live examples are a front-end job now, not a research question.
3. **Everything a reference page needs is already data.** `wf registry --json` (111 built-ins, 83 icons), `wf commands --json`, `wf config schema --json`, `wf theme --json`, and `diagnostics::codes` (128 codes).

---

## 1. Goals, and how we will know

| Goal | Measured by | Target |
|---|---|---|
| A newcomer gets something running quickly | Time from Home to a changed, running page, in the in-browser tour | Under 2 minutes, with nothing installed |
| Nothing in the docs is wrong | Every `wf` block parsed, checked and built in CI; every whole-page example run | 100%; a broken sample fails CI |
| Reference never drifts | Hand-written reference pages | 0 (all generated from the compiler) |
| Every finding has a page | Codes in `diagnostics::codes` with an `/errors/<CODE>` page | 128 of 128 |
| A reader lands on the right kind of page | Pages labelled with their kind; pages that mix kinds | Every page labelled; none mixed |
| Readers from other frameworks find things | Search for `useState`, `v-for`, `$:`, `onClick` returns the WebFluent term first | Every synonym in the table (W7) |
| It is fast and accessible | `wf verify` budget; WCAG AA in Night and Day; keyboard and screen-reader pass | FCP < 1 s on every route; no AA failures |
| Agents get the same docs | `.md` twin per page, `llms.txt`, `llms-full.txt` | Every page has a twin |
| Links from old binaries keep working | Every `docs:` URL printed by 5.x and 6.0, and every URL in today's sitemap (132) | All resolve to the new page |

---

## 2. What is already decided

| Decision | Choice | Where to look |
|---|---|---|
| UX direction | **All ten options combined** into one experience | Canvas page *Combined experience* |
| Visual system | **Blueprint**, Night first, Day as the alternate | Canvas pages *Blueprint: the site* and *Design system: pick one* |
| Colours and logo | The current palette (`site/src/theme.wf`) and the blue `wf` tile | §7 |
| Type | Instrument Sans for reading and the interface; JetBrains Mono for code and measurements | §7 |
| Live examples | The real compiler, in the browser, as WebAssembly | §5.5 |

**Design canvas:** https://claude.ai/artifact/Q7U2qAypKASeJ1oTYdV6ve

- **Blueprint: the site:** Home and the Button reference page. This is the visual target.
- **Combined experience:** every page type, in the earlier light styling. This is the structural target.
- **UX options (10), Desktop · dark / light, Phone:** the research directions and the current site.

**In this folder:**

- `blueprint-pages/`: ten pages drawn in Blueprint (Home, Learn, Recipe, Reference, Error, Coming from, Search, Playground, and two phone views). Use them as the visual spec.
- `generators/`: the scripts that drew the artboards. They are scratch tools, not part of the build.

**Research:** `research/Programming language documentation UX.md` (the sourced report) and `research/notes/`.

**Rejected, so we don't revisit them:**

- Light-first designs.
- The first three systems (Compiler, Workbench, Field Manual).
- A reference page built around a measuring diagram with leader labels everywhere. Measurements appear only when the reader turns on *Measure*.

---

## 3. What changed in the project since the first plan

| Change | What it means for the docs |
|---|---|
| **The compiler runs in a browser** (5.14). `crates/wf-wasm`; `wf.js` exports `compile`, `check`, `explain`, `version`, `preview`, `previewConfig`, `preload` and `registerFont`; `preview-sw.js` serves a compiled project to a frame. The web build is 4.8 MB (about 1.6 MB gzipped). The PDF build is a separate 8.4 MB file, loaded only for a PDF or slides config. | The live editor, Try it, *Break it* and the Playground can be built now. The size decides how they load (§5.5): never on page load, only on first interaction. A PDF example loads the larger build only when asked. |
| **6.0: a page can declare `lang:` and `translates:`**, and `R05` checks them. `?lang=` is no longer an `hreflang` alternate. | Arabic pages become real pages at `/ar/…`, each indexed in its own language. This settles the "Arabic at launch?" question (§12). The feature itself needs a recipe and a Learn mention. |
| **6.0: the Rust library has a documented facade**, typed errors, `#[non_exhaustive]` and the `internals` feature. | A new *Library* section in Reference: Rust (`Template`, `build`, `memory`, `vfs`), Node (`webfluent` on npm) and the browser (`wf.js`). docs.rs stays the item-level reference; we link to it and don't copy it. |
| **6.0 hardening:** `E120` and `E121`; `--allow-output-outside`; `wf serve --host`; `WF_ALLOW_SHELL_ENV`; stricter escaping and CSP read-back. | New error pages. The Security essay and Environments recipe are updated. The site's own build now needs `--allow-output-outside`, because it writes to `../docs`. |
| **Builds are fast** (the docs site builds in 0.36 s; `wf check` takes 62 ms). | Building the whole site and compiling every sample in CI is cheap, so every PR can afford it. |
| **The registry grew** to 111 built-ins (including PDF and Slides elements), 83 icons, 8 retired modifiers, and the universal props. | The reference index is grouped by family (Layout, Navigation, Data display, Form, Feedback, Actions, Media, Typography, PDF, Slides, Routing). The first plan's "77" was out of date. |
| **The editors live in their own repositories** (tree-sitter-webfluent, webfluent-zed, webfluent-vscode). | An *Editors* reference page links out and lists what each supports. Install steps live in Learn 1 and the editors' READMEs. |
| **The agent skill and `AGENTS.md` are versioned with the release** (`skills/writing-webfluent`, v6.0.0). | The agent pages (`llms.txt`, `.md` twins) and the skill must say the same things. One generator feeds both where it can (W7). |

---

## 4. How the docs are built today, and what depends on it

You can't move anything safely until you know this.

```
md-docs/NN-slug.md   (53 files, about 156,000 words; the source)
   │
   ├─ scripts/site-from-guide.py ─▶ site/src/pages/guide/NN-slug.wf   (generated; "edit the guide, not this file")
   ├─ scripts/components-reference.py ◀─ wf registry --json ─▶ md-docs/40-components-reference.md
   ├─ scripts/site-data.py ─▶ site/src/registry.json, site/public/search-index.json
   │
   ├─ build.rs ─▶ guide_files.rs ─▶ `wf context --guide full` (every chapter, inside the binary)
   ├─ src/diagnostics/codes.rs: include_str!("md-docs/43-diagnostics.md")
   │      ├─ guide_section(code) ─▶ `wf explain CODE` and the editor's "explain"
   │      └─ docs_url(code) ─▶ https://webfluent.monzeromer.dev/docs/guide/diagnostics#<anchor>
   │                           (printed under every finding, by every shipped `wf`)
   └─ tests
          ├─ docs_parse.rs: every `wf` block parses and checks; the sidebar lists every chapter;
          │                 site pages are current with md-docs; the tokens chapter matches the baseline
          └─ guide_runs.rs + tests/js/guide.test.mjs: every whole-page example builds, mounts, is clicked and typed into
```

The site (`site/`) builds with `wf build` into `../docs` (GitHub Pages, `CNAME`). It is a static build with `clean_urls: "file"`, a strict CSP, Day/Night themes, `en`/`ar` interface strings (`site/src/translations`, about 250 keys) and Google Analytics behind a consent banner. Redirects are pages with `guard: false` and `redirect:` (`site/src/pages/Redirects.wf`).

**Rules this sets for the rewrite:**

1. **Markdown in the repository stays the source of truth.** The compiler embeds it, the tests read it, and agents read it. Pages are generated from it, never the other way round.
2. **The compiler's `docs:` URLs are a public contract.** Old binaries keep printing `/docs/guide/diagnostics#t05-…` forever. A fragment never reaches a server, so the old diagnostics page must stay, as a page whose script sends `#t05-…` to `/errors/T05`.
3. **Every test above keeps passing at every phase.** Each is rewritten for the new tree in the same change that moves the content (W2).

---

## 5. The architecture of the rewrite

### 5.1 Sources: one tree, by kind

`md-docs/` is reorganised by kind. Every file carries front matter instead of the HTML comment it carries today.

```
md-docs/
├── README.md                     the contents, generated
├── learn/01-first-page.md … 17-pdf-and-slides.md
├── recipes/<slug>.md             one task each (about 45; list in §6.3)
├── about/<slug>.md               essays: why-webfluent, reactivity, security, what-a-build-ships, …
├── coming-from/{react,vue,svelte,html-css,latex}.md
├── reference/                    the prose around generated tables
│   ├── language.md  grammar.md  expressions.md  types.md  runtime.md
│   ├── project.md  browser-support.md  glossary.md  library.md  editors.md
│   └── components/<slug>.md      optional: hand-written notes merged into a generated page
├── errors/<CODE>.md              why the rule exists and how to fix it (128 files, split from 43-diagnostics.md)
└── changelog.md                  generated from RELEASE_NOTES.md
```

Front matter (one schema, checked by a test):

```yaml
---
kind: tutorial | recipe | about | reference | error | coming-from
title: Keep the active tab in the address
description: One sentence a search result shows.   # ≤ 160 chars (S03)
route: /recipes/active-tab-in-address
outputs: [spa, static]          # "works in" badges; omit for all
since: "5.9"                    # first release the page's feature is in
prereqs: [learn/07-state]       # Learn chapters a recipe assumes
related: [recipes/filters-in-address, reference/link]
quiz: learn/07-state.quiz.yaml  # tutorials only
lang: en
translates: null                # for /ar/ pages: the English page's route
---
```

### 5.2 One generator, run by `cargo xtask site`

Three scripts (`site-from-guide.py`, `site-data.py`, `components-reference.py`) become one package, `scripts/site/`, with a single entry point: `cargo xtask site`. The xtask runs `wf` from `target/` and the Python package.

| Module | Reads | Writes |
|---|---|---|
| `content.py` | `md-docs/{learn,recipes,about,coming-from}` | `site/src/pages/<kind>/<slug>.wf`, framed by the kind's layout |
| `reference.py` | `wf registry/commands/config schema/theme --json`, `md-docs/reference/` | `site/src/data/{components,commands,config,tokens}.json` plus the prose pages |
| `errors.py` | `diagnostics::codes` (through `wf explain --json`, a new flag; see W3), `md-docs/errors/` | `site/src/data/errors.json` |
| `examples.py` | every fenced `wf` block | `site/public/examples/<hash>/` (built output for the static preview) and `site/src/data/examples.json` (the HTML/CSS/JS tabs, and the findings for a block meant to fail) |
| `search.py` | everything above | `site/public/search-index.json`, with kinds and synonyms |
| `twins.py` | the Markdown sources and the data | `site/public/**/*.md`, `llms.txt`, `llms-full.txt` |

Generated files carry the "generated, edit `md-docs/…`" header they carry today, and a test holds them current (`docs_parse.rs::the_sites_guide_pages_are_current_with_the_guide`, generalised).

### 5.3 Generated pages use the language's own features

The reference and error pages are **not** one `.wf` file each. They are one page per family, using `data` and `paths:`, which is how a static build writes a page per item. That dogfoods the feature the docs teach.

```wf
data components: [Component] = "data/components.json"

page ComponentPage(path: "/reference/:slug", slug: String,
                   paths: components.map(c => c.slug),
                   title: "{components.find(c => c.slug == slug)?.name ?? slug} — Reference",
                   layout: ReferenceShell(slug: slug)) {
    derived c = components.find(c => c.slug == slug)
    …
}
```

The same pattern gives `/errors/:code`, `/reference/cli/:command` and `/ar/learn/:chapter`.

### 5.4 The compiler moves with the docs

These changes go in the same PR as the content they follow (W3):

- `docs_url(code)` returns `https://webfluent.monzeromer.dev/errors/<CODE>`.
- `guide_section(code)` reads `md-docs/errors/<CODE>.md`, embedded by `build.rs` as a table, instead of searching one chapter.
- `wf context` reads the new tree. The route, title and kind come from front matter, and `--guide full` keeps every page.
- `wf explain --json` (new) prints a code's title, family, severity, whether `lints` may lower it, its example and its fix kind. `errors.py` reads it, so the site never parses `codes.rs`.
- **Old binaries:** `/docs/guide/diagnostics` stays as a small page with a plain script (`src/legacy-anchors.js`, on `wf:render`). It maps every old anchor to `/errors/<CODE>`, from a table generated by `errors.py`.

### 5.5 Live examples

Every code block is one component, `Example`, with three levels. Each level costs nothing until the reader asks for more:

| Level | What the reader gets | What it costs |
|---|---|---|
| **Static** (default) | The code, highlighted, with copy. For a page example: tabs for **Preview · HTML · CSS · JS** (and **PDF** for a document), all captured at site build time. Preview is a same-origin `Frame(src: "/examples/<hash>/")` of the real compiled output. | No compiler download; `frame-ancestors 'self'` already allows it |
| **Editable** | An **Edit** button turns the code into an editor. The first keystroke loads `wf.js`, which loads the web build (about 1.6 MB gzipped). Findings appear under the code as the compiler prints them. **Break it** inserts the block's prepared mistake. | Cached after the first time; preloaded on hover or focus of Edit |
| **Playground** | **Open in Playground** carries the files to `/playground`. That page has several files, the phone and desktop preview, every output tab, `wf test`, Share (the project in the URL hash) and Download (a zip). | The PDF build loads only for a PDF or slides project |

Fence info strings say what a block is. The generator and the tests read the same strings:

````
```wf                          a fragment, checked inside a page (today's behaviour)
```wf page                     a whole page: built, previewed, run by guide_runs
```wf page live                as above, with Edit
```wf expect=T05               must fail with exactly this code (error pages, "common mistakes")
```wf break="nmae:name"        Break it swaps these two strings
```wf files=src/App.wf,src/pages/Home.wf   one block of a multi-file example
```wf no-check                 rare, needs a comment saying why (grammar excerpts)
````

**Site config the live pieces need:** `meta.wasm: true` (`'wasm-unsafe-eval'`) and the preview service worker's scope. Phase 4 also checks whether the preview frame needs `meta.frame` or a `worker-src` entry.

### 5.6 Translation

Arabic is done with 6.0's own features, not by switching the locale on one address:

- The interface strings (header, sidebar, buttons, footer) go through `t()`, as today, with both tables complete at every phase (`I03` is held at error for the site).
- A translated page is its own `.md` file, `md-docs/ar/<kind>/<slug>.md`, with `lang: ar` and `translates:` set. It is published at `/ar/…` and listed as an `hreflang` alternate of the English page. Untranslated pages have no `/ar/` twin, and the language switch on them says the page is English only.
- Code stays code: identifiers may be Arabic in an Arabic example (the language allows it), but keywords and built-ins are English.
- RTL is checked from the first paint, in both themes, with `raqeeb`, on every template.

---

## 6. Information architecture

### 6.1 Top navigation

`Learn · Recipes · Reference · Errors · Playground · Showcase`, then:

- search (⌘K);
- the **Building** switch (Website · App · PDF · Slides · Elements · Desktop/phone), kept with `persist`, which filters badges and examples but never forks content;
- Night/Day;
- English/العربية;
- GitHub.

### 6.2 URL map

| Route | Page | Template |
|---|---|---|
| `/` | Home | T1 |
| `/learn` | The book: contents and progress | T2 index |
| `/learn/<nn>-<slug>` | A chapter (1–3 are the tour) | T2 |
| `/about`, `/about/<slug>` | Essays | T2 variant |
| `/recipes`, `/recipes/<slug>` | Recipe finder, a recipe | T3 |
| `/reference` | Index: built-ins by family; language, CLI, config, tokens, runtime, library, editors | T4 index |
| `/reference/<component>` | A built-in (111) | T4 |
| `/reference/{language,grammar,expressions,types,runtime,project,tokens,config,browser-support,glossary,library,editors}` | Reference pages | T4 variant |
| `/reference/cli`, `/reference/cli/<command>` | Every command, generated | T4 variant |
| `/errors`, `/errors/<CODE>` | Index by family, one code (128) | T5 |
| `/coming-from/<framework>` | Cheat sheets | T6 |
| `/playground` | The compiler in the browser | T7 |
| `/showcase` | Real outputs: sites, PDFs, decks (from `showcase.json`) | T8 |
| `/changelog` | Release notes, by version, with anchors | T4 variant |
| `/ar/…` | Arabic translations of the pages that have one | same templates, RTL |
| `**/*.md`, `/llms.txt`, `/llms-full.txt` | Machine-readable twins | generated |
| 404 | Search box, likely matches for the address, the main sections | T9 |

**Redirects.** Each of the 132 URLs in today's `docs/sitemap.xml` maps to one new route. The map is a generated table (`site/src/data/redirects.json`) read by one `Redirect` page family (`paths:` again). Each old page is `noindex`, carries `<link rel="canonical">` to its new home, and redirects. A test reads the old sitemap from git and checks that every URL has an entry and that the target exists.

### 6.3 Content map: every current chapter, and where it goes

The guide has 52 chapters in `md-docs/`. The site shows 51 of them; chapter 40, the components reference, is generated from the registry and lives only in `md-docs/`. Each chapter goes to exactly one kind of page.

**Learn: one path, 17 chapters**

| # | New chapter | From | Quiz |
|---|---|---|---|
| 1 | Your first page (tour; install, `wf init`, `wf serve`, change it) | 02 Getting started | |
| 2 | A small site: Shelf (tour) | 03 Tutorial: Shelf | |
| 3 | An app that remembers: Todos (tour) | 05 Tutorial: Todos | |
| 4 | Language basics | 09 | |
| 5 | Pages and routing (now including `lang:` and `translates:`) | 10 | |
| 6 | Elements | 11 | |
| 7 | State and reactivity | 12 | ✓ |
| 8 | Events | 13 | |
| 9 | Control flow | 14 | |
| 10 | Components | 15 | ✓ |
| 11 | Stores | 16 | ✓ |
| 12 | Types | 17 | |
| 13 | Styling | 19 | |
| 14 | Forms and validation | 23 | |
| 15 | Data from servers | 24 | ✓ |
| 16 | Testing | 29 | |
| 17 | PDFs and slides | 37 | |

**Recipes: one task each.** About 45 recipes come from 08, 06, 07, 20–21, 25–28, 34–36, 38–39, and the new 6.0 features:

- **Interaction:** debounce a search · page through results · load more · keep filters in the address · keep the active tab in the address · a confirm dialog · a responsive navigation · count down · copy to the clipboard · keyboard shortcuts · drag to reorder · a long list (`VirtualList`) · a resizable sidebar (`Splitter`).
- **Data:** an API described once (`api`) · from an OpenAPI file · optimistic updates · sign in with a session cookie · validation messages · a server's 422 errors on a form.
- **Realtime:** a socket · server-sent events · tabs that talk (`broadcast`) · two readers on one line (`peer`) · a worker.
- **Content:** a static site from Markdown (Field Notes) · a page per data item (`data` and `paths:`) · images at every width.
- **Look:** a dark-mode switch · a theme from a brand colour · animate in and out · route transitions · draw a chart.
- **Reach:** translate a site (`i18n`) · **a site in two languages with `lang:` and `translates:`** · right-to-left layouts · works offline · environments and secrets.
- **Ship:** deploy to GitHub Pages / Netlify / Cloudflare / nginx · a desktop app with Tauri (Pocket) · the same app on a phone · Electron.
- **Interop:** your own JavaScript · a library from a CDN · a custom element · publish components as custom elements · **compile in the browser (`wf.js`)** · render templates from Rust · render templates from Node.

**About:** why WebFluent (01) · how reactivity works · accessibility the compiler gives you (22) · the security model (30, updated for 6.0) · what a build ships (31) · static or single-page (33) · upgrading, including 5 → 6 (51) · contributing (52).

**Reference:**

| Page | Source |
|---|---|
| Language | 18 Expressions, plus the language parts of 09 |
| Grammar | 47 |
| Project structure | 32 |
| CLI | 41, generated from `wf commands --json` |
| Configuration | 42, generated from `wf config schema --json` |
| Built-ins | 40 and 44, generated from `wf registry --json` |
| Design tokens | 45, generated from `wf theme --json` |
| Runtime API | 46 |
| Browser support | 48 |
| Glossary | 49 |
| **Library** (new) | Rust 6.0 facade, Node, `wf.js` |
| **Editors** (new) | The three editor repositories |

**Errors:** 43 Diagnostics splits into 128 pages. 50 Troubleshooting folds into the error pages and the recipes.

**Coming from:** 04 splits into React, Vue and Svelte pages. HTML & CSS and LaTeX/Typst are new.

**New pages with no source today:** Library · Editors · Changelog · Coming from HTML & CSS · Coming from LaTeX · the 6.0 recipes · the Learn tour's in-browser steps · the quizzes.

---

## 7. The design system: Blueprint

### 7.1 Colour (Night first)

| Role | Night | Day | Used for |
|---|---|---|---|
| Ground | `#0E1012` | `#F9FAFB` | Page background; the measuring grid where things are measured |
| Sheet | `#17191C` | `#FFFFFF` | Panels, the active tab |
| Raised | `#1F2225` | `#F4F6F7` | Inputs, hovered rows |
| Ink | `#EEF0F2` | `#17191C` | Text |
| Muted | `#A9AEB4` | `#5A5F64` | Secondary text |
| Line | `#2C3035` | `#E2E5E8` | Hairlines, borders |
| Action | `#3D6FA3` | `#2B5178` | Primary buttons |
| Link | `#8FBDE7` | `#2B5178` | Links, references |
| Construction (cyan) | `#6FD3E6` | `#0D6274` | **Only** focus rings, the active-tab rule and measurements |
| Error | `#D94F45` | `#B3261E` | Diagnostics, destructive actions |
| Console | `#101315` | `#1B1F22` | Code blocks (dark in both themes) |

Measured contrast:

- ink on ground: about 17:1;
- muted on ground: 8.5:1 (Night), 6.2:1 (Day);
- white on action: 5.2:1 (Night), 8.2:1 (Day).

Everything passes AA. `A13` holds it in the build.

### 7.2 Type, shape, depth

- **Instrument Sans**, self-hosted woff2 next to the current fonts, preloaded with `meta.preload`:
  - display 56/1.02 at weight 650, −0.035em;
  - H1 40/1.08, H2 26/1.2, H3 19/1.3;
  - body 16/1.65; small 13.
- **JetBrains Mono** for code (13/1.65) and every number that measures.
- **Text:**
  - Sentence case, never all caps.
  - No eyebrows over headings.
  - No middle-dot meta strings.
  - No arrows appended to links.
- **Radius:** 4px (3px for labels).
- **Spacing:** a 4px step.
- **Depth:** a hairline, not a shadow. Only overlays float.
- **Grid:** a 24px grid at `rgba(143,189,231,.07)`, only behind previews and stages.

### 7.3 Signature behaviours

- **Active tab:** a sheet pulled forward, with a 2px cyan top rule.
- **Sidebar current item:** a cyan inset rule on its left edge, or its right edge in RTL.
- **Measure:** a toggle on a reference stage that draws the real height, padding and radius in cyan. It is off by default.
- **Asides:** a cyan dot and a leader line. No boxed callouts with a coloured left border.

### 7.4 Components (all `.wf`, in `site/src/components/`, using `$tokens`)

| Group | Components |
|---|---|
| Shell | `SiteHeader`, `BuildingSwitch`, `ThemeToggle`, `LangSwitch`, `SiteFooter`, `SkipLink` |
| Navigation | `BookSidebar` (progress), `IndexSidebar`, `OnThisPage`, `PrevNext`, `Breadcrumbs` |
| Page furniture | `KindLabel`, `OutputBadges`, `SinceBadge`, `Aside`, `Prereqs`, `Related` |
| Code | `Example` (static, editable and playground levels, §5.5), `CodeTabs`, `Terminal`, `Diagnostic` (a finding drawn as `wf` prints it) |
| Reference | `Stage` (preview, grid, Measure, controls generated from props), `PropsTable`, `EventsTable`, `SlotsAndParts`, `ExampleRow`, `CodeFamily` |
| Learning | `Quiz` (one question, choices, why each is right or wrong), `TourStep`, `Progress` |
| Finding things | `SearchPalette` (⌘K), `RecipeCard`, `RecipeFilter`, `CheatRow` |
| Playground | `FileTree`, `Editor`, `PreviewPane` (desktop and phone), `OutputTabs`, `TestPanel` |

### 7.5 Templates

| | Template | What a page of it has, top to bottom |
|---|---|---|
| T1 | Home | "What are you making?" (six outputs) → live hero editor (static until clicked) → the three ways in (Learn, Recipes, Reference) → one example per output → install |
| T2 | Chapter / essay | Kind label, title, one-sentence summary, prerequisites → body with `Example`s and asides → quiz (if any) → "What you learned" → prev/next. Book sidebar with progress (kept with `persist`) |
| T3 | Recipe | Title as the task → outputs and since badges → the complete code first → how it works, step by step → variations → related recipes and reference |
| T4 | Reference item | Name, one-line summary, signature → `Stage` with controls → props, events, slots and parts → composition examples → codes it can raise → since and works-in → hand notes (if any) |
| T5 | Error | Code, title, severity, family → what the compiler prints (real output, captured) → why → the broken code → the fixed code (diff) → "may be lowered by `lints`?" → related codes and chapter |
| T6 | Coming from | Two-column cheat sheet (their idiom → ours), grouped by concept → three "this works differently" sections → a port of one small component |
| T7 | Playground | Files, editor, preview, outputs, findings, tests, Share, Download, examples menu |
| T8 | Showcase | Filter by output → cards with preview, size, pages, and links to source and recipe |
| T9 | 404 | What happened, the closest routes for the address, search, sections |

**Designs still to draw:** Showcase (T8), 404 (T9), the About essay variant of T2, the Changelog, and the Arabic RTL version of T2 and T4. They are drawn in Blueprint on the canvas in Phase 0, before they are built.

---

## 8. How we write

The rules come from the research. Each one is short enough to check in review.

**Every page:**

1. **One kind per page.** A tutorial never stops to list every prop; a reference page never teaches; an error page never explains a feature from scratch.
2. **The first sentence says what the page is**, for a reader who arrives from search.
3. **Lead with a working example**, then explain it. Introduce each example with one sentence.
4. **Examples are complete.** No `...`. If something is left out, a comment says what. (The test enforces this.)
5. **Name things as the compiler names them.** `derived`, not "computed"; "finding", not "warning", when it could be either. The glossary is the arbiter.
6. **Plain verbs, sentence case, active voice.** No "simply", "just", "easy", "obviously", "powerful" or "seamless". A new `scripts/site/prose.py` flags them; `check-vocabulary.mjs` is about the editors' grammar, not prose.
7. **Say what fails and how to fix it.** Every Learn chapter ends with its two or three most likely mistakes, each a `wf expect=CODE` block linked to its error page.
8. **Interference is explicit.** Where a React/Vue/Svelte habit breaks (61% of cross-language questions show it), an aside names the habit and the WebFluent way, and links the Coming-from page.

**By kind:**

- **Tutorial:** a goal, then small steps, each ending in something that runs. Never a choice between two ways. At most one new idea per section.
- **Recipe:** the title is the task ("Keep filters in the address"). The full code comes first. Variations come after. No theory beyond one paragraph.
- **Reference:** complete, neutral, scannable. Generated wherever data exists; hand prose only for what the data cannot say.
- **Error:** what you see, why, then the fix. The fix is a diff.
- **About:** an argument, with a point of view. It is allowed to be read once and never again.

**Quizzes:**

- Two to four questions at the end of chapters 7, 10, 11 and 15.
- Each question targets one misconception from the research or from real findings (a `state` from `now` that never ticks, `X06`; `for` without `by`; a store read without `use`).
- Each wrong answer explains itself.
- Answers are counted as anonymous GA events behind the existing consent banner, so there's no new endpoint and no CSP change. A chapter whose answers run more than 40% wrong gets its section rewritten.

---

## 9. Workstreams

Each workstream can move on its own; dependencies are named. Sizes: S (a day or less), M (a few days), L (a week or more).

### W1. Compiler in WebAssembly: **done** (5.14)

`crates/wf-wasm`, `wf.js`, `preview-sw.js`, and two builds (web and PDF).

What remains belongs to W5:

- Serve `wf.js` and the `.wasm` files from the site, under `/wasm/<version>/`, cached forever.
- Copy them in at site build from `crates/wf-wasm/pkg*`.
- Check the size against the budget in CI.

### W2. The verification floor (M). No dependencies; first.

- Teach `docs_parse.rs` and `guide_runs.rs` the new tree and the fence strings (§5.5).
- A `wf expect=CODE` block must produce exactly that code, and nothing else at error level.
- A front-matter test: the schema, unique routes, `description` within 160 characters, `prereqs` and `related` that exist, and every page reachable from the index.
- A link test: every internal link in `md-docs/` points to a route that exists.
- An old-URL test: every `<loc>` in the 6.0 `docs/sitemap.xml`, and every anchor `docs_url` has ever produced (generated from `codes.rs` at the 6.0 tag), resolves.
- `cargo xtask site --check` regenerates into a scratch directory and fails on a difference, as the docs test does today.
- **Done when:** a deliberately broken sample, a dead link and a removed old URL each fail CI.

### W3. Generated reference and errors (L). Needs W4's T4/T5.

- Add `wf explain --json`, with the fields listed in §5.4, and a test.
- Split `43-diagnostics.md` into `md-docs/errors/<CODE>.md` (scripted, then reviewed). Fill the missing sections, so every code has a *why*, a broken example and a fix.
- Change `codes.rs`:
  - `guide_section` reads the per-code files;
  - `docs_url` returns `/errors/<CODE>`.
- Update the snapshot of `wf explain` output and the editor's explain test.
- Write `reference.py` and `errors.py`, plus the page families (`/reference/:slug`, `/errors/:code`, `/reference/cli/:command`).
- `Stage` controls generated from each prop's kind: enum → segmented control, `Bool` → switch, `String` → field.
- Each component page lists the codes it can raise (a table kept in the registry or in `errors.py`, tested against `codes.rs`).
- Write the Library and Editors reference pages.
- **Done when:** all 111 built-ins, every command and all 128 codes have pages, `wf explain T05` prints the new file, and a `docs:` link opens the new page.

### W4. Theme, shell and templates (L). No dependencies; first.

- **Theme:**
  - `theme.wf` makes `Night` the default and `Day` the alternate;
  - add the Blueprint tokens (§7);
  - self-host Instrument Sans and drop Inter once nothing uses it.
- Build the components in §7.4 and the templates T1–T9 from `blueprint-pages/`.
- Draw the missing designs first (T8, T9, the essay variant, Changelog, RTL).
- `BuildingSwitch` is kept with `persist`. It filters badges and examples, and its default comes from the page the reader arrived on.
- The site builds with `--allow-output-outside` (recorded in `justfile` and CI).
- **Done when:**
  - each template renders with placeholder content;
  - `wf verify` passes at 390, 768 and 1440 px, in Night and Day, in English and Arabic;
  - `A13` raises nothing.

### W5. Live pieces (L). Needs W1 (done) and W4.

- `Example`, the editable level:
  - an editor (a `textarea` with highlighting from `WF.highlight` first; a richer editor only if it is needed);
  - `wf.js` loaded on first edit and preloaded on intent;
  - findings drawn with `Diagnostic`;
  - **Break it**;
  - "Open in Playground".
- The Learn tour (chapters 1–3): `TourStep`s that check the reader's change with `compile` (for example, "the heading now says your name") before the next step unlocks.
- Home hero: the same editor, static until clicked.
- **Playground:**
  - files, examples, preview (desktop and phone);
  - outputs (HTML, CSS, JS, PDF);
  - tests;
  - Share (the project compressed into the URL hash);
  - Download.
- The reference stage's controls recompile the preview live once the compiler is loaded. Until then, they swap between pre-built variants.
- CSP for the live pieces: `meta.wasm`, the preview worker, and the frame. Verify with `wf verify`.
- **Done when:** the hero, every "Edit", the tour and the Playground run the real compiler, with nothing sent to a server, and a first visit to any page downloads no `.wasm`.

### W6. Content: move, rewrite, write (L, the largest). Uses W4's templates.

1. **Move (scripted):** split and move every chapter to its place in §6.3, convert the comment headers to front matter, and add redirects. The site keeps working at the end of this step.
2. **Rewrite, by kind, to §8:**
   - Learn 1–17 (tour steps and quizzes included);
   - about 45 recipes;
   - eight essays;
   - five Coming-from pages.
3. **Write the new pages** listed at the end of §6.3.
4. **Arabic:**
   - the interface strings are complete at every phase;
   - at launch, translate Home, Learn 1–3 and the Coming-from index;
   - after launch, chapter by chapter.

- **Done when:**
  - every old URL redirects to a page that is not a stub;
  - every page has a kind, a description and passes `prose.py`;
  - every Learn chapter ends with its mistakes.

### W7. Search and agent surfaces (M). Needs W6's sources.

- **⌘K palette over `search-index.json`:**
  - an exact jump for a component name or a code (`T05`, `Button`);
  - results grouped by kind;
  - recent pages;
  - keyboard only.
- **Synonyms:**
  - `useState`/`ref`/`$state` → `state`;
  - `useMemo`/`computed`/`$derived` → `derived`;
  - `useEffect`/`watch`/`$effect` → `effect`;
  - `v-for`/`{#each}`/`.map(` → `for`;
  - `v-if`/`{#if}` → `if`;
  - `props`/`defineProps` → props;
  - `slot`/`children` → slots;
  - `onClick`/`@click` → `on click`;
  - `className`/`:class` → `class:`;
  - `fetch`/`useSWR`/`useQuery` → `resource`;
  - `context`/`provide` → stores;
  - `\section`/`#set page` → `Document`.
- `.md` twin of every page, **Copy page**, `llms.txt` (the map) and `llms-full.txt` (everything).
- Check the skill's references and `AGENTS.md` against the new pages for contradictions, with a script that diffs the claims each makes about the same feature names. Contradictions are fixed in the source that is wrong.
- **Later, only if wanted:** an MCP endpoint, and an "Ask the docs" assistant that answers only from cited pages.
- **Done when:** `useState` finds `state` first, `T05` jumps to its page, and every page has a twin.

### W8. Quality and launch (M). Last.

- `wf verify` on every route, with a budget.
- `build.budget` for `app.js` and `styles.css`.
- **Accessibility pass:** keyboard through every template, a screen reader (Orca) on Learn, Reference, Error and Playground, contrast, focus visible, and reduced motion.
- **Right to left:** with `raqeeb` on every template.
- **Old URLs:** the sitemap and anchor tests (W2) pass against the deployed site, not only the build.
- **Analytics:** pages, searches with no result, and quiz answers, behind consent.
- **Launch checklist:**
  - `docs/` rebuilt;
  - `CNAME` kept;
  - sitemap submitted;
  - old site tagged in git;
  - the release notes and README point to the new routes.

---

## 10. Phases

| Phase | Ships | Workstreams | Exit check | Size |
|---|---|---|---|---|
| **0. Foundations** | The missing designs drawn; Blueprint theme, shell and templates; the new source tree with every chapter moved as-is; every test on the new tree | W2, W4, W6 step 1 | Templates pass `wf verify` (sizes, themes, languages); a broken sample, a dead link and a lost old URL each fail CI; the site looks new but says what it said before | L |
| **1. Reference and errors** | `/reference/*`, `/errors/*`, Library, Editors, Changelog; `wf explain --json`; new `docs:` URLs; old diagnostics anchors forwarded | W3 | 111 built-ins, every command and 128 codes have pages; `wf explain` reads the new files | L |
| **2. Do things** | `/recipes/*` (about 45), `/coming-from/*` (5), the new Home with a static hero | W6 | Every old URL that maps to a recipe redirects; every recipe's code builds | L |
| **3. The book** | Learn 1–17 rewritten, essays, quizzes | W6 | Learn published; quizzes record answers; every chapter ends with its mistakes | L |
| **4. Live** | Edit on every example, the tour, the hero, the Playground, live reference controls | W5 | The compiler runs on every live block; no `.wasm` on first load | L |
| **5. Search and agents** | ⌘K with synonyms, `.md` twins, `llms.txt`, the skill/`AGENTS.md` check | W7 | `useState` finds `state`; every page has a twin | M |
| **Launch** | The old site retired; Arabic Home, Learn 1–3 and Coming-from index | W8 | Everything in W8 passes on the deployed site | M |

Phases 0–3 need no WebAssembly, so the new structure can ship before the live pieces land. Until Phase 4, every example shows its code and its output captured at build time.

Each phase is one branch, merged to `master` and deployed when its exit check passes, so the live site improves phase by phase rather than in one cut-over.

**Running in parallel:** once Phase 0 is merged, Phase 1 (compiler-side work and generators) and Phase 2 (writing) touch different files and can run in separate sessions. Phase 4 can start beside Phase 3 once T2 and `Example` exist.

---

## 11. Risks

| Risk | What we do about it |
|---|---|
| Moving `md-docs/` breaks the compiler's embedded guide (`wf context`, `wf explain`) | Phase 0 moves the files *and* updates `build.rs`, `context.rs` and `codes.rs` in the same change; the tests that read them run on every commit |
| Old binaries print URLs we can't redirect on the server (fragments) | The `legacy-anchors` page (§5.4), with a test over every anchor the 6.0 tag can print |
| GitHub Pages can't send real 301s | Redirect pages carry `noindex` and a canonical to the new page; the sitemap lists only new routes |
| 1.6 MB of compiler slows the site | Never loaded on page load; only on Edit, the tour or the Playground; cached by version; the budget is checked in CI |
| The rewrite stalls half-way, with two styles of page | Phase 0 moves everything into the new templates first; rewriting happens in place, so a half-done phase still ships one coherent site |
| Docs and the skill/`AGENTS.md` drift apart | The W7 contradiction check; the skill's references point to site pages where they can |
| The Arabic translation lags | Only translated pages exist under `/ar/`; the switch says plainly when a page is English only; `I03` keeps the interface complete |
| Quizzes count nothing because readers decline analytics | Accepted: the counts guide rewrites, they aren't a metric we report |

---

## 12. Decisions, all settled

**Settled by 6.0, 5.14 and this plan:**

1. **WebAssembly size:** about 1.6 MB gzipped for the web build. It is never loaded on page load, only on intent (§5.5).
2. **Arabic:** real pages with `lang:` and `translates:` (§5.6). The interface is complete from Phase 0. Home, Learn 1–3 and the React cheat sheet (in place of a Coming-from index, which the site does not have) were translated at launch; the rest follow.
3. **Quiz analytics:** GA events behind the existing consent banner. No new endpoint and no CSP change.
4. **Versions:** latest only, with `since` badges and a Changelog. A version switcher waits for the first breaking language change that needs one.

**Settled at launch (2026-11-13), each as recommended:**

5. **Typeface:** Instrument Sans, self-hosted, Latin subset. Arabic text uses a system font.
6. **Ask the docs:** no assistant. Agents read the Markdown twins and `llms.txt`; revisit after launch if readers ask for one.
7. **The editor in the browser:** the plain highlighted textarea. No new origin and no CSP change; upgrade only if the Playground needs it.
8. **`md-docs/` name:** kept, since `build.rs`, the tests and the scripts name it.

Nothing in this section needs a call any more.

---

## 13. Next step

The plan is complete (see §14). What comes next is the post-launch review: every page swept at three widths, the content read part by part, and the Playground tested, with the findings in `plans/docs-redesign/review-2026-11-14/`.

---

## 14. Progress

### Phase 0

**Done (2026-11-13, commits `5543637f`, `dc98a223`; grammar repo `4e1e7c0`, not pushed):**

- `md-docs/` is arranged by kind:
  - `learn/01–17` (numbered in reading order);
  - `recipes/`, `about/`, `reference/`, `errors/` and `coming-from/`.
- Each page opens with front matter: `chapter`, `kind`, `route`, `group`, `blurb`, `description`. Quoted values are JSON strings.
- Every link between pages, and from `README.md`, `RELEASE_NOTES.md` and `spec/`, follows the move.
- The compiler reads the new tree:
  - `build.rs` embeds every page;
  - `wf context` takes number and route from front matter;
  - `wf explain` reads `errors/diagnostics.md`.
- The three site scripts walk the tree.
- **Tests:**
  - shared helpers in `tests/common/mod.rs` (`guide_pages`, `guide_chapters`, `front_matter`);
  - links resolve from the page's folder;
  - each page lives in its kind's folder.
- **The old-URL safety net (W2):** `tests/docs_urls.rs` holds the site to every URL in the 6.0 sitemap, and the diagnostics page to every `docs:` link 6.0 prints. Both lists are in `tests/fixtures/docs-urls/`.
- A deliberately broken sample, a dead link and a lost URL were each checked to fail.

**Decided while doing it:**

- **Routes stay as they are in Phase 0.** A page's route changes in the phase that rewrites it (Phases 1–3), with a redirect left behind. Moving routes now would chain redirects twice for every chapter that is later split.
- **Chapter numbers stay** (`chapter:`) for the sidebar, prev/next and the `ch.NN` translation keys, until Phase 3 replaces the book's navigation.
- **Whole chapters moved, none split yet.** The splits happen in the phase that rewrites them:
  - `errors/diagnostics.md` into one file per code (Phase 1);
  - `recipes/common-tasks.md` and the multi-task recipe chapters (Phase 2);
  - `coming-from/react-vue-svelte.md` into three pages (Phase 2).

**Also done (merged 2026-11-13, `9d934cb7`, `294b9e85`, site rebuilt in `9dc34e4f`):**

- **Designs:** Showcase, 404, essay, Changelog, and Arabic T2 and T4, on the canvas and in `blueprint-pages/`.
- **Theme and shell:**
  - Night by default, Day as the alternate;
  - Instrument Sans;
  - every component in §7.4;
  - templates T1–T9 (`DocsShell` is T2), previewed at `/blueprint/*` and `/ar/blueprint/*`, marked `noindex`;
  - the header with search, the Building switch, the theme toggle and the language switch.
- **Known gaps:**
  - The header and footer link to routes that arrive in Phases 1–3.
  - `Guide.wf`'s chapter list is written by hand: the generator should write it, then retire `DocSidebar.wf` (`U03`).
  - **Compiler:** a reader who chose Day sees one Night frame first, because the config can't say "dark by default".
  - **Compiler:** `t()` ignores a page's `lang:`, so pre-rendered `/ar/` pages show English interface text until the reader switches language.

**Finished (2026-11-13, `52544575` and the rebuild after it):**

- **`cargo xtask site [--check]`:** the three scripts are one package, `scripts/docsite/` (`guide.py`, `data.py`, `components.py`, `examples.py`, `fences.py`).
  - The package is `docsite`, not `site`, because `site` would shadow Python's own module.
  - `--check` writes into a scratch directory and fails when a committed file is behind.
- **The chapter list:** the generator writes the book's list, `site/src/components/Guide.wf`, from front matter. `DocSidebar.wf` is gone, and the tour is `learn/01–03`.
- **Fence words** (§5.5): `page`, `live`, `no-check`, `expect=CODE`, `break="from:to"` and `files=a,b`.
  - They are read by `tests/common/mod.rs::fence` and `scripts/docsite/fences.py`.
  - A word no fence takes fails the docs tests and the site check.
  - The guide's `wf expect CODE` fences became `wf expect=CODE`.

**Left from §9 for later phases, by design:**

- The front-matter fields `outputs`, `since`, `prereqs` and `related` are added, and checked, as pages are rewritten (Phases 1–3).
- `examples.py`'s captured outputs, `twins.py` and the `errors.py` and `reference.py` split of `data.py` arrive with the pages that need them (Phases 1, 4 and 5).

**Notes for whoever continues:**

- The grammar submodule commit must be pushed before this repository's.
- `site/` builds with `--allow-output-outside`.

### Phase 1 — errors and reference

**Done (2026-11-13; merged in `9d0f66c9`):**

- A page per diagnostic at `/errors/CODE`, written from `md-docs/errors/` by `errors.py`. Each page shows its program fixed (the `fixed` fence word) beside the broken one.
- `wf explain --json`; `docs_url` points at `/errors/CODE`.
- The reference pages list the codes each built-in can raise, and each entry's since badge.
- Old `#diagnostics` anchors forward through `site/src/old-links.js`.

### Phase 2 — recipes and Coming-from

**Done (2026-11-13, `ef2083bc`, `dc2f3664`):**

- 75 single-task recipes and two project recipes at `/recipes/*`, from `md-docs/recipes/` by `recipes.py`. The shape: intro, code, How it works, Variations.
- `/recipes` lists them by group, with a filter.
- Old chapter routes redirect to `Moved.wf`, and old `#section` links go through `old-links-map.js`.
- Five cheat sheets at `/coming-from/*`: React, Vue, Svelte, HTML & CSS and LaTeX / Typst, from `comingfrom.py`.

### Phase 4 — the compiler in the reader's browser

**Done (2026-11-13; merged in `c0a9011d`, site rebuilt in `e8b8bd9e`):**

- Live examples: Edit, Break it and Apply the fix.
- The Playground compiles in the browser through `site/src/live.js` and `crates/wf-wasm`. Its PDF build loads on demand.

### Phase 3 — the book

**Done (2026-11-13, `d2aa6f01`, `e2d5a821`, site rebuilt in `6dc25cae`):**

- The book is `learn/01–17` at `/learn/NN-slug`, numbered 1–17 in three parts: the tour, the language, building with it. `/learn` is its contents page, with progress.
- The essays are at `/about/*`: Why WebFluent, the new How reactivity works, Accessibility, Security, What a build ships, Static or single-page, Upgrading and Contributing.
- Other pages keep a number only for order (essays 18–25, reference 30–42, errors 43–44) and carry none in their titles.
- Every old address is in its page's `from:`. `guide.py` writes a redirect page for each (`pages/guide/Moved.wf`), and `old-links-pages.js` keeps the `#section`.
- Each chapter:
  - says what it is in its first sentence;
  - has a `learned:` list;
  - has interference asides linking the Coming-from sheets;
  - ends with `## Common mistakes`: two or three `wf expect=CODE` programs, each linked to its error page.
- Chapters 7, 10, 11 and 15 close on a quiz. A ```` ```quiz ```` block in Markdown becomes `Quiz`, and a checked answer is sent through `track("quiz_answer")`, behind consent.
- `docs_parse::the_book_is_written_to_its_rules` holds the book to all of this, and to the banned words.

**Found while writing, not fixed:**

- A `derived` built with `reduce` renders empty in a `wf test` render.
- A component placed only in a test draws U03.

### Phase 5 — search and agents

**Done (2026-11-13, `f2200252`, site rebuilt in `7a94573b`):**

- **Search:**
  - Each row of `search-index.json` has a `kind`, and the palette's tabs filter by it.
  - Ranking: a name or code that is the term comes first (`Button`, `T05`, `wf build`), then a section another framework's word leads to (`data.py`'s `SYNONYMS`: `useState` finds `state`, `\\newpage` finds `PageBreak`), then headings, then the words of a section. Enter opens the first result.
  - Error codes, the cheat sheets and the commands have rows of their own.
- **Markdown twins:**
  - `twins.py` writes a twin of every published page to `site/public/<route>.md`, 395 in all. Each comes from its `md-docs` source, or from the registry and commands for the reference, with its links pointed at the other twins.
  - It also writes `llms.txt` (the map) and `llms-full.txt` (the prose, about 900 kB).
  - The rail's **Copy page as Markdown** and **Open as Markdown** use the twins.
- **Agent files:** `docs_parse::the_agent_files_agree_with_the_site_and_the_compiler` checks that every site page `AGENTS.md`, the README and the skill link to is one the sitemap publishes. It also checks that `AGENTS.md` names every built-in, and that its icon list and count are the runtime's. It found and fixed two stale links.
- **Tests:** `every_published_page_has_a_markdown_twin` and `the_search_knows_other_frameworks_words` hold the rest.

### Launch

**Done (2026-11-13; site rebuilt in `60b9a5ed`, old site tagged `site-v1`):**

- **Decisions:** all four took the plan's recommendation: Instrument Sans; no assistant (Markdown twins and `llms.txt` only); the plain-text editor; `md-docs/` keeps its name.
- **Compiler fixes the launch needed:**
  - A page's `lang:` now picks its `t()` table, in the pre-render and live (`pageLocale` in the router), so `/ar/` pages are Arabic before any script runs.
  - A program that calls `setTheme` gets `theme.js`, linked blocking and first, so a reader's chosen theme is on the first frame.
  - The engine leaves out its English skip link when a page opens with its own.
- **Arabic:**
  - `/ar` (Home), `/ar/learn/01–03` and `/ar/coming-from/react`, from `md-docs/ar/` (`lang:`, `translates:`).
  - `guide.py` and `comingfrom.py` write translations beside their pages, with `GUIDE_AR` for the sidebar.
  - Anchors keep Arabic words.
- **Other changes:**
  - `/docs` sends readers to Home.
  - The README links Learn and Recipes.
  - A search with no results is counted when the palette closes.
  - Chapters show their `learned:` lists; Phase 3 had dropped them.
- **Checks:**
  - `wf verify` loaded all 558 pages with no problems, far under a 1.5 s paint budget.
  - `build.budget`: `app.js` 135 kB and `styles.css` 30 kB, gzipped.
  - **Keyboard:** the skip link comes first, and focus shows a 2 px outline.
  - **Contrast:** Night passes A13, and every Day pair is at least 4.6:1.
  - **Reduced motion:** the engine's sheet stops all motion.
  - **Right to left:** the only physical left/right rules are on code, deliberately.
  - **Old URLs:** all 132 of the 6.0 site's addresses answer 200 on the deployed site.

**Left for you:** submit `sitemap.xml` in Search Console, and listen to Learn, Reference, an error page and the Playground with a screen reader (Orca).

**Follow-up, done (2026-11-14):** the route table was 102 kB of `app.js` before compression, carrying every page's title and layout. A split build now sends those with each page's chunk, which leaves 36 kB of paths and page names (6.1 kB gzipped); `app.js` went from 126 kB to 108 kB gzipped.
