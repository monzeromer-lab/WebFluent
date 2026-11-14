# Post-launch review, 2026-11-14

This review covers the whole published docs site: every page, every part's content, the Playground, and how each page lays out on a phone, a tablet and a desktop. It was checked against `docs/` and `md-docs/` at `6cdad705`.

| File | What it covers | High | Medium | Low |
|---|---|---|---|---|
| [site.md](site.md) | All 558 pages at 375, 768 and 1440px; axe-core in both themes; every link and anchor; every page type by hand; the Playground | 3 | 11 | 11 |
| [learn.md](learn.md) | The 17 chapters, read in order; both tutorials replayed step by step | 4 | 11 | 14 |
| [recipes.md](recipes.md) | The 78 recipes; 107 code blocks pasted into a fresh project and run through `wf check` | 4 | 16 | 18 |
| [reference.md](reference.md) | Reference prose, the generated built-in and CLI pages, the five Coming-from sheets | 5 | 9 | 17 |
| [errors-about-ar.md](errors-about-ar.md) | All 127 error pages run against the compiler; the essays; the Arabic pages | 3 | 12 | 19 |

**Total: 157 findings: 19 high, 59 medium, 79 low.** Each one has a place (`file:line` or a route), what is wrong and a concrete fix.

## What holds up

- **Every page answers.** No console errors, no failed requests, and no sideways scrolling at any width.
- **Every page has the basics.** One `h1`, a description and a canonical link on each, and every Arabic page right to left.
- **Error pages match the compiler.** All 127 codes have a page and every page has a code. Each broken example raises its code, and each fixed one clears it.
- **Both tutorials build and pass `wf test`.** Every full example in the reference and the cheat sheets compiles.
- **The light theme passes contrast** everywhere except the Changelog.

## Fix first

These break something for a reader today, or teach something false.

1. **The Playground and every "Edit" button are broken on the live site.** The compiler files (`/wasm/6.0.0/…`) were never copied into the site. [site 1]
2. **Some documented features don't do what the docs promise.** These are compiler bugs:
   - `optimistic()` throws every time it runs. [reference 1]
   - `state x = $primary` resolves to a token that does not exist. [reference 2]
   - `log(a, b)` is a syntax error, though the docs show it. [reference 5]
3. **Three tutorial steps don't match what happens:**
   - Getting started's "Write a site from scratch" deletes the theme file the config still names, so the build fails while `wf check` says all is well. [learn 2]
   - The Todos tutorial ends in a `wf verify` failure because no step creates the favicon. [learn 3]
   - Shelf step 4 promises an error that does not appear. [learn 4]
4. **The stores chapter's `Cart.add` example silently does nothing.** [learn 1]
5. **Four recipes fail when followed exactly:**
   - The Node recipe installs `webfluent` from npm, where the package does not exist. [recipes 1]
   - GitHub Pages deep links 404 on reload with the default project. [recipes 2]
   - The session sign-in loses the reader on reload. [recipes 3]
   - The two-language site writes its Arabic page as `lang="en"`. [recipes 4]
6. **Arabic pages scramble inline code**: `.muted` shows as `muted.`. One CSS rule fixes it. [errors-about-ar 1]
7. **On a phone, a chapter has no contents and no "On this page".** Previous and Next are the only way around. [site 3]
8. **The generated reference pages are broken in two places:**
   - "Edit this page" is a broken link on 137 pages. [reference 3]
   - `/reference/cli/config` lists none of its subcommands. [reference 4]
9. **Links that lead nowhere:**
   - The Changelog's 18 anchors all miss, so the reader lands at the top of the page. [site 8]
   - E108 links to a page that 404s. [errors-about-ar 2]
   - Arabic chapters send readers to the English React sheet. [errors-about-ar 3]

### Fixed (2026-11-14)

All 19 high items are fixed:

- **site 1, the Playground on the live site:** the compiler is copied in under `/wasm/6.0.0/` by `scripts/site-wasm.sh`, which the release script already ran and the launch's hand rebuild skipped.
- **site 2, no service worker:** the Playground, the live examples and the home page's example fall back to `/wasm/<release>/run.html`. It writes the build inlined into one document; an `srcdoc` frame would inherit the site's policy and run no script. Checked in the desktop app's browser pane, which refuses service workers.
- **site 3, phones:** a "Contents" button under every docs page's title opens its headings, and below 768px the pages of its part.
- **reference 1, `optimistic()`:** it is now handed the state (or a getter and setter over a store's state). Two more bugs turned up and are fixed:
  - An action that awaited nothing wrote `await` in a plain function, which stopped the page.
  - A store's action never rolled back.

  Both are checked in a browser.
- **reference 2, tokens as values:** `$primary` as a value is the one group that has it, `$color-primary`.
- **reference 3 and 4, the generated pages:** "Edit this page" points at `src/registry/builtins.rs` or `src/cli/app.rs`, and `wf config` lists its four subcommands, each with its options.
- **reference 5, `log`:** documented as taking one value. `log(a, b)` now says how to log several.
- **The Learn chapters:**
  - learn 1: `Cart.add` is rewritten, and click-tested.
  - learn 2: Getting started keeps `theme.wf`, and `wf check` now reports a theme the config names but no file declares (`E111`); before, only `wf build` did.
  - learn 3: Todos creates its favicon.
  - learn 4: Shelf step 4 shows the warnings that really appear.
- **The recipes:**
  - recipes 1: the Node binding installs from a clone until it is on npm.
  - recipes 2: GitHub Pages builds static, and names the `404.html` step a single-page build needs.
  - recipes 3: the sign-in recipe loads the session in `app` (a call at the top of `app` is now set-up code) and redirects once the check answers. Checked on reload, signed in and out.
  - recipes 4: the two-language recipe shows its config, and `R05` now knows the site's language.
- **The errors and Arabic pages:**
  - errors-about-ar 1: inline code is isolated left to right.
  - errors-about-ar 2: E108 links to the environments recipe, and error pages rewrite links to other parts.
  - errors-about-ar 3: Arabic chapters link the Arabic React sheet.

Still open from these: publishing `bindings/node` to npm, which is yours to do.

## Then: site-wide fixes, each done once

- **Icons in the pre-rendered HTML.** The header's icon buttons are empty until the script runs. This is a compiler change. [site 4]
- **One `main` landmark, not two nested.** This is a compiler change, like the skip link at launch. [site 5]
- **Code blocks a keyboard can scroll**, and rail buttons 24px tall. [site 6, 7]
- **Links in place of the recipes index.** About 66 links across Learn and Reference point at the recipes index although a specific recipe exists. [learn 7, reference 9, recipes 16]
- **Examples that paste into the starter.** Most error-page and recipe examples claim `path: "/"` or reuse the starter's names, so pasted beside `Home.wf` they fail with `S04` or `E102`. Either the examples should use their own path, or the pages should say "replace `Home.wf`". [errors-about-ar 6, recipes 6]
- **The "Next" endings in Learn chapters.** Six chapters end with a "Next" that disagrees with the site's own Next button. [learn 6]
- **The Playground's smaller gaps:**
  - It needs a fallback when a browser refuses service workers. [site 2]
  - Picking an example should build it. [site 12]
  - On a phone, it needs its file list. [site 13]

## Improvements worth planning

These are not bugs:
- A "Contents" sheet on phones. [site 3]
- "Open in the Playground" on every chapter example. [learn 15]
- A one-page view of every built-in (`components.md` exists but has no page). [reference 13]
- A "Needs" line under each recipe. [recipes 35]
- Recipes for the obvious missing tasks: drag-and-drop, shortcuts, `persist`, upload, a custom 404, a PDF report, a test. [recipes 18]
- On error pages, the fix before "Why this rule exists". [errors-about-ar 9]
- A compact recipe index on phones. [site 23]

## How to reproduce

The sweep's scripts (Node with `puppeteer-core`, `axe-core` and the system Chrome) are kept in the session scratchpad, not in the repo. To keep the link, anchor and Playground-import checks for good, the natural home is `tests/` or `wf verify`.
