# Pages and routing

> Page attributes, routes and typed parameters, query strings, layouts, guards, pages in other languages, head tags and the 404 page.

The page: https://webfluent.monzeromer.dev/learn/05-pages-and-routing

This chapter covers WebFluent's pages and routes: a page is a route and what
it shows. Each page names its own path, so there is no separate route table
to keep in step with your pages.

In this chapter:

- the attributes a page takes
- the `app` shell that frames every page, and the `Router`
- links, `navigate()` and how the active link is marked
- route parameters, query strings and fragments
- layouts, guards and per-page head tags
- a page in another language, with `lang:` and `translates:`
- the catch-all page
- what runs, and in what order, when a page opens

## Declaring a page

A page is a `page` declaration with a name, a `path:` and a body. This page
answers `/about`:

```wf
page About(path: "/about", title: "About us", description: "Who we are and why we build this.") {
    Container {
        Heading("About us").h1
        Paragraph("We are three people and a dog.")
    }
}
```

The page's header takes these attributes:

| Attribute | Required | Meaning |
|---|---|---|
| `path:` | yes | The route: `"/"`, `"/about"`, `"/posts/:slug"`, or `"*"` for the catch-all |
| `title:` | strongly advised | The `<title>`, the history entry, and what a screen reader announces on arrival. It may name the page's parameters, `"{slug} — Blog"`, or splice an expression over them and the program's constants |
| `description:` | strongly advised | The meta description: the snippet a search result and a link preview show. Keep it under about 160 characters (`S03`) |
| `image:` | | The image a shared link previews with, site-relative or absolute |
| `image_alt:` | | What that image shows, for a reader who cannot see it (`og:image:alt`). `meta.image_alt` in the config describes the site's image |
| `type:` | | `og:type`: `"website"` (the default) or `"article"` |
| `noindex:` | | `true` keeps the page out of search results and out of the sitemap |
| `lang:` | | The language the page is written in, when it is not the site's: `"en"`, `"ar"`, `"pt-BR"`. It sets `<html lang dir>`, `og:locale` and the JSON-LD's `inLanguage` ([A page in its own language](https://webfluent.monzeromer.dev/recipes/site-in-two-languages.md)) |
| `translates:` | | The page this one is a translation of, by name: `translates: Home`. Translations list each other as `hreflang` alternates |
| `layout:` | | The component that frames the page ([Layouts](#layouts)) |
| `guard:` | | An expression that must hold for the route to render ([Guards](#guards)) |
| `redirect:` | | Where the reader goes when the guard fails. The default is `/` |
| `paths:` | | The values a static build renders a `:param` page for ([Static or single-page](https://webfluent.monzeromer.dev/about/static-or-single-page.md#pre-render-a-dynamic-route-with-paths)) |
| `name: Type` | | A route parameter, typed ([Route parameters](#route-parameters)) |

A page without a `title:` builds with warning `S01`, and one without a
`description:` with warning `S02`. The compiler cannot write those for you,
and a page without them looks bare when someone shares or searches for it.

> [!NOTE]
> **Coming from React Router, Vue Router or SvelteKit?** There is no route
> table and no `routes/` folder. Each `page` names its own `path:`, and the
> compiler builds the router from every page. See
> [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md#things-that-work-differently).

## The app shell

The `app` declaration is the shell: what every page renders inside, such as a
navbar and a footer. The `Router` marks where the current page goes:

```wf
app {
    Navbar {
        Navbar.Brand { Link("Acme", to: "/") }
        Navbar.Links {
            Link("Docs", to: "/docs").prefix
            Link("Pricing", to: "/pricing")
        }
        Navbar.Actions { Button("Sign in").sm { on click { navigate("/login") } } }
    }
    Router
    Footer { Text("© Acme").muted.sm }
}
```

The `Router` behaves like this:

- It is bare: it takes no children, and it may sit at any depth inside `app`
  (in a `Row`, a `Container`, a `Stack`).
- Its route table is built from every page's `path:`, ordered by
  specificity: static segments before `:params`, and `*` last. The order you
  declare pages in can never shadow a route.
- A project with no `app` declaration gets a bare router.
- `Router(transition: .fade, duration: "200ms")` animates page changes
  ([Motion](https://webfluent.monzeromer.dev/recipes/route-transitions.md)).

## Links and navigation

A `Link` is an `<a>` whose clicks the router handles. To move from code, for
example after a button press, call `navigate(path)`:

```wf
page Start(path: "/", title: "Start", description: "Where the tour begins.") {
    Heading("Start here").h1
    Link("Home", to: "/")
    Link("Guide", to: "/docs").prefix           // also active on /docs/anything
    Button("Take the tour") { on click { navigate("/about") } }
}

page Guide(path: "/docs", title: "Guide", description: "The guide.") { Heading("Guide").h1 }
page About(path: "/about", title: "About", description: "Who we are.") { Heading("About").h1 }
```

### The active link

The link whose `to:` matches the current route carries the class `.active`
and `aria-current="page"`, so you can style it and a screen reader announces
it. Two rules widen or narrow the match:

- `.prefix` (the flag for `active: .prefix`) also makes a link active on the
  routes beneath it, which suits a section link such as **Guide**.
- A `to:` with a query, such as `"/?show=open"`, is active only when the
  address carries those values too. A row of filter links then marks the one
  that is on, and a plain link to the same path gives way to it.

### Paths under a sub-path

Write every path from the site's root, `"/about"`, even when the site is
served under a sub-path. With `build.base_path` set to `"/docs"`, the
compiler prefixes every `Link`, every `navigate()` and every asset for you.

### Static and single-page navigation

How a link moves depends on the build:

- In a static build, a link is a real page load, and so is `navigate()`.
- In a single-page app, a link swaps the page without a reload. Focus moves
  to the new page's heading, the scroll goes back to the top (or to the
  `#fragment` the address names), and the new title is announced.

## Route parameters

A `:name` segment in the path is a typed parameter of the page. You read it
as a plain name, or as `params.name`:

```wf
type Post { slug: String, title: String, body: String }

store Posts {
    state all: [Post] = [
        Post(slug: "hello", title: "Hello", body: "First post."),
        Post(slug: "again", title: "Again", body: "Second post.")
    ]
}

page PostPage(path: "/posts/:slug", title: "Post", description: "One post.", slug: String) {
    use Posts
    derived post = Posts.all.find(p => p.slug == slug)
    if let p = post {
        Heading(p.title).h1
        Paragraph(p.body)
    } else {
        Heading("No such post").h1
        Text("Nothing lives at /posts/{params.slug}.")
    }
}
```

A parameter is a `String` unless you declare another type, such as
`id: Number`. A static build renders a `:param` page once for each value its
`paths:` names ([Pre-render a dynamic route](https://webfluent.monzeromer.dev/about/static-or-single-page.md#pre-render-a-dynamic-route-with-paths)).

## Query strings and fragments

`query` is the URL's query string as a map, and `hash` is its `#fragment`.
The runtime keeps both current, so a page that reads them follows the
address:

```wf
page Products(path: "/products", title: "Products", description: "Everything we sell.") {
    derived tab = query.tab ?? "all"
    derived sort = query.sort ?? "name"
    Heading("Products").h1
    Row(gap: .sm) {
        Link("All", to: "/products?tab=all")
        Link("On sale", to: "/products?tab=sale")
    }
    Text("Showing {tab}, sorted by {sort}; section: {hash}")
}
```

A query string is not part of the route: `/products?tab=sale` is the
`/products` page. To change the query from code, call
`navigate("/products?tab=sale")`.

The *static paint* is the HTML the build pre-renders for a page. In it,
`query` and `hash` are unknown, so anything that depends on them is drawn
once the page is live in the browser.

## Layouts

A layout is a component that frames a page. You name it with `layout:`, and
the page renders in its default slot. Use a layout for one section's chrome,
such as a docs sidebar or an app rail; the `app` shell is for the whole
site's.

In this example, two docs pages share one sidebar and breadcrumb:

```wf
component DocsShell(_ crumb: String) {
    slot
    Row(gap: .lg) {
        Sidebar {
            Sidebar.Item(to: "/docs") { Text("Guide") }
            Sidebar.Item(to: "/docs/reference") { Text("Reference") }
        }
        Stack {
            Breadcrumb {
                Breadcrumb.Item(to: "/docs") { Text("Docs") }
                Breadcrumb.Item { Text(crumb) }
            }
            children
        }
    }
}

page Guide(path: "/docs", title: "Guide", description: "The guide.", layout: DocsShell("Guide")) {
    Heading("Guide").h1
}

page Reference(path: "/docs/reference", title: "Reference", description: "The reference.", layout: DocsShell("Reference")) {
    Heading("Reference").h1
}
```

A layout follows these rules:

- The compiler checks the layout's arguments like any component call
  (`DocsShell("Guide")`).
- The layout component must declare a default `slot`, and place `children`
  where the page goes. Without the `slot` line the build stops with error
  `E104`. A component that is only ever called, not named as a layout,
  needs `children` alone ([Slots](https://webfluent.monzeromer.dev/learn/10-components.md#slots)).
- The page's skip link jumps past the layout's chrome to the page's first
  element. That element gets `id="wf-content"` when it has no id of its own.

## Guards

A guard keeps a route from rendering until a condition holds. `guard:` names
an expression, usually a store's derived value. When it is false, the reader
is sent to `redirect:` (by default, `/`):

```wf
store Auth {
    state user = null
    derived loggedIn = user != null
    action login() { user = { name: "Sam" } }
    action logout() { user = null }
}

page Account(path: "/account", title: "Your account", description: "Your account.", guard: Auth.loggedIn, redirect: "/login") {
    use Auth
    Heading("Welcome back").h1
    Button("Sign out") { on click { Auth.logout() } }
}

page Login(path: "/login", title: "Sign in", description: "Sign in.") {
    use Auth
    Heading("Sign in").h1
    Button("Sign in").primary { on click { Auth.login()  navigate("/account") } }
}
```

The browser evaluates the guard on every navigation.

> [!CAUTION]
> A guard is a gate in the browser, not a secret. A static build still
> pre-renders the guarded page's HTML, so keep anything private behind an
> API that the page fetches.

## Per-page head tags

A `head { }` block adds tags of the page's own to the document's `<head>`.
The build writes them into the static paint where it knows the value, the
live page keeps them current, and they are removed when the reader leaves
the page:

```wf
page Share(path: "/share", title: "Share", description: "A page with its own tags.") {
    state image = "/og-share.png"
    head {
        meta(property: "og:image", content: image)
        link(rel: "canonical", href: "https://example.com/share")
        script(src: "/analytics.js", defer: true)
    }
    Heading("Share").h1
}
```

A tag's `src:` and `href:` go through the same scheme check as every
address, at build time and on the live page, so a value never points one at
`javascript:` or `data:`. A `script(src:)` is written out, never read from a
value, and a `meta(http-equiv: "refresh")` may not take its address from
one (`E107`): whoever controls the value would choose what the page runs or
where the reader goes.

`title:` and `description:` are page attributes, not head tags. From them
and the config, the compiler writes the standard tags itself: charset,
viewport, description, canonical, Open Graph, Twitter card and JSON-LD.

## A page in another language

A page written in a language other than the site's says so with `lang:`, and
names the page it translates with `translates:`. This site is in English,
with its home page in Arabic at `/ar`:

```wf
page Home(path: "/", title: "Home", description: "What we make.") {
    Heading("Welcome").h1
}

page HomeAr(path: "/ar", title: "الرئيسية", description: "ما نصنعه.", lang: "ar", translates: Home) {
    Heading("أهلاً بك").h1
}
```

What the two attributes do:

- **`lang:`** writes the page as `<html lang="ar" dir="rtl">`, with its
  `og:locale` and its JSON-LD's `inLanguage` to match. The router sets
  `lang` and `dir` when it shows the page, and puts the site's back on a page
  without one.
- **`translates:`** joins the pages. Each lists the others as `hreflang`
  alternates, each at its own address, with `x-default` for the one in the
  site's language, so a search engine finds every language.
- A `translates:` that names no page, or two translations of one page in one
  language, is `R05`.

This is different from `t()` and `setLocale`, which translate one address in
the browser. [Publish a page in each language](https://webfluent.monzeromer.dev/recipes/site-in-two-languages.md)
compares the two.

## The catch-all page

A page with `path: "*"` answers every route no other page claims. The router
shows it in the browser, and a static host serves it as `404.html`:

```wf
page NotFound(path: "*", title: "Not found", description: "Nothing lives here.", noindex: true) {
    Container {
        Heading("Nothing here").h1
        Link("Back home", to: "/")
    }
}
```

## Set-up code: what runs when a page opens

A page's body is a render block: it declares state, derived values and
actions, and lists what to show. The top of the body runs once, when the page
renders. A call written there, such as `Deploys.load(id)`, is *set-up code*:
it runs once as the page opens.

```wf
store Deploys {
    state rows = []
    action load(id: String) { rows = [{ id: id, ok: true }] }
}

page Deploy(path: "/deploys/:id", title: "Deploy", description: "One deploy.", id: String) {
    use Deploys
    Deploys.load(id)
    Heading("Deploy {id}").h1
    for row in Deploys.rows by row.id {
        Text("{row.id}: {row.ok}")
    }
}
```

### The life of a page

A page goes through these stages, in order:

1. The route matches. In a split build, the page's chunk loads, and its
   stylesheet with it.
2. The body runs **once**: each `state` takes its initial value, set-up calls
   run, and the elements are created. Nothing re-renders after this; whatever
   reads a state updates on its own.
3. Each `effect` runs, and runs again when what it reads changes. Timers
   start.
4. The page is placed. In a single-page app, focus moves to its heading, the
   scroll goes to the top and the title is announced.
5. When the route changes, everything the page created is disposed of:
   effects, timers, listeners, connections and `.route` stores. Their
   `cleanup { }` blocks run.

There is no `onMount` to write. Set-up code runs at stage 2, an `effect` at
stage 3, and `after(0) { … }` once the page is on screen.

> [!NOTE]
> **Coming from React?** A page's body is not a function called on every
> render; it runs once. Set-up code does what `useEffect(…, [])` did, and
> nothing re-renders after it. See
> [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md#things-that-work-differently).

## Quick reference

| Syntax | Meaning |
|---|---|
| `page Name(path: "/x", title: "…", description: "…") { … }` | A page and its route |
| `path: "/posts/:slug"`, `slug: String` | A typed route parameter, read as `slug` or `params.slug` |
| `path: "*"` | The catch-all page, served as `404.html` |
| `app { … Router … }` | The shell around every page, and where the page goes |
| `Link("Docs", to: "/docs").prefix` | A link, active on its route and the routes beneath it |
| `navigate("/about")` | Move to a route from code |
| `query.tab`, `hash` | The query string and the fragment, kept current |
| `layout: Shell("Crumb")` | Frame the page with a component's default slot |
| `guard: Auth.loggedIn, redirect: "/login"` | Render only while a condition holds |
| `head { meta(…) link(…) script(…) }` | Tags of the page's own in `<head>` |
| `lang: "ar", translates: Home` | A page in another language, joined to the page it translates |

The [configuration reference](https://webfluent.monzeromer.dev/reference/config.md) covers `build.base_path`,
and [Static or single-page](https://webfluent.monzeromer.dev/about/static-or-single-page.md) covers how each build
serves pages.

## Common mistakes

**A route names a parameter the page does not declare.** Each `:name` in
the path is a parameter of the page, and the page must declare it:

```wf expect=R02
page Post(path: "/posts/:slug", title: "Post", description: "One post.") {
    Heading("A post").h1
}
```

Add `slug: String` to the page's header, beside `path:`, and read it as
`slug` ([R02](https://webfluent.monzeromer.dev/errors/R02.md)).

**A link to a page that does not exist.** The compiler knows every route, so
a `Link` or a `navigate()` to a path no page answers stops the build:

```wf expect=R01
page Home(path: "/", title: "Home", description: "The front page.") {
    Heading("Home").h1
    Link("About us", to: "/about")
}
```

Declare the page, `page About(path: "/about", …)`, or fix the path
([R01](https://webfluent.monzeromer.dev/errors/R01.md)).

**A layout with nowhere to put the page.** A `layout:` component must
declare a default `slot` and place `children` where the page goes:

```wf expect=E104
component Shell(_ crumb: String) {
    Stack {
        Text(crumb)
    }
}

page Home(path: "/", title: "Home", description: "The front page.", layout: Shell("Home")) {
    Heading("Home").h1
}
```

Add `slot` at the top of `Shell` and `children` inside its `Stack`
([E104](https://webfluent.monzeromer.dev/errors/E104.md)).

## Next

[Elements](https://webfluent.monzeromer.dev/learn/06-elements.md).
