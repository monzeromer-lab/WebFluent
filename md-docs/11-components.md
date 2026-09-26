# 8. Components

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

## Somebody else's code

A chart, a map, a rich-text editor: a library that wants a DOM node and
gives back a handle. Two declarations cover it, and between them there is
no `window.X` and no hoping.

### `external` — a typed import

```wf
external Chart from "https://cdn.jsdelivr.net/npm/chart.js@4.4.1/+esm" {
    integrity: "sha384-…"
    fn Chart(canvas: Any, config: Map) -> ChartHandle
    type ChartHandle {
        update(data: Map)
        destroy()
    }
}
```

The compiler cannot read the other side, so **the declaration is the
contract**: `Chart.Chart(node, config)` is checked against it, and a call
that does not match is an error where it is written rather than a
`TypeError` in a browser.

What the build does with it: writes `externals.js`, a module holding the
`import`s; links it from every page with `<script type="module">`; adds the
origin to the Content-Security-Policy, so a declared import is never
blocked by the policy shipped beside it; and, where `integrity:` is given,
emits `<link rel="modulepreload" integrity="…">` — the one place the
platform lets subresource integrity reach a module.

It is a separate file, not part of the bundle, because a module has its own
scope and the page chunks a split build writes are classic scripts. Each
import is bound to a global there, so the bundle stays what it was.

A bare specifier — `external d3 from "d3"` — is a module specifier like any
other: it resolves the way the page's import map or your host resolves it.

### `Host` — a node with a lifetime

```wf
page Sales(path: "/sales", title: "Sales", description: "How it is going.") {
    resource rows = Backend.sales()
    Host(tag: "canvas",
         mount: (node) => Chart.Chart(node, { type: "bar", data: rows.data }),
         update: (chart) => chart.update(rows.data),
         cleanup: (chart) => chart.destroy())
}
```

`mount` runs once with the element and gives back whatever the library
hands over. `update` runs again whenever the state it reads changes.
`cleanup` runs when the page, branch or list item that owns it leaves —
tied to the same scope as every effect and timer, so it cannot be
forgotten. That last part is the reason this exists: the hand-written
version is a `ref`, an `effect` and a `cleanup`, and the cleanup is what
people leave out.

`tag:` is the element the library is handed — `div` by default, and
`span`, `canvas`, `svg`, `section`, `figure`, `pre`, `p`, `ul` or `table`
where it wants one of those. A library that throws on mount takes itself
out; the rest of the page stays.

### Somebody else's custom element

```wf
external element Stripe("stripe-pricing-table") {
    prop publishableKey: String
    prop pricingTableId: String
    event ready()
}
```

```wf
Stripe(publishableKey: key, pricingTableId: "prctbl_1") { on ready { loaded = true } }
```

It is placed like a component. A prop becomes the attribute a framework
would write — `publishableKey` is `publishable-key` — and follows state
like any other value; an event it declares is a DOM event the page hears.

## Publishing yours

```json
{ "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] } }
```

Each name becomes a standards-based custom element: `PriceTag` is
`<price-tag>`, and a one-word component is prefixed so its tag has the
hyphen a custom element needs (`Rating` is `<wf-rating>`). The build writes
`elements.js`, `styles.css` and a page listing the tags it published.

```html
<script src="/elements.js" defer></script>
<link rel="stylesheet" href="/styles.css">

<price-tag label="Pro" amount="29" sale></price-tag>
```

An attribute is a prop, read when the element is connected and again
whenever it changes: a `Number` prop is read as a number, a `Map` or a list
as JSON, and a `Bool` is true when the attribute is present — as HTML reads
`disabled` — and false when it is `="false"`, so a framework that writes
the string still works. An event the component declares is dispatched as a
`CustomEvent` that bubbles. What the component created goes when the
element leaves the document.

React, Vue, Svelte, Angular, Rails, WordPress and a plain page can all
place a tag. That is the whole reason this is the answer rather than an
adapter per framework: **the shared interface between frameworks is the
platform.**

## Components in the static paint

A static build expands every component call into HTML at build time — its
props substituted, its slots filled, its loops and branches over seeded state
painted — so a page built from components is a real page before any script
runs.

## Next

[Stores](09-stores.md).
