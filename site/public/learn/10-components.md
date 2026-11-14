# Components

> Declare components: typed props, enum props, events, slots and scoped slots, parts, local state, stores, and how to design them.

The page: https://webfluent.monzeromer.dev/learn/10-components

This chapter shows how to declare a *component*: a reusable element of your
own. It has typed
props, the events it fires, the slots its caller fills, the parts it
offers, and a body that draws it. The one thing to remember: the compiler
checks a call to your component exactly as it checks a call to a built-in
such as `Button`, so a missing prop or a misspelled flag is an error where
you wrote it.

**In this chapter**

- Declaring a component and calling it
- Props, including enum props
- Events, slots, scoped slots and parts
- A component's own state, and stores inside a component
- What a call puts on the component's root element
- How to design a component's interface

## A first component

This page declares a `Greeting` component and places it once:

```wf
component Greeting(_ name: String) {
    Text("Hello, {name}!")
}

page Home(path: "/", title: "Home", description: "A greeting.") {
    Heading("Welcome").h1
    Greeting("Ada")
}
```

The page shows the heading **Welcome** and the line **Hello, Ada!**.

- `component Greeting(…)` declares the component. Its name is capitalised,
  as every element's is.
- `_ name: String` is a prop. The `_` lets a call pass it without its name,
  so `Greeting("Ada")` sets `name` to `"Ada"`.
- The block is the body: what the component draws each time it is placed.
- `Greeting("Ada")` in the page is a call, written like any built-in.

## Props

A *prop* is a value the caller hands to the component. This component takes
four, with a type and, for three of them, a default:

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

page Team(path: "/", title: "Team", description: "Three people, three cards.") {
    Heading("Team").h1
    UserCard("Monzer", role: "Developer")
    UserCard("Sam", role: "Designer", active: false, avatar: "/sam.png")
    UserCard("Ann", role: "PM").active
}
```

The rules for props:

- `_ name: Type` marks the one prop a call may pass positionally. It must be
  the first prop. Every other prop is passed by name.
- Every prop has a written type: `String`, `Number`, `Bool`, `Map`, `Any`,
  `[T]` (a list), `T?` (may be `null`), or a declared `type` or `enum`. See
  [Types](https://webfluent.monzeromer.dev/learn/12-types.md) for the whole type language.
- `= expr` gives a prop a default, so a call may leave it out. A `T?` prop
  with no default defaults to `null`.
- A `Bool` prop is set with a flag, `.active`, or by name, `active: false`.
- A `///` comment above the component or above a prop is its
  documentation. The editor shows it on hover, and `wf docs` prints it.

> [!NOTE]
> **Coming from React?** There is no `...rest` that forwards whatever the
> caller passed. A prop the component does not declare is an error (`C02`);
> `class:`, `aria-*`, `data-*` and a few others go to the root element (see
> [What a call puts on the root element](#what-a-call-puts-on-the-root-element)).
> See [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md).

A prop that reads the caller's state stays live inside the component: a
`Pill(pressed: isOn)` repaints when `isOn` changes. You do not need to pass
a callback or re-render anything yourself.

The checker holds every call to the declaration. Leaving out a prop that
has no default is `C01`; passing a prop the component does not declare is
`C02`; a second positional argument is `E113`. A positional argument to a
component whose first prop is not marked `_` is bound to that first prop,
with a `C06` warning asking you to add the `_`.

## Enum props

An *enum prop* takes one case of a declared `enum`. The caller picks the
case with a flag or by name:

```wf
enum Tone { calm, loud }

component Note(_ text: String, tone: Tone = .calm) {
    Text(text, class: "note") {
        style { padding: $sm; border-radius: $radius-sm }
    }
}

page Notes(path: "/", title: "Notes", description: "One note in each tone.") {
    Heading("Notes").h1
    Note("Quiet")
    Note("Loud!", tone: .loud)
    Note("Loud too").loud
}
```

`.loud` and `tone: .loud` mean the same. A flag the enum does not have, such
as `.huge`, is an error (`E103`) that lists the cases it does have.

The compiler writes a component's enum props to its root element as
`data-<prop>="<case>"` — here `data-tone="loud"` — so a stylesheet can
select on them: `[data-tone="loud"] { font-weight: 700 }`. See
[Styling](https://webfluent.monzeromer.dev/learn/13-styling.md) for project stylesheets.

## Events

An *event* is how a component tells its caller that something happened. The
component declares it with `event name(param: Type)` and fires it with
`emit name(value)`. The caller handles it with `on name(value) { }`, as it
handles a click:

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

page Todos(path: "/", title: "Todos", description: "Rows that report back.") {
    state todos: [Todo] = [Todo(id: "1", title: "Milk"), Todo(id: "2", title: "Eggs")]
    action toggle(id: String) {
        todos = todos.map(todo => if todo.id == id { Todo(id: todo.id, title: todo.title, done: !todo.done) } else { todo })
    }
    Heading("Todos").h1
    for todo in todos by todo.id {
        TodoRow(todo) {
            on toggle(id) { toggle(id) }
            on remove(id) { todos = todos.filter(todo => todo.id != id) }
        }
    }
}
```

The checker holds each `emit` to the parameters its event declares (`T09`).
A handler on a call must name a declared event or a DOM event; an `emit`
outside a component is `E114`. A DOM event handler written on a component
call, such as `on click`, attaches to the component's root element (see
[What a call puts on the root element](#what-a-call-puts-on-the-root-element)).
For events in general, see [Events](https://webfluent.monzeromer.dev/learn/08-events.md).

> [!NOTE]
> **Coming from React?** A callback prop such as `onToggle={…}` becomes a
> declared `event toggle(id: String)`, fired with `emit toggle(id)` and handled
> on the call as `on toggle(id) { … }`. The checker holds both ends to the
> declaration. See [Coming from React](https://webfluent.monzeromer.dev/coming-from/react.md).

### An event with several values

An event may send several values. The handler names them in the order the
event declares them, and each name has the type declared for it. In this
example `key` is a `String`, and `x` and `y` are `Number`s:

```wf
component Entry(_ key: String) {
    event menu(key: String, x: Number, y: Number)
    Button(key) {
        on contextmenu(e) {
            e.preventDefault()
            emit menu(key, e.clientX, e.clientY)
        }
    }
}

page Files(path: "/", title: "Files", description: "A menu that knows where it opened.") {
    state at = "nothing yet"
    Heading("Files").h1
    Entry("notes.wf") { on menu(key, x, y) { at = "{key} at {x}, {y}" } }
    Text("Menu for {at}")
}
```

A handler may name fewer values than the event sends: `on menu(key)` reads
the key and ignores the rest. It may not name more. A name past the last
value would always be `undefined`, so the checker reports it (`T22`). You
handle an event that sends nothing with no names: `on close { }`.

## Slots

A *slot* is a place in the component where the caller's content goes. The
component places the caller's block with `children`, and may declare named
slots that the caller fills with `name { … }`:

```wf
component Panel(_ title: String) {
    slot trailing
    Card {
        Row(justify: .between, align: .center) {
            Heading(title).h2
            trailing
        }
        children
    }
}

page Keys(path: "/", title: "Keys", description: "A panel with a slot.") {
    Heading("Keys").h1
    Panel("API keys") {
        trailing { Badge("Beta").info }
        Text("Rotate every 90 days.")
        Button("Generate").primary
    }
}
```

- `children` places the call's block. It needs no declaration: a component
  that writes `children` takes a block.
- `slot trailing` declares a named slot, which the body places by its name.
- Anything in the call's block that is not a named fill goes to
  `children`. Here that is the `Text` and the `Button`.
- A bare `slot` line declares the default slot by name. You need it only on
  a component a page names as its `layout:`, where it is required
  (`E104`); see [Layouts](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md#layouts).

A fill is compiled in the caller's scope: it reads the caller's state and
loop variables, and its handlers are the caller's. The component only
decides where the fill lands. A page's `layout:` is a component with a
default slot, called with the page as its block (see
[Pages and routing](https://webfluent.monzeromer.dev/learn/05-pages-and-routing.md#layouts)).

### Scoped slots

A *scoped slot* hands values to its fill. The component declares what it
hands over, places the slot with those values by name, and the caller's
fill names them in order:

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

page Use(path: "/", title: "Todos", description: "A list that hands each row to its caller.") {
    state todos: [Todo] = [Todo(id: "1", title: "Milk")]
    Heading("Todos").h1
    TodoList(items: todos) {
        row(todo, i) { Text("{i + 1}. {todo.title}") }
        empty { Text("Nothing to do.").muted }
    }
}
```

The page shows **1. Milk**. The fill's names take their types from the slot
declaration: `todo` is a `Todo` and `i` a `Number`. The fill is drawn again
when a value it is handed changes.

## Parts

A *part* is a component of the component's own, called under its name —
the way `Card.Header` is a part of `Card`:

```wf
component Panel(_ title: String) {
    part Header(_ text: String) { Heading(text).h2 }
    part Footer { Divider  Text("End").muted.sm }
    Card { Text(title)  children }
    slot
}

page Parts(path: "/", title: "Parts", description: "A panel with parts of its own.") {
    Heading("Parts").h1
    Panel("Keys") {
        Panel.Header("Rotate")
        Text("Body")
        Panel.Footer
    }
}
```

You declare a part inside its component. A part has props, events and slots
of its own, but it cannot declare parts: a part inside a part is an error.
Calling a part outside its owner, such as `Panel.Header` with no `Panel`
around it, is `C03`.

## Local state and actions

A component may have its own `state`, `derived` values, `effect`s, actions,
timers and handles. Each placed instance gets its own copy, so the two
`Stepper`s on this page count separately:

```wf
component Stepper(_ start: Number, step: Number = 1) {
    event change(value: Number)
    state n = start
    derived even = n % 2 == 0
    action move(delta: Number) { n = n + delta  emit change(n) }
    Row(gap: .sm, align: .center) {
        Button("−").sm { on click { move(-step) } }
        Text("{n}").bold
        Button("+").sm { on click { move(step) } }
        if even { Badge("even").info }
    }
}

page Counters(path: "/", title: "Counters", description: "Two steppers, two counts.") {
    state last = 0
    Heading("Counters").h1
    Stepper(0) { on change(v) { last = v } }
    Stepper(10, step: 5) { on change(v) { last = v } }
    Text("Last value: {last}")
}
```

What an instance creates — effects, timers, listeners — is disposed of when
the instance leaves the page. See [State and reactivity](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md)
for `state`, `derived` and `effect`.

## Stores inside a component

`use StoreName` brings a store into a component, as it does into a page.
Every instance reads and writes the same store:

```wf
store Cart {
    state items: [String] = []
    action add(name: String) { items.push(name) }
}

component AddToCart(_ name: String) {
    use Cart
    Button("Add {name}").primary { on click { Cart.add(name) } }
}

page Shop(path: "/", title: "Shop", description: "Buttons that share one cart.") {
    use Cart
    Heading("Shop").h1
    AddToCart("Pen")
    Text("{Cart.items.length} in cart")
}
```

Clicking **Add Pen** changes the page's text from **0 in cart** to
**1 in cart**. [Stores](https://webfluent.monzeromer.dev/learn/11-stores.md) covers stores in full.

## What a call puts on the root element

A component's *root element* is the one element its body draws at the top.
Some of what you write on a call goes to that element rather than to a
prop:

- A DOM event handler: `Pill("x") { on click { … } }` attaches to the root,
  so a styled button component is clickable wherever you place it.
- `class:`, which is added to the root's own classes.
- A `style { }` block, which styles the root: `Tile(label: "a") { style {
  grid-column: 2 / span 2 } }` places that tile in its grid. Its rule's class
  joins the root's classes, as `class:` does.
- An attribute the component takes no prop for: `aria-*`, `data-*`, `id`,
  `role`, `title` and `hidden`. You write it as you would on an element:
  `data-open: true` is `data-open="true"`, and `false` writes no attribute,
  except for an ARIA state, which keeps `"false"`.
- `mount:`, `update:` and `cleanup:` (see
  [Hand an element to a script](https://webfluent.monzeromer.dev/recipes/hand-an-element-to-a-script.md)).
- `drag:`, the `drop…:` props, `on receive`, `on dragmove`, `on longpress`
  and `on swipe`. These need one root element: on a component whose body is
  an `if`, a `for` or several elements they are `C07`.

When the component declares a prop of that name, it receives the value
instead and decides where it goes. Declare the prop when the root is not
the right element for it. This `Pill` takes the class and the description
as props and puts them on its `Badge`:

```wf
component Pill(_ label: String, extra: String = "", note: String = "") {
    Badge(label, class: extra, aria-description: note)
}

page Pills(path: "/", title: "Pills", description: "Styled by the caller.") {
    Heading("Pills").h1
    Pill("Beta", extra: "chip-beta", note: "Not finished yet") { on click { log("beta") } }
}
```

The `on click` still goes to the root, which here is the `Badge`.

## Designing components

A component's interface is its props, events, slots and parts. These
guidelines keep it clear to callers.

### A value down, an event up

A component does not take `bind:`. To make a control of your own, take the
value as a prop and fire an event when it should change. The caller decides
what to do with the new value:

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

Clicking **+** sets `seats` to 2, and the component shows **Seats: 2**
because its `value` prop reads `seats`.

> [!NOTE]
> **Coming from Vue or Svelte?** There is no `v-model` or `bind:` on your own
> component. Take the value as a prop and fire a `change` event; the caller
> assigns its state in the handler. `bind:` belongs to the built-in controls.
> See [Coming from Vue](https://webfluent.monzeromer.dev/coming-from/vue.md) and
> [Coming from Svelte](https://webfluent.monzeromer.dev/coming-from/svelte.md).

### Pick the right way in

Choose the part of the interface by what the caller wants to do:

| The caller wants to… | Use |
|---|---|
| configure it with a value | a prop |
| choose one of a few looks | an `enum` prop, set with a flag |
| put any content inside | the default slot, `children` |
| put content in a particular place | a named slot |
| render each item its own way | a scoped slot, `slot row(item: T)` |
| offer a sub-element with its own props | a part, `Panel.Header` |
| hear that something happened | an `event` |

### Keep state where it is read

A component's own `state` is right for what only that component cares
about, such as whether its menu is open or it is hovered. What the page or
other components care about comes in as a prop and goes out as an event —
or lives in a store.

### Document it

A `///` comment above the component, a prop or an event is shown on hover
in the editor and in the gallery `wf docs` writes.

## Other people's code

To use a JavaScript library — a chart, a map, an editor — call it from a
`.js` file under `src/` and hand it an element with `mount:` and
`cleanup:`. To place somebody else's custom element, use
`Element("tag-name", …)`. You can also publish your components as custom
elements for another framework to place. See [Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md),
[Hand an element to a script](https://webfluent.monzeromer.dev/recipes/hand-an-element-to-a-script.md),
[Place a custom element](https://webfluent.monzeromer.dev/recipes/place-a-custom-element.md) and
[Publish components as custom elements](https://webfluent.monzeromer.dev/recipes/publish-custom-elements.md).

## Components in the pre-rendered page

A static build pre-renders each page to HTML — the *static paint*. It
expands every component call into HTML at build time: props substituted,
slots filled, and loops and branches over the seeded state drawn. A page
built from components is therefore a complete page before any script runs.
See [Static sites and SPAs](https://webfluent.monzeromer.dev/about/static-or-single-page.md).

## Quick reference

| Syntax | Meaning |
|---|---|
| `component Name(_ a: T, b: T = v) { … }` | Declare a component; `_` marks the positional prop |
| `Name(x, b: y).flag` | Call it; a flag sets a `Bool` prop or picks an enum case |
| `/// text` | Documentation for the component, a prop or an event |
| `event name(p: T)` | Declare an event |
| `emit name(v)` | Fire it from the body |
| `on name(v) { … }` | Handle it on the call |
| `slot` · `children` | Declare and place the default slot |
| `slot name` · `name` | Declare and place a named slot; the caller fills it with `name { … }` |
| `slot row(item: T)` · `row(item: v)` | A scoped slot; the caller fills it with `row(x) { … }` |
| `part Name(…) { … }` · `Owner.Name` | Declare a part and call it |
| `use Store` | Read a store inside the component |

[Components reference](https://webfluent.monzeromer.dev/reference.md) lists every built-in,
and [Diagnostics](https://webfluent.monzeromer.dev/errors.md) every code named here.

## Quiz

```quiz
A page declares `state count = 3` and places `Panel("Totals") { Text("{count}") }`. `Panel` declares no `count`. What does the fill show?
+ 3, the page's `count` -- A fill is compiled in the caller's scope: it reads the caller's state, and the component only decides where it lands.
- Nothing, because `Panel` has no `count` -- The component never reads the fill's names; they belong to the caller.
- An error until `count` is passed to `Panel` as a prop -- A fill needs no prop to read the caller's state.
```

```quiz
`component Note(_ text: String)` is called as `Note("Saved", tone: "success")`. What happens?
- It builds, and `tone="success"` lands on the root element -- Only what *What a call puts on the root element* lists goes to the root without a prop, and `tone` is not on that list.
- It builds, and `tone` is ignored -- Nothing is ignored in silence: the compiler checks a call to your component as it checks a built-in's.
+ The build stops with `C02`: `Note` declares no prop `tone` -- Declare the prop, or correct its spelling.
```

```quiz
A page places `Stepper(0)` twice, and `Stepper` declares `state n = start`. You press **+** on the first. What does the second show?
+ 0 -- Each placed instance has its own copy of the component's state.
- 1 -- State shared by every instance belongs in a store, not in a component.
- It depends on which instance was drawn first -- No instance reads another's state, whatever the order.
```

```quiz
`for row in rows { Stepper(row.start) }` draws three steppers, and you press **+** on each. Then `rows` gains an item. What happens to the three?
- They keep their counts, matched by position -- Without `by` there is nothing to match an item by; the list is rebuilt.
+ They start again from `row.start`, since a list with no key is rebuilt when it changes -- Key it, `for row in rows by row.id`, and each keeps its state; the build warns of this (U09).
- The build refuses the loop until it has a key -- It builds; the missing key is a warning, U09.
```

## Common mistakes

**A required prop left out.** A prop with no default must be passed on every
call:

```wf expect=C01
component UserCard(_ name: String, role: String) {
    Card { Text(name).bold  Text(role).muted }
}

page P(path: "/", title: "T", description: "D") {
    Heading("Team").h1
    UserCard("Ada")
}
```

Pass it, `UserCard("Ada", role: "Engineer")`, or give it a default in the
declaration, `role: String = ""`. See [C01](https://webfluent.monzeromer.dev/errors/C01.md).

**A prop the component does not declare.** Often a misspelling, or a prop
written as though the component forwarded what it was given:

```wf expect=C02
component Note(_ text: String, muted: Bool = false) { Text(text) }

page P(path: "/", title: "T", description: "D") {
    Heading("Notes").h1
    Note("Saved", tone: "success")
}
```

Declare the prop, or correct its name. See [C02](https://webfluent.monzeromer.dev/errors/C02.md).

**A handler that names more values than its event sends.** An event sends
what it declares, and nothing else:

```wf expect=T22
component Stepper(_ value: Number) {
    event change(value: Number)
    Button("+") { on click { emit change(value + 1) } }
}

page P(path: "/", title: "T", description: "D") {
    state seats = 1
    state last = 0
    Heading("Seats: {seats}").h1
    Stepper(seats) { on change(next, previous) { last = previous  seats = next } }
    Text("Was {last}")
}
```

Name only what the event sends, `on change(next)`, or declare a second
parameter and `emit` it. See [T22](https://webfluent.monzeromer.dev/errors/T22.md).

## Next

[Stores](https://webfluent.monzeromer.dev/learn/11-stores.md).
