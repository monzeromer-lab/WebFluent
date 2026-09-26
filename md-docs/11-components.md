# 11. Components

<!--
route: guide/components
group: basics
blurb: A component is a reusable element you declare: typed props, events, slots, parts and a body, checked like a built-in.
description: Declaring components: props, enum props, events, slots and scoped slots, parts, local state, stores, and patterns for designing them.
-->

A component is a reusable element you declare: typed props, the events it
fires, the slots its caller fills, the parts it offers, and a body. A call
to it is checked like a call to a built-in.

## Declaring and calling

```wf
/// A person, at a glance.
component UserCard(_ name: String, role: String, active: Bool = true, avatar: String? = null) {
    Card.elevated {
        Row(align: .center, gap: .md) {
            if let src = avatar { Avatar(src: src, alt: name) } else { Avatar(initials: name.slice(0, 2)).primary }
            Stack {
                Text(name).bold
                Text(role).muted
            }
            if active { Badge("Active").success }
        }
    }
}

page Team(path: "/") {
    UserCard("Monzer", role: "Developer")
    UserCard("Sam", role: "Designer", active: false, avatar: "/sam.png")
    UserCard("Ann", role: "PM").active
}
```

- `_ name: Type` marks the one prop a call may pass positionally; it must be
  first. Every other prop is passed by name.
- Prop types: `String`, `Number`, `Bool`, `Map`, `Any`, `[T]`, `T?`, or a
  declared `type` or `enum`. They must be written; `= expr` gives a default,
  and a `T?` defaults to `null`.
- A `Bool` prop is set with a flag: `.active`. An enum prop with a flag or by
  name: `.loud` or `tone: .loud`.
- A `///` comment above the component or a prop is its documentation.

A prop that reads the caller's state stays live inside the component: a
`Chip(pressed: isOn)` repaints when `isOn` changes.

## Enum props

```wf
enum Tone { calm, loud }

component Note(_ text: String, tone: Tone = .calm) {
    Text(text) {
        style { padding: $sm; border-radius: $radius-sm }
    }
}

page Notes(path: "/") {
    Note("Quiet")
    Note("Loud!", tone: .loud)
    Note("Loud too").loud
}
```

A component's enum props are written to its root element as
`data-<prop>="<case>"` — `data-tone="loud"` — so a stylesheet can select on
them: `.note[data-tone="loud"] { font-weight: 700 }`.

## Events

A component declares the events it fires with `event name(param: Type)`,
fires them with `emit name(value)`, and the caller handles them with
`on name(value) { }`:

```wf
type Todo { id: String, title: String, done: Bool = false }

component TodoRow(_ todo: Todo) {
    event toggle(id: String)
    event remove(id: String)
    Row(align: .center, gap: .sm) {
        Checkbox(checked: todo.done, label: todo.title) { on change { emit toggle(todo.id) } }
        IconButton(icon: "trash", label: "Remove {todo.title}").sm { on click { emit remove(todo.id) } }
    }
}

page Todos(path: "/") {
    state todos: [Todo] = [Todo(id: "1", title: "Milk"), Todo(id: "2", title: "Eggs")]
    action toggle(id: String) {
        todos = todos.map(t => if t.id == id { Todo(id: t.id, title: t.title, done: !t.done) } else { t })
    }
    for t in todos by t.id {
        TodoRow(t) {
            on toggle(id) { toggle(id) }
            on remove(id) { todos = todos.filter(t => t.id != id) }
        }
    }
}
```

The checker holds `emit` to the declared parameters (`T09`) and a handler on
a call to a declared event or a DOM event; a DOM event on a component call
attaches to its root element.

## Slots

A component places its caller's block with `children`, and may declare named
slots the caller fills with `name { … }`:

```wf
component Panel(_ title: String) {
    slot
    slot trailing
    Card {
        Row(justify: .between, align: .center) {
            Heading(title).h3
            trailing
        }
        children
    }
}

page Keys(path: "/") {
    Panel("API keys") {
        trailing { Badge("Beta").info }
        Text("Rotate every 90 days.")
        Button("Generate").primary
    }
}
```

`slot` declares the default slot (`children`); `slot trailing` a named one. A
fill is compiled in the caller's scope — it reads the caller's state and
loop variables — and the component only decides where it lands. A page's
`layout:` is a component with a default slot, called with the page as its
block.

### Scoped slots

A slot may hand values to its fill. The component declares what it hands,
uses the slot with those values by name, and the caller's fill names them in
order:

```wf
type Todo { id: String, title: String, done: Bool = false }

component TodoList(items: [Todo]) {
    slot row(item: Todo, index: Number)
    slot empty
    if items.length == 0 {
        empty
    } else {
        for it, i in items by it.id { row(item: it, index: i) }
    }
}

page Use(path: "/") {
    state todos: [Todo] = [Todo(id: "1", title: "Milk")]
    TodoList(items: todos) {
        row(t, i) { Text("{i + 1}. {t.title}") }
        empty { Text("Nothing to do.").muted }
    }
}
```

The fill's names are typed by the declaration (`t` is a `Todo`), and the fill
is drawn again when a value it is handed changes.

## Parts

A component may declare parts — components of its own, called under its
name, the way `Card.Header` is a part of `Card`:

```wf
component Panel(_ title: String) {
    part Header(_ text: String) { Heading(text).h3 }
    part Footer { Divider  Text("End").muted.sm }
    Card { Text(title)  children }
    slot
}

page Parts(path: "/") {
    Panel("Keys") {
        Panel.Header("Rotate")
        Text("Body")
        Panel.Footer
    }
}
```

A part is declared inside the component and has props, events and slots of
its own; a part declares no parts.

## Local state and actions

A component has its own `state`, `derived`, `effect`, actions, timers and
handles; each instance gets its own:

```wf
component Stepper(_ start: Number, step: Number = 1) {
    event change(value: Number)
    state n = start
    derived even = n % 2 == 0
    action move(by: Number) { n = n + by  emit change(n) }
    Row(gap: .sm, align: .center) {
        Button("−").sm { on click { move(-step) } }
        Text("{n}").bold
        Button("+").sm { on click { move(step) } }
        if even { Badge("even").info }
    }
}
```

## Using stores inside a component

`use StoreName` brings a store into a component as it does a page:

```wf
store Cart {
    state items: [String] = []
    action add(name: String) { items.push(name) }
}

component AddToCart(_ name: String) {
    use Cart
    Button("Add {name}").primary { on click { Cart.add(name) } }
}

page Shop(path: "/") {
    use Cart
    AddToCart("Pen")
    Text("{Cart.items.length} in cart")
}
```

## Designing components

**A value down, an event up.** A component does not take `bind:`. To make a
control of your own, take the value as a prop and fire an event when it
should change; the caller decides what to do with it:

```wf
component Stepper(_ value: Number, label: String, min: Number = 0) {
    event change(value: Number)
    Row(gap: .sm, align: .center) {
        Button("−", disabled: value <= min).sm { on click { emit change(value - 1) } }
        Text("{label}: {value}")
        Button("+").sm { on click { emit change(value + 1) } }
    }
}

page Booking(path: "/", title: "Booking", description: "How many seats.") {
    state seats = 1
    Heading("Book").h1
    Stepper(seats, label: "Seats", min: 1) { on change(v) { seats = v } }
}
```

**Handlers reach the root; classes and attributes do not.** A DOM event
handler on a call — `Chip("x") { on click { … } }` — attaches to the
component's root element. A `class:`, an `id:` or an `aria-*` on the call is
not forwarded: the component decides its own markup. When a caller needs to
style or label it, declare the prop and place it yourself:

```wf
component Chip(_ label: String, extra: String = "", note: String = "") {
    Badge(label, class: extra, aria-description: note)
}

page Chips(path: "/", title: "Chips", description: "Styled by the caller.") {
    Heading("Chips").h1
    Chip("Beta", extra: "chip-beta", note: "Not finished yet") { on click { log("beta") } }
}
```

**Pick the right door.**

| The caller wants to… | Use |
|---|---|
| configure it with a value | a prop |
| choose one of a few looks | an `enum` prop, set with a flag |
| put arbitrary content inside | the default slot, `children` |
| put content in a particular place | a named slot |
| render each item its own way | a scoped slot, `slot row(item: T)` |
| offer a sub-element with its own props | a part, `Panel.Header` |
| hear that something happened | an `event` |

**Keep state where it is read.** A component's own `state` is right for what
only it cares about (an open menu, a hover). What the page or other
components care about is a prop coming in and an event going out — or a
store.

**Document it.** A `///` comment above the component, a prop or an event is
shown on hover in the editor and in `wf docs`.

## Other people's code

A JavaScript library — a chart, a map, an editor — is declared with
`external` and given a node with `Host`; a component can be published as a
custom element for another framework to place. Both are in
[JavaScript interop](32-javascript-interop.md).

## Components in the static paint

A static build expands every component call into HTML at build time — its
props substituted, its slots filled, its loops and branches over seeded state
painted — so a page built from components is a real page before any script
runs.

## Next

[Stores](12-stores.md).
