# Update at once, and roll back on failure

> Show a change before the server agrees: update the state at once, send the request, and put the old value back if it fails.

The page: https://webfluent.monzeromer.dev/recipes/optimistic-updates

A like that waits for the server feels slow. To show the change at once, update the state before the request, keep the old value, and put it back if the request fails:

**Needs:** a server that answers `POST /api/like/<id>`. It claims `/`: paste it into a new project, or replace the starter's `src/pages/Home.wf` with it.

```wf
type Item { id: String, liked: Bool }

page Likes(path: "/", title: "Likes", description: "A like that shows at once.") {
    state items: [Item] = [Item(id: "a", liked: false)]
    action like(id: String) {
        let before = items
        items = items.map(i => if i.id == id { Item(id: i.id, liked: !i.liked) } else { i })
        match await http.post("/api/like/{id}") {
            .ok { }
            .err(_) {
                items = before
                toast("Could not save. Try again.", tone: .danger)
            }
        }
    }
    Heading("Likes").h1
    for i in items by i.id {
        Button(if i.liked { "♥" } else { "♡" }, aria-label: "Like").sm { on click { like(i.id) } }
    }
}
```

## How it works

- `let before = items` keeps the list as it was. A [`let`](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#actions-and-locals) is local to the action.
- The next line gives `items` a new list with the one item flipped. The button repaints at once, before any request leaves.
- The request is [awaited](https://webfluent.monzeromer.dev/learn/15-data.md#one-off-requests-with-http) and its result matched. If it failed, the `.err` arm puts `before` back and a `toast(…)` tells the reader — a second failure, a second toast.

## Variations

### Let the action roll itself back

`optimistic(holder, change)` does the same with less code inside an action. It applies the change at once, and if anything later in the action fails — a `try` passing its error up, or a throw — the change is taken back. The action declares `-> !`, so `try` may pass the failure up ([Fetching data](https://webfluent.monzeromer.dev/learn/15-data.md#optimistic-updates)):

```wf
action like(id: String) -> ! {
    optimistic(items, list => list.map(i => if i.id == id { Item(id: i.id, liked: !i.liked) } else { i }))
    try await http.post("/api/like/{id}")
}
```

### Change what a read shows

When the list comes from an endpoint of an `api`, change the read itself: `.update(change)` shows the change in every `resource` over that endpoint, and returns what takes it back. This store adds a todo at once, and takes it out again if the server refuses it:

```wf
type Todo { id: String, title: String }
type NewTodo { title: String }

api Backend(base: "/api") { }

api Backend.todos {
    get list() at "todos" -> [Todo]!
    post create(body: NewTodo) at "todos" -> Todo!
        invalidates [list]
}

store Todos {
    action add(title: String) {
        let undo = Backend.todos.list.update(list => list.concat([Todo(id: "new", title: title)]))
        if let .err(_) = await Backend.todos.create(body: NewTodo(title: title)) { undo() }
    }
}

page List(path: "/todos", title: "Todos", description: "Todos that show at once.") {
    use Todos
    resource todos = Backend.todos.list()
    Heading("Todos").h1
    Button("Add bread") { on click { Todos.add("Bread") } }
    for todo in todos.value ?? [] by todo.id { Text(todo.title) }
}
```

When the create succeeds, `invalidates [list]` asks for the list again, and the server's copy, with the todo's real `id`, takes the place of the one shown ([Refresh a list after a write](https://webfluent.monzeromer.dev/recipes/refresh-after-a-write.md)).
