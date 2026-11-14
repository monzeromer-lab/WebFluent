# Review: md-docs/errors, md-docs/about, md-docs/ar

Checked against `target/debug/wf` built from HEAD (6cdad705). The installed
`~/.cargo/bin/wf` is older: its `docs:` links still point at
`/docs/guide/diagnostics#…`, so don't use it to check these pages.

## What was run

- **Registry vs pages.** 127 codes in `src/diagnostics/codes.rs` and 127
  `md-docs/errors/CODE.md` pages, the same set. No code is missing a page,
  and no page is for a code the compiler no longer has. Every page's H1
  title matches the registry title. Where a page states a severity, it
  matches the registry.
- **Examples in a real project.** All 106 `wf expect=` blocks and 105
  `wf fixed` blocks were run through `wf check --format json`. Each block
  went into `src/App.wf` of a `wf init x -t spa` project, with the
  starter's `.wf` files removed. The script is
  `scratchpad/run_examples.py` and the results are `scratchpad/iso.txt`.
  - All 106 broken examples draw their code.
  - All 105 fixed examples clear it and draw no error. A13 and V06 do this
    only once the starter's `"theme": {"name": "Day"}` is removed (see
    item M5).
  - The A01, A09 and R04 fixed blocks still draw `D01` (the file is not in
    `public/`). This is expected.
- **Shown messages.** `cargo test --test it -- docs_parse::every_diagnostic_example` passes, so every `text` block is what the compiler prints today.
- **Same examples pasted beside the starter** (`src/pages/Example.wf`,
  the reader's likely move): 103 of 105 fixed blocks still fail. Results
  are in `scratchpad/pasted.txt`.
- **About essays.** Every `wf` block in `md-docs/about` was compiled. I
  checked claims against the runtime, codegen, `wf --help`, `--stats` and
  the recipes.
- **Links.** Every relative link and anchor in errors/, about/ and ar/ was
  checked with the site's slug rule (`scratchpad/links.py`). None is
  missing as a file. The broken ones are listed below.
- **Arabic on screen.** I rendered `/ar/learn/01-getting-started` from the
  built `docs/` in a browser to check right-to-left display.

---

## High

1. **high** — `site/src/blueprint.css` / `site/src/docs.css:44-54` (shows in every `md-docs/ar/**/*.md`). Inline code in Arabic prose is not isolated (`unicode-bidi` is `normal`; checked with getComputedStyle on the rendered page, `<html dir="rtl">`). A code span that starts or ends with punctuation reorders: in `ar/learn/01-getting-started.md:145-147`, `.muted` and `.mutd` render as `muted.` / `mutd.`, and the hint “Did you mean `.muted`?” comes out scrambled. The same thing breaks `-t spa`, `--locales en,ar`, `:slug`, `?.`, `{ … }`, `No problems in shelf.` and so on in all four Arabic pages. **Fix:** add `[dir="rtl"] .wf-markdown :not(pre) > code, [dir="rtl"] .bp-prose :not(pre) > code { direction: ltr; unicode-bidi: isolate; }` (the same as `.bp-ltr`). Do the same for table cells.

2. **high** — `md-docs/errors/E108.md:11-13`. The fix links to `([chapter 30](../recipes/index.md))`. "chapter 30" is a leftover from the old numbered guide. `scripts/docsite/errors.py:link_paths` rewrites only `CODE.md` and `diagnostics.md` links, so the site keeps the raw `../recipes/index.md`. From `/errors/E108` that resolves to `/recipes/index.md`, and the built `docs/` has no such file, so the link is a 404. **Fix:** write `([Environments and secrets](/recipes/environments-and-secrets))`, or teach `link_paths` to resolve `../` links through `guide.route_of`. Also add a test that fails on a raw `.md` link left in `site/src/data/errors.json`.

3. **high** — `md-docs/ar/learn/01-getting-started.md:306`, `ar/learn/02-tutorial-shelf.md:152,238,856`, `ar/learn/03-tutorial-todos.md:374`. These Arabic pages link "قادم من React" to `../../coming-from/react.md`, the English sheet, even though its Arabic translation `md-docs/ar/coming-from/react.md` exists (route `/ar/coming-from/react`). `guide.py:rewrite_links` resolves the path literally, so Arabic readers are sent to the English page. **Fix:** change each to `../coming-from/react.md`. Better, have `rewrite_links` prefer a translation of the target when the current page has `lang:` and one exists.

## Medium

4. **medium** — `md-docs/about/introduction.md:145-146`. It says "`wf` is the compiler, the dev server, the formatter, the test runner, the language server and the template renderer". `wf --help` has no language-server command. The language server is a separate binary, `wf-lsp` (`crates/wf-lsp`), as `learn/01-getting-started.md` ("`wf-lsp` is a second binary") and `about/contributing.md:24` say. **Fix:** "`wf` is the compiler, the dev server, the formatter, the test runner and the template renderer; its sibling `wf-lsp` is the language server, built from the same compiler."

5. **medium** — `md-docs/errors/A13.md:10-18` and `md-docs/errors/V06.md:10-15`. In a project made by `wf init`, the config names `"theme": {"name": "Day", "dark": "Night"}`. Pasting the A13 example then draws nothing, because the declared `theme Faint` is not the theme in use. I reproduced this; it works only once `theme` is removed from the config. The V06 example depends on the same thing. **Fix:** add one line to A13: "The theme must be the one the build uses: the only one declared, or the one `theme.name` names." Better, have A13 also check every declared theme.

6. **medium** — Most error pages, for example `md-docs/errors/T04.md:10` and `T19.md:12`, and every page using `page P(path: "/", …)`. A reader who pastes an example into the project `wf init` made (beside `src/pages/Home.wf`) gets `S04` ("Pages P and Home both claim the route /") on 103 of 105 fixed blocks. The T04 and T19 examples also declare `type User`, which the starter already has, so they draw `E102`/`C01`/`T01` instead of the documented code (see `scratchpad/pasted.txt`). **Fix:** say once at the top of each example where it runs ("a whole program: put it in `src/App.wf` of an empty project"), or give examples a unique path (`/example`) and names (`ExUser`). Adding a "Try it in the playground" button per example would avoid the problem.

7. **medium** — `md-docs/errors/diagnostics.md:174-178`. It says "Each code's page has the same parts: … a program that draws it, the message the compiler prints for that program, and the fix." 22 pages have no program and no message: D01, D04, D05, E003, E005, E108, E109, E110, E111, E112, E115, E119, E120, E121, E901, E902, I01–I05 and V02. Several are easy to give one: D01 (an `Image(src: "/missing.png")`), E108 (`Heading(env.STRIPE_SECRET)` draws it with no config, as I checked), E109 (with a one-line config note), and E003 (the example exists but sits in a `text` fence, not a checked `wfx` block). **Fix:** soften the sentence ("most pages"), add examples for D01, E108 and E003, and drop E108 and D01 from `EXAMPLE_EXEMPT` in `tests/docs_parse.rs:1222`, since a single program draws both.

8. **medium** — `site/src/components/ErrorShell.wf:83-87` (error-page UX). The footer row always shows `"lints": { "CODE": "warn" }`:
   - For a non-lowerable error (E002, T05), it prints "lints cannot lower it" next to the exact config the build refuses.
   - For a warning (U01, A12), "warn" is already the default, so the snippet does nothing.

   **Fix:** show nothing for a non-lowerable error. Show `"warn"` for a lowerable error, and `"off"` plus the `// wf-allow(CODE)` one-liner for a warning.

9. **medium** — `site/src/components/ErrorShell.wf:45-81` (error-page UX). For a reader arriving from a terminal `docs:` link, the order is summary → "What the compiler said" → "Why this rule exists" → "The fix". The fix comes after the rationale, and for a long "why" (E120, E119) it starts below the fold. **Fix:** put "The fix" (broken/fixed side by side) right after the printed message and move "Why" below it. Consider a "Jump to fix" link in the label row.

10. **medium** — `md-docs/about/security.md:280-281`. It says the `_headers` file is for "Netlify, Cloudflare Pages, Vercel". This contradicts `md-docs/recipes/deploy-to-vercel.md:12,44` ("Vercel does not read the `_headers` file"). The comment at `src/build/mod.rs:1211` has the same error. **Fix:** "(Netlify, Cloudflare Pages; on Vercel copy them into `vercel.json`, see [Deploy to Vercel](../recipes/deploy-to-vercel.md))".

11. **medium** — `md-docs/ar/coming-from/react.md:30,32,36,37`. The WebFluent column was left in English prose: "handled as", "read as `Cart.items` after `use Cart`", "read with `match users { … }`", and "flags (…), `style { }` on the element, design tokens". In an RTL cell these English runs also reorder against the code. **Fix:** translate, e.g. `…`، تُعالَج بـ `on change(value) { last = value }`; `…`، تُقرأ `Cart.items` بعد `use Cart`; تُقرأ بـ `match users { … }`; أعلام (`Button("Save").primary`)، و`style { }` على العنصر، ورموز التصميم.

12. **medium** — Arabic terms clash (all of `md-docs/ar`):
    - **"compiler".** `مترجم` is used for the compiler (01: 9 times, react: 5, 02: 3), while the same pages use `مترجمًا` / `ترجمه` / `التدويل` for i18n. For example, `ar/learn/01-getting-started.md:238` says "مشروع `spa` أو `static` مترجمًا" (a *translated* project) a few lines after "مترجم WebFluent" (the *compiler*). `ar/learn/02-tutorial-shelf.md:271` calls a T04 a "خطأ ترجمة", which reads as "translation error".
    - **"attribute" and "theme".** `سمة` means "attribute" in `ar/coming-from/react.md:16,42` and "theme" in 02/03.

    **Fix:** use `المُصرِّف` for compiler and `خطأ تصريف` / `خطأ وقت البناء` for a compile error, keep `ترجمة` for i18n, and use `خاصية HTML` / `صفة` for attribute, keeping `سمة` for theme. Then add a glossary for translators.

13. **medium** — More Arabic terms used two ways:
    - **store:** `متجر` in `ar/learn/01-getting-started.md:108` and `ar/coming-from/react.md:14`, `مخزن` in 02 and 03.
    - **flag:** `علم`/`الأعلام` in `ar/learn/01-getting-started.md:160,298,372,382`, `علامة`/`علامات` in `ar/learn/02-tutorial-shelf.md:188` and `ar/coming-from/react.md:47,115`.
    - **pre-render:** `مُصيَّرة مسبقًا` (01, 02), `العرض المسبق` (react:61), `مُعدّة مسبقًا` (03:20).
    - **runtime:** `وقت التشغيل` (01) vs `بيئة تشغيل` (react:64).

    **Fix:** choose one term for each (suggested: مخزن, علم, مُصيَّر مسبقًا, وقت التشغيل) and apply it to all four pages.

14. **medium** — `md-docs/about/upgrading.md:34`. The "At a glance" row for 6.0 says "Nothing for a project". The 6.0 section (`:77-109`) lists changes that can break a build or change behaviour: `build.output` outside the project refused, `public_env` no longer read from the shell, `wf serve` bound to localhost, cross-origin `navigate()` refused, values that used to be clamped now refused (`T01`, `E120`), and it ends "Run `wf check`." **Fix:** "Run `wf check`; check `build.output`, `public_env` names from the shell, and `wf serve --host` if you test on a phone."

15. **medium** — Seven places in `md-docs/ar/learn` where a line ends in an attached prefix: `ar/learn/01-getting-started.md:193` (`بـ`), `ar/learn/02-tutorial-shelf.md:151,379,451,593` (`و`), `:155` (`لـ`), `ar/learn/03-tutorial-todos.md:397` (`و`). Markdown joins the lines with a space, so the page shows a detached `و `persist`` / `بـ `wf build``, which is an orthographic error. The spacing is also inconsistent elsewhere (`بـ `x`` at 02:282,540 vs `بـ`x`` at 03:441). **Fix:** move the prefix to the next line, attached (`و`persist``), and use one form throughout.

## Low

16. **low** — `md-docs/about/performance.md:36-37`. It says a one-page static-text build is "an `app.js` of 4.9 kB, 1.8 kB gzipped". HEAD builds 4.9 kB / **1.9 kB**, as AGENTS.md and `learn/01-getting-started.md` (“1.9 kB”) say. **Fix:** 1.9 kB. Better, generate the number in the test that checks the guide.

17. **low** — `md-docs/about/performance.md:68-91`. The `--stats` sample is slightly stale. Today: app.js 16.8 kB gzipped (not 16.7), `pages/Profile.js` 5.6 kB, total 100.0 kB, runtime 123.5 of 406.2 kB, router 13.9 kB. **Fix:** refresh it, or say "about".

18. **low** — `md-docs/about/performance.md:43`. "A site of one page is one file" is wrong as written: a one-page build writes `index.html`, `app.js` and `styles.css`. **Fix:** "A site of one page is not split: its code is all in `app.js`."

19. **low** — `md-docs/errors/diagnostics.md:37`. It says "`docs:` — a link to the code's entry in this chapter". It now links to the code's own page, `/errors/CODE`. **Fix:** "a link to the code's page".

20. **low** — `md-docs/errors/diagnostics.md:177` and `about/contributing.md:188`. Both say the suite checks that each example "draws exactly the code it is filed under". The test (`tests/docs_parse.rs`, `every_diagnostic_example_draws_its_code`) only checks that the code is among the findings. `about/introduction.md:50` also draws A12, and V06 draws A12. **Fix:** "draws the code it is filed under", or make the test exact.

21. **low** — `md-docs/errors/diagnostics.md:155-169` vs the page front matter `family:` values. The family names are worded two ways: the table says `P` "persisted values" and `U` "declared and never used", but the page groups (and `codes.rs`) say "What is kept in the browser" and "Declared and never read". `D` is "data, assets and what is kept in the browser" in the table but "Data and assets" as a group. **Fix:** use the group names in the table.

22. **low** — `wf explain CODE` output (`src/diagnostics/codes.rs:guide_section`, as printed by the CLI). In a terminal, the broken program, the message and the *fixed* program follow one another with no label, so the fixed block looks like more of the problem. **Fix:** print "Broken:" / "Fixed:" before the two programs, as the site does.

23. **low** — `md-docs/errors/E003.md`. The example is in a `text` fence, so it is not compiled and has no `wfx` highlighting. It does draw E003 when saved as `src/App.wfx` (I checked). **Fix:** make it a checked ```` ```wfx expect=E003 ```` block, if the fence parser can take `wfx`.

24. **low** — `md-docs/about/accessibility.md:34`. It says "`Header`, `Section` and `Footer` are landmarks". `Section` renders `<section>`, which is a landmark (region) only with an accessible name. **Fix:** "`Header` and `Footer` are landmarks, and so is a `Section` given an `aria-label`."

25. **low** — `md-docs/about/accessibility.md:83`. It says `A03` means every `Input` "has a label", but `A03` accepts a `placeholder:` (AGENTS.md: "neither `label:` nor `placeholder:`"), and the same essay then advises against placeholders. **Fix:** "has a label or at least a placeholder (prefer a label)".

26. **low** — `md-docs/ar/learn/02-tutorial-shelf.md:428,791,792` (and the English source `learn/02-tutorial-shelf.md:430,793,794`). The links labelled Environments / Offline / Internationalisation all go to `../recipes/index.md`, though `recipes/environments-and-secrets.md`, `recipes/work-offline.md` and `recipes/translate-a-site.md` exist. **Fix:** point each at its recipe, in both languages.

27. **low** — `md-docs/ar/learn/02-tutorial-shelf.md:793` (and EN `:795`). "Build a bigger project" links to the Todos tutorial (20 minutes), which is smaller than Shelf (30 minutes). **Fix:** link Pocket (`../../recipes/build-a-budget-app.md`), or change the wording to "another project".

28. **low** — `md-docs/ar/learn/03-tutorial-todos.md:15` (and EN `learn/03-tutorial-todos.md:15`). "It is the first of three tutorials" sits in chapter 3, after the Shelf tutorial (chapter 2). Its table lists Field Notes and Pocket, which live under recipes. **Fix:** "the first of three project tutorials (after Shelf)", or drop "first".

29. **low** — `md-docs/ar/**`. Links from Arabic pages to English-only chapters carry Arabic labels, for example `[الأنواع](../../learn/12-types.md)` and `[المخازن](../../learn/11-stores.md)`, so readers land in English without warning. **Fix:** mark them "(بالإنجليزية)", or let the site add a small "EN" badge to links that leave the translated set.

30. **low** — `md-docs/ar/learn/02-tutorial-shelf.md:8`. The "learned" item "اقرأ ملف JSON وقت البناء بيانات ذات أنواع" is ungrammatical (it is missing "as"). The style is also mixed: 01 lists nouns (تثبيت…، إنشاء…) while 02/03 use imperatives. **Fix:** "اقرأ ملف JSON وقت البناء بوصفه بيانات ذات أنواع، بـ`type` و`data`", and use imperatives in 01 too, matching the English.

31. **low** — `md-docs/ar/learn/01-getting-started.md:143`, `ar/learn/02-tutorial-shelf.md:341,404`, `ar/learn/03-tutorial-todos.md:398`. English UI strings in bold or italic with a final period or dash (**Welcome back.**, *Nothing yet — add your first todo above.*) next to Arabic text render with the period on the wrong side; I saw this on the rendered page. **Fix:** wrap UI strings in a span with `dir="ltr"`, or render them as code once item 1 isolates code. Alternatively, put the period outside the bold.

32. **low** — Whether each essay earns its place (`md-docs/about`):
    - `introduction`, `how-reactivity-works` and `contributing` are accurate and earn their place. I checked `how-reactivity-works` line by line against the compiled `Page_Home` and `WF.signal`'s `!==` check, and it matches.
    - `accessibility`, `security` and `static-and-spa` are half essay, half how-to: imperative headings, config recipes, a checklist. They duplicate recipes (404 page, `base_path`, `clean_urls`, CSP origins).
    - `upgrading` (625 lines) is a changelog digest, not an essay.

    **Fix:** keep the argument in each essay. Move the how-to sections to recipes (`cache-and-security-headers`, deploy recipes) and link to them. Move `upgrading` to the reference or help group.

33. **low** — UX idea for error pages: show the *actual* `wf fix` / quick-fix title where one exists. In `wf check --format json` I saw "Change to `name`" (T05), "Add `Router`" (R03), "Change to `derived clock`" (X06), "Add an `error` arm" (T21). Showing "Your editor can fix this: *Change to `name`*" or "`wf fix --code T05` applies it" right under the message gives a terminal reader the fastest way out. 12 codes the index says have fixes offer none on their own page's example: T02, T06, E103, C02, C04, V06, T04, T17, C01, A09, U09 and C05. Most have no near name in the example, but T04's example uses a method call (`.toUpperCase()`), for which no `?.` fix is offered (`src/check/sema/types/method.rs:234-241` vs `expr.rs:725-738`). **Fix:** use a field read in the T04 example, or add the `?.` plan to the method path.

34. **low** — UX idea for error pages: the H1 is the title ("A field or method that does not exist") and the code is only a small label. A reader who searched for "T05" or clicked a terminal link should see the code in the H1 ("T05 — A field or method that does not exist", as the Markdown source and the `<title>` already have it).
