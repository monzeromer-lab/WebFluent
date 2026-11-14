# Reference and coming-from review

Scope: `md-docs/reference/*.md`, `md-docs/coming-from/*.md`, and the generated pages under `docs/reference/` (per built-in, `docs/reference/cli/*`). Checked against `target/debug/wf` 6.0.0 (rebuilt for this pass), `wf registry --json`, `wf commands --json`, `wf config schema --json`, the runtime source and AGENTS.md. Scratch project: `/tmp/claude-1000/-home-monzer-omer-Code-WebFluent/2fabfa5e-ad79-4cc9-a8f7-e6f2f213e67c/scratchpad/review-ref/x`.

What passed:
- Every full `wf` example in reference/ and coming-from/ (except components.md) was run through `wf check` in its own project. All of them compile. The only findings are the two below (A11, U01).
- The ported counter (React/Vue/Svelte) runs in a browser `wf test`: clicking "+" shows "Last change: 3", and "even" goes away.
- The LaTeX report renders to a PDF with "Page 1 of 1".
- Every relative link and anchor in these pages resolves. The one exception is the components.md mirror (item 13).
- Every CLI option in `wf commands --json` appears in cli.md, and every config key in `wf config schema` appears in configuration.md.

---

## High

1. **high — `md-docs/reference/built-ins.md:289-306`, `expressions.md:479` (and AGENTS.md "Showing a change before the server agrees"): `optimistic()` crashes at run time in every form.** The docs describe `optimistic(holder, change)` with "a state, or a store's state" as the holder. The code generator passes the holder's *value* (`WF.optimistic(_n(), …)` for a page state; `WF.optimistic(store.items, …)` inside a store). The runtime (`src/web/runtime/modules/net.js:99`) then calls `holder()`. Two browser tests confirm it:
   - a page state fails with `uncaught: TypeError: holder is not a function`;
   - a store action `optimistic(items, …)` fails the same way.

   The test in `tests/language_4_0.rs:515` only checks that `WF.optimistic(` appears in the output.

   Fix: make the codegen pass the signal (for a page `_n`; for a store, a getter/setter pair over the proxy), and add a `wf test` that clicks one. Until that lands, mark the function as broken in built-ins.md and in the recipe.

2. **high — `md-docs/reference/expressions.md:70` and `:401-412` ("Tokens as values"): `state accent = $primary` compiles to `var(--primary)`, which no theme declares.** The border in the example gets no colour. Short names (`$primary` → `color-primary`) are only resolved inside a `style { }` property, as `design-tokens.md:214-231` says. As a value there is no property to resolve them by. `wf check` reports nothing: no V06.

   I confirmed this in the built output:
   - `accent=WF.signal("var(--primary)")`;
   - `:root` has no `--primary`;
   - the same token written directly in a style becomes `var(--color-primary)`.

   Fix the docs to write `$color-primary` and add a sentence: "outside a style block, write the full name". Compiler: either resolve `$primary` as a value by the colour group, or raise V06 on an undeclared full name.

3. **high — 137 generated pages (`docs/reference/<builtin>.html` ×111, `docs/reference/cli/<cmd>.html` ×26): the "Edit this page" link is broken.** Its href is `https://github.com/monzeromer-lab/WebFluent/blob/master/wf registry --json` (or `…/wf commands --json`).

   Fix in `scripts/docsite/reference.py`. For a built-in, point at the registry source (or `scripts/docsite/examples.py` for its example). For a CLI page, point at the clap definitions, or at `md-docs/reference/cli.md#wf-<cmd>`. Or drop the link when there is no single source file.

4. **high — `docs/reference/cli/config.html` (generated): `wf config` shows no subcommands and no options.** The usage line is just `wf config` and the options table is missing. `wf commands --json` describes `config` through a `subcommands` key (schema/get/set/unset, `--json`, `-d`), which `reference.py` ignores.

   Fix: render `subcommands`, one usage line and one options table per subcommand.

5. **high — `md-docs/reference/built-ins.md:38` and `:252-266`, `expressions.md:471`: `log(x, …)` / "One or more values to print" is wrong.** `log(a, b)` is `error[E002]: Expected ')', got ','`.

   Fix: document `log(value)` with one argument and suggest `log("{a} {b}")` or `console.log(a, b)`. Or teach the parser to accept several arguments, since `console.log` takes them.

## Medium

6. **medium — generator (`scripts/docsite/components.py`, `reference.py`): the universal Bool props `dropInside` and `trapFocus` are appended to every component's flag list.** This happens on 112 generated pages and in 105 entries of `components.md`, so `Router`, `Spacer` and `Tcell` read as if they had a `.trapFocus` of their own. The other universal flags (`.fadeIn`, `.fast`, `.slow`…) are not appended, which is inconsistent.

   Fix: drop props whose owner is `universal` from the per-component flag list, and list them once under "On every element".

7. **medium — generated built-in pages, e.g. `docs/reference/button.html` "Props"/"Events": the universal-props sentence names 5 of 21 universal props.** The page says "Plus the universal props on every element: animate:, exit:, speed:, class:, ref:". The registry's `universal.props` also has duration, delay, easing, stagger, on, count, shared, drag, dragGroup, dropValue, dropAxis, dropInside, trapFocus, mount, update and cleanup. The "Events" section is empty apart from a pointer. Button's registry `events` is `[]`, so nothing says `on click` is how a button acts (only the one-line summary mentions it).

   Fix:
   - write "Plus the 21 universal props (animate:, exit:, … mount:, update:, cleanup:)", or just the link;
   - in Events, list the common DOM events the element is meant for (click for Button/IconButton/Link, change/input for controls).

8. **medium — `/reference` index and sidebar: `reference/editors.md` and `reference/library.md` are not listed.** The only inbound links to `/reference/editors` come from `/reference/library`, and `/reference/library` is reached only from the glossary/editors "Next" chain. The sidebar lists Expressions … Glossary and stops.

   Fix: add both to the reference sidebar and to the "Language and tools" grid on `/reference`.

9. **medium — 25 links to the generic `../recipes/index.md` where a specific recipe exists.**

   | Place | Link text | Better target |
   |---|---|---|
   | `configuration.md:330`, `expressions.md:460`, `project-structure.md:57`, `cli.md:1030` | Environments | `environments-and-secrets.md` |
   | `configuration.md:318`, `cli.md:121`, `cli.md:856`, `expressions.md:474`, `project-structure.md:120` | i18n | `translate-a-site.md` |
   | `configuration.md:348` | Offline | `work-offline.md` |
   | `configuration.md:238`, `cli.md:767` | Motion | `motion-defaults.md` |
   | `configuration.md:185`, `cli.md:705`, `project-structure.md:119` | Media | `add-a-picture.md` |
   | `configuration.md:257` (owner), `:267` (preload), `:170` (clean_urls) | Content | `search-and-link-previews.md` (clean_urls: the deploy recipes) |
   | `configuration.md:175`, `runtime-api.md:43`, `project-structure.md:116` | JavaScript interop | `call-your-own-script.md` / `call-wf-from-a-script.md` |
   | `expressions.md:480` | Real-time | `send-on-unload.md` |
   | `expressions.md:484` | Real-time | `open-a-socket.md` |
   | `cli.md:587`, `glossary.md:177` | Server rendering | `render-a-template.md` |

   At minimum, link to the index's section anchors (`#interop`, `#realtime`, `#content`), which do exist.

10. **medium — `cli.md:1032-1152` duplicates `editors.md` and disagrees with it.** On Zed, `cli.md:1093` says the server is "downloaded from the latest release". `editors.md:124-128` says the extension downloads the release it *pins*, checked against SHA256SUMS. VS Code download behaviour is described only in editors.md. The Neovim/Helix/Emacs configs are only in cli.md, and editors.md links over to them.

    Fix: move the editor setup (the wf-lsp feature list and the Neovim/Helix/Emacs blocks) into editors.md, and leave cli.md with one paragraph and a link.

11. **medium — `runtime-api.md:106-109`, `:144-145`, `:39-43`: some module names are wrong, and the runtime-full advice is misleading.**
    - `onKey`/`keyIs`/`keyLabel`/`keyAria` are in module `keys`, not `core`.
    - `theme()`/`setTheme` are in module `theme`, not `core` (`src/web/runtime/mod.rs:79-83`).
    - The "Modules" bullet says a script calling a module nothing else uses needs `runtime: "full"`. But the build scans project scripts: a `src/support.js` calling `WF.toast`/`WF.announce`/`WF.storeSnapshot` gets all three shipped with `auto`. AGENTS.md says the same.

    Fix:
    - correct the two module cells;
    - reword the bullet: "only a call the build cannot see (a name built from a string, a script loaded from another origin) needs `runtime: \"full\"`".

    `askRules` and `playRoute` are exported but unlisted. They are internal, but could be listed as such.

12. **medium — generated CLI pages (`docs/reference/cli/*.md|html`) are much thinner than `cli.md` and do not link to it.**
    - No examples apart from the usage line.
    - No exit codes, no `--format` value table, no "preview" flag (which `wf commands --json` provides).
    - No link to the full section (e.g. `/reference/cli#wf-build`).
    - `wf build`'s summary "compile .wf files to HTML + CSS + JS" omits PDF/slides/elements.

    Fix: have `reference.py` pull each command's prose and examples from the matching `### \`wf X\`` section of cli.md, or at least add "Full reference → /reference/cli#wf-x" and the `preview` flag.

13. **medium — `md-docs/reference/components.md` (route `/reference`): it is published only as the Markdown mirror `docs/reference.md`.** The HTML at `/reference` is a different page, an index that links to per-built-in pages. The mirror's contents list links to `#container`, `#row` … (77 anchors), and none of those ids exist on `/reference`. The site has no HTML one-page view of every built-in, which is the fastest Ctrl+F lookup.

    Fix: give components.md its own route (e.g. `/reference/all-components`), publish it as HTML, and link it from the index. Or have the generator write each contents entry as `/reference/<name>`.

14. **medium — generated Markdown mirrors (`docs/reference/*.md`): 99 pages end "Since WebFluent 5.0."** This includes built-ins that predate 5.0 (Button) and the "original spelling" pages (`tcell.md`, `trow.md`, `route.md`). The HTML correctly shows "Before 5.0" (`reference.py:54,457`).

    Fix: have the Markdown writer emit "Before 5.0." when `since == OLDER`.

## Low

15. **low — `cli.md:18`:** "`wf --version` prints the version (`wf 5.11.0`)", but the binary is 6.0.0. **`cli.md:1028`:** `WF_VERSION` example `v5.11.0`. Fix: use `wf 6.0.0` / `v6.0.0`, or a placeholder that cannot go stale (`wf X.Y.Z`).

16. **low — `cli.md:1024-1030` environment-variable table is incomplete.** The source reads `WF_CHROME_NO_SANDBOX` (described only in prose at `cli.md:442`) and `WF_ALLOW_SHELL_ENV` (AGENTS "Security"). Neither is in the table. Fix: add both rows.

17. **low — `configuration.md:363`:** "`C02` — a prop nothing reads", but the code's title (`wf explain C02`) is "A prop the component does not declare". Fix: use the title.

18. **low — `configuration.md:200` `page_size`:** `paper_size()` (`src/paged/mod.rs:65`) also accepts `B5`, `Ledger` and `WIDTHxHEIGHT` (`800x600`). Fix: add them.

19. **low — `coming-from/html-css.md:94-96`:** the ported `Person` uses `Heading(name).h3` right after the page's `h1`, which draws `A11` (heading level skipped). Fix: use `.h2`, or `Text(name).bold`.

20. **low — `coming-from/svelte.md:51`:** "Svelte checks your script with TypeScript; WebFluent checks the markup too" understates Svelte, because `svelte-check` type-checks props and expressions in markup as well. Fix: "Svelte's checker types your props; WebFluent also knows every built-in's flags, events and slots, so `Button("Save").primray` fails."

21. **low — `coming-from/react.md:30`:** `useQuery(…)` is TanStack Query, not React. Fix: write "TanStack Query's `useQuery(…)`, or `useEffect` + `fetch`".

22. **low — coming-from maps are uneven.** React has Fetching and Test rows. Vue has neither (`<Suspense>`/`useFetch`, Vitest + Vue Test Utils). Svelte has `{#await}` but no Test row. Fix: add the missing rows so each sheet answers the same questions.

23. **low — `coming-from/latex.md:21-22` and `:72-93`:** the map says `\section` → `.h1` and `\subsection` → `.h2`, but the ported report uses `h1` for the title and `h2` for `\section{Sales}`. Fix: map `\section` → `.h2` (the title is the `h1`), and keep the example as it is.

24. **low — `grammar.md:278-279`:** "two on one line are separated by a `;` or two spaces". A single space also parses (`n = n + 1 log(n)` checks clean), so the rule as written is stricter than the parser. Fix: say "a `;` (two spaces also work)", or make the parser require one.

25. **low — `glossary.md`:** terms are bold paragraphs with no ids, so `/reference/glossary#signal` cannot be linked and the TOC is empty. Fix: make each term a `###` heading or give it an explicit anchor.

26. **low — `built-ins.md:26-50` and `:506-516`:** the function and browser-value summary tables are not linked to their entries below. Fix: link each name (`[after](#after)`). With 22 functions on a 770-line page, that is the main lookup path.

27. **low — `cli.md:1163-1179` "Building a tool on the compiler":** some paths are not real crate paths. `print`/`reprint` are under `syntax::print`, `style_value` under `paged::style_value`, `assets` under `build::assets`; `cli.md:702` itself says `webfluent::build::assets::{list, plan}`. The table also omits that most of these need the `internals` feature, which the per-command text does say. Fix: give full paths and an "internals" column.

28. **low — registry summaries shown on generated pages:**
    - `Button`: "Button. Its block holds what it shows; `on click` what it does."
    - `Table.Row`: "A row." `Table.Body`: "Body section." `Carousel.Slide`: "One slide." `DatePicker`: "Date input."

    Separately, Button's `to:` prop reuses Link's text ("the link whose `to` matches … carries `.active`"), and the Element summary writes `publishable-key: key` while its example writes `publishableKey:`. Both compile.

    Fix the registry summaries: "A button: its label or block is what it shows, `on click` what it does"; "A row of cells"; "Navigates to this route when pressed"; use one attribute spelling.

29. **low — legacy pages `docs/reference/{tcell,trow,thead,tbody,tabpage,option,route}`:** no example, and no link to the modern spelling. Their summary only names it ("the original spelling of Table.Cell"). Fix: add "Use [`Table.Cell`](/reference/table-cell)" as a link, or leave them out of the index and search.

30. **low — `cli.md:36` summary table:** `wf explain [CODE]` omits `--json`, which its own entry (`:379`) and `wf commands` list.

31. **low — `expressions.md:60-73`:** the Literals example draws three `U01` warnings (`note`, `slug`, `accent` never read). Fix: show them in the `Text` line so the example is clean.

## UX ideas specific to the reference

- The two longest pages (`cli.md`, 1,182 lines and 216 KB of HTML; `configuration.md`) should open with a jump table that links to each section. The per-command and per-key pages should carry the prose, so a search hit lands on a short page that has everything.
- Each per-built-in page should start with the example, then the props, which is the order a reader looking something up wants. Right now the Stage and props come first and the example is near the bottom.
- `built-ins.md` and `expressions.md` both list the string, list and number methods and the browser globals, and the lists differ: expressions has 3 number methods, built-ins has 6. Keep the lists in built-ins.md and have expressions.md link to them.
