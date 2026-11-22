### Showing a change before the server agrees

`optimistic(holder, change)` shows it at once. If anything later in the
action fails — a `try` passing its error up, or a throw — what was shown is
taken back: the action rolls itself back.

```wf
action rename(id: String, name: String) -> ! {
    optimistic(Todos.items, items => items.map(i => if i.id == id { { ...i, title: name } } else { i }))
    try await Backend.todos.rename(id: id, body: Rename(title: name))
}
```

