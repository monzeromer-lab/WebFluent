# 23. Security

<!--
route: guide/security
group: building
blurb: What the compiler guarantees, what it cannot, and the decisions left to you: markup, secrets, storage, headers and a deployment checklist.
description: The threat model, what WebFluent enforces, markup from outside, sessions, env and secrets, uploads, CSP and headers, audits and a checklist.
-->

A web page is a program you hand to a stranger's machine, and the strings
in it come from everywhere: a form, a URL, a CMS, a database, another
service. Most of what goes wrong is one mistake — a string arriving
somewhere it becomes *code* rather than *content*.

This chapter says what the compiler stops, what it cannot, and what is
left for you to decide.

## What the compiler guarantees

These are properties of the output, not advice. You do not opt in.

- **Text is text.** Every string an element shows goes in as a text node.
  There is no path by which `"<script>"` in your data becomes a script. The
  one exception has `Unsafe` in its name.
- **No inline handlers.** Events bind through `addEventListener`. Writing
  `Button("x", onclick: expr)` is a compile error, because that attribute
  is the one place an attribute value is executed.
- **No URL a browser would run.** `href:`, `src:`, `to:`, `poster:` and
  `navigate()` take a URL whose scheme is `http`, `https`, `mailto`, `tel`,
  `sms`, `ftp` or nothing at all. A literal is refused where it is written;
  a value that only exists at run time is refused at the moment it is used.
  `javascript:`, `data:`, `vbscript:`, `blob:` and `file:` are not URLs a
  page may point at.
- **No handle on your page.** A link with `target:` carries
  `rel="noopener noreferrer"`, so the page it opens cannot navigate yours.
- **Markdown is escaped first.** `Markdown(text)` renders the small
  Markdown it documents and shows any HTML in the text rather than running
  it.
- **A `Secret` cannot escape.** It may not be shown, spliced into a string,
  logged or persisted — each is a `T12`, at the line that does it.
- **A non-public `env` name may not be read from a page.** See below.

## What it cannot

- **What your server does with what you send it.** Injection into a query,
  a template or a shell is on the other side of the wire.
- **Whether a reader may see a thing.** `guard:` on a page hides a route;
  it does not authorise anything. Authorisation is the server's answer and
  has to be checked there, on every request.
- **A dependency you add yourself.** A `<script>` in `public/`, a font from
  another origin, an embed — each is code you are choosing to trust.
- **Your hosting.** HTTPS, HSTS, cookie flags and rate limits are the
  host's.

## Markup you did not write

A CMS body, a Markdown renderer of your own, an email — sometimes the
content really is HTML. There is one door:

```wf
type Post { title: String, body: String }

api Backend(base: "/api") {
    get post(slug: String) at "posts/:slug" -> Post
}

page PostPage(path: "/p/:slug", title: "Post", description: "One post.", slug: String) {
    resource post = Backend.post(slug: slug)
    match post {
        ready(p) { Unsafe.Html(sanitize(p.body)) }
        else { Skeleton }
    }
}
```

`Unsafe.Html(markup)` puts markup in as markup. It is named so that a
review greps for `Unsafe` and finds every one, and the build prints a `V03`
at each — including whether it went through `sanitize`.

`sanitize(html)` is an **allow-list**: a fixed set of elements and, per
element, a fixed set of attributes. A tag nobody has thought of is dropped
rather than permitted. No `<script>`, no `<style>`, no `<iframe>`, no
`<form>`, no `on*`, and an `href` or `src` goes through the same scheme
check every other URL does. An element that is dropped keeps its text, so
removing a `<font>` does not remove the sentence inside it.

It runs at build time as well as in the browser, from the same allow-list,
so a pre-rendered page shows the sanitised body immediately rather than an
empty box that fills in when the script loads.

`Unsafe.Html(raw)` without `sanitize` is for markup **your project
produced**. Anything that arrived from outside goes through the sanitiser.

## Sessions and tokens

`persist` writes to `localStorage` or `sessionStorage`. **Neither is a
place for a session token.** Any script that reaches your page can read
both, which is the whole payload of a cross-site scripting bug: without a
readable token it is a defacement, with one it is an account takeover.

Keep the session in an **httpOnly cookie**, set by the server:

```
Set-Cookie: session=…; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=1209600
```

`HttpOnly` puts it out of reach of every script, including yours;
`SameSite=Lax` stops another site's form from riding on it; `Secure` keeps
it off plaintext connections. The browser sends it with each request, so
the page needs no token at all:

```wf
api Backend(base: "/api/v1") {
    credentials: .sameOrigin      // the cookie goes with every call
    get me() -> User
    post signIn(body: Map) -> User
    post signOut()
}

store Session {
    state user: User? = null
    action load() { user = await Backend.me() }
    action out() { await Backend.signOut()  user = null  navigate("/") }
}
```

What a page *may* keep is what a page may show: a display name, a theme, a
preference. If a value would matter to someone who stole it, it belongs
behind a request.

The compiler enforces the part it can: `persist token: Secret = ""` is a
`T12`. Declaring a token `Secret` is how you ask it to.

## `env`, and what ends up in the bundle

`env.NAME` is replaced by its value **at build time**, and the value lands
in the JavaScript the browser downloads. That is right for an API base and
wrong for an API key, and nothing in the spelling said which was which.

Now a name is public when it says so, or when the config says so:

```json
{
  "env": { "PUBLIC_API": "/api/v1", "ANALYTICS_ID": "UA-1", "STRIPE_SECRET": "sk_…" },
  "public_env": ["ANALYTICS_ID"]
}
```

`env.PUBLIC_API` and `env.ANALYTICS_ID` may be read from a page.
`env.STRIPE_SECRET` read from a page, a component, a store or an `api`
block is a compile error naming the file and the line. The value is still
there for `wf render`, which runs on a server where a secret is a secret.

A secret that has to reach a third party belongs in a request your server
makes, not in a page.

## Uploads

A `File` from `FileUpload` is a handle on a file on the reader's machine.
`accept:` and `.size` are conveniences for them, **not** checks: both are
the page's, and a page is the attacker's to rewrite. The server decides
what it accepts — the type from the bytes rather than the name, a size
limit it enforces itself, a filename it generates rather than one it was
given, and storage somewhere nothing is served from.

## Content-Security-Policy

`"build": { "csp": true }` — on for a project `wf init` creates — ships:

- a `<meta http-equiv="Content-Security-Policy">` in every page, and
- a `_headers` file for hosts that read one (Netlify, Cloudflare Pages,
  Vercel; others ignore it).

`frame-ancestors` is only in `_headers`, because a browser ignores it in a
meta tag — a policy that carried it there would say one thing and deliver
another.

The policy is `'self'` throughout, with no `'unsafe-inline'` for scripts:
the compiler writes external files and binds events with
`addEventListener`, so nothing needs it. It is widened by exactly the
origins `meta.fonts`, `meta.stylesheets` and `meta.scripts` declare, so a
declared font or library is never blocked by the policy shipped beside it.
The project's own scripts under `src/` are served from the site itself, so
they need nothing added.

Where a page may *send* requests is the same rule: its own origin, unless
`meta.connect` names others — an API on another domain, an analytics
endpoint. Each is an origin, `https://api.example.com`, or
`https://*.example.com` for its subdomains, and joins `connect-src`:

```json
{
  "meta": {
    "scripts": [{ "src": "https://www.googletagmanager.com/gtag/js?id=G-XXXXXXX", "async": true }],
    "connect": ["https://*.google-analytics.com", "https://*.analytics.google.com",
                "https://www.googletagmanager.com"],
    "img": ["https://*.google-analytics.com", "https://www.googletagmanager.com"]
  }
}
```

Images follow the same rule: a page's own origin and `data:`, unless
`meta.img` names others, which join `img-src` — a CDN, or the pixels a tag
requests (`https://www.googletagmanager.com/a`, `/td`), which a tag
assistant otherwise reports as blocked.

A tag manager's own set-up — the `dataLayer` and the `gtag('config', …)`
call a vendor pastes as an inline `<script>` — is a plain script under
`src/`, which the policy already allows.

**The build checks its own output against it.** Every HTML file is read
back and held to the policy it carries: an inline `<script>`, a `<style>`,
a `style=` attribute, an `on*` attribute, or a script or stylesheet from an
origin the policy never named stops the build. A policy is a promise about
what a page contains, and this is how the promise is kept — the browser
would enforce it and show a blank page. A page's own `head { script(src:
…) }` or stylesheet `link` is added on the live page, after the HTML is
written, so it is held to the policy where it is written instead: one from
an origin the config does not declare is an `E902` on that line.

One thing is allowed rather than refused: a `style { }` value that **reads
state** is written on the element, because there is nowhere else for it to
go. Where a build emits one, `style-src` gains `'unsafe-inline'` and the
build names the pages that caused it, so the stricter policy is one edit
away. A literal in a `style { }` block is already a shared class and costs
nothing.

## Another origin's files

A stylesheet or a font from somewhere else is code that origin can change
after you read it. Declare its hash:

```json
{
  "meta": {
    "stylesheets": ["https://cdn.example.com/brand.css"],
    "integrity": { "https://cdn.example.com/brand.css": "sha384-…" }
  }
}
```

The build emits `integrity` and `crossorigin="anonymous"`, and warns about
a declared external asset that has none. It never fetches the file to work
the hash out — that would make a build depend on the network. A Google
Fonts stylesheet is exempt: it is written per user-agent, so its bytes
differ between readers and no hash can match.

## Your own scripts

A `.js` file under `src/` runs on every page exactly as written — the
compiler reads its names, it does not rewrite or sandbox it. It is code you
trust, like the rest of `src/`: what it puts in the page with `innerHTML`
is markup, so markup from outside the project goes through
`WF.sanitize(…)` there too. A library from another origin is listed in
`meta.scripts` with an integrity hash, so a change at that origin cannot
reach your readers.

## `wf audit`

One command for the questions a review asks:

```bash
wf audit
```

Every `Unsafe.*` and whether it is sanitised, the project's own scripts and
the names each declares, every origin the pages load from, everything
written to the reader's machine and where, every `env` name and whether it
is public, the policy this build ships, and what the compiler itself
depends on. `--json` for a tool. It reports; it never fails
a build.

## Before you deploy

- `wf audit`, and read all of it.
- `"csp": true`, and the build passing its own check.
- `_headers` reaching your host, or the same headers set another way.
- HTTPS, with HSTS.
- Session in an httpOnly, Secure, SameSite cookie — not in `persist`.
- No non-public `env` name in the bundle (the build refuses one).
- Every `Unsafe.Html` either sanitised or provably your own markup.
- Authorisation checked on the server for every request, not by `guard:`.
- Uploads checked by the server: type from the bytes, a size limit, a
  generated filename, storage nothing serves from.

## Next

[Testing](24-testing.md).
