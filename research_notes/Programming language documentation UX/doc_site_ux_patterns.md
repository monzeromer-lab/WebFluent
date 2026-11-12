# UX and interface patterns for developer documentation websites (as of late 2026)

Scope: concrete UI patterns, who does them well, and trade-offs, to feed 10 UX directions for the WebFluent docs site. That site has a ~50-chapter guide, a 77-component reference, a showcase and a home page, and it can render live previews of its own components.

Method note: this pass made about 15 tool calls. The facts cited below were checked in this session. Items under "Inferences" also draw on general familiarity with the named sites (Tailwind, Docusaurus, VitePress, Rust/Go docs and others) and were **not** re-verified here. Treat them as leads to confirm on the live sites.

---

## Navigation: sidebar trees, tabs, hub pages, rails, prev/next, breadcrumbs, progress

### Takeaway
The standard layout is a three-zone shell: a section sidebar tree on the left, content in the middle, and an "on this page" rail or code column on the right. Above it sit a few top-level tabs or hubs, one per audience or doc mode. Practitioner guidance caps the top level at about 5–7 items and asks for task-based groups and no more than two levels of depth. Navigation must back up search, not be replaced by it.

### Cited Findings
- NN/g, "Search Is Not Enough: Synergy Between Navigation and Search": when sites put search ahead of navigation, "users must invest cognitive effort to create queries and to deal with the weak implementations of site search." Navigation teaches users the "search space." — [NN/g video](https://www.nngroup.com/videos/search-box-vs-navigation/); [NN/g, Do we still need IA?](https://www.nngroup.com/videos/do-we-still-need-ia/)
- Auth0's design team cites the same NN/g article when arguing that docs need both search and navigation. Their research was used to "untangle" their docs. — [Auth0 Design, Medium](https://medium.com/auth0-design/using-research-to-untangle-our-documentation-db610379519f)
- Mintlify's navigation guidance:
  - Keep top-level sections to 7 or fewer (5–7 is ideal). More than 7–8 counts as "overloaded."
  - Keep labels under about 4 words.
  - Prefer depth over breadth, but don't bury critical content more than 2 levels deep.
  - Promote high-traffic pages.
  - Group by user journey or task, not by product feature. Use verbs for task sections ("Deploy") and nouns for reference sections ("SDKs").
  - Use tabs when audiences have fundamentally different journeys.
  - [Mintlify navigation guide](https://mintlify.com/docs/guides/navigation)
- Tom Johnson (I'd Rather Be Writing) suggests sidebar groupings of roughly 10–30 related topics, and writes about "zooming out and in" with navigation. — [idratherbewriting, doc navigation principles](https://idratherbewriting.com/files/doc-navigation-wtd/design-principles-for-doc-navigation/); [zooming out and in](https://idratherbewriting.com/2016/09/25/zooming-out-and-in-with-navigation/)
- MongoDB's card sorts and tree tests showed that users navigate from their own experience, which led to a task-centric navigation. Fern also recommends task-based over feature-based IA (Feb 2026). — [Fern, IA best practices](https://buildwithfern.com/post/information-architecture-best-practices-documentation)
- Diátaxis splits docs into four needs/forms: tutorials, how-to guides, reference and explanation. It says docs should be organized around those structures. This is widely used as the top-level split. — [diataxis.fr](https://diataxis.fr/)
- Next.js puts a router switcher (App Router / Pages Router) at the top of the sidebar, and the breadcrumbs show which router's docs you are reading. — [Next.js FAQ](https://nextjs.org/docs/faq)
- Docsio's teardown of Vercel/Next.js docs (Apr 2026):
  - It praises splitting framework docs (nextjs.org) from platform docs (vercel.com), each with an audience-specific sidebar, but notes inconsistent cross-links.
  - It criticizes the parallel Pages Router tree: "often 2 to 3 months behind," with key differences buried.
  - It recommends a persistent variant/version indicator in the sidebar and per-page notices.
  - [Docsio Vercel teardown](https://docsio.co/blog/vercel-docs-teardown)
- MDN's 2026 frontend rebuild puts most top-level links behind dropdowns in the top navigation. These work as native CSS dropdowns before JavaScript loads (progressive enhancement), and the navigation toggle uses `aria-expanded`. — [MDN front-end deep dive (Apr 8, 2026)](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)

### Inferences
- **Pattern N1, mode tabs.** Use top tabs or hubs that map to Diátaxis modes: "Learn" (tutorial), "Guide" (how-to and explanation), "Reference" (components, CLI, diagnostics, config), plus "Showcase." Each tab owns its sidebar. For WebFluent this keeps 77 components out of the guide's sidebar.
  - Trade-off: users must learn which tab holds what. Cross-linking discipline is required, which is exactly the Vercel/Next.js weakness.
- **Pattern N2, single long sidebar tree** with collapsible groups, auto-expanding the current section (Docusaurus/Starlight default).
  - Trade-off: simple and scannable at 50 chapters plus 77 components, but long. It needs collapse-by-default and a sidebar filter box.
- **Pattern N3, hub pages.** Section landing pages show a card grid per group (Stripe, Supabase, Starlight "LinkCards").
  - Trade-off: hubs are good for orientation but add a click. They work best as section index pages, not as the only route in.
- **Pattern N4, "on this page" rail** with scroll-spy (right side). Standard on Tailwind, Docusaurus, Starlight, VitePress and Nextra. It collapses into a "On this page" dropdown under about 1280 px.
- **Pattern N5, prev/next footer links** for a linear guide, combined with a tutorial progress indicator. Suggested forms are "Chapter 12 of 50," a progress bar in the sidebar group, or check marks on visited pages stored in `localStorage`. Svelte's tutorial and Next.js Learn use chapter progress.
  - Trade-off: progress tracking implies a linear path. The WebFluent guide reads more like a reference than a course.
- **Pattern N6, breadcrumbs** that carry context (section › page), and a variant label if there is one (for WebFluent, possibly output type: web / PDF / slides).
- **Mobile.** Use a hamburger drawer for the sidebar, with the "on this page" rail as a top dropdown. The search trigger must stay one tap away.

### Gaps
- I did not fetch NN/g's article text directly, only the video summary. I found no NN/g article specific to developer docs.
- I did not verify Tailwind's 2025 docs redesign details in a primary source. Search results only confirmed that a redesign shipped with v4 (Jan 2025). — [Tailwind Weekly #181](https://tailwind-weekly.beehiiv.com/p/issue-181)

---

## Search: Cmd-K palettes, DocSearch, scoped search, AI "ask the docs"

### Takeaway
A keyboard-first Cmd/Ctrl-K modal that searches as you type is table stakes. The differentiators are:
- scoping/filtering (by section, version or variant)
- indexing code examples
- treating search as navigation (jump to headings and components)

AI assistants are now common but draw criticism. They add a step, drop code and edge cases, and can answer for the wrong version. They need scoped retrieval, citations and version metadata.

### Cited Findings
- The Vercel/Next.js Cmd+K palette is praised as keyboard-first, live as you type, indexing pages *and code examples*, and ranking pages where the topic is the main subject. It is criticized because version-qualified queries mix App and Pages Router results; the author suggests an "in:pages-router" prefix filter. — [Docsio](https://docsio.co/blog/vercel-docs-teardown)
- The same teardown calls the Next.js AI chat "mostly theater." It cites real pages and knows the current version, but adds a step, drops code examples and edge-case detail, and can't compare your code to current docs. — [Docsio](https://docsio.co/blog/vercel-docs-teardown)
- Algolia DocSearch v4 Ask AI:
  - Algolia recommends `facetFilters` to narrow retrieval for accuracy, plus testing in a playground.
  - Its prompting guide lists pitfalls; vague prompts give generic answers.
  - Ask AI now sits within Algolia's Agent Studio and is no longer offered standalone for new apps.
  - [DocSearch Ask AI](https://docsearch.algolia.com/docs/v4/askai); [Ask AI prompts](https://docsearch.algolia.com/docs/v4/askai-prompts)
- Version-tagging each retrieved chunk and filtering before retrieval is recommended to avoid wrong-version answers (third-party). — [lawzava, AI docs that don't lie](https://lawzava-com.pages.dev/blog/2025-07-21-ai-documentation-systems)
- Starlight ships built-in search, and newer versions let you swap in Algolia. — [Starlight overview via LogRocket](https://blog.logrocket.com/starlight-vs-docusaurus-building-documentation)
- MDN has a search button that opens a search modal, delivered as a progressively enhanced web component. — [MDN deep dive](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)
- AI-consumption affordances are now common (2025–26):
  - `/llms.txt` and `/llms-full.txt`, and `.md` appended to any page URL (Mintlify, Cloudflare).
  - A "Copy page as Markdown / Open in ChatGPT/Claude" menu on each page (PostHog, dbt).
  - [Mintlify llms](https://mintlify.com/docs/settings/llms); [PostHog](https://posthog.com/docs/ai-engineering/markdown-llms-txt); [Cloudflare AI consumability](https://developers.cloudflare.com/style-guide/how-we-docs/ai-consumability/index.md); [dbt](https://docs.getdbt.com/navigation-options)
- MDN launched an MCP server (listed on the MDN blog). — [MDN blog index](https://developer.mozilla.org/en-us/blog/)

### Inferences
- **Pattern S1, Cmd-K modal** with grouped results (Guide / Components / Diagnostics / CLI) and type icons. It should deep-link to headings, and show a preview pane with the live rendered component for component hits. This last point is a WebFluent-specific differentiator.
- **Pattern S2, search-as-navigation.** Typing `Button` jumps to the component page, `T05` jumps to the diagnostic entry, and `wf build` jumps to the CLI entry. WebFluent's codes (E/T/C/…) and component names suit exact-match shortcuts.
- **Pattern S3, scoped search chips** ("in: components", "in: PDF") to avoid the mixed-results problem.
- **Pattern S4, a self-hosted static index** (Pagefind-style), consistent with a compiler that "never fetches anything" and a strict CSP. This avoids a third-party origin.
- **Pattern S5, AI assistant.** If added, it must cite pages, keep code blocks intact and be scoped to the current version. Cheaper wins are `llms.txt`, per-page `.md`, and the existing `wf context` / AGENTS.md. Its value is contested.

### Gaps
- I found no quantitative study comparing Cmd-K palettes to classic search boxes for docs.
- I found no independent measurements of AI docs-assistant accuracy or usage.

---

## Code presentation: tabs, copy, highlighting, annotations, side-by-side, playgrounds, output panes

### Takeaway
Expected baseline:
- a copy button
- line, substring and diff highlighting
- file-name frames and terminal frames
- tabs synced across the page

The high end:
- Stripe's personalized, synced code column next to prose
- MDN's "Try it" editors with a rendered output or console
- Svelte/Next.js Learn's in-browser project tutorials

For a language that can render its own components, an "edit and see" pane is the main differentiator.

### Cited Findings
- **Stripe.**
  - Docs are written in Markdoc, open-sourced in 2022.
  - The docs insert the reader's own API test key into code samples and show content conditionally by location or by the products the reader uses.
  - They support checklists, collapsible sections, tab switchers, multi-language samples, callouts and asides, and interactive React Flow diagrams with clickable links.
  - Planned: hover overlays showing API reference for a function or parameter inside code examples.
  - [Stripe, How Stripe builds interactive docs with Markdoc](https://stripe.dev/blog/markdoc)
- **Stripe API reference layout.** Navigation on the left, prose in the middle, a code column on the right that scrolls in step with the reference. The layout is widely copied (GitHub, Twilio, Plaid, Mintlify, ReadMe). An older critique called three columns "a massive waste of space." Sources disagree slightly on column order. — [Moesif teardown](https://www.moesif.com/blog/best-practices/api-product-management/the-stripe-developer-experience-and-docs-teardown/); [Medium/Oxford Comma](https://medium.com/the-oxford-comma/stripes-new-api-reference-documentation-e6b1fc46896d) (403 on fetch; summary from search snippet)
- **MDN "Try it" interactive examples**, in three layouts:
  - JavaScript: an editor plus a console.
  - HTML: a tabbed editor (HTML/CSS) plus rendered output.
  - CSS: a table of selectable code variants plus rendered output.
  - They are built from reusable `<play-editor>`, `<play-console>`, `<play-runner>` and `<play-controller>` components. A full Playground also exists.
  - Each component's JS/CSS loads only when it is on the page.
  - Scrimba screencasts load only on click, for privacy and weight.
  - [MDN deep dive (2026)](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)
- **Starlight / Expressive Code.**
  - Line markers `{2-3}`, plus substring and regex markers.
  - `ins=`/`del=` and diff syntax combined with language highlighting.
  - Editor frames with file-name titles, and terminal frames for shell.
  - Components: Tabs, Steps, File Tree, Cards, Link Cards, Badges, Asides (note/tip/caution/danger).
  - [Starlight authoring content](https://starlight.astro.build/guides/authoring-content)
- **Svelte's tutorial** (learn.svelte.dev / svelte.dev/tutorial) runs in-browser with WebContainers, offers a "Solve" button, and encourages typing the code by hand. — [learn.svelte.dev](https://learn.svelte.dev); [WebContainers community](https://webcontainers.io/community-projects/sveltekit)
- **Next.js Learn** embeds editors in a project built across chapters, and examples fork in one click to templates or CodeSandbox. The caveat: "interactive code isn't a substitute for clear prose." — [Docsio](https://docsio.co/blog/vercel-docs-teardown)

### Inferences
- **Pattern C1, Preview | Code tabs** (shadcn/Chakra style) on every example. The preview is rendered by the WebFluent compiler at build time, so it works without JS.
- **Pattern C2, side-by-side source and output.** Show `.wf` source on the left and the live result on the right. Optionally add tabs for the generated HTML/CSS/JS, and a PDF thumbnail for document examples. This is unique to a compile-to-web language: "what it compiles to" is itself teaching material.
- **Pattern C3, an editable playground** (MDN Try-it / Svelte REPL style). This needs the compiler in WASM in the browser.
  - Trade-off: bundle weight. Mitigate as MDN does: load on demand, show a static first paint, run on click.
- **Pattern C4, `.wf` / `.wfx` syntax tabs** synced across the whole site, persisted in `localStorage` like Starlight's `syncKey` tabs. This is analogous to language selectors on Stripe and the package-manager tabs common elsewhere.
- **Pattern C5, annotated code.** Numbered callouts or hover tooltips on tokens that link to reference entries; Stripe planned this. WebFluent already has a registry, so a hover on `Button` or `.primary` could show the props.
- **Pattern C6, diagnostics shown as rendered compiler output.** Show the `error[T05]` block next to the faulty code.

### Gaps
- I did not directly verify Tailwind's, Docusaurus's or VitePress's current code-block features in this session.
- I did not verify whether Starlight tabs sync by `syncKey`; the fetched page did not mention it.

---

## Component/API reference pages: props tables, live demos, variant galleries, anatomy

### Takeaway
The common anatomy is:
1. a title and one-line description
2. a live preview with code
3. installation or import
4. usage
5. examples and variants
6. a composition/anatomy tree
7. a props/API table
8. accessibility notes

shadcn (April 2026) added explicit "Composition" trees, partly so LLM agents nest parts correctly. This matches WebFluent's parts (`Card.Header`, …) and its AGENTS.md focus.

### Cited Findings
- shadcn/ui added "Composition" sections (Apr 2026). These are trees showing what wraps what, e.g. `Card` > `CardHeader` > `CardTitle`/`CardDescription`/`CardAction`, plus `CardContent` and `CardFooter`. The stated reasons: show valid nesting at a glance, and because "LLMs and coding agents compose elements more reliably when they can see the full structure." Pages include composition, usage and examples, and the CLI `shadcn docs <component>` prints the docs for an agent. — [shadcn changelog 2026-04](https://ui.shadcn.com/docs/changelog/2026-04-component-composition)
- Community component-docs guidance recommends:
  - rendered output plus code for each example
  - copy buttons and syntax highlighting
  - tabbed install variants
  - "Props and API Reference," accessibility and responsive-behavior sections
  - [components.build](https://www.components.build/docs)
- MDN's CSS Try-it shows a table of selectable code variants with one rendered output. This is in effect a variant gallery driven by one preview. — [MDN deep dive](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)

### Inferences
- **Pattern R1, a single-page component template.**
  - Header with name, a one-line description, tags (category, output support: web/PDF/slides).
  - Live preview.
  - Flags/variants gallery: every tone and size rendered in a grid, click to copy.
  - Props table generated from the registry: name, type/enum cases, default, description.
  - Events, slots and parts as an anatomy tree.
  - Accessibility notes: the A-codes it can trigger.
  - Related diagnostics.
  - WebFluent's `wf registry --json` already holds the data. `wf docs` already writes a gallery.
- **Pattern R2, an interactive "playground props" panel** (Storybook Controls style). Toggles for flags and dropdowns for enums re-render the preview and update the code snippet. This is a strong demo of "flags are props."
  - Trade-off: more JS, and it risks overshadowing the prose.
- **Pattern R3, a component index as a visual grid** of thumbnails (Material, shadcn "blocks"), not just a list. 77 components benefit from filtering by category (Layout, Navigation, Data, Forms, Feedback, Media, Typography, Document/PDF).
- **Pattern R4, Stripe-style three columns for reference**: prose and props in the middle, sticky code and preview on the right, scrolling in step. Good on wide screens; it collapses to stacked on mobile.

### Gaps
- I did not fetch Storybook, Radix, Material or Chakra pages in this session. Their specific layouts (Radix "Anatomy" + "API Reference" sections, Storybook Controls/ArgsTable) are from memory and need verification.

---

## Landing/home and "choose your path"; persona routing

### Takeaway
Effective docs homes route by intent or persona: "Get started / Learn / Reference / Examples," or by audience. They use card grids, a prominent search and a quickstart. Separating audiences into distinct sites or tabs helps when journeys truly differ, at the cost of cross-link discipline.

### Cited Findings
- Mintlify: use tabs when audiences have fundamentally different journeys. — [Mintlify](https://mintlify.com/docs/guides/navigation)
- Vercel splits its framework docs and platform docs into separate sites by audience. This is praised, with the caveat about cross-links. — [Docsio](https://docsio.co/blog/vercel-docs-teardown)
- Diátaxis's four modes give a standard "choose your path" split. — [diataxis.fr](https://diataxis.fr/)
- Starlight ships Cards and Link Cards for hub pages. — [Starlight](https://starlight.astro.build/guides/authoring-content)

### Inferences
- **Pattern H1, "What are you making?" routing.** Choices: a website, an app (SPA), a PDF/report, a slide deck, custom elements, a desktop app. Each card links to a tailored quickstart (`wf init -t …`). WebFluent's multiple output types are a natural persona axis.
- **Pattern H2, a live hero**: an editable `.wf` snippet with its rendered result beside it on the home page (Svelte/Tailwind-style "see it work in 10 seconds").
- **Pattern H3, a split home**: the marketing home stays separate from a docs home that is a dense hub (search box plus section cards plus "popular pages").
- **Pattern H4, an "I'm coming from…" path** (React, Vue, HTML/CSS), using migration-style comparison pages.

### Gaps
- I found no primary-source data on how well persona routing converts on docs homes.

---

## Versioning, changelog, i18n/RTL, dark mode, mobile, typography, accessibility

### Takeaway
A version or variant switcher must persist and be visible on every page, with per-page notices; otherwise users get lost (the Next.js lesson). Theme switching should be progressively enhanced and support light/dark images. A WebFluent docs site should be bilingual with RTL from the first paint, which few doc frameworks do well.

### Cited Findings
- Next.js:
  - The version switcher only goes back to v13.
  - "Latest" defaults to v15.
  - Pages lack per-page version warnings.
  - The suggested fix is per-page version notices and a persistent indicator in the sidebar.
  - [Docsio](https://docsio.co/blog/vercel-docs-teardown)
- MDN:
  - The theme-switcher dropdown is progressively enhanced.
  - Images and Baseline status icons have light and dark variants.
  - Components are visually stable before JS (minimal global CSS to prevent layout shift).
  - `<dialog>` is used for modal views, with `aria-expanded`/`aria-label` on the navigation toggle.
  - [MDN deep dive](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)
- MDN shows Baseline browser-support indicators on pages. This is a status badge at the top of a reference page. — [MDN deep dive](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)
- Starlight markets out-of-the-box i18n (0.28 release), plus dark mode and accessibility. — [Astro blog, Starlight 0.28](https://astro.build/blog/starlight-028/); [Astro themes: Starlight](https://astro.build/themes/details/starlight/)
- MDN's 2022 redesign coincided with MDN Plus; the newer frontend replaced the React "Yari" app with server-rendered web components (Lit). Visible changes were mostly styles. — [Mozilla Hacks 2022](https://hacks.mozilla.org/2022/03/a-new-year-a-new-mdn/); [MDN deep dive](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)

### Inferences
- **Version or variant badge on every page.** Mark "Since 4.2" or "5.0" on features. Also show output-type badges (Web, PDF, Slides), analogous to MDN's Baseline. WebFluent's AGENTS.md has many version-specific notes (4 → 5, 4.2 `external` removal) that could become inline "Changed in 5.0" chips.
- **Changelog as a first-class section** with dated entries linked from the features they change, as shadcn's changelog does.
- **Reading width.** Prose at about 65–75 characters per line, code blocks allowed wider, and the TOC rail hidden on narrow screens. This is general typographic practice, not sourced here.
- **RTL.** Mirror the sidebar and TOC rail, keep code LTR, and isolate Latin runs in Arabic text. WebFluent's own guidance (a `.ltr` class) covers this.
- **Accessibility.**
  - Skip link to content.
  - Visible focus.
  - Keyboard-operable Cmd-K (focus trap, Esc).
  - `aria-current` on the active sidebar link.
  - `prefers-reduced-motion` for any animated previews.
  - Code blocks reachable by keyboard (scrollable regions with `tabindex=0`).
- **Mobile.**
  - Sticky top bar with a menu button and a search icon.
  - Sidebar as a drawer.
  - "On this page" as a collapsible list.
  - Previews must not overflow; use horizontal scroll inside the preview frame.

### Gaps
- I found no sourced reading-width figures specific to docs, and no NN/g mobile-docs study, in this pass.
- I did not research Docusaurus versioning mechanics (versioned folders, "unreleased" banners) in this session; they are widely known but unverified here.

---

## Feedback widgets, edit-on-GitHub, last-updated

### Takeaway
The common trio is a "Was this page helpful? Yes/No" control (sometimes with a free-text follow-up), an "Edit this page" link to the source, and a "Last updated" date. In 2025–26 these sit next to a "Copy page / Open in AI" menu. Effectiveness data is scarce.

### Cited Findings
- "Was this page helpful?" Yes/No at the bottom of pages, with a thank-you state after voting, is used by Cloudflare, dbt and Modular. — [Modular docs](https://docs.modular.com/api/mojo/nn/rope/); [dbt](https://docs.getdbt.com/navigation-options)
- Per-page "Copy as Markdown / View raw / Open in ChatGPT/Claude/Perplexity" menus are used by PostHog and dbt, and Mintlify has a shortcut to copy a page's Markdown source. — [PostHog](https://posthog.com/docs/ai-engineering/markdown-llms-txt); [Mintlify](https://mintlify.com/docs/settings/llms)
- GitBook markets "LLM-ready docs." — [GitBook](https://docs.gitbook.com/llm-ready-docs)

### Inferences
- **Feedback widget.** A binary vote plus an optional "what was missing?" box. Without a backend it could open a pre-filled GitHub issue. That fits WebFluent's no-third-party, CSP-strict posture, but needs user consent since it leaves the site.
- **"Edit this page on GitHub" and "Last updated <date>"** generated at build time from git history.
- **"Copy for AI" / per-page `.md`.** Especially fitting for WebFluent, which already ships AGENTS.md and `wf context`.

### Gaps
- I found no published study quantifying the effect of "was this helpful" widgets on doc quality or support load.
- I found no data on how much "copy page for AI" buttons are used.
