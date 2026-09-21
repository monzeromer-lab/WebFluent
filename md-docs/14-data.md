# 14. Data

Three ways to get data onto a page: `resource` for something fetched while
the page shows, `await fetch` inside an action for something fetched on
demand, and `data` for a file read at build time. Plus `const` and `env`
for values that are the same for the whole build.

## `resource`

A resource is a request the page makes and a `match` shows:

```wf
type Deploy { id: String, status: String, startedAt: String }

page Deploys(path: "/", title: "Deploys", description: "Recent deploys.") {
    resource deploys: [Deploy] = fetch("/api/deploys")
    match deploys {
        loading { Spinner }
        error(e) { Alert("Could not load: {e.message}").danger }
        ready(rows) {
            if rows.length == 0 { Text("No deploys yet.").muted }
            for d in rows by d.id {
                Row(justify: .between) {
                    Text(d.id).bold
                    Badge(d.status, tone: if d.status == "ok" { .success } else { .danger })
                    Text(ago(d.startedAt)).muted.sm
                }
            }
        }
    }
    Button("Refresh") { on click { deploys.reload() } }
}
```

- `resource name = fetch(url)` starts the request when the page renders.
  The annotation (`: [Deploy]`) types the value; without it the value is
  `Any`.
- `match` over a resource has three arms — `loading`, `error(e)`, `ready(v)`
  — and `else`. The static paint renders the `loading` arm.
- `name.reload()` makes the request again; `name.state` is `"loading"`,
  `"ready"` or `"error"`; `name.data` the last value (`T?`), `name.error`
  the last error.
- A failed response (`!ok`) is an error with `e.message` of `HTTP 500`.

### Options

```wf
page Search(path: "/") {
    state q = ""
    state token = "abc"
    resource hits = fetch("/api/search?q={q}", headers: { Authorization: "Bearer {token}" })
    resource created = fetch("/api/items", method: "POST", body: { name: q })
    Input(bind: q, label: "Search")
    match hits {
        loading { Spinner.sm }
        error(e) { Text(e.message).danger }
        ready(list) { Text("{list.length} hits") }
    }
}
```

Options are named after the URL: `method:`, `headers:` (a map), `body:` (a
map, sent as JSON with the content type set). The body of a response is
parsed as JSON.

### A reactive URL

A URL that reads state is re-fetched whenever that state changes — a
search box that fetches as you type, or a detail page that follows its
parameter:

```wf
type Post { slug: String, title: String, body: String }

page PostPage(path: "/posts/:slug", slug: String) {
    resource post: Post = fetch("/api/posts/{slug}")
    match post {
        loading { Skeleton }
        error(e) { Alert("Not found").warning }
        ready(p) {
            Heading(p.title).h1
            Markdown(p.body)
        }
    }
}
```

Debounce a fast-changing URL by deriving it from a slower value, or by
keeping the query in state and copying it to a `submitted` state on Enter.

## `await fetch` in an action

For a request made on demand — a save, a delete, a step in a flow —
`fetch` returns the parsed body and throws on a failed response:

```wf
type Todo { id: String, title: String, done: Bool = false }

page Todos(path: "/") {
    state todos: [Todo] = []
    state draft = ""
    state error = ""

    action load() {
        let r: [Todo] = await fetch("/api/todos")
        todos = r
    }

    action add() {
        if draft.trim() == "" { return }
        try {
            let created: Todo = await fetch("/api/todos", { method: "POST", body: { title: draft } })
            todos.push(created)
            draft = ""
        } catch e {
            error = "Could not save: {e.message}"
        }
    }

    action toggle(id: String) {
        let t = todos.find(x => x.id == id)
        if let todo = t {
            let updated: Todo = await fetch("/api/todos/{id}", { method: "PATCH", body: { done: !todo.done } })
            todos = todos.map(x => if x.id == id { updated } else { x })
        }
    }

    load()
    Input(bind: draft, label: "New todo") { on key("Enter") { add() } }
    Button("Add", disabled: add.pending).primary { on click { add() } }
    if error != "" { Alert(error).danger }
    for t in todos by t.id {
        Checkbox(checked: t.done, label: t.title) { on change { toggle(t.id) } }
    }
}
```

The second argument is a map: `method`, `headers`, `body`. `name.pending`
is true while the action runs. The browser's own `fetch`, for a response
you want raw, is `window.fetch`.

## `data`: a file at build time

`data name = "file.json"` reads a JSON file (from the project root or
`src/`) at build time and makes its value a constant. It is in every page
and component, costs nothing at run time, and is painted into the static
build:

```wf
type Post { slug: String, title: String, date: String, body: String }

data posts: [Post] = "posts.json"

page Blog(path: "/", title: "Blog", description: "Every post.") {
    derived sorted = posts.sortBy(p => p.date).reverse()
    for p in sorted by p.slug {
        Card {
            Link(p.title, to: "/posts/{p.slug}")
            Text(format(p.date, .date)).muted.sm
        }
    }
}
```

```json
[
  { "slug": "hello", "title": "Hello", "date": "2026-01-10", "body": "# Hi\nFirst post." },
  { "slug": "second", "title": "Second", "date": "2026-02-02", "body": "More." }
]
```

Annotate it (`data posts: [Post]`) and the checker knows every field.

### Static paths

A `:param` page is rendered once per value in `paths:` by a static build —
one HTML file per post, each with its own title and description:

```wf
type Post { slug: String, title: String, date: String, body: String }

data posts: [Post] = "posts.json"

page PostPage(path: "/posts/:slug", slug: String, title: "Post", description: "A post.", paths: posts.map(p => p.slug)) {
    derived post = posts.find(p => p.slug == slug)
    head { meta(property: "og:title", content: post?.title ?? "Post") }
    if let p = post {
        Heading(p.title).h1
        Markdown(p.body)
    } else {
        Alert("No such post").warning
    }
}
```

`paths:` must be knowable at build time — a `data` file, a `const`, a
literal. For a route with several parameters, each value is a map naming
them: `paths: pairs.map(x => { year: x.y, slug: x.s })`. Routes not in
`paths:` still work in the SPA; only the static build needs the list.

## `const` and `env`

```wf
const PAGE_SIZE = 25
const API = env.API_URL ?? "/api"
const FEATURES = { billing: env.BILLING == "on", beta: false }

page Rows(path: "/") {
    resource rows = fetch("{API}/rows?limit={PAGE_SIZE}")
    if FEATURES.billing { Link("Billing", to: "/billing") }
    match rows {
        loading { Spinner }
        error(e) { Text(e.message).danger }
        ready(r) { Text("{r.length} rows") }
    }
}
```

`env.NAME` reads the build environment — the shell's variables and a
`.env` file in the project root — and is `null` when unset. Everything
here is inlined into the bundle, so a secret does not belong in `env`
reads; only what a browser may see.

## Where data lands in the static paint

| Source | In the pre-rendered HTML |
|---|---|
| `data`, `const`, `env` | Fully painted |
| `state` initial values, `derived` over them | Painted |
| `resource` | The `loading` arm; the live page fetches and replaces it |
| `await fetch` | Nothing until the action runs |

## Next

[Internationalisation](15-i18n.md).
