# 8. Recipes

<!--
route: guide/recipes
group: tutorials
blurb: Small, complete pages for the jobs most projects meet: debounced search, paging, optimistic updates, a confirm dialog, dark mode and more.
description: WebFluent recipes: debounced search, pagination, optimistic updates, confirm dialogs, sessions, load more, dark mode, clipboard and charts.
-->

These are smaller pieces that most projects need. Each is a whole page you
can build and adapt.

## Debounce a search

To wait until the reader stops typing before you search, keep what they
typed apart from what you search for:

```wf
page Search(path: "/", title: "Search", description: "Search as you type, without a request per key.") {
    state typed = ""
    state q = ""
    action commit(value: String) { if value == typed { q = value } }
    effect {
        let pending = typed
        setTimeout(() => commit(pending), 300)
    }
    resource hits = fetch("/api/search?q={q}")
    Heading("Search").h1
    Input(bind: typed, placeholder: "Search", aria-label: "Search")
    match hits {
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} results for {q}") }
    }
}
```

The effect runs again on every keystroke, and each run schedules a check.
Only the check whose value is still current writes `q`, and the resource
follows `q`. The timer is the browser's `setTimeout`, because `after(ms)`
is a declaration with a body, not a statement an effect can hold.

## Page through results

To show a long list a page at a time, keep the page number in state and
splice it into the address:

```wf
page Rows(path: "/", title: "Rows", description: "Every row, twenty at a time.") {
    state page = 1
    state perPage = 20
    resource rows = fetch("/api/rows?page={page}&per={perPage}")
    Heading("Rows").h1
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

A change of `page` changes the address, so the resource asks again.

## Update at once, and roll back on failure

To show a change before the server agrees, keep the old value and put it
back if the request fails:

```wf
type Item { id: String, liked: Bool }

page Likes(path: "/", title: "Likes", description: "A like that shows at once.") {
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
    Heading("Likes").h1
    for i in items by i.id {
        Button(if i.liked { "♥" } else { "♡" }, aria-label: "Like").sm { on click { like(i.id) } }
    }
    if error != "" { Toast(error).danger }
}
```

`optimistic(holder, change)` does the same with less code inside an
action; see [Fetching data](24-data.md#show-a-change-before-the-server-agrees).

## Build a confirm dialog component

To ask before a destructive action, wrap a `Dialog` in a component that
fires `confirm` or `cancel`:

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

page Account(path: "/", title: "Account", description: "Close your account.") {
    state asking = false
    state gone = false
    Heading("Account").h1
    Button("Delete account").danger { on click { asking = true } }
    Confirm("Delete your account?", visible: asking, danger: true) {
        on confirm { gone = true  asking = false }
        on cancel { asking = false }
        Text("Everything goes. This cannot be undone.")
    }
    if gone { Alert("Account deleted.").info }
}
```

The caller's block fills the default slot, so the question's detail is the
caller's to write.

## Make navigation responsive

To show links on a wide screen and a menu button on a narrow one, branch on
`viewport`:

```wf
page Nav(path: "/", title: "Home", description: "Navigation that fits the screen.") {
    state open = false
    Heading("Home").h1
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

## Show validation messages on a form

To check a form as the reader fills it in, put a `validate` block beside
each state:

```wf
page Signup(path: "/", title: "Sign up", description: "Make an account.") {
    state email = ""
    state password = ""
    state confirm = ""
    state sent = false
    validate email { required  email "Enter a valid email" }
    validate password { required  minLength(8) "At least 8 characters" }
    validate confirm { matches(password) "The passwords differ" }
    Heading("Sign up").h1
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
that fails moves focus to the first problem. [Forms](23-forms.md) lists
every rule.

## Keep the active tab in the address

To make a tab survive a reload, read it from `hash`:

```wf
page Account(path: "/account", title: "Account", description: "Your profile and billing.") {
    derived tab = if hash != "" { hash } else { "profile" }
    Heading("Account").h1
    Row(gap: .sm) {
        Link("Profile", to: "/account#profile")
        Link("Billing", to: "/account#billing")
    }
    if tab == "profile" { Text("Profile settings") }
    if tab == "billing" { Text("Billing settings") }
}
```

## Pass a store's value to a component as a prop

A component that needs only one value from a store can take it as a prop
instead of using the store. That keeps the component reusable outside that
store:

```wf
store Cart { state count = 0 }

component CartBadge(count: Number) {
    if count > 0 { Badge("{count}").primary }
}

page Shop(path: "/", title: "Shop", description: "Everything we sell.") {
    use Cart
    Heading("Shop").h1
    CartBadge(count: Cart.count)
}
```

## Count down

To count down once a second, change a state in an `every` timer:

```wf
page Countdown(path: "/", title: "Countdown", description: "Ten seconds, counted down.") {
    state left = 10
    every(1000) { if left > 0 { left = left - 1 } }
    Heading("Countdown").h1
    Progress(value: 10 - left, max: 10)
    Text(if left == 0 { "Done" } else { "{left}s" })
}
```

The timer stops when the page leaves.

## Sign in with a session cookie

To sign a reader in without holding a token in the page, describe the
service with `credentials: .sameOrigin` and keep the user in a store:

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

The session is an httpOnly cookie the server sets, so the page never holds
a token. `guard:` hides the page from a reader who is signed out, and the
server still checks every request
([Security](30-security.md#keep-the-session-in-a-cookie)). Call `Session.load()`
from the app's set-up, so a returning reader is signed in.

## Load more

To add the next page of results below the ones shown, call an endpoint with
`paginate: .page`:

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

`list.items` is every item loaded so far, and `list.loadMore()` asks for the
next page.

## Keep filters in the address

To make a filter survive a reload, a shared link and the back button, read
it from `query` instead of holding it in a `state`:

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

## Add a dark-mode switch

To let the reader pick light, dark or the system's choice, call `setTheme`
and read `theme`:

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

The switch needs a dark theme named in the config
([Styling](19-styling.md#dark-mode)).

## Copy to the clipboard

To copy text, call the browser's clipboard from an action:

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

## Draw a chart from a library

To use a charting library, list its URL in `meta.scripts`, call it from a
script of your own, and hand it a canvas with `Host`. The full example is
in [JavaScript interop](39-javascript-interop.md#load-a-library-from-a-cdn).

## Where to go next

- The [components reference](40-components-reference.md), for every prop
  and flag.
- `examples/` in the repository: the tutorials, reference PDFs and slide
  decks, and the `tests/fixtures/` projects — a gallery, a dashboard, a
  docs site, an offline site — each complete.
- `wf docs` in your own project, for what *your* components take.

## Next

[Language basics](09-language-basics.md) starts the concepts: how the language itself works.
