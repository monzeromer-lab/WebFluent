# Recipes review (md-docs/recipes, 78 pages)

How this was checked: `target/debug/wf` (6.0.0, built from master). Every `wf` fence that is not `no-check`
was pasted on its own into `src/pages/Recipe.wf` of a fresh `wf init x -t spa` project (the recipe's
`// src/*.js` blocks written beside it; `App.wf` removed when the block declares `app`, and `Home.wf` when
it claims `/`) and run through `wf check`. The noise the starter's own `StatCard`/`TaskStore.completed`
produce when `Home.wf` is removed is ignored. Configs were applied where a recipe shows one, templates were rendered
with `wf render`, the Rust program was compiled and run against the crate, and three runtime claims were
checked in headless Chrome (`wf test`, `--dump-dom`). The tutorial projects (`build-a-*`) were checked with
`wf check` on `examples/tutorials/*`, and all three pass. Relative links and anchors all land (0 broken).
Scratch projects: /tmp/claude-1000/-home-monzer-omer-Code-WebFluent/2fabfa5e-ad79-4cc9-a8f7-e6f2f213e67c/scratchpad/review-recipes/work/

## High

1. **high** `render-from-node.md:42`: `npm install webfluent express` fails. `https://registry.npmjs.org/webfluent`
   returns 404, so the package is not published (the crate is, at 6.0.0). Every Node step after it fails too.
   Fix: publish `bindings/node` (6.0.0) to npm. Until then, add a note saying to install it from a checkout
   (`npm install /path/to/WebFluent/bindings/node`).

2. **high** `deploy-to-github-pages.md:19-27, 74`: following this recipe with the default `spa` project gives a site whose deep links 404 on reload.
   The build writes `404.html` only for a static build (`ssg: true`) that has a `path: "*"` page
   (`src/build/mod.rs:1104`). A `spa` build with `clean_urls: "file"` writes only `index.html` (verified).
   GitHub Pages has no rewrite, so `/tasks` reloads into GitHub's own 404. Line 74 ("GitHub Pages serves
   `404.html`") implies the build provides one. Fix: add `"ssg": true` to the recipe's config, and either a
   `page NotFound(path: "*")` step or, for a single-page build, a workflow step
   `cp build/index.html build/404.html`. Say which build kind the recipe assumes.

3. **high** `sign-in-with-a-session-cookie.md:66, 95-96`: after a reload, the guarded `Account` page sends a signed-in reader to
   `/sign-in`. Nothing calls `Session.load()`, and `wf check` reports `U04 Session.load / Session.checked is
   declared but never read`. The only fix offered is one sentence ("Call `Session.load()` from the app's
   set-up") with no code, and even when `load` is called, `guard: Session.signedIn` is read before its
   `await` resolves. Fix: show the `app { use Session  Session.load()  … }` set-up, and make the guard wait
   (for example `guard: !Session.checked || Session.signedIn`, with the page showing a `Spinner` until
   `checked`, or a check of `checked` in the page). Use `checked`, which is declared and never read.

4. **high** `site-in-two-languages.md:17-28`: pasted into a default project (`meta.lang: "en"`), the Arabic home
   page is written `<html lang="en">`, no `hreflang` alternates are written, and `wf check` is silent.
   Two "en" translations should be `R05`, but no `R05` fires. Verified: alternates appear only with
   `meta.lang: "ar"` and `meta.site_url` set (plus `ssg: true` for them to be in the HTML). The recipe says
   "This site is written in Arabic" but never shows that config. Fix: add the config block
   `{ "meta": { "lang": "ar", "site_url": "https://…" }, "build": { "ssg": true } }` above the code. Separately
   (compiler), R05 should fire when both pages resolve to the same language.

## Medium

5. **medium** `index.md:8-10`: "Copy the code into a page and it runs; every example here is compiled by the
   test suite". `tests/docs_parse.rs::check_strictly` runs only sema/type checks on blocks that are pure
   declarations. Project lints (R01, D01, A12, E102 against the starter) and the build's I/O are not run, so
   these main examples fail `wf check` as pasted: `environments-and-secrets.md:25` (`R01 /billing`),
   `responsive-navigation.md:22-31` (four `R01`s), and `markdown-pages.md:22-23` (`R01 /guide/pages`; only
   `install.md` is shown). These fail `wf check` with exit 2 ("IO Error", no code) because they need a file
   the recipe never provides: `add-a-picture.md:21` (`hero.jpg`) and `search-and-link-previews.md:21`
   (`posts.json`). Fix: in the index, say "each example parses and type-checks" (or make the test run the
   project checks). In the recipes, add the target pages (or link to `/`), show `pages.md`, and give the data
   file / picture as a step ("Put a `hero.jpg` in `public/`", show a two-line `posts.json`).

6. **medium** systemic, 39 recipes (e.g. `add-a-picture.md:23`, `animate-a-list.md:19`, `dark-mode-switch.md:36`,
   `sign-in-with-a-session-cookie.md:17`, `translate-a-site.md:22-24`): code pasted into the starter
   `wf init -t spa` makes collide with it. `path: "/"` gives `S04` against `Home.wf`. These names give `E102`:
   `page Tasks` (also a bogus `R05 "Tasks and Tasks are translations of each other"`), `page Settings` (+ `S04
   /settings`), `page Home`, and `type User` (the starter's `stores/auth.wf`). `translate-a-site` also gets `C01` because the
   starter's `User` has more fields. Fix: one line on the index and in each recipe's intro: "Paste into a
   new file under `src/pages/`; if it claims `/`, replace `Home.wf`". Better: give recipe pages their own
   paths/names (`/recipe`, `type Member`) so they paste cleanly. Compiler: the `R05` raised for a duplicate
   page name is a misleading message.

7. **medium** `active-tab-in-the-address.md:20-21`: both tab links get `class="wf-link active"
   aria-current="page"` on `/account#billing`, so neither the look nor a screen reader tells which tab is
   open (verified with headless Chrome `--dump-dom`). Fix: mark the current tab from `tab`, e.g.
   `Link("Profile", to: "/account#profile", class: { "is-current": tab == "profile" }, aria-current: if tab == "profile" { "page" } else { null })`,
   or use `Tabs`/buttons with `navigate("/account#billing")`. Say why a hash link alone is not enough.

8. **medium** `copy-to-the-clipboard.md:16-23`: the toast shows only on the first click. `Toast` compiles to a
   one-shot `WF.toast(...)` when the `if copied` branch is drawn (`src/web/codegen/js/dom/element.rs:172`),
   and `copied` never goes back to `false`, so the second "Copy" shows nothing. A refused clipboard also
   throws out of the action with no message. Fix: call `WF.toast` directly, or reset first
   (`copied = false` at the start of `copy`, or `after(2000) { copied = false }`), and wrap the `await` in
   `try … catch e { … }` as `call-wf-from-a-script.md:32-37` does. The same applies to
   `optimistic-updates.md:35` (`error` never resets, so a second failure shows no toast).

9. **medium** `show-a-page-in-a-frame.md:21`: the YouTube frame is blocked at run time. The build's CSP has no
   `frame-src` (verified: `_headers` has only `default-src 'self'` and `frame-ancestors 'self'`), and the
   build and `wf check` say nothing. The prose (line 41) mentions `meta.frame` but the config is not shown.
   Fix: add the block `{ "meta": { "frame": ["https://www.youtube-nocookie.com"] } }` beside the code.

10. **medium** `portal.md:40-43`: the recipe depends on a CodeMirror `meta.scripts` module entry with
    `"as": "CM"` exposing `EditorView`, `basicSetup` and `hoverTooltip` together, but never shows it. No
    single published CodeMirror module exports all three (`hoverTooltip` is `@codemirror/view`, `basicSetup`
    is `codemirror`). Fix: show the exact `meta.scripts` entries (and `meta.integrity`), or a small
    `src/vendor` bundle step, so the recipe runs.

11. **medium** `search-and-link-previews.md:30-36`: the SEO recipe's own post page is built with
    `<title>Post</title>` for every post (verified on `build/posts/hello/index.html`), and its hand-written
    `og:image` is the relative `/c.png`. Link previews need an absolute `og:image`. The fix for the title
    is described further down (line 120-124) but not used in the example. Fix: use
    `title: "{posts.find(p => p.slug == slug)?.title ?? slug} — Acme"` in the example, and either drop the
    `og:image` meta (pass `image:`) or make it absolute (`"{SITE}{post.cover}"`). Also give a per-post
    `description:`.

12. **medium** `serve-with-nginx.md:36-41`: the `add_header` in the `location = /sw.js` and image locations
    stops nginx inheriting the server-level `add_header`s, so those responses carry no
    `X-Content-Type-Options`/`X-Frame-Options`/`Referrer-Policy`. Also, line 37 caches *every* `.png/.jpg/.webp`
    for a year as `immutable`, including `public/` files that keep their names. That contradicts
    `cache-and-security-headers.md:52` ("`public/` files — your choice") and keeps a replaced `/logo.png`
    stale for a year. Fix: match only hashed names (`location ~* \.[0-9a-f]{8,}\.\d+\.(webp|avif|jpg|jpeg|png)$`,
    adjusting to the real hash pattern), and repeat the security headers in those locations, or put them in
    an `include` file. Also add the `ssl_certificate`/`ssl_certificate_key` lines the `listen 443 ssl` needs
    (low).

13. **medium** `serve-with-apache.md:49-50`: "write `Options MultiViews` … and add `AddEncoding gzip .gz`"
    does not serve `app.js.gz` for a request of `app.js`. Content negotiation by MultiViews happens only
    when the requested file does not exist, and `build/app.js` exists. This is from Apache's documented
    behaviour, not run on a live server. Fix: the usual
    `RewriteCond %{HTTP:Accept-Encoding} gzip` / `RewriteCond %{REQUEST_FILENAME}.gz -f` /
    `RewriteRule ^(.*)$ $1.gz [L]` with `<FilesMatch "\.js\.gz$"> ForceType text/javascript; Header set Content-Encoding gzip`,
    or recommend only `mod_deflate`. Also add `Referrer-Policy` to match the other host recipes (low).

14. **medium** `deploy-to-netlify.md:22`, `deploy-to-vercel.md:24`, `deploy-to-cloudflare-pages.md:21`,
    `docker-image.md:23`: install `wf` unpinned, while `install-wf-in-ci.md:19-27` (which each links to)
    says to pin. Unpinned, `install.sh:105` asks the GitHub API for the latest release and dies with
    "could not determine the latest version (GitHub API rate limit?)", which is common on shared CI IPs. All
    use `curl -sSL` without `-f`, so an error page gets piped to bash. `install.sh:4` recommends
    `--proto '=https' --tlsv1.2 -fsSL`. Fix: use
    `curl --proto '=https' --tlsv1.2 -fsSL …/install.sh | WF_VERSION=v6.0.0 bash` everywhere.

15. **medium** `send-on-unload.md:16-21`: the title and description promise "as the page unloads", but the
    example sends on a button click. Fix: show the unload case. This checks clean:
    `on visibilitychange { if document.visibilityState == "hidden" { beacon("/analytics", { event: "left" }) } }`
    at the top of the page. Keep the click as a variation.

16. **medium** `build-a-budget-app.md:67,69,428,457,470,511,524,526,527,528` and
    `build-a-notebook-site.md:74,317,454,459`: 14 links labelled "Internationalization", "Offline",
    "JavaScript interop", "Realtime", "Deploying" and "Content" all point at `recipes/index.md` (leftovers of
    the old guide chapters). A reader clicking "Offline" lands on the recipe index. Fix: point each one at the
    matching recipe (`translate-a-site.md`, `work-offline.md`, `call-your-own-script.md`,
    `talk-between-tabs.md`, `install-wf-in-ci.md`, `search-and-link-previews.md`) or the learn chapter.

17. **medium** `index.md` grouping: the published index (live `/recipes`) has groups of 8/5/7/4/**17**/7/**14**/**13**/2.
    "Look and motion" mixes media (picture, video, icon, carousel, frame, dark mode) with 11 motion
    recipes. "Ship" mixes hosts, web servers and desktop apps, sorted alphabetically, so the prerequisite
    "Install wf on a build machine" sits 7th and "Check a deploy" 2nd. "Interop" holds server rendering
    (`render-a-template`, `render-from-node`, `render-from-rust`). "Real time" holds `show-the-network` and
    `send-on-unload`. "Reach" holds `environments-and-secrets`. Fix: split "Media" from "Motion"; split
    "Desktop apps" (electron, tauri) and "Server rendering" out; order "Ship" by workflow (install → deploy
    per host → serve per server → headers → check); move env to Ship. The markdown twin `index.md` itself
    lists no recipe at all (sections are empty headings), so an agent or a GitHub reader sees nothing. Generate
    the list into the file.

18. **medium** findability, tasks with no recipe (checked by grep over all 78): drag-and-drop reorder
    (`drag:`/`on receive`), a keyboard shortcut or command palette (`on key("mod+k")`), keeping a value
    across visits (`persist`, outside dark mode), file upload with progress (`FileUpload`, `.progress`), a
    long list (`VirtualList`), menus/context menus/popovers (`Menu`, `Popover`), submitting a form to an API
    and showing the server's 422 (`form.apply`), a sortable table, a custom 404 page (`path: "*"`, which
    nginx/Apache/GitHub recipes rely on), a PDF report or slide deck from a project (`Document`, `Chart`,
    `Presentation`), and writing a test (`wf test`). Fix: add recipes for at least drag-and-drop, a
    shortcut, `persist`, upload, the 404 page, and a PDF report; the 404 one is needed by four ship recipes.

19. **medium** `build-a-notebook-site.md:21-40` (same in `build-a-budget-app.md:21-30`): it says "work in the
    finished project" (`cd examples/tutorials/medium-field-notes && wf serve`), then Step 1 says
    `wf init field-notes -t static`. A reader cannot tell whether to build along or read along. Fix: pick
    one: "Read along in the finished project; to build it yourself, start with `wf init …` and copy each
    file as its step shows it". These two are tutorials (`from: /docs/tutorials/…`) filed as recipes. Link
    them from the Learn section too.

## Low

20. **low** `dark-mode-switch.md:28`: the starter already declares `theme Night` (`src/theme.wf`). Pasting the
    recipe gives two `theme Night`, which `wf check` does not report (the recipe's silently wins). Fix: say
    "replace `src/theme.wf`", and mention that the starter already has a `ThemeToggle` component doing this.
    Compiler: a duplicate `theme` should be `E102`.

21. **low** `translate-by-the-words.md:70`: "the default locale's … file may be empty". With an empty `en.json`,
    `wf check` warns `I03: en.json has no {files} files.other … so a reader in en sees the key`. In Chrome
    the English reader sees `11 files` (verified with `wf test`), so the recipe is right and the warning is
    wrong. Fix (compiler): don't raise I03 for the default locale under `literals`. Until then, mention the
    warning in the recipe.

22. **low** `translate-a-site.md:37, 71, 112-114`: `es` is configured and offered in the picker, but no
    `es.json` is shown. `wf check` then says `I03: the locale es has no es.json, so every message shows its key`,
    which contradicts line 113 (falls back to the default). Chrome confirms the fallback, so the message is
    what's wrong. Also `state chosen = "en"` (line 25) does not follow a kept choice, so a returning Arabic
    reader sees "English" selected. Fix: show `es.json` (or drop `es`), and use `state chosen = locale`.

23. **low** `environments-and-secrets.md:136-151`: "Each build writes its own `build/` folder" after three
    `wf build`s in a row that all write `./build` (there is no output flag). Fix: "Each build overwrites
    `build/`; deploy it before the next", or set `build.output` per environment with `wf config set`.

24. **low** `chart-from-a-library.md` vs `library-from-a-cdn.md`: the same Sales page and the same
    `drawSales` (in `src/sales-chart.js` vs `src/charts.js`). A reader who follows both gets the
    "two scripts declare one name" error. Fix: make `library-from-a-cdn` about loading (use a different
    library, e.g. confetti with `"load": "after"`), and leave charts to `chart-from-a-library`.

25. **low** `keyframes.md:44-46` and `animate-in-and-out.md:144-146`: "`Card.Drop` is an error". The error a
    reader actually gets is `E103 Card.Drop is not a part of Card` plus `C03 Card.Drop is a part of Card, and
    it is not inside one`, which talks about parts, not flags. Fix: quote the real message in Troubleshooting
    (and, compiler, make it say "`Drop` is your animation: write `animate: .Drop`").

26. **low** `cache-and-security-headers.md:49`: the "no-cache" row lists HTML, `app.js`, `pages/*`,
    `styles.css`, but a build also writes an unhashed `theme.js` (verified in `build/`). Fix: add `theme.js`
    (and `js/*`, the project's scripts, and `elements.js`).

27. **low** `responsive-navigation.md:15-35`: a site's navigation belongs in `app`, not a page, and the menu
    button has no `aria-expanded`. Fix: show it in `app { … Router }` and add
    `aria-expanded: open, aria-controls: "nav-menu"` with `id: "nav-menu"` on the `Stack`.

28. **low** `render-markdown.md:17-22`: `A12 Page has no h1` (the heading comes from inside `Markdown`).
    `show-a-carousel.md:21-23` and `video-and-audio.md:21-22`: `D01` for each file not in `public/`.
    Fix: add `Heading(...).h1` (or `// wf-allow(A12)`), and a line saying which files to put in `public/`.

29. **low** `debounce-a-search.md:24`: the resource asks `/api/search?q=` once on load before anything is
    typed. Fix: `if q != ""` around the `match`, or say it's intended.

30. **low** `filters-in-the-address.md:15-25`: titled "Keep filters", but the example sorts nothing and filters
    nothing (it only prints "Sorted by …"). Fix: sort a small `const` list by `sort`, so the reader sees the
    effect.

31. **low** `electron-app.md:196-205`, `tauri-app.md:60-69`: "Two keys in `webfluent.app.json`" followed by
    three bullets (the third is "leave `offline` out"). Fix: "Two keys, and one to leave out".

32. **low** `tauri-app.md:22`: `https://v2.tauri.app/start/prerequisites/` times out here (the host resolves
    to IPv6 only); `https://tauri.app/start/prerequisites/` answers 200. Fix: use the canonical
    `tauri.app` URL.

33. **low** `migrate-from-external.md:9`: the title doesn't say the task or the version. Fix: "Upgrade a
    WebFluent 4 project that uses `external`", so it is found by people upgrading from 4.

34. **low** clarity, no stated result: `animation-sequence.md`, `store-value-as-a-prop.md`,
    `page-through-results.md`, `load-more.md`, `talk-between-tabs.md` (has it), `check-a-deploy.md`. Most
    recipes say what the reader will see ("Press Load and the revenue counts…"), but these don't. Fix: one
    "You will see…" sentence after the code. `check-a-deploy.md:16-21` also gives no commands. Add
    `curl -sI https://site/deep/link`, `curl -s https://site/ | grep '<h1'`, and the preview-debugger URLs.

## UX ideas specific to recipe pages

35. **medium** Put a "Needs" line under each recipe title, generated from what the page already shows: config
    keys (`meta.frame`, `offline`, `i18n`), files (`public/hero.jpg`, `src/posts.json`, `src/*.js`) and
    other recipes. Items 1, 4, 5, 9 and 10 are all a missing "you also need X", and the reader finds out
    only from a blank frame or an `IO Error`.

36. **low** A "Copy as files" or download button on a recipe that bundles every block with a `// src/…` or
    file-name label (wf + js + json) into a ready project, e.g. `wf init --recipe chart-from-a-library`. That
    would make "copy-paste-able" true for the multi-file recipes (chart, portal, tauri, electron, call-wf).

37. **low** Show the `outputs:` front matter (website/app/desktop/pdf) as badges on the card and the
    page, and filter the index by it, so a PDF or desktop reader skips web-only recipes.

38. **low** End each recipe with a "Related" row built from its links (several already end with prose
    links). The motion recipes form a natural path (in/out → list → sequence → route → shared → reduced
    motion) that a "next in Motion" link would walk.
