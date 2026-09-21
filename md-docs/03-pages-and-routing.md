# 3. Pages and routing

A page is a route and what it shows. Pages own their paths: there is no route
table to keep in step with them.

## Declaring a page

```wf
page About(path: "/about", title: "About us", description: "Who we are and why we build this.") {
    Container {
        Heading("About us").h1
        Paragraph("We are three people and a dog.")
    }
}
```

The header takes these attributes:

| Attribute | Required | Meaning |
|---|---|---|
| `path:` | yes | The route: `"/"`, `"/about"`, `"/posts/:slug"`, or `"*"` for the catch-all |
| `title:` | strongly | The `<title>`, the history entry, what a screen reader announces on arrival |
| `description:` | strongly | The meta description and the snippet a search result and a link preview show (~150 characters) |
| `image:` | | The image a shared link previews with, site-relative or absolute |
| `type:` | | `og:type` — `website` (default) or `article` |
| `noindex:` | | `true` keeps the page out of search results and the sitemap |
| `layout:` | | The component that frames the page (below) |
| `guard:` | | An expression that must hold for the route to render |
| `redirect:` | | Where the reader goes when the guard fails (default `/`) |
| `paths:` | | The values a static build renders a `:param` page for (chapter 14) |
| `name: Type` | | A route parameter, typed (below) |

A page without a `title` or `description` builds with a warning (`S01`,
`S02`); the compiler cannot write those for you, and a page without them ships
unshareable.

## The app shell

`app` declares what every page renders inside — a navbar, a footer, the
`Router` where the current page goes:

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

`Router` is bare — it takes no children — and the table is built from every
page's `path`, ordered by specificity (static segments before `:params`, `*`
last), so declaration order can never shadow a route. A project with no `app`
declaration gets a bare router.

`Router(transition: .fade, duration: "200ms")` animates page changes
(chapter 13).

## Links and navigation

```wf
page Nav(path: "/nav") {
    Link("Home", to: "/")                       // an <a>; the router handles the click
    Link("Guide", to: "/docs").prefix           // also active on /docs/anything
    Button("Go") { on click { navigate("/about") } }
}
```

The link whose `to:` matches the current route carries the class `.active`
and `aria-current="page"`; `.prefix` makes a section link active for the
routes beneath it. `navigate(path)` moves in code. In a static build, links
are real page loads and `navigate` is too; in a single-page app they swap
pages without a reload, move focus to the new page's heading, reset the
scroll and announce the title.

## Route parameters

A `:name` segment is declared as a typed parameter of the page and read as a
plain name, or as `params.name`:

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

A parameter is a `String` unless you say otherwise (`id: Number`). A static
build renders a `:param` page once per value named in `paths:` — see
[Data](14-data.md#static-paths).

## Layouts

A `layout:` is a component with a default slot; the page renders in it. Use
it for a section's own chrome — a docs sidebar, an app rail — where the
`app` shell is the site's.

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

The layout's arguments are checked like any component call. The layout
component is required to declare a `slot` (its default slot); a layout
without one is an error, since the page would have nowhere to go.

## Guards

`guard:` names an expression — usually a store's derived value — that must be
true for the route to render; otherwise the reader is sent to `redirect:`.

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
    Button("Sign in").primary { on click { Auth.login()  navigate("/account") } }
}
```

The guard is evaluated on every navigation, in the browser. A static build
still pre-renders the page's HTML (the guard is a client-side gate, not a
secret), so keep anything private behind an API the page fetches.

## Per-page head tags

A page may add tags of its own to the document's `<head>`; they are painted
at build time where the value is known, kept current on the live page, and
gone when the page is:

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

`title` and `description` are page attributes, not head tags; the compiler
writes the standard tags (charset, viewport, description, canonical, Open
Graph, Twitter card, JSON-LD) from them and the config.

## The catch-all page

`path: "*"` is the page a static host serves as `404.html` and the router
shows for a route no page claims:

```wf
page NotFound(path: "*", title: "Not found", description: "Nothing lives here.", noindex: true) {
    Container {
        Heading("Nothing here").h1
        Link("Back home", to: "/")
    }
}
```

## Where the code runs

A page's body is a render block: it declares state, derived values and
actions, and lists what to show. Everything at the top of the body runs once
when the page renders; a call at the top — `Store.load(id)` — is set-up code.

```wf
store Deploys {
    state rows = []
    action load(id: String) { rows = [{ id: id, ok: true }] }
}

page Deploy(path: "/deploys/:id", title: "Deploy", description: "One deploy.", id: String) {
    use Deploys
    Deploys.load(id)
    for row in Deploys.rows by row.id {
        Text("{row.id}: {row.ok}")
    }
}
```

## Next

[Elements](04-elements.md) — what goes inside a page.
