# Review: the book, `md-docs/learn/*.md`

Checked against a fresh debug build of `wf` (6.0.0, `target/debug/wf`, built from master at 6cdad705). Scratch projects are under
`/tmp/claude-1000/-home-monzer-omer-Code-WebFluent/2fabfa5e-ad79-4cc9-a8f7-e6f2f213e67c/scratchpad/review-learn/`.
Both tutorials (Shelf, Todos) were replayed step by step, including `wf test` and `wf verify`. Every item below was checked. Where an item says "verified", the compiler or a test run showed it.

Most severe first.

## High

1. **high — `11-stores.md:105-111`.** The "fuller `Cart`" `add` action does not do what the prose says ("raises the quantity of an item already in the cart"). `let found = items.find(…)` makes a local copy (ch. 7:181-183 says this itself), so `found.qty = found.qty + 1` changes the local and leaves `items` alone. Verified: a click test (`click "Add a pen"` twice) shows `1 in cart`, not `2 in cart`, and `wf check` reports nothing. **Fix:** write it the way ch. 15:605-610 does: `items = items.map(i => if i.id == item.id { { ...i, qty: i.qty + 1 } } else { i })`, guarded by `if items.some(i => i.id == item.id)`. Add a sentence that ties it to the `let` rule in ch. 7. (Separate compiler issue: assigning a field of a `let` copied from state could warn.)

2. **high — `01-getting-started.md:257-299`, "Write a site from scratch".** The reader is told to "replace the contents of `src/` with two files". That deletes `src/theme.wf`, but the `spa` starter's `webfluent.app.json` still names `"theme": { "name": "Day", "dark": "Night" }`. Verified: `wf build` and `wf serve` then fail with ``Config Error: No `theme Day` is declared in this project``, so the promised "Save, and the dev server shows a navigation bar" never happens. `wf check` meanwhile prints `No problems in hello.`, which hides the cause. **Fix:** say "replace every file in `src/` except `theme.wf`", or tell the reader to delete the `"theme"` key from the config. Separately, `wf check` should report the missing theme the way `wf build` does.

3. **high — `03-tutorial-todos.md:77-90` with `:470-472`.** The config names `"favicon": "/favicon.svg"`, but no step creates `public/favicon.svg` (the `spa` template ships an empty `public/`). Step 8 tells the reader to run `wf verify`. Verified: it fails with `404 …/favicon.svg` and `1 page(s), 2 problem(s)`. **Fix:** add a step that creates `public/favicon.svg` (or copies it from the finished project), or drop `favicon` from the config in step 1.

4. **high — `02-tutorial-shelf.md:198-201`, step 4 "Check it".** The text says `wf check` "reports one error: ``error[R01]: `/books/{…}` is not a route``". Verified: it reports two warnings and no error: ``warning[U03]: `BookCard` is declared but never placed`` and ``warning[U04]: `Shelf.count` is declared but never read``. The R01 check does not run inside a component nothing places. A newcomer's first "Check it" that does not match is a real stumble. **Fix:** replace the expected output with the two warnings, and explain them as step 3 does. Optionally keep the point about link checking for step 5. All the other "Check it" outputs in Shelf matched, including `wf test` (2 passed, 1 snapshot) and `wf verify` (6 routes, 0 problems).

## Medium

5. **medium — `06-elements.md:119-130`, the universal props table.** The "Chapter" column shows numbers from the old guide that match nothing in the book: `[15]` links to 13-styling, `[8]` to 07-state, `[32]` to a recipe, and `[20]` six times to recipes. Three of those `[20]`s go to `../recipes/index.md`. **Fix:** rename the column "Read more" and use titles: "Styling", "State and reactivity", "Hand an element to a script", "Animate in and out", "Animate a list", "Shared element", "Count a number". Point `animate:`, `delay:` and `on:` at `recipes/animate-in-and-out.md` and `recipes/motion-defaults.md`.

6. **medium — "## Next" sections that disagree with the site's own Previous/Next bar.** The built page has an automatic "Pages either side" nav. For example, `docs/learn/12-types.html` shows "Next: Styling", while the body's "## Next" says "Expressions". The body links that leave the chapter order are:
   - `02-tutorial-shelf.md:858`: goes to Coming from React, skipping ch. 3.
   - `03-tutorial-todos.md:552`: goes to the Field Notes recipe, not ch. 4.
   - `12-types.md:618`: goes to `reference/expressions.md`, not ch. 13.
   - `13-styling.md:659`: goes to `recipes/index.md` ("Motion"), not ch. 14.
   - `15-data.md:821`: goes to `recipes/index.md` ("Real-time"), not ch. 16.
   - `16-testing.md:395`: goes to `about/security.md`, not ch. 17.

   **Fix:** make "## Next" always the next chapter. Move the side trips into a "Going further" line above it.

7. **medium — 27 links to the generic `../recipes/index.md` where a specific recipe exists.** Concrete mapping:
   - Deploying (01:158, 02:597, 02:787, 03:469): use `recipes/deploy-to-github-pages.md`, or list the four `deploy-to-*` recipes.
   - Environments (02:430, 15:42): use `recipes/environments-and-secrets.md`.
   - Offline (02:793, 07:288, 15:40): use `recipes/work-offline.md`. For `update`, use `recipes/offer-a-new-version.md`.
   - Internationalisation, i18n and Translations (02:794, 07:289, 14:306): use `recipes/translate-a-site.md`. For 14:306, use `recipes/validation-messages.md`.
   - Real-time (11:367, 15:39, 15:821): use `recipes/open-a-socket.md`.
   - Desktop apps (11:355): use `recipes/tauri-app.md`.
   - JavaScript interop (06:303, 10:389, 10:483): use `recipes/call-your-own-script.md` and `recipes/hand-an-element-to-a-script.md`.
   - Media (06:301): use `recipes/video-and-audio.md` and `recipes/add-a-picture.md`.
   - Motion (09:16, 13:289, 13:659, 06:124/126/128): use `recipes/animate-in-and-out.md`.
   - Server rendering (17:46, 17:491): use `recipes/render-a-template.md`.

8. **medium — `07-state-and-reactivity.md:318-329`, "Where values come from, in order".** The list puts "loop variables and arm bindings" (4) below `state` (2) and store members (3), which says a state wins over a loop variable of the same name. Verified, the opposite happens: with `state name = "outer"` and `for name in items by name { Text(name) }`, the text reads the loop's `$name`. AGENTS.md also says a `let`, a `for` binding, a lambda parameter or a caught error shadows a prop, state or store member. The list also leaves out `let` locals and lambda parameters. **Fix:** order it innermost first: `let` locals and lambda/catch parameters, then loop and arm bindings, then route params and props, then body members, then `use`d stores (by name), then constants and `data`/`image`, then browser values, then globals. Say "the nearest declaration wins".

9. **medium — the default slot rule is taught three different ways.**
   - `10-components.md:217-246` says "`slot` declares the default slot, which the body places as `children`", which implies `slot` is required.
   - `05-pages-and-routing.md:245-246,456-471` says a layout needs both.
   - `11-stores.md:296-300` uses `children` with no `slot`.

   Verified: a plain component with `children` and no `slot` builds and works. Only a `layout:` component needs the `slot` line (E104). A component with `slot` and no `children` passes silently. **Fix:** in ch. 10, say plainly that `children` alone is enough for a call's block, and that `slot` is needed (and then required) only when the component is a page's `layout:`. Point to ch. 5.

10. **medium — `03-tutorial-todos.md:16-24`.** "It is the first of three tutorials that each build a whole project", but the reader has just finished ch. 2, also a tutorial ("Tutorial: Shelf"). The other two live under `recipes/`, outside the book's chapter order. **Fix:** say "the second of the book's tutorials, after Shelf". Present Field Notes and Pocket as "longer projects" in recipes, and list Shelf in the table.

11. **medium — `14-forms.md:409-416`, the `form.apply` snippet.** It calls `await Backend.signUp(email, password)` with two positional arguments. The only `signUp` the chapter declares (:480-483) is `post signUp(body: Map)`, called by name at :495. A reader who copies the snippet gets a call that does not match the endpoint. Verified that `wf check` does not flag it (no T10), so the mistake would ship. **Fix:** use `await Backend.signUp(body: { email: email, password: password })` and show the `api` it calls. Separately, positional arguments to an endpoint should be T10, as ch. 15:322-323 promises.

12. **medium — `08-events.md:238-540` and `09-control-flow.md:149-218`: walls of reference detail in teaching chapters.**
    - Ch. 8 covers the overlay stack, Escape routing between layers, every `Menu.Item` option, `Splitter`'s `reset: 0` semantics and drag-and-drop in about 300 lines.
    - Ch. 9's `VirtualList` section explains first-frame layout behaviour (199-205).

    A newcomer needs "open a modal from state", "a menu", "drag to reorder". **Fix:** keep one short example and the rules a beginner needs. Move "Layers", the full `Menu`/`Popover` option lists, gestures and the `VirtualList` "How it draws" list into `reference/components.md` or recipes (`confirm-dialog`, a new "drag to reorder" recipe), and link to them.

13. **medium — `Footer` (and `Header`) mean two things, and the book never says so.**
    - `02-tutorial-shelf.md:336`, `05-pages-and-routing.md:87` and `06-elements.md:269-287` teach `Footer` as a web `<footer>` landmark (verified: `<footer class="wf-footer">`).
    - `17-pdf-and-slides.md:183` makes it the page-margin footer of a `Document`, and :447-448 says "`Footer` is already the document's footer".

    **Fix:** in ch. 6, note "inside a `Document`, `Header` and `Footer` are drawn in every page's margin (ch. 17)". In ch. 17, say the same built-in is the `<footer>` landmark on the web.

14. **medium — `08-events.md:121-131`.** The first page-level shortcut example pairs `on key("ctrl+k")` and `on key("cmd+k")`, and the modifier list (:130) leaves out `mod`, which appears only 20 lines later (:151) as "the one to use". Readers copy the first example. **Fix:** use `on key("mod+k")` in the first example, add `mod` first in the modifier list, and keep `ctrl`/`cmd` as "when you mean exactly that key".

15. **medium — no runnable examples.** The site has a Playground (`/playground`, `site/src/pages/Playground.wf`) whose share links carry a project in the hash (`PlaygroundShell.wf:5`). No chapter links to it. **Fix:** add an "Open in Playground" link to each full `page` example, at least the ones in ch. 1, 4, 7, 9, 10 and 13, and to each "Common mistakes" block, so a reader sees the finding without installing anything.

## Low

16. **low — `09-control-flow.md:91-93`.** The text is right that "the right-hand side of the `&&` is not narrowed". Verified: `if user != null && user.name == "x"` is T04. But the T04 hint itself says "or a check for `!= null`", and JavaScript readers will write exactly this. **Fix:** add the wrong and right lines side by side (`user != null && user?.name == "x"`, or nest an `if let`).

17. **low — `01-getting-started.md:104-109`.** The shown `wf init` output is two lines. Verified: it also prints ``.agents/skills/ and .claude/skills/ teach your coding agent to write it well`` and a "cd hello / wf build / wf serve" block. **Fix:** show the full output, or write "You see, among other lines:".

18. **low — examples that build with warnings, against the book's own advice.** Ch. 5:61-63 says to always give `title:`/`description:` (S01/S02) and an `h1` (A12). Many later examples omit them, for example `page Team(path: "/")` (10:74), `Notes` (10:125), `Todos` (10:159), `Files` (10:203), `Keys` (10:234), `Use` (10:273), `Parts` (10:299), `Shop` (10:362) and `Typed`/`Records`/`Enums`/`Shapes`/`Infer`/`Narrow`/`Deploys` in ch. 12. Ch. 10's `Panel` uses `.h3` directly under nothing (A11, verified). **Fix:** give every full `page` example `title:`, `description:` and an `h1`, or state once that examples drop them for brevity.

19. **low — `11-stores.md:68`.** "changes **0 items, $0.00** to **1 items, $2.50**." "1 items" reads as a bug in a teaching example. **Fix:** use `Text("Cart: {Cart.count}, {format(…)}")`, or point to plural `t()` forms.

20. **low — `10-components.md:522-526`, quiz 2.** The wrong-answer explanation says "Only `class:`, `aria-*`, `data-*`, `id`, `role`, `title` and `hidden` go to the root without a prop". The chapter's own list (:378-392) also sends `style { }`, `mount:`/`update:`/`cleanup:`, `drag:`/`drop…:` and DOM handlers there. **Fix:** "Only the attributes and props listed in *What a call puts on the root element* go to the root…".

21. **low — `06-elements.md:254`.** `Markdown(…)   // see Content`: there is no "Content" chapter. **Fix:** link `recipes/render-markdown.md`, or drop the comment.

22. **low — `08-events.md:188`.** `Text("Commands: {keyLabel(\"mod+shift+p\")}")` escapes quotes inside a splice. Ch. 17:137 writes `{format("2026-09-01", .date, "long")}` with plain nested quotes, which works. **Fix:** write it the ch. 17 way, so readers do not think escaping is required.

23. **low — `17-pdf-and-slides.md:364`.** `ImageSlide(src: "chart.png", …)` names a file the reader does not have. Verified: the deck builds with 6 slides and no warning at all (D01 only checks site-relative `/` paths). **Fix:** say "put a `chart.png` in `public/`", or use a site-relative path so D01 catches a missing file.

24. **low — `13-styling.md:426`.** `.note[data-tone="loud"]` assumes the component's root carries the class `note`, but the `Note` component it refers to (10:119-123) sets no class. **Fix:** use `[data-tone="loud"]`, or add `class: "note"` to the component in ch. 10.

25. **low — `12-types.md:535-541`.** "What the checker does not check" includes "It does not change a name's type across assignments… assigning a string to it is `T01`". That describes a check that does happen. **Fix:** move it to "Where types come from" as "A name keeps the type it was declared or inferred with".

26. **low — `12-types.md:134,138`.** `Text("Out on {checkout}, a weekday of {weekday}")` shows a bare number ("a weekday of 4"). **Fix:** `Text("Out on {checkout:.date(long)}")`, which also shows the splice format.

27. **low — inconsistent spelling of trivial calls across chapters.** `Spinner()` (11:422) against `Spinner` everywhere else. `component CartSummary()` with `use Cart` (11:494) against `component CartSummary {` without `use` (16:208). **Fix:** pick one form, and say once that empty parentheses and `use` are optional.

28. **low — `04-language-basics.md:157-159`.** "separate them with `;` or with nothing at all; the house style is two spaces". "Nothing at all" contradicts the space that is needed. Verified: a single space works too (`state home = 1 state away = 2`). **Fix:** "with `;`, or just whitespace — two spaces is the house style".

29. **low — `01-getting-started.md:251-255` against `:100`.** The chapter recommends starting with `static`, but its own first command is `wf init hello` (spa), and Shelf uses `-t static`. **Fix:** say why the first project is an `spa`, or use `-t static` in the first command.

## UX ideas specific to these pages

- **Ch. 7:** a small diagram of "state, the readers of it, a click" showing that only the `Text` repaints. The quiz at :350 tests exactly this idea, and a picture would land it.
- **Ch. 5 "The life of a page":** turn the numbered list into a timeline diagram (route match, chunk, body once, effects, placed, disposed), with where `after(0)` and set-up code sit.
- **Ch. 15:** a decision diagram for `resource` against `api` against `await fetch` against `data`. The table at :46-51 is close, and a flow ("known at build time?", "shown when the page opens?") is what newcomers ask.
- **Both tutorials:** add a collapsible "Stuck? Compare with the finished file" link per step to `examples/tutorials/...` (Todos does this only for the CSS). Show the expected `wf check` output in a `text` block, as step 9 of Shelf does, rather than inline prose.
- **Ch. 2 "The whole program" (:600-770):** the 170-line listing repeats every step. Collapse it by default, or replace it with a link to a finished `examples/` project.
