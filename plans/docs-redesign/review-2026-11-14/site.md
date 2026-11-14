# Review: the site as built: every page, the Playground, responsiveness

Checked on 2026-11-14 against `docs/` at `6cdad705`, the build GitHub Pages serves.

## How it was checked

- **Every page.** All 558 routes in `docs/` were loaded in headless Chrome at 375×812, 768×1024 and 1440×900. Each load recorded:
  - errors and failed requests;
  - whether the page scrolls sideways, and which element sticks out;
  - text that spills out of its box;
  - text smaller than 12px;
  - broken images;
  - every link, and whether the page and the `#anchor` it names exist.

  axe-core ran three ways: every WCAG 2.2 AA rule plus best practice in the dark theme at desktop width, contrast in the light theme, and target size and contrast at 375px. The script is in the session scratchpad (`sweep/sweep.js`, `sweep/analyze.js`).
- **Every kind of page by hand** in a browser, at phone, tablet and desktop widths:
  - Home and the Arabic Home;
  - a Learn chapter and an Arabic chapter;
  - the recipe index;
  - a reference built-in;
  - an error page;
  - the React sheet;
  - the search palette and the mobile menu.
- **The Playground** in headless Chrome:
  - Run;
  - clicking in the preview;
  - typing an error, the findings and "Apply the fix";
  - every example;
  - the PDF build;
  - Share;
  - the phone layout.

## What is fine

- All 558 pages answer 200, with no console errors and no uncaught exceptions.
- **No page scrolls sideways** at any of the three widths. Only one line of text spills out of its box: a list item on `/recipes/render-markdown` at 768px, by 3px.
- Every page has one `h1`, a description and a canonical link. No page repeats an `id`. Every `/ar` page is right to left. The `/blueprint` mock-ups are `noindex`.
- Pages load in about 1.0s (median and p95 locally; slowest 1.5s, `/recipes/library-from-a-cdn`).
- The light theme passes contrast on every page sampled except the Changelog (below).
- In real Chrome, with the compiler files served (see 1 below), the Playground works:
  - It compiles in about 50ms.
  - The preview is interactive.
  - An error keeps the last good page, and the findings link to their error pages.
  - "Apply the fix" is offered.
  - Every example loads its own files (a store, translations, a PDF page, a deck).
  - The PDF build loads on demand and offers "Open the PDF".
  - Share writes the project into the address and says "Link copied".
  - At 375px it stacks with no sideways scroll.

## Findings

### High

1. **The Playground and every "Edit" button are broken on the live site.** Both import `/wasm/6.0.0/wf.js`, which `https://webfluent.monzeromer.dev/wasm/6.0.0/wf.js` answers with 404.
   - Plan §9 W1 says to copy `crates/wf-wasm/{wf.js, preview-sw.js, pkg/, pkg-pdf/}` into the site at build, under `/wasm/<version>/`. That step was never done, so there is no `site/public/wasm` and no `docs/wasm`.
   - The reader sees "1 error, the preview keeps the last good page" and an empty preview.
   - *Fix:* copy those files in from `cargo xtask site` (or the site build), and serve them as long-cached files. The PDF build alone is 2.7 MB plus its fonts, so check whether the gzipped `.wasm` stays under GitHub Pages' limits.
   - *Also:* add a check, to `wf verify` or a test, that fetches the Playground's import, so this cannot ship broken again.
2. **The Playground's preview needs a service worker and has no fallback.**
   - Where a browser refuses one (Firefox private windows, some embedded browsers, the Claude desktop browser pane), the findings show the raw "Failed to register a ServiceWorker … An unknown error occurred" and the preview stays empty.
   - *Fix:* fall back to `Frame(srcdoc:)` for a single-page build, and show "The preview needs service workers, which this browser has turned off" instead of the browser's message.
3. **On a phone, a chapter has no way to reach the book's contents or its own sections.**
   - Below 768px, both the book sidebar and "On this page" are `display: none`, and the menu lists only the six top-level parts.
   - A reader on a phone can only press Previous or Next, or go back to `/learn`.
   - The same holds on every page with a sidebar: Reference, Errors, About, Recipes.
   - *Fix:* add a "Contents" button under the title that opens the book list and the page's sections as a sheet, or put them in the menu when the page has them.

### Medium

4. **Icons are drawn only after the script loads.**
   - Every icon is an empty `<span data-icon="…">` in the pre-rendered HTML.
   - The header's search, theme and menu buttons are empty squares for the first ~0.5–1s on every page (the second screenshot of a phone page shows this). A reader with JavaScript off never sees them.
   - This is a compiler gap: the static paint should inline the icon's SVG, as the live page does.
5. **Landmarks are nested and doubled on every page** (axe, 557 pages):
   - The engine's `<main id="wf-main">` wraps the site's own `<main id="bp-main">` and its `banner`.
   - Screen-reader landmark lists show two "main" regions.
   - *Fix:* when a layout draws its own `main`, the engine should not draw one (the same rule as the skip link at launch), or the shell should use a plain `div`.
6. **Code blocks cannot be scrolled from the keyboard** (axe `scrollable-region-focusable`, 228 pages, 667 blocks).
   - A wide `pre` scrolls sideways but cannot take focus, so a keyboard user cannot reach the rest of the line.
   - *Fix:* `tabindex="0"`, plus a `role="region"` and label, on the code block when it overflows (or always).
7. **The "Copy page" and "Open as Markdown" buttons are 15px tall** (axe `target-size`, 525 pages; WCAG 2.5.8 asks for 24px). *Fix:* give `.bp-rail-action` at least 24px of height, or padding.
8. **Every Changelog anchor is stale: 18 of them.** For example:
   - `/learn/14-forms#controls-and-bind`
   - `/learn/11-stores#how-long-it-lives`
   - `/reference#markdown`
   - `/reference#navbar`
   - `/reference/config#buildmedia`
   - `/about/security#content-security-policy`

   The page exists but the anchor does not, so the reader lands at the top. `RELEASE_NOTES.md` still uses the old guide's section names.
   - *Fix:* map them in `scripts/docsite/changelog.py` through the same old-to-new table `old-links-map.js` uses.
   - *Also:* add a test that every anchor the site links to exists. The sweep's link check is a ready start.
9. **Redirect targets that lose their anchor:**
   - Old `/docs/guide/{elements,events,state,…}` addresses forward to `/coming-from/react#the-map` and `#things-that-work-differently`, but the React sheet has neither id.
   - `/reference#icons` is linked from `learn/06-elements` and has no such anchor (the icons list lives in `components.md`, which has no page).
10. **The Changelog's code is unreadable in the light theme.**
    - Its fenced blocks keep the dark theme's token colours (`#6fd3e6`, `#8fbde7`, `#b3bac0`) on the light page background: 125 failing nodes, contrast as low as 1.6:1.
    - Elsewhere code sits in a dark console box, so the problem shows only here. *Fix:* give the Changelog's `pre` the same `bp-console` treatment, or light-theme token colours.
11. **One link colour fails contrast in the dark theme.**
    - `#3d6fa3` on `#17191c` is 3.35:1, short of the 4.5:1 that text needs.
    - It shows on the Home page's "What are you making?" links, `/ar`, the Changelog, and the "Every command" back link on all 26 `/reference/cli/*` pages.
12. **Picking an example in the Playground clears the preview.**
    - The editor says "Type, and the compiler in your browser builds it as you go", but choosing "Todo list" or "Invoice (PDF)" shows "Nothing built yet." until Run is pressed.
    - *Fix:* build on pick, as on typing. For the PDF and deck examples, open the PDF tab with "Build it as a PDF" in focus.
13. **The Playground on a phone has no file list.** The Files column is hidden below 768px, so the store in "Todo list" and the translations in "Arabic" cannot be opened. *Fix:* turn the file list into a row of tabs above the editor.
14. **The Playground cannot add or rename a file.** That limits it to the examples' shapes; you cannot try a component in its own file.

### Low

15. **On a phone, the React sheet's pairs lose their labels.** The "React" and "WebFluent" column heads are hidden below 768px, so the stacked pairs of code do not say which is which. *Fix:* a small label above each block.
16. **`aria-label` on a plain `div`** ("Works in", and its Arabic "يعمل في"), on 275 pages. It is ignored without a role. *Fix:* `role="group"`.
17. **Two accessible names drop visible text:**
    - The logo link is named "WebFluent, home" but shows "6.0" too.
    - The search button is named "Search the docs" but shows "Ctrl+K".
    - *Fix:* `aria-hidden` on the version and the key hint.
18. **Empty table header cells** on `/about/static-or-single-page`, `/recipes/open-a-socket` and two others: the first `th` of a comparison table is blank. *Fix:* give it a visually hidden label.
19. **The `Slider` demo on `/reference/slider` has no label.** axe rates it critical. *Fix:* give the demo `label:`.
20. **Inline code is 11.2px on a phone**, at 0.7em of 16px, on 96 pages. *Fix:* 0.85em is easier to read on a small screen.
21. **The mobile menu stays open if the window is widened** past the breakpoint, and it overlays the page. *Fix:* close it when the media query changes.
22. **The `/blueprint` and `/ar/blueprint` mock-ups ship with 16 dead links** (`/learn/07-data`, `/recipes/pdf-invoice`, …). They are `noindex`, but anyone with the address can reach them. *Fix:* leave them out of the production build, or move them to a `/design` page nothing links to.
23. **The recipe index on a phone is 78 full-width cards**, each about 90px tall: over 7,000px of scrolling before the last group. *Fix:* a compact list on small screens, with the group headings as a jump list.
24. **The error page shows "Error" twice** in the chips above the title: once for the kind, once for the family.
25. **The Playground's Counter example says "Clicked 1 times".** A starter is a good place to show `t(…, { count })` plurals, or at least to avoid the slip.
