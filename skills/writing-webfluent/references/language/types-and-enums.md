### Types and enums

`type` declares a record and `enum` a set of cases; both are checked everywhere they are used.

```wf
enum Tone { calm, loud }
type Todo { id: String, title: String, done: Bool = false, tone: Tone = .calm, note: String? = null }

store Todos {
    state items: [Todo] = []
    action add(title: String) { items = items.concat([Todo(id: uuid(), title: title)]) }
}
```

