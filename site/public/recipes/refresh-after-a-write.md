# Refresh a list after a write

> Keep a list current after a create or a delete: the contract says which reads a write makes stale, and they ask again by themselves.

The page: https://webfluent.monzeromer.dev/recipes/refresh-after-a-write

After adding a todo, the list should show it, without a `reload()` written after every write. Say once, in the contract, which reads a write makes stale: when the write succeeds, every `resource` over one of them asks again, and the answer is drawn in place.

**Needs:** a server with `GET /api/todos`, `POST /api/todos` and `DELETE /api/todos/<id>`. It claims `/`: paste it into a new project, or replace the starter's `src/pages/Home.wf` with it.

```wf
type Todo { id: String, title: String }
type NewTodo { title: String }

api Backend(base: "/api") { }

api Backend.todos {
    get list() at "todos" -> [Todo]!
    post create(body: NewTodo) at "todos" -> Todo!
        invalidates [list]
    delete remove(id: String) at "todos/{id}" -> !
        invalidates [list]
}

page Todos(path: "/", title: "Todos", description: "Todos that stay current.") {
    state draft = ""
    state said = ""
    resource todos = Backend.todos.list()

    Heading("Todos").h1
    Input(bind: draft, label: "New todo")
    Button("Add").primary {
        on click {
            match await Backend.todos.create(body: NewTodo(title: draft)) {
                .ok(_) { draft = "" }
                .err(e) { said = e.message }
            }
        }
    }
    match todos {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list, refreshing) {
            if refreshing { Spinner.sm }
            for todo in list by todo.id {
                Row(justify: .between) {
                    Text(todo.title)
                    Button("Remove {todo.title}") {
                        on click {
                            match await Backend.todos.remove(id: todo.id) {
                                .ok { }
                                .err(e) { said = e.message }
                            }
                        }
                    }
                }
            }
        }
    }
    Text(said)
}
```

## How it works

The list stays on the screen while it asks again — the read is `.loaded` with `refreshing` true — and a row whose key is still there keeps its nodes. A write's `invalidates` names reads of its own group, each with the write's own parameters as its arguments (`invalidates [list, byId(id)]`); a name it does not have is `N05`. For a contract `wf api` writes, say the same in the service's `invalidations { }` (see [Call a service from its OpenAPI spec](https://webfluent.monzeromer.dev/recipes/service-from-openapi.md)).
