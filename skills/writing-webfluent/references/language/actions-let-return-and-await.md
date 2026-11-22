### Actions, `let`, `return` and `await`

```wf
type Me { token: String }

store AuthStore {
    state accessToken = ""

    action getHeaders() -> [String: String] {
        let h = {}
        h["Authorization"] = "Bearer " + accessToken
        return h
    }

    action load() -> ! {
        let me: Me = try await http.get("/api/me")
        accessToken = me.token
    }
}
```

`let` declares a local of an action or handler, which — like a `for`
binding, a lambda's parameter or a caught error — shadows a store member,
a prop or a state of the same name until its block ends (`let refs = …` in
a store with `state refs` is the action's own); `await` inside one makes
it async; `return` leaves it with a value or without. `for` loops (with an
index, over a range), `try { } catch e { }` and destructuring `let` are
imperative statements too:

```wf
action sync() -> ! {
    let r: Sync = try await http.get("/api/sync")
    let { items, total } = r
    let [first, second] = items
    for item, i in items { log("{i}: {item.title}") }
    for n in 1..=total { count = n }
    try {
        saveDraft(items)                  // a project script that may throw
    } catch e {
        error = e.message
    }
}
```

