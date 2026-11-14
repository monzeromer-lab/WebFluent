# Security

> What WebFluent enforces and why, and where its guarantees stop: outside markup, sessions, env secrets, uploads, the CSP and code you choose to trust.

The page: https://webfluent.monzeromer.dev/about/security

This essay sets out WebFluent's security model: what the compiler
guarantees about every page it writes, what no compiler can guarantee, and
the decisions it leaves to you.

A web page is a program you hand to a stranger's machine, and its strings
come from everywhere: a form, a URL, a CMS, a database, another service.
Most security bugs are one mistake, a string that arrives somewhere it
becomes *code* instead of *content*. Advice against that mistake has been
written for twenty years, and the mistake is still the most common one,
because advice has to be followed on every line and a deadline only has
to win once.

So WebFluent does not advise. It makes the safe spelling the only one the
compiler accepts, and names the one door that is not safe so a review can
find it. What follows is the list of those guarantees, the places they
stop, and what lies on the other side of each.

## What the compiler guarantees

These are properties of the output, not advice. You do not have to turn
them on.

- **Text is text.** Every string an element shows goes in as a text node.
  `"<script>"` in your data can never become a script. The one exception
  has `Unsafe` in its name ([One door for markup](#one-door-for-markup)).
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
  (`#0F766E`, `red`, `rgb(…)`, `var(--x)`) or the default is drawn. A
  design token whose value holds `<`, `{`, `}` or `;` is left out of the
  stylesheet, so a theme cannot close a rule and open another.
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
  no link or picture at all ([Render Markdown](https://webfluent.monzeromer.dev/recipes/render-markdown.md)).
- **A `Secret` cannot escape.** It may not be shown, spliced into a string,
  logged or persisted. Each is a `T12`, reported at the line that does it.
- **A non-public `env` name cannot be read from a page.** See
  [Secrets stay out of the bundle](#secrets-stay-out-of-the-bundle).
- **A build stays in its project.** `data`, `image`, `api … from` and
  `i18n.dir` name paths inside the project; an absolute path or a `..` is
  an error. A link that points out of the project, at nothing, or back on
  itself is left out of every walk with a warning. A `build.output` outside
  the project, or a link inside it, is refused unless the command line says
  `--allow-output-outside`, and the build removes only the `.gz` files it
  wrote itself. A cloned project cannot make `wf build` read or delete
  files elsewhere on your machine.

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

## One door for markup

Sometimes the content really is HTML: a CMS body, the output of a Markdown
renderer of your own, an email. There is one way to show it, and it takes
two calls, `Unsafe.Html(sanitize(body))`, because they do different jobs:

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

`Unsafe.Html` without `sanitize` is for markup the project itself produced;
anything that arrived from outside goes through `sanitize`.
[Show HTML from outside the project](https://webfluent.monzeromer.dev/recipes/show-html-from-outside.md)
is a page that does it.

## Where a session belongs

`persist` writes to `localStorage` or `sessionStorage`. **Neither is a
place for a session token.** Any script that reaches your page can read
both, and that is the whole payload of a cross-site scripting bug: without
a readable token the bug is a defacement; with one it is an account
takeover.

A session belongs in an **httpOnly cookie** the server sets, and each of
its flags closes one door:

- `HttpOnly` puts the cookie out of reach of every script, including yours.
- `SameSite=Lax` stops another site's form from riding on it.
- `Secure` keeps it off plaintext connections.

The browser sends the cookie with each request, so the page needs no token
at all: an `api` with `credentials: .sameOrigin` sends it with every call.
[Sign in with a session cookie](https://webfluent.monzeromer.dev/recipes/sign-in-with-a-session-cookie.md)
builds the whole flow, including what to forget when the reader signs out.

A page *may* keep what a page may show: a display name, a theme, a
preference. If a value would matter to someone who stole it, it belongs
behind a request.

The compiler enforces the part it can. Declare a token as a `Secret`, and
keeping it with `persist` is a `T12`:

```wf expect=T12
page Account(path: "/", title: "Account", description: "Your account.") {
    persist _token: Secret = ""
    Heading("Account").h1
}
```

## Secrets stay out of the bundle

`env.NAME` is replaced by its value **at build time**, and the value lands
in the JavaScript the browser downloads. That is right for an API's base
address and wrong for an API key. So a page, a component, a store or an
`api` block may read only a name that is public: one that begins `PUBLIC_`,
or one that `public_env` in the config lists. Any other name is an `E108`
compile error naming the file and the line, and every build warns of each
`public_env` name it bundles, since its name does not say so.

`public_env` comes with the project, so it cannot reach into the shell of
whoever builds it: a name it lists without the `PUBLIC_` prefix takes its
value from the config or `.env`, and from the shell only when
`WF_ALLOW_SHELL_ENV` names it.

A private value is still there for `wf render`, which runs on a server
where a secret stays secret
([Render a template](https://webfluent.monzeromer.dev/recipes/render-a-template.md)). A secret that has to
reach a third party belongs in a request your server makes, not in a page.
[Use environment values, and keep secrets out](https://webfluent.monzeromer.dev/recipes/environments-and-secrets.md)
sets values, lists public ones, and builds once per environment.

## Uploads are checked on the server

A `File` from `FileUpload` is a handle on a file on the reader's machine.
`accept:` and `.size` help the reader pick the right file. They are **not**
checks: both live in the page, and a page is the attacker's to rewrite.

The server decides what it accepts. It reads the type from the file's
bytes, not its name; enforces a size limit itself; generates the stored
file's name, never using the one it was given; and stores uploads
somewhere nothing is served from.

## The Content-Security-Policy

A Content-Security-Policy (CSP) tells the browser which scripts, styles,
images and connections a page may use. `"build": { "csp": true }` ships
one, and a project `wf init` creates has it on. The build then writes:

- a `<meta http-equiv="Content-Security-Policy">` in every page, right
  after `<meta charset>`: a meta policy governs only what the browser
  parses after it, so it stands before the title, the sharing tags and a
  page's own `head { }` tags, which a page's data reaches;
- a `_headers` file for hosts that read one (Netlify, Cloudflare Pages;
  other hosts ignore it — on Vercel the headers go in `vercel.json`, see
  [Deploy to Vercel](https://webfluent.monzeromer.dev/recipes/deploy-to-vercel.md), and
  [Set cache and security headers](https://webfluent.monzeromer.dev/recipes/cache-and-security-headers.md)
  covers the rest).

`frame-ancestors` is only in `_headers`, because a browser ignores it in a
meta tag. A policy that carried it there would say one thing and do
another.

The policy is `'self'` throughout, with no `'unsafe-inline'` for scripts:
the compiler writes external files and binds events with
`addEventListener`, so nothing needs it. The build widens it by exactly the
origins that `meta.fonts`, `meta.stylesheets` and `meta.scripts` declare,
so the policy never blocks a font or library you declared. Your own scripts
under `src/` are served from the site itself and need nothing added.

### Who may frame a page

By default no other site may frame your pages. `_headers` says
`frame-ancestors 'none'` and `X-Frame-Options: DENY`, and `wf serve` sends
the same, so nobody can lay your page under theirs to trick a reader into
clicking it.

A project that frames a page of its own — a `Frame` whose `src:` is
site-relative, such as `Frame(src: "/preview/", title: "Preview")` — is
relaxed to its own origin and no further: `frame-ancestors 'self'` and
`X-Frame-Options: SAMEORIGIN`. A `Frame` of another origin's page changes
nothing here; that is `meta.frame`'s `frame-src`, below.

### What else a page may reach

Anything beyond that is named, origin by origin: `meta.connect` for the
origins a page sends requests to, `meta.img` for images, `meta.frame` for
frames, each joining its directive of the policy, plus an app's own
schemes, `meta.wasm` for WebAssembly and `meta.style_elements` for scripts
that write `<style>`. Each widens the policy by exactly what it names, and
no more. [Allow other origins in the security policy](https://webfluent.monzeromer.dev/recipes/allow-other-origins.md)
sets each one.

### The build checks its own output

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

## Code you choose to trust

A `.js` file under `src/` runs on every page exactly as written. The
compiler reads its names; it does not rewrite or sandbox it. It is code you
trust, like the rest of `src/`. What it puts in the page with `innerHTML`
is markup, so markup from outside the project goes through
`WF.sanitize(…)` there too.

A file from another origin — a library, a stylesheet, a font — is code
that origin can change after you read it. It belongs in `meta.scripts`,
`meta.stylesheets` or `meta.fonts` with an integrity hash in
`meta.integrity`, so a change at that origin cannot reach your readers; the
build warns (`D05`) about one without
([Allow other origins in the security policy](https://webfluent.monzeromer.dev/recipes/allow-other-origins.md#variations),
[Load a library from a CDN](https://webfluent.monzeromer.dev/recipes/library-from-a-cdn.md)).

## What a review reads

Everything the compiler cannot refuse is something the project chose to
trust, so a review starts from a list of those choices. `wf audit` prints
it — every `Unsafe.*`, every script, origin and stored value, every `env`
name, the policy — and never fails a build.
[Review a site's security before you deploy](https://webfluent.monzeromer.dev/recipes/review-security.md)
reads it, and goes through the checklist for the build, the host and the
server.

## Next

[What a build ships](https://webfluent.monzeromer.dev/about/what-a-build-ships.md): what a build already does to be fast, and how to see what it weighs.
