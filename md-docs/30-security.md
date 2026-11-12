# 30. Security

<!--
route: guide/security
group: quality
blurb: What the compiler guarantees, what it cannot, and the decisions left to you: markup, secrets, storage, headers and a deployment checklist.
description: What WebFluent enforces, and how to handle outside markup, sessions, env secrets, uploads, CSP headers, other origins and a deploy checklist.
-->

This chapter shows you how to keep a WebFluent site safe: what the compiler
already stops, what it cannot stop, and the decisions it leaves to you.

A web page is a program you hand to a stranger's machine, and its strings
come from everywhere: a form, a URL, a CMS, a database, another service.
Most security bugs are one mistake — a string that arrives somewhere it
becomes *code* instead of *content*. The compiler closes most of those
doors for you. The rest are listed here, with what to do about each.

In this chapter:

- what the compiler guarantees, and what it cannot;
- how to show markup from outside, keep a session, keep secrets out of the
  bundle and handle uploads;
- how to ship a Content-Security-Policy and trust other origins' files;
- how to audit a project, and a checklist for deploying.

## What the compiler guarantees

These are properties of the output, not advice. You do not have to turn
them on.

- **Text is text.** Every string an element shows goes in as a text node.
  `"<script>"` in your data can never become a script. The one exception
  has `Unsafe` in its name ([Show markup you did not write](#show-markup-you-did-not-write)).
  An `html:` argument on an element is an attribute called `html`, as any
  argument an element does not declare is (`C04`), and never markup.
- **No inline handlers.** Events are bound with `addEventListener`.
  `Button("x", onclick: expr)` is a compile error, because an `on*`
  attribute is the one place a browser runs an attribute's value. Write
  `on click { }` instead. `style:` is refused the same way (`E106`): a
  value from data would be the element's whole inline style. A
  `style { }` block sets one property at a time.
- **An attribute holds its value.** A `class:` from data, a page title from
  a route parameter or front matter, a chart's or a QR code's colour: each
  is escaped for where it is written, and a colour must be a colour
  (`#0F766E`, `red`, `rgb(…)`, `var(--x)`) or the default is drawn.
- **No URL a browser would run.** `href:`, `src:`, `to:`, `poster:` and
  `navigate()` take a URL whose scheme is `http`, `https`, `mailto`, `tel`,
  `sms` or `ftp`, or a URL with no scheme at all. The compiler refuses a
  literal where you write it, and the runtime refuses a value that exists
  only at run time at the moment it is used. A page may never point at
  `javascript:`, `data:`, `vbscript:`, `blob:` or `file:`.
- **Except a picture in its address.** A `data:image/png;base64,…` or
  `data:image/svg+xml,…` URL is allowed where a picture is drawn: an
  `Image`'s or an `Avatar`'s `src:`, a Markdown picture, and an `<img>` that
  `sanitize` keeps. An `<img>` never runs what it shows, not even an SVG's
  scripts. A link, a frame or a form never takes one, and `data:text/html`
  is not a picture.
- **No handle on your page.** A link with `target:` carries
  `rel="noopener noreferrer"`, so the page it opens cannot navigate yours;
  `sanitize` adds both to such a link whatever its `rel` said, and keeps
  `opener` on none.
- **`navigate()` stays on the site.** A value the page was handed —
  `navigate(query.next ?? "/")` after a sign-in — goes to a page of the
  site; an address on another origin (`//elsewhere.example`) is refused, and
  the reader stays where they are. Leave the site with a `Link`.
- **A frame drawing HTML from a value runs none of it.** A
  `Frame(srcdoc: value)` with no `sandbox:` is given `sandbox=""`, and the
  build names it (`V03`). Its `on message` hears only what that frame's
  own document posts, from the origin that document has, not a page it
  navigated to.
- **A page's `head { }` points nowhere a browser runs.** Its `src:` and
  `href:` go through the scheme check at build time and in the browser. A
  `script(src:)` must be written out, and a refresh may not take its
  address from a value (`E107`): either would let whoever controls the
  value choose what the page runs or where the reader goes.
- **Markdown is escaped first.** `Markdown(text)` shows any HTML in the
  text instead of running it. Its links and pictures go through the same
  scheme check as every other URL: a link to a `javascript:` URL is drawn
  as its words, with no link, at build time and in the browser alike. So is
  one that puts a backslash before its colon (`javascript\:`), which
  Markdown reads as the colon it escapes. For text somebody else wrote,
  such as a language server's documentation, `Markdown(text).inert` draws
  no link or picture at all ([Content](27-content.md#show-text-somebody-else-wrote-with-inert)).
- **A `Secret` cannot escape.** It may not be shown, spliced into a string,
  logged or persisted. Each is a `T12`, reported at the line that does it.
- **A non-public `env` name cannot be read from a page.** See
  [Keep secrets out of the bundle](#keep-secrets-out-of-the-bundle).

## What it cannot guarantee

Some decisions are outside the compiler's reach:

- **What your server does with what you send it.** Injection into a SQL
  query, a template or a shell happens on the other side of the wire.
- **Whether a reader may see a thing.** `guard:` on a page hides a route;
  it does not authorise anything. Authorisation is the server's answer, and
  the server checks it on every request.
- **A dependency you add yourself.** A `<script>` in `public/`, a font from
  another origin, an embed — each is code you choose to trust.
- **Your hosting.** HTTPS, HSTS, cookie flags and rate limits are the
  host's job.

## Show markup you did not write

Sometimes the content really is HTML: a CMS body, the output of a Markdown
renderer of your own, an email. To show it, pass it through `sanitize` and
into `Unsafe.Html`, the one door for markup.

This page fetches a post and shows its HTML body:

```wf
type Post { title: String, body: String }

api Backend(base: "/api") {
    get post(slug: String) at "posts/:slug" -> Post
}

page PostPage(path: "/p/:slug", title: "Post", description: "One post.", slug: String) {
    resource post = Backend.post(slug: slug)
    match post {
        ready(p) {
            Heading(p.title).h1
            Unsafe.Html(sanitize(p.body))
        }
        else { Skeleton }
    }
}
```

The two calls do different jobs:

- `Unsafe.Html(markup)` puts markup in as markup. Its name lets a review
  find every one with a search for `Unsafe`, and the build prints a `V03`
  at each, saying whether the markup went through `sanitize`.
- `sanitize(html)` is an **allow-list**: a fixed set of elements and, for
  each element, a fixed set of attributes. A tag nobody has thought of is
  dropped, not permitted. It keeps no `<script>`, `<style>`, `<iframe>` or
  `<form>`, and no `on*` attribute, and every `href` and `src` goes through
  the same scheme check as every other URL. A dropped element keeps its
  text, so removing a `<font>` does not remove the sentence inside it.

`sanitize` runs at build time as well as in the browser, from the same
allow-list. A pre-rendered page therefore shows the sanitised body at once,
not an empty box that fills in when the script loads.

> [!CAUTION]
> Use `Unsafe.Html(raw)` without `sanitize` only for markup **your project
> produced**. Anything that arrived from outside goes through `sanitize`.

## Keep the session in a cookie

`persist` writes to `localStorage` or `sessionStorage`. **Neither is a
place for a session token.** Any script that reaches your page can read
both, and that is the whole payload of a cross-site scripting bug: without
a readable token the bug is a defacement; with one it is an account
takeover.

To keep a session safe, have the server set it as an **httpOnly cookie**:

```text
Set-Cookie: session=…; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=1209600
```

Each flag closes one door:

- `HttpOnly` puts the cookie out of reach of every script, including yours.
- `SameSite=Lax` stops another site's form from riding on it.
- `Secure` keeps it off plaintext connections.

The browser sends the cookie with each request, so the page needs no token
at all. An `api` with `credentials: .sameOrigin` sends it with every call:

```wf
type User { id: String, name: String }

api Backend(base: "/api/v1") {
    credentials: .sameOrigin
    get me() -> User
    post signIn(body: Map) -> User
    post signOut()
}

store Session {
    state user: User? = null
    action load() { user = await Backend.me() }
    action out() { await Backend.signOut()  user = null  Backend.me.invalidate()  navigate("/") }
}
```

`Backend.me.invalidate()` on sign-out drops what the endpoint cached, so
the next reader of the tab is not shown the last one's; do the same for
every endpoint whose answer is one reader's. An answer cached
under one `Authorization` header is never served for another, but a cookie
cannot be read, so the cache cannot tell one cookie's reader from the next.

A page *may* keep what a page may show: a display name, a theme, a
preference. If a value would matter to someone who stole it, it belongs
behind a request.

The compiler enforces the part it can. Declare a token as a `Secret`, and
keeping it with `persist` is a `T12`:

```wf expect T12
page Account(path: "/", title: "Account", description: "Your account.") {
    persist _token: Secret = ""
    Heading("Account").h1
}
```

## Keep secrets out of the bundle

`env.NAME` is replaced by its value **at build time**, and the value lands
in the JavaScript the browser downloads. That is right for an API's base
address and wrong for an API key. So a page may read only a name that is
public: one that begins `PUBLIC_`, or one that `public_env` in the config
lists.

```json
{
  "env": { "PUBLIC_API": "/api/v1", "ANALYTICS_ID": "UA-1", "STRIPE_SECRET": "sk_…" },
  "public_env": ["ANALYTICS_ID"]
}
```

With this config:

- a page may read `env.PUBLIC_API` and `env.ANALYTICS_ID`;
- `env.STRIPE_SECRET`, read from a page, a component, a store or an `api`
  block, is an `E108` compile error naming the file and the line.

The value of `STRIPE_SECRET` is still there for `wf render`, which runs on
a server where a secret stays secret
([Server rendering](38-server-rendering.md)). A secret that has to reach a
third party belongs in a request your server makes, not in a page.
[Environments](35-environments.md) covers where `env` values come from.

## Check uploads on the server

A `File` from `FileUpload` is a handle on a file on the reader's machine.
`accept:` and `.size` help the reader pick the right file. They are **not**
checks: both live in the page, and a page is the attacker's to rewrite.

The server decides what it accepts. Have it:

- read the type from the file's bytes, not its name;
- enforce a size limit itself;
- generate the stored file's name, never use the one it was given;
- store uploads somewhere nothing is served from.

## Ship a Content-Security-Policy

A Content-Security-Policy (CSP) tells the browser which scripts, styles,
images and connections a page may use. To ship one, set `"build": { "csp":
true }`. A project `wf init` creates has it on. The build then writes:

- a `<meta http-equiv="Content-Security-Policy">` in every page, right
  after `<meta charset>`: a meta policy governs only what the browser
  parses after it, so it stands before the title, the sharing tags and a
  page's own `head { }` tags, which a page's data reaches;
- a `_headers` file for hosts that read one (Netlify, Cloudflare Pages,
  Vercel; other hosts ignore it).

`frame-ancestors` is only in `_headers`, because a browser ignores it in a
meta tag. A policy that carried it there would say one thing and do
another.

The policy is `'self'` throughout, with no `'unsafe-inline'` for scripts:
the compiler writes external files and binds events with
`addEventListener`, so nothing needs it. The build widens it by exactly the
origins that `meta.fonts`, `meta.stylesheets` and `meta.scripts` declare,
so the policy never blocks a font or library you declared. Your own scripts
under `src/` are served from the site itself and need nothing added.

### Keep other sites from framing your pages

By default no other site may frame your pages. `_headers` says
`frame-ancestors 'none'` and `X-Frame-Options: DENY`, and `wf serve` sends
the same, so nobody can lay your page under theirs to trick a reader into
clicking it.

A project that frames a page of its own — a `Frame` whose `src:` is
site-relative, such as `Frame(src: "/preview/", title: "Preview")` — is
relaxed to its own origin and no further: `frame-ancestors 'self'` and
`X-Frame-Options: SAMEORIGIN`. A `Frame` of another origin's page changes
nothing here; that is `meta.frame`'s `frame-src`, below.

### Allow requests, images and frames from other origins

A page may send requests only to its own origin. To allow another — an API
on another domain, an analytics endpoint — name it in `meta.connect`. Each
entry is an origin, such as `https://api.example.com`, or
`https://*.example.com` for its subdomains, and it joins `connect-src`.
An entry with a path, or a keyword such as `'self'`, is `E111`.

Images follow the same rule: a page may load them from its own origin and
`data:`, unless `meta.img` names others, which join `img-src`. Name a CDN
there, or the pixels a tag requests (`https://www.googletagmanager.com/a`,
`/td`), which a tag assistant otherwise reports as blocked.

Frames are the same again: `meta.frame` names the origins a page may
embed — a video player (`https://www.youtube-nocookie.com`), a map — and
they join `frame-src`.

This config loads Google's tag manager after the page and allows what it
sends and draws:

```json
{
  "meta": {
    "scripts": [{ "src": "https://www.googletagmanager.com/gtag/js?id=G-XXXXXXX", "load": "after" }],
    "connect": ["https://*.google-analytics.com", "https://*.analytics.google.com",
                "https://www.googletagmanager.com"],
    "img": ["https://*.google-analytics.com", "https://www.googletagmanager.com"]
  }
}
```

A tag manager's own set-up — the `dataLayer` and the `gtag('config', …)`
call a vendor gives you as an inline `<script>` — goes in a plain script
under `src/`, which the policy already allows.

### Allow an app's own schemes, WebAssembly and style elements

Three settings widen the policy for pages that need more:

- **An app's own scheme.** An app that serves its pages itself — a desktop
  shell — has schemes of its own. `meta.connect` and `meta.frame` take one
  as a scheme alone: `"ipc:"` for the channel a shell's pages call it on,
  `"wfpreview:"` for a frame the app fills from its own protocol. A scheme
  named in `meta.frame` is also one a frame's `src` may use: the address is
  kept, where any other unknown scheme is emptied. The web's schemes are
  refused: `https:` alone would allow every origin, and `data:`, `blob:`
  and `javascript:` run whatever a page writes.
- **WebAssembly.** A page that compiles WebAssembly — a parser, an image
  codec — sets `meta.wasm: true`, and `script-src` gains
  `'wasm-unsafe-eval'`. WebAssembly compiles; `eval` is still refused.
- **Style elements.** A page whose scripts write `<style>` elements — an
  editor such as CodeMirror, a chart library — sets
  `meta.style_elements: true`, and `style-src` gains `'unsafe-inline'`, for
  styles only.

### Let the build check its own output

**The build holds every page to the policy it ships.** It reads each HTML
file back and stops on anything the policy would block: an inline
`<script>`, a `<style>`, a `style=` attribute, an `on*` attribute, a
script or stylesheet from an origin the policy never named or from an
address that names no place (`data:`, `javascript:`), a `<base>`, or a
refresh that sends the reader to another site. It reads the HTML as a
browser does — names in any case, attributes apart by white space or a
`/`, values quoted or not. A policy is a promise about what a page
contains; without this check, the browser would enforce it and show a
blank page. A page the check refuses is removed from the output, so a
deploy that ignores the failed build does not publish it.

A page's own `head { script(src: …) }` or stylesheet `link` is added on the
live page, after the HTML is written, so the compiler checks it where you
write it instead: one from an origin the config does not declare is an
`E902` on that line.

One thing is allowed rather than refused: a `style { }` value that **reads
state** is written on the element, because it has nowhere else to go. When
a build emits one, `style-src` gains `'unsafe-inline'` and the build names
the pages that caused it, so the stricter policy is one edit away. A
literal in a `style { }` block is already a shared class and costs nothing.

## Pin another origin's files

A stylesheet or a font from another origin is code that origin can change
after you read it. To pin it, declare its hash in `meta.integrity`:

```json
{
  "meta": {
    "stylesheets": ["https://cdn.example.com/brand.css"],
    "integrity": { "https://cdn.example.com/brand.css": "sha384-…" }
  }
}
```

The build emits `integrity` and `crossorigin="anonymous"` on the tag, and
warns (`D05`) about a declared external asset that has no hash. It never
fetches the file to work the hash out, because that would make a build
depend on the network. A Google Fonts stylesheet is exempt: Google writes
it per browser, so its bytes differ between readers and no hash can match.

## Sanitize what your own scripts insert

A `.js` file under `src/` runs on every page exactly as written. The
compiler reads its names; it does not rewrite or sandbox it. It is code you
trust, like the rest of `src/`. What it puts in the page with `innerHTML`
is markup, so markup from outside the project goes through
`WF.sanitize(…)` there too.

A library from another origin belongs in `meta.scripts`, with an integrity
hash, so a change at that origin cannot reach your readers
([JavaScript interop](39-javascript-interop.md)).

## Audit the project

To answer the questions a security review asks, run:

```bash
wf audit
```

It lists:

- every `Unsafe.*`, and whether it is sanitised;
- the project's own scripts, and the names each declares;
- every origin the pages load from;
- everything written to the reader's machine, and where;
- every `env` name, and whether it is public;
- the policy this build ships;
- what the compiler itself depends on.

Add `--json` to get the same report for a tool. `wf audit` only reports; it
never fails a build.

## Before you deploy

- Run `wf audit`, and read all of it.
- Set `"csp": true`, and make sure the build passes its own check.
- Check that `_headers` reaches your host, or set the same headers
  another way.
- Serve over HTTPS, with HSTS.
- Keep the session in an httpOnly, Secure, SameSite cookie — not in
  `persist`.
- Read no non-public `env` name in a page (the build refuses one).
- Make sure every `Unsafe.Html` is sanitised, or provably your own
  markup.
- Check authorisation on the server for every request, not with
  `guard:`.
- Have the server check uploads: type from the bytes, a size limit, a
  generated filename, storage nothing serves from.

## Next

[Performance](31-performance.md): learn what a build already does to be fast, see what it weighs, set budgets that warn, time pages in a browser, and avoid what makes a site slow.
