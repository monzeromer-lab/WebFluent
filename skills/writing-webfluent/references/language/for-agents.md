## For agents

The grammar, the idioms a model reaches for, and the mistakes it makes
most; the rest of this file is the reference.

### The grammar in brief

- A file declares `page Name(path: "/x", title: "…") { … }`,
  `component Name(_ first: String, other: Bool = false) { … }` or
  `store Name { … }`. Names of declarations are capitalised.
- Inside: `state x = 0` (changes), `derived y = x * 2` (computed),
  `action go(n: Number) { … }` (does things), `persist z = ""` (kept).
- A page or component reads a store after `use Name`, as `Name.member`.
  Only members the store declares exist.
- An element is capitalised: its first argument positional, the rest
  `name: value`, flags after a dot (`Button("Save").primary`), children in
  a block, handlers as `on click { … }`.
- `if … { } else { }`, `for item in list by item.id { }`, `match` for an
  enum or a resource.
- A string splices `{expression}`; `t("key")` is a translation.
- Commas separate arguments, never statements; a `style { }` block is CSS.

### Ten idioms

A list with an empty state:

```wf
type Todo { id: String, title: String }

store Todos { state items: [Todo] = [] }

page List(path: "/todos", title: "Todos") {
    use Todos
    if Todos.items.length == 0 {
        Text("Nothing to do.")
    } else {
        for item in Todos.items by item.id {
            Text(item.title)
        }
    }
}
```

A field checked as it is typed:

```wf
page Signup(path: "/signup", title: "Sign up") {
    state email = ""
    derived wrong = email != "" && !email.includes("@")
    Input(bind: email, label: "Email address").email
    if wrong {
        Text("Enter an email address.")
    }
}
```

A store action that waits for the network:

```wf
type Account { name: String }

store Profile {
    state name = ""
    state failed = false

    action load() {
        let me: Account! = await http.get("/api/me")
        match me {
            .ok(m) { name = m.name }
            .err(_) { failed = true }
        }
    }
}
```

A route with a parameter:

```wf
page Post(path: "/posts/:slug", slug: String, title: "{slug}") {
    Heading(slug).h1
}
```

A class that follows state:

```wf
component Tab(_ label: String, active: Bool = false) {
    Button(label, class: if active { "tab on" } else { "tab" })
}
```

A component that says something happened:

```wf
component Choice(_ label: String) {
    event pick(label: String)
    Button(label) { on click { emit pick(label) } }
}

page Picker(path: "/pick", title: "Pick") {
    state picked = ""
    Choice("First") { on pick(l) { picked = l } }
    Text("Picked: {picked}")
}
```

A component that places what its caller gives it:

```wf
component Panel(_ title: String) {
    Card {
        Heading(title).h3
        children
    }
}

page Keys(path: "/keys", title: "Keys") {
    Panel("Keys") { Text("Rotate every 90 days.") }
}
```

A value worked out from a store:

```wf
type Deploy { id: String, service: String, status: String }

store Deploys {
    state items: [Deploy] = []
    derived live = items.filter(d => d.status == "live").length
}

page Live(path: "/live", title: "Live") {
    use Deploys
    Text("{Deploys.live} live")
}
```

A choice kept across visits:

```wf
store Prefs { persist scheme = "light" }

page Theme(path: "/theme", title: "Theme") {
    use Prefs
    Button("Switch theme") {
        on click { Prefs.scheme = if Prefs.scheme == "light" { "dark" } else { "light" } }
    }
}
```

One arm per case of an enum:

```wf
enum Tone { calm, loud }

component Banner(tone: Tone = .calm) {
    match tone {
        .calm { Text("All is well.") }
        .loud { Text("Look here!") }
    }
}
```

### The mistakes made most

- **T06, a member the store does not have** (`Cart.total` for `sum`): read
  the store before you call one; never guess a name.
- **C02, a prop the component does not declare**: read its entry in the
  registry first.
- **X05, `use` of a store nothing declares**, and **T13, a name nothing
  declares**: a store or a value is named exactly as it is declared, with
  its capital.
- **E002, a syntax error**: a comma between two statements, a brace left
  open, a `.flag` before the call's parentheses close. Arguments are
  separated by commas; statements by new lines.
- **T17, an async result used before it is awaited**: an action that
  fetches is awaited where its result is read.

A diagnostic's code is in "Compiler diagnostics", with its fix.

### A page, whole

```wf
type Deploy { id: String, service: String, status: String }

store Deploys {
    state items: [Deploy] = []
    state search = ""
    derived shown = items.filter(d => d.service.includes(search))

    action add(service: String) {
        items = items.concat([Deploy(id: service, service: service, status: "queued")])
    }
}

page DeployList(path: "/deploys", title: "Deploys", description: "Every deploy.") {
    use Deploys
    Heading("Deploys").h1
    Input(bind: Deploys.search, placeholder: "Search by service", aria-label: "Search")
    Button("Add a deploy").primary { on click { Deploys.add("new-service") } }
    if Deploys.shown.length == 0 {
        Text("No deploys yet.")
    } else {
        for d in Deploys.shown by d.id {
            Row(gap: .sm) {
                Text(d.service)
                Badge(d.status)
            }
        }
    }
}
```

<!-- core -->
