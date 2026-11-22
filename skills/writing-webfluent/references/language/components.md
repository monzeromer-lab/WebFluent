### Components

```wf
/// A person, at a glance.
component UserCard(_ name: String, role: String, active: Bool = true) {
    Card.elevated {
        Row(align: .center, gap: .md) {
            Avatar(initials: "U").primary
            Stack {
                Text(name).bold
                Text(role).muted
            }
            if active {
                Badge("Active").success
            }
        }
    }
}

// Usage
UserCard("Monzer", role: "Developer")
UserCard("Sam", role: "Designer").active
```

- Prop types: `String`, `Number`, `Bool`, `[T]` (a list), `T?`
  (may be null), `[K: V]` (a dictionary), `(A) -> R` (a function), `Json`,
  or a declared `type` or `enum`
- `_ name: Type` — the one prop a call may pass positionally; it must be first
- Default values: `active: Bool = true`; a `Bool` prop is set with a flag
  (`.active`), an enum prop with a flag or by name (`.loud`, `tone: .loud`)
- A component named like a built-in takes its place in the project: every
  call of the name draws the project's own (`V11`, information), so a built-in a
  later WebFluent adds never changes what a project draws
- A `///` comment above a declaration, a prop or an event is its
  documentation, shown on hover; above a `state`, `persist`, `derived` or
  `action`, it is that member's

A prop that reads state stays live inside the component: `Chip(pressed:
showErrors)` repaints when `showErrors` changes.

#### Slots

A built-in takes the named blocks its entry lists: a `Tree`'s `row(item)`,
a `DataTable`'s `empty` and a column's `cell(row)`, a `Combobox`'s
`option(o)` and `empty`. Named blocks come before the children in a
block (`E002` otherwise).

A component places its caller's block with `children`, and may declare
named slots the caller fills with `name { … }`. A fill is compiled in the
caller's scope, so it reads the caller's state and loop bindings; the
component only decides where it lands.

```wf
component Panel(_ title: String) {
    slot trailing
    Card {
        Row(justify: .between) { Heading(title).h3  trailing }
        children
    }
}

Panel("Keys") {
    trailing { Badge("Beta") }
    Text("Rotate every 90 days.")
    Button("Generate").primary { on click { generate() } }
}
```

A page's `layout:` is a component with a default slot, called with the page
as its block.

A **scoped slot** hands its fill values: the component declares what it
hands over, uses the slot with those values by name, and the caller's fill
names them in order.

```wf
component Rows(items: [Todo]) {
    slot row(item: Todo, index: Number)
    for it, i in items by it.id { row(item: it, index: i) }
}

Rows(items: todos) { row(todo, i) { Text("{i}: {todo.title}") } }
```

A component may declare **parts** — components of its own, called under
its name:

```wf
component Panel(_ title: String) {
    part Header(_ text: String) { Heading(text).h3 }
    part Footer { Divider }
    Card { Text(title)  children }
}

Panel("Keys") { Panel.Header("Rotate")  Text("…")  Panel.Footer }
```

A component's enum props are written to its root element as
`data-<prop>="<case>"` (`data-tone="loud"`), so a stylesheet can select on
them.

What a call puts on the component's root element: a DOM event's handler,
`class:`, an attribute the component takes no prop for (`aria-*`, `data-*`,
`id`, `role`, `title`, `hidden`), and a `style { }` block, which styles the
root — `Tile(label: "a") { style { grid-column: 2 / span 2 } }` places that
tile in its grid.

#### Events

A component declares the events it fires and their arguments; `emit` fires
one; the caller handles it with `on name(args) { }`, a name for each value
the event sends, in order — `event menu(key: String, x: Number, y: Number)`
is handled `on menu(key, x, y) { }`, each name typed as declared. It may
name fewer (`on menu(key)`), never more (T22). A DOM event written on
a component call — `on click { }` — attaches to the component's root
element, so a styled button component is clickable wherever it is used.
So does an attribute the component takes no prop for — `aria-label`,
`data-*`, `id`, `role`, `title`, `hidden` — on the live page, in the
static paint and in a test's render alike, written as on an element
(`data-open: true` is `data-open="true"`, `false` none but an ARIA
state's): `Avatar2(user, aria-label: "Profile")`.

```wf
component TodoRow(_ label: String, done: Bool = false) {
    event toggle(id: String)
    Row {
        Checkbox(checked: done) { on change { emit toggle(label) } }
        Text(label)
    }
}

TodoRow("Milk") { on toggle(id) { Todos.toggle(id) } }

component Entry(_ key: String) {
    event menu(key: String, x: Number, y: Number)
    Button(key) { on contextmenu(e) { e.preventDefault()  emit menu(key, e.clientX, e.clientY) } }
}

Entry("row-1") { on menu(key, x, y) { Menus.open(key, x, y) } }
```

