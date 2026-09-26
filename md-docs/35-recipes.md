# 35. Cookbook

<!--
route: cookbook
group: shipping
blurb: Three complete applications you can paste into a fresh project, and recipes for the things every site needs.
description: Three complete applications — todos, a static blog, a guarded dashboard — and recipes for search, pagination, forms, auth, uploads and more.
-->

Three complete applications, each a single file you can paste into
`src/App.wf` of a fresh `wf init`, then a set of recipes for things that
come up in every project. Every block here builds.

## App 1: Todos

A keyed list, a store with `persist`, filters in the URL, keyboard
shortcuts, and an empty state.

```wf
enum Filter { all, open, done }

type Todo { id: Number, title: String, done: Bool = false, createdAt: String }

store Todos {
    persist items: [Todo] = []
    persist nextId = 1
    derived open = items.filter(t => !t.done)
    derived done = items.filter(t => t.done)
    derived count = items.length

    action add(title: String) {
        let text = title.trim()
        if text == "" { return }
        items.push(Todo(id: nextId, title: text, createdAt: "{Date.now()}"))
        nextId = nextId + 1
    }
    action toggle(id: Number) {
        items = items.map(t => if t.id == id { Todo(id: t.id, title: t.title, done: !t.done, createdAt: t.createdAt) } else { t })
    }
    action rename(id: Number, title: String) {
        items = items.map(t => if t.id == id { Todo(id: t.id, title: title, done: t.done, createdAt: t.createdAt) } else { t })
    }
    action remove(id: Number) { items = items.filter(t => t.id != id) }
    action clearDone() { items = items.filter(t => !t.done) }
}

component TodoRow(_ todo: Todo) {
    use Todos
    event edit(id: Number)
    state editing = false
    state draft = todo.title

    Row(align: .center, gap: .sm, exit: .fadeOut) {
        style { padding: $sm 0; border-bottom: 1px solid $border; &:hover { background: $surface } }
        Checkbox(checked: todo.done, aria-label: "Done: {todo.title}") { on change { Todos.toggle(todo.id) } }
        if editing {
            Input(bind: draft, ref: box, aria-label: "Edit {todo.title}") {
                on key("Enter") { Todos.rename(todo.id, draft)  editing = false }
                on key("Escape") { draft = todo.title  editing = false }
                on blur { Todos.rename(todo.id, draft)  editing = false }
            }
        } else {
            Text(todo.title, class: if todo.done { "is-done" } else { "" }) {
                style { flex: 1; &.is-done { text-decoration: line-through; color: $text-muted } }
                on click { editing = true  emit edit(todo.id) }
            }
        }
        IconButton(icon: "trash", label: "Remove {todo.title}").sm { on click { Todos.remove(todo.id) } }
    }
}

app {
    Container {
        style { max-width: 40rem; padding: $lg }
        Router
    }
}

page Home(path: "/", title: "Todos", description: "A small list of things to do.") {
    use Todos
    state draft = ""
    state lastEdited: Number? = null
    derived filter: Filter = if query.filter == "open" { .open } else if query.filter == "done" { .done } else { .all }
    derived shown = match filter { .open { Todos.open } .done { Todos.done } else { Todos.items } }

    on key("/") { box.focus() }

    Heading("Todos").h1
    Row(gap: .sm) {
        Input(bind: draft, ref: box, placeholder: "What needs doing? (press / to focus)", aria-label: "New todo") {
            on key("Enter") { Todos.add(draft)  draft = "" }
        }
        Button("Add", disabled: draft.trim() == "").primary { on click { Todos.add(draft)  draft = "" } }
    }

    Row(gap: .sm, align: .center) {
        style { margin: $md 0 }
        Link("All", to: "/?filter=all", active: .prefix)
        Link("Open ({Todos.open.length})", to: "/?filter=open")
        Link("Done ({Todos.done.length})", to: "/?filter=done")
        Spacer
        if Todos.done.length > 0 { Button("Clear done").sm { on click { Todos.clearDone() } } }
    }

    if shown.length == 0 {
        Card(animate: .fadeIn) {
            Text(if Todos.count == 0 { "Nothing yet. Add your first todo above." } else { "Nothing here." }).muted
        }
    }
    for todo in shown by todo.id {
        TodoRow(todo, animate: .fadeIn) { on edit(id) { lastEdited = id } }
    }
    if let id = lastEdited { Text("Editing #{id}").muted.sm }
}
```

What to notice:

- `persist items` keeps the list across visits with no storage code.
- `for … by todo.id` keeps each row's `editing` state and its input while
  the list is filtered or reordered.
- The filter lives in the URL (`?filter=open`), read through `query`, so a
  reload and a shared link keep it.
- `on key("/")` at page level and `on key("Enter")`/`("Escape")` on the
  input are the whole keyboard story.
- `TodoRow` declares `event edit` and the page listens with `on edit(id)`.

## App 2: A blog, statically built

A `data` file, one `.md`-free blog written entirely in WebFluent with
static paths, a layout, SEO tags, and a dark theme.

```wf
theme Paper {
    color-primary: #7C3AED
    color-background: #FFFFFF
    color-surface: #FAF5FF
    color-text: #1F1235
    color-text-muted: #5B5470
    font-family: 'Georgia', serif
    radius-md: 4px
}

theme Ink {
    color-background: #14101C
    color-surface: #1D1728
    color-text: #F3EEFF
    color-text-muted: #B9AECF
    color-border: #2E2540
}

type Post {
    slug: String
    title: String
    summary: String
    date: String
    tags: [String]
    body: String
}

data posts: [Post] = "posts.json"

component Shell(_ crumb: String) {
    slot
    Navbar {
        Navbar.Brand { Link("Paper & Ink", to: "/") }
        Navbar.Links {
            Link("Posts", to: "/", active: .prefix)
            Link("Tags", to: "/tags")
            Link("About", to: "/about")
        }
        Navbar.Actions {
            Button(if theme == "dark" { "Light" } else { "Dark" }).sm {
                on click { setTheme(if theme == "dark" { "light" } else { "dark" }) }
            }
        }
    }
    Container {
        style { max-width: 44rem; padding: $lg $md }
        Breadcrumb {
            Breadcrumb.Item(to: "/") { Text("Home") }
            Breadcrumb.Item { Text(crumb) }
        }
        children
    }
}

component PostCard(_ post: Post) {
    Card {
        style { margin-bottom: $md }
        Link(post.title, to: "/posts/{post.slug}") { style { font-size: $xl; font-weight: 700 } }
        Text(format(post.date, .date, "long")).muted.sm
        Text(post.summary)
        Row(gap: .xs) {
            for tag in post.tags by tag { Tag(tag) }
        }
    }
}

app { Router(transition: .fade) }

page Home(path: "/", title: "Paper & Ink", description: "Notes on making things.", layout: Shell(crumb: "Posts")) {
    derived sorted = posts.sortBy(p => p.date).reverse()
    Heading("Posts").h1
    for p in sorted by p.slug { PostCard(p) }
}

page PostPage(path: "/posts/:slug", slug: String, title: "Paper & Ink", description: "A post.", type: "article", layout: Shell(crumb: "Post"), paths: posts.map(p => p.slug)) {
    derived post = posts.find(p => p.slug == slug)
    derived related = posts.filter(p => p.slug != slug && p.tags.some(t => post?.tags.includes(t) ?? false)).take(3)
    head {
        meta(property: "og:title", content: post?.title ?? "Post")
        meta(property: "og:description", content: post?.summary ?? "")
        meta(property: "article:published_time", content: post?.date ?? "")
    }
    if let p = post {
        Heading(p.title).h1
        Text(format(p.date, .date, "long")).muted
        Markdown(p.body)
        Divider
        if related.length > 0 {
            Heading("Related").h2
            for r in related by r.slug { Link(r.title, to: "/posts/{r.slug}") }
        }
    } else {
        Alert("No such post.").warning
        Link("Back to posts", to: "/")
    }
}

page Tags(path: "/tags", title: "Tags", description: "Every tag on the blog.", layout: Shell(crumb: "Tags")) {
    derived tags = posts.flatMap(p => p.tags).unique().sort()
    Heading("Tags").h1
    for tag in tags by tag {
        derived count = posts.filter(p => p.tags.includes(tag)).length
        Row(gap: .sm, align: .center) {
            Link(tag, to: "/tags/{tag}")
            Badge("{count}").info
        }
    }
}

page TagPage(path: "/tags/:tag", tag: String, title: "Tag", description: "Posts under a tag.", layout: Shell(crumb: "Tag"), paths: posts.flatMap(p => p.tags).unique()) {
    derived hits = posts.filter(p => p.tags.includes(tag))
    Heading("Tagged {tag}").h1
    for p in hits by p.slug { PostCard(p) }
}

page About(path: "/about", title: "About", description: "Who writes here.", layout: Shell(crumb: "About")) {
    Heading("About").h1
    Markdown("Written with **WebFluent**. Every page here is a static file, hydrated in place.")
}

page NotFound(path: "*", title: "Not found", description: "Nothing lives here.", noindex: true, layout: Shell(crumb: "404")) {
    Heading("Not found").h1
    Link("Back home", to: "/")
}
```

`posts.json` beside it:

```json
[
  { "slug": "hello", "title": "Hello, world", "summary": "The first post.", "date": "2026-01-10", "tags": ["meta"], "body": "# Hello\n\nThis is the **first** post." },
  { "slug": "static", "title": "Why static", "summary": "Files are fast.", "date": "2026-02-02", "tags": ["meta", "web"], "body": "Static files are cached everywhere." }
]
```

And the config: `"theme": { "name": "Paper", "dark": "Ink" }`,
`"build": { "ssg": true }`, `"meta": { "site_url": "https://example.com" }`.

What to notice:

- `paths:` over the `data` file renders one HTML file per post and per
  tag; `head { }` gives each its own sharing tags.
- The `Shell` layout takes an argument (`crumb:`) like any component.
- `derived count` inside a `for` body is per item.
- `setTheme` and a `dark:` theme are the whole dark-mode feature.

## App 3: A dashboard behind a login

Guarded routes, a session store, resources with reload, a form with a
handle, a modal, a table with sorting, and a command palette.

```wf
type Deploy { id: String, service: String, status: String, startedAt: String, by: String }
type Service { name: String, healthy: Bool, region: String }

enum Sort { newest, service, status }

store Session {
    persist user: Map? = null
    state error = ""
    derived loggedIn = user != null
    action login(email: String, password: String) {
        error = ""
        try {
            let r = await fetch("/api/login", { method: "POST", body: { email: email, password: password } })
            user = r.user
            navigate("/")
        } catch e {
            error = "Sign-in failed: {e.message}"
        }
    }
    action logout() { user = null  navigate("/login") }
}

store Ui {
    state palette = false
    state confirmId: String? = null
    action openPalette() { palette = true }
    action close() { palette = false  confirmId = null }
}

component Shell {
    use Session
    use Ui
    slot
    on key("cmd+k") { Ui.openPalette() }
    on key("ctrl+k") { Ui.openPalette() }
    on key("Escape") { Ui.close() }
    Row {
        style { min-height: 100vh }
        Sidebar {
            style { width: 220px }
            Sidebar.Header { Text("Ops").bold }
            Sidebar.Item(to: "/", icon: "home") { Text("Overview") }
            Sidebar.Item(to: "/deploys", icon: "upload") { Text("Deploys") }
            Sidebar.Item(to: "/settings", icon: "settings") { Text("Settings") }
            Sidebar.Divider
            if let u = Session.user {
                Text(u.email).muted.sm
                Button("Sign out").sm { on click { Session.logout() } }
            }
        }
        Container {
            style { flex: 1; padding: $lg }
            children
        }
    }
    Palette
}

component Palette {
    use Ui
    state q = ""
    derived commands = [
        { label: "Go to overview", to: "/" },
        { label: "Go to deploys", to: "/deploys" },
        { label: "Go to settings", to: "/settings" },
    ]
    derived hits = commands.filter(c => c.label.toLowerCase().includes(q.toLowerCase()))
    Modal(visible: Ui.palette, title: "Command palette") {
        Input(bind: q, placeholder: "Type a command", aria-label: "Command")
        for c, i in hits {
            Button(c.label).sm { on click { navigate(c.to)  Ui.close() } }
        }
        if hits.length == 0 { Text("No matches").muted }
    }
}

app { Router }

page Login(path: "/login", title: "Sign in", description: "Sign in to the console.") {
    use Session
    state email = ""
    state password = ""
    Container {
        style { max-width: 24rem; margin: 4rem auto }
        Heading("Sign in").h1
        Form(bind: form) {
            on submit { Session.login(email, password) }
            Input(bind: email, name: "email", label: "Email").email.required
            Input(bind: password, name: "password", label: "Password").password.required
            Button("Sign in", type: .submit, disabled: !form.valid || Session.login.pending).primary
        }
        if Session.error != "" { Alert(Session.error).danger }
    }
}

page Overview(path: "/", title: "Overview", description: "Service health.", guard: Session.loggedIn, redirect: "/login", layout: Shell) {
    resource services: [Service] = fetch("/api/services")
    every(30000) { services.reload() }
    Heading("Overview").h1
    match services {
        loading { Skeleton }
        error(e) { Alert("Could not load services: {e.message}").danger { Button("Retry") { on click { services.reload() } } } }
        ready(list) {
            derived healthy = list.filter(s => s.healthy).length
            Row(gap: .md) {
                Card { Text("Services").muted.sm  Heading("{list.length}").h2 }
                Card { Text("Healthy").muted.sm  Heading("{healthy}").h2 }
                Card { Text("Regions").muted.sm  Heading("{list.map(s => s.region).unique().length}").h2 }
            }
            Grid(columns: if viewport.lg { 3 } else { 1 }, gap: .md) {
                for s in list by s.name {
                    Card {
                        Row(justify: .between, align: .center) {
                            Text(s.name).bold
                            Badge(if s.healthy { "healthy" } else { "down" }, tone: if s.healthy { .success } else { .danger })
                        }
                        Text(s.region).muted.sm
                    }
                }
            }
        }
    }
}

page Deploys(path: "/deploys", title: "Deploys", description: "Recent deploys.", guard: Session.loggedIn, redirect: "/login", layout: Shell) {
    use Ui
    state sort: Sort = .newest
    state q = ""
    state rolledBack = ""
    resource deploys: [Deploy] = fetch("/api/deploys")
    action rollback(id: String) {
        await fetch("/api/deploys/{id}/rollback", { method: "POST" })
        rolledBack = id
        Ui.close()
        deploys.reload()
    }
    Row(justify: .between, align: .center) {
        Heading("Deploys").h1
        Row(gap: .sm) {
            Input(bind: q, placeholder: "Filter by service", aria-label: "Filter")
            Select(bind: sort, label: "Sort") {
                Select.Option("Newest", value: "newest")
                Select.Option("Service", value: "service")
                Select.Option("Status", value: "status")
            }
        }
    }
    match deploys {
        loading { Spinner }
        error(e) { Alert(e.message).danger }
        ready(rows) {
            derived shown = rows.filter(d => d.service.toLowerCase().includes(q.toLowerCase()))
            derived sorted = match sort { .service { shown.sortBy(d => d.service) } .status { shown.sortBy(d => d.status) } else { shown.sortBy(d => d.startedAt).reverse() } }
            Table(caption: "Recent deploys") {
                Table.Head {
                    Table.Row { Table.Cell("Service")  Table.Cell("Status")  Table.Cell("When")  Table.Cell("By")  Table.Cell("") }
                }
                Table.Body {
                    for d in sorted by d.id {
                        Table.Row {
                            Table.Cell(d.service)
                            Table.Cell { Badge(d.status, tone: if d.status == "ok" { .success } else if d.status == "failed" { .danger } else { .info }) }
                            Table.Cell(ago(d.startedAt))
                            Table.Cell(d.by)
                            Table.Cell {
                                Menu(trigger: "Actions", aria-label: "Actions for {d.service}") {
                                    Menu.Item { Link("Details", to: "/deploys/{d.id}") }
                                    Menu.Divider
                                    Menu.Item { Button("Roll back").sm.danger { on click { Ui.confirmId = d.id } } }
                                }
                            }
                        }
                    }
                }
            }
            if sorted.length == 0 { Text("No deploys match.").muted }
        }
    }
    Modal(visible: Ui.confirmId != null, title: "Roll back this deploy?") {
        Text("The previous build will be restored.")
        Modal.Footer {
            Button("Cancel") { on click { Ui.close() } }
            Button("Roll back", disabled: rollback.pending).danger { on click { if let id = Ui.confirmId { rollback(id) } } }
        }
    }
    if rolledBack != "" { Toast("Rolled back {rolledBack}").success }
}

page DeployDetail(path: "/deploys/:id", id: String, title: "Deploy", description: "One deploy.", guard: Session.loggedIn, redirect: "/login", layout: Shell) {
    resource deploy: Deploy = fetch("/api/deploys/{id}")
    Breadcrumb {
        Breadcrumb.Item(to: "/deploys") { Text("Deploys") }
        Breadcrumb.Item { Text(id) }
    }
    match deploy {
        loading { Skeleton }
        error(e) { Alert("Not found").warning }
        ready(d) {
            Heading("{d.service} — {d.status}").h1
            Text("Started {ago(d.startedAt)} by {d.by}")
        }
    }
}

page Settings(path: "/settings", title: "Settings", description: "Your preferences.", guard: Session.loggedIn, redirect: "/login", layout: Shell) {
    persist density = "comfortable"
    persist notify = true
    Heading("Settings").h1
    Radio(bind: density, value: "comfortable", label: "Comfortable")
    Radio(bind: density, value: "compact", label: "Compact")
    Switch(bind: notify, label: "Email me when a deploy fails")
    Row(gap: .sm) {
        Button("Light") { on click { setTheme("light") } }
        Button("Dark") { on click { setTheme("dark") } }
        Button("System") { on click { setTheme("system") } }
    }
}
```

What to notice:

- `guard: Session.loggedIn, redirect: "/login"` on each private page; the
  session itself is a `persist` in a store, so a reload stays signed in.
- `Shell` is both a layout and the place the global shortcuts live — its
  `on key` handlers exist while any page inside it shows.
- `every(30000) { services.reload() }` polls, and stops when the page
  leaves.
- The confirm modal's state is in `Ui`, so any page can open it; `Ui.close()`
  on Escape covers the palette and the modal.
- `Table.Cell { … }` holds elements; `Table.Cell(text)` holds text.

## Recipes

### Debounced search

```wf
page Search(path: "/") {
    state typed = ""
    state q = ""
    action commit(value: String) { if value == typed { q = value } }
    effect {
        let pending = typed
        setTimeout(() => commit(pending), 300)
    }
    resource hits = fetch("/api/search?q={q}")
    Input(bind: typed, placeholder: "Search", aria-label: "Search")
    match hits {
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} results for {q}") }
    }
}
```

The effect re-runs on every keystroke; each run schedules a check, and only
the one whose value is still current commits to `q`, which the resource
follows. (`after(ms)` is a declaration of a body, not a statement, so a
timer inside an effect is the browser's `setTimeout`.)

### Pagination

```wf
page Rows(path: "/") {
    state page = 1
    state perPage = 20
    resource rows = fetch("/api/rows?page={page}&per={perPage}")
    match rows {
        loading { Spinner }
        error(e) { Text(e.message).danger }
        ready(r) {
            for row in r.items by row.id { Text(row.name) }
            Row(gap: .sm, align: .center) {
                Button("Previous", disabled: page == 1) { on click { page = page - 1 } }
                Text("Page {page} of {r.pages}")
                Button("Next", disabled: page >= r.pages) { on click { page = page + 1 } }
            }
        }
    }
}
```

### Optimistic update with rollback

```wf
type Item { id: String, liked: Bool }

page Likes(path: "/") {
    state items: [Item] = [Item(id: "a", liked: false)]
    state error = ""
    action like(id: String) {
        let before = items
        items = items.map(i => if i.id == id { Item(id: i.id, liked: !i.liked) } else { i })
        try {
            await fetch("/api/like/{id}", { method: "POST" })
        } catch e {
            items = before
            error = "Could not save. Try again."
        }
    }
    for i in items by i.id {
        Button(if i.liked { "♥" } else { "♡" }, aria-label: "Like").sm { on click { like(i.id) } }
    }
    if error != "" { Toast(error).danger }
}
```

### A confirm dialog component

```wf
component Confirm(_ title: String, visible: Bool, danger: Bool = false) {
    event confirm()
    event cancel()
    slot
    Dialog(visible: visible, title: title) {
        children
        Row(gap: .sm, justify: .end) {
            Button("Cancel") { on click { emit cancel() } }
            Button("Confirm", tone: if danger { .danger } else { .primary }) { on click { emit confirm() } }
        }
    }
}

page Use(path: "/") {
    state asking = false
    state gone = false
    Button("Delete account").danger { on click { asking = true } }
    Confirm("Delete your account?", visible: asking, danger: true) {
        on confirm { gone = true  asking = false }
        on cancel { asking = false }
        Text("Everything goes. This cannot be undone.")
    }
    if gone { Alert("Account deleted.").info }
}
```

### Responsive navigation

```wf
page Nav(path: "/") {
    state open = false
    if viewport.md {
        Row(gap: .md) {
            Link("Home", to: "/")
            Link("Docs", to: "/docs")
            Link("Pricing", to: "/pricing")
        }
    } else {
        IconButton(icon: "menu", label: "Open navigation") { on click { open = !open } }
        show open {
            Stack(gap: .sm, animate: .slideDown) {
                Link("Home", to: "/")
                Link("Docs", to: "/docs")
                Link("Pricing", to: "/pricing")
            }
        }
    }
}
```

### A form with validation messages

```wf
page Signup(path: "/") {
    state email = ""
    state password = ""
    state confirm = ""
    state sent = false
    validate email { required  email "Enter a valid email" }
    validate password { required  minLength(8) "At least 8 characters" }
    validate confirm { matches(password) "The passwords differ" }
    Form(bind: form) {
        on submit { sent = true  form.reset() }
        Input(bind: email, name: "email", label: "Email").email
        Input(bind: password, name: "password", label: "Password").password
        Input(bind: confirm, name: "confirm", label: "Confirm password").password
        Button("Create account", type: .submit, disabled: !form.valid).primary
    }
    if sent { Alert("Welcome aboard.").success }
}
```

Each control shows its own message when the reader leaves it, and a submit
that fails focuses the first problem. [Forms](16-forms.md) has every rule.

### Tabs with the active tab in the URL

```wf
page Account(path: "/account") {
    derived tab = if hash != "" { hash } else { "profile" }
    Row(gap: .sm) {
        Link("Profile", to: "/account#profile")
        Link("Billing", to: "/account#billing")
    }
    if tab == "profile" { Text("Profile settings") }
    if tab == "billing" { Text("Billing settings") }
}
```

### Reading a store from a component without `use`

A component that only needs one value takes it as a prop, which keeps it
reusable outside that store:

```wf
store Cart { state count = 0 }

component CartBadge(count: Number) {
    if count > 0 { Badge("{count}").primary }
}

page Shop(path: "/") {
    use Cart
    CartBadge(count: Cart.count)
}
```

### Countdown

```wf
page Countdown(path: "/") {
    state left = 10
    every(1000) { if left > 0 { left = left - 1 } }
    Progress(value: 10 - left, max: 10)
    Text(if left == 0 { "Done" } else { "{left}s" })
}
```

### Signing in with a session cookie

```wf
type User { id: String, name: String }

api Auth(base: "/api") {
    credentials: .sameOrigin
    get me() -> User
    post signIn(body: Map) -> User
        errors { 401 -> Map }
    post signOut()
}

store Session {
    state user: User? = null
    state checked = false
    derived signedIn = user != null
    action load() {
        try { user = await Auth.me() } catch e { user = null }
        checked = true
    }
    action signIn(email: String, password: String) {
        user = await Auth.signIn(body: { email: email, password: password })
        navigate("/account")
    }
    action signOut() {
        await Auth.signOut()
        user = null
        navigate("/")
    }
}

page SignIn(path: "/sign-in", title: "Sign in", description: "Sign in to your account.", noindex: true) {
    state email = ""
    state password = ""
    state problem = ""
    validate email { required  email }
    validate password { required }
    action send() {
        problem = ""
        try { await Session.signIn(email, password) } catch e { problem = "That email and password do not match." }
    }
    Heading("Sign in").h1
    if problem != "" { Alert(problem).danger }
    Form(bind: form) {
        on submit { send() }
        Input(bind: email, label: "Email").email
        Input(bind: password, label: "Password").password
        Button("Sign in", type: .submit, disabled: send.pending).primary
    }
}

page Account(path: "/account", title: "Your account", description: "Your account.", guard: Session.signedIn, redirect: "/sign-in") {
    use Session
    Heading("Your account").h1
    if let u = Session.user { Text("Signed in as {u.name}") }
    Button("Sign out") { on click { Session.signOut() } }
}
```

The session is an httpOnly cookie the server sets; the page never holds a
token. `guard:` hides the page from the signed-out, and the server still
checks every request ([Security](23-security.md#sessions-and-tokens)).
Call `Session.load()` from the app's set-up so a returning reader is signed
in.

### Load more

```wf
type Item { id: String, name: String }

api Catalog(base: "/api") {
    get items(page: Number = 1) -> [Item]
}

page Items(path: "/items", title: "Items", description: "Everything, a page at a time.") {
    state page = 1
    resource list = Catalog.items(page: page, paginate: .page)
    Heading("Items").h1
    for item in list.items by item.id { Text(item.name) }
    if list.hasMore {
        Button("Load more", disabled: list.state == "loading") { on click { list.loadMore() } }
    }
}
```

### Filters in the URL

```wf
page Products(path: "/products", title: "Products", description: "Everything we sell.") {
    derived sort = query.sort ?? "name"
    Heading("Products").h1
    Row(gap: .sm) {
        Link("By name", to: "/products?sort=name")
        Link("By price", to: "/products?sort=price")
    }
    Text("Sorted by {sort}")
}
```

A filter in the query string survives a reload, a shared link and the back
button, where a `state` would not.

### A dark-mode switch

```wf
page Settings(path: "/settings", title: "Settings", description: "How the site looks.") {
    Heading("Settings").h1
    Row(gap: .sm) {
        Button("Light", outlined: theme != "light") { on click { setTheme("light") } }
        Button("Dark", outlined: theme != "dark") { on click { setTheme("dark") } }
        Button("Match the system", outlined: theme != "system") { on click { setTheme("system") } }
    }
}
```

It needs a dark theme named in the config ([Styling](15-styling.md#dark-mode)).

### Copy to the clipboard

```wf
page Share(path: "/share", title: "Share", description: "Copy the link.") {
    state copied = false
    action copy(text: String) {
        await navigator.clipboard.writeText(text)
        copied = true
    }
    Heading("Share").h1
    Button("Copy the link") { on click { copy(location.href) } }
    if copied { Toast("Copied").success }
}
```

### A chart

A charting library goes through `external` and `Host`; the full example is in
[JavaScript interop](32-javascript-interop.md#a-library-end-to-end).

## Where to go next

- The [components reference](36-components-reference.md) for every prop
  and flag.
- The `tests/fixtures/` folder of the repository: a gallery, a marketing
  site, a dashboard, a docs site, an invoice, a deck, an offline site and a
  bespoke design, each a complete project.
- `wf docs` in your own project, for what *your* components take.

## Next

[Built-in components — reference](36-components-reference.md).
