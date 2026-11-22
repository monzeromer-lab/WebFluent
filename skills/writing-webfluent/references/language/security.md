## Security

What the compiler guarantees, as properties of the output rather than
advice:

- **Text is text.** Every string an element shows goes in as a text node.
  `Unsafe.Html(markup)` is the one door for markup, named so a review
  greps for it, and every use draws a `V03`. An `html:` (or `markdown:`)
  written on an element is an attribute of that name, as any argument an
  element does not declare is (`C04`), never markup.
- **No inline handlers.** `Button("x", onclick: expr)` is a compile error
  pointing at `on click { }`; `style:` is one too (`E106`), pointing at
  `style { }`.
- **An attribute holds its value.** A page title (from a route parameter,
  data or front matter), a `class:` from data, a chart's or a QR code's
  colour are escaped where they are written, and a colour must be a CSS
  colour or the default is drawn. A design token holding `<`, `{`, `}` or
  `;` is left out of the sheet.
- **No URL a browser would run.** `href:`, `src:`, `to:`, `poster:` and
  `navigate()` take `http`, `https`, `mailto`, `tel`, `sms`, `ftp` or a
  relative URL. A literal is refused where it is written; anything else
  goes through `WF.safeUrl` at the moment it is used. `javascript:`,
  `data:`, `vbscript:`, `blob:` and `file:` are refused — but for a
  picture carried in its address (`data:image/png;base64,…`,
  `data:image/svg+xml,…`), which an `Image`'s or an `Avatar`'s `src:`, a
  Markdown picture and a sanitised `<img>` take (`WF.safePicture`): an img
  never runs what it shows. A link, a frame or a form never takes one, and
  no other `data:` is a picture.
- **`rel="noopener noreferrer"`** on any link with `target:`; a sanitised
  link with `target` loses any `rel="opener"`.
- **A frame drawing HTML from a value runs none of it**: `Frame(srcdoc:
  value)` with no `sandbox:` is `sandbox=""`, and draws a `V03`.
- **A `Secret` cannot escape** — shown, spliced, logged or persisted is a
  `T12`.

`sanitize(html)` is an allow-list of elements and attributes — no script,
no style, no frame, no form, no `on*`, and every `href`/`src` through the
scheme check. It runs at build time and in the browser from the same list,
so a pre-rendered page shows the sanitised body immediately.

```wf
Unsafe.Html(sanitize(post.body))     // markup from outside the project
```

**Sessions belong in an httpOnly cookie**, not in `persist`: any script
that reaches the page can read `localStorage`. `api B(…) { credentials:
.sameOrigin }` sends the cookie with every call and the page holds no
token at all.

**`env` splits in two.** `env.NAME` is replaced at build time and the value
is in the bundle, so only a public name may be read from a page, a
component, a store or an `api` block: one that begins `PUBLIC_`, or one
`public_env` lists. Any other name is a compile error naming the file and
the line; the values still reach `wf render`, which runs on a server.

```json
{ "env": { "PUBLIC_API": "/api/v1", "STRIPE_SECRET": "sk_…" }, "public_env": [] }
```

A name `public_env` lists without the prefix takes its value from the
config or `.env`, never the shell — unless `WF_ALLOW_SHELL_ENV` names it —
since a cloned project could list a shell secret there; each build warns of
every such value it bundles.

**A build stays in its project.** `data`, `image`, `api … from`,
and `i18n.dir` are paths inside it (an absolute path or a
`..` is an error); a link out of the project, to nothing, or looping back is
left out of every walk with a warning; `build.output` outside the project,
or a link in it, is refused unless the command line says
`--allow-output-outside`; and the build removes only the `.gz` files it
wrote itself.

`wf audit` prints what a review asks for: every `Unsafe.*` and whether it
is sanitised, the project's scripts and what each declares, every other
origin, everything kept on the reader's machine,
every `env` name and whether it is public, the policy this build ships, and
what the compiler depends on. `--json` for a tool; it never fails a build.

### Headers and the policy

`"build": { "csp": true }` — on for a project `wf init` creates — emits a
`Content-Security-Policy` meta tag — right after `<meta charset>`, before
anything a page's data reaches — and a `_headers` file for hosts that
read one. `frame-ancestors` is only in `_headers`, because a browser
ignores it in a meta tag. It is `'none'`, with `X-Frame-Options: DENY`
(`wf serve` sends the same); a project that places a `Frame` with a
site-relative `src:` (`"/preview/"`) gets `'self'` and `SAMEORIGIN`, so it
may frame its own pages and no one else may. The policy is widened by exactly the origins
`meta.fonts`, `meta.stylesheets` and `meta.scripts` declare, so a declared
font or library is never blocked by the policy that ships beside it; the
project's own scripts are same-origin and need nothing. `meta.connect` names
the origins a page may send requests to besides its own — an API elsewhere,
an analytics endpoint (`https://*.google-analytics.com`) — as `connect-src`;
`meta.img` names those it may load images from — a CDN, a tag's tracking
pixel (`https://www.googletagmanager.com`) — as `img-src`; `meta.frame` the
origins a page may frame, as `frame-src`; `connect` and `frame` take an
app's own scheme alone too (`"ipc:"`, `"wfpreview:"`), never the web's
(`https:`, `data:`, `blob:`); `meta.wasm: true` adds `'wasm-unsafe-eval'`
for a page that compiles WebAssembly, `meta.style_elements: true` adds
`'unsafe-inline'` to `style-src` for one whose scripts write `<style>`; an entry with a path, or a keyword,
is `E111`. A vendor's inline set-up
snippet (`dataLayer`, `gtag('config', …)`) is a plain script under `src/`.

**The build reads its own output back and holds it to that policy**: an
inline `<script>` or `<style>`, a `style=` attribute, an `on*` attribute,
a script or stylesheet from an origin the policy never named or from a
`data:`/`javascript:` address, a `<base>`, or a refresh to another site
stops the build — read as a browser reads it, in any case, attributes apart
by white space or `/` — and the refused page is removed from the output. One exception is widened rather than refused — a `style { }` value
that reads state is written on the element, so `style-src` gains
`'unsafe-inline'` and the build names the pages that caused it. A page's
own `head { script(src: …) }` or stylesheet `link` is added on the live
page, so it is held to the policy where it is written: one from an origin
the config does not declare is `E902` on that line.

`meta.integrity` gives a declared external asset its subresource-integrity
hash (emitted with `crossorigin="anonymous"`); a build warns about one that
has none. The build never fetches the file to work the hash out.

