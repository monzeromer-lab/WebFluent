---
chapter: 19
kind: "about"
route: "/about/how-reactivity-works"
group: "about"
blurb: "What happens between a click and the one text node it changes: signals, derived values, effects, branches and keyed lists, with no virtual DOM in between."
description: "How WebFluent's signals, derived values, effects and keyed lists work under the hood, and why what reads a value is what redraws."
---
# How reactivity works

This essay explains what WebFluent does when a value changes: how a
`state` knows who reads it, why only those readers redraw, and how
branches, lists and refreshed data keep the nodes they already drew.

The claim it defends: a page should be drawn once, and after that a
change should touch exactly the nodes that read what changed. Everything
below is the machinery that makes that true, and the price it charges.

## What a page compiles to

This page has a value that changes, a value worked out from it, and a
button that changes it.

```wf
page Home(path: "/", title: "Count", description: "A counter.") {
    state count = 0
    derived double = count * 2
    Heading("Count").h1
    Text("{count} doubled is {double}")
    Button("Add one") { on click { count = count + 1 } }
}
```

`wf build` turns it into one function that runs once, when the route is
shown. Here it is, laid out for reading:

```js
function Page_Home(params) {
  const _count = WF.signal(0);
  const _root = document.createDocumentFragment();
  const _double = WF.computed(() => _count() * 2);
  _root.appendChild(WF.el("h1", { className: "wf-heading" }, "Count"));
  _root.appendChild(WF.el("p", { className: "wf-text" },
    () => `${_count()} doubled is ${_double()}`));
  const button = WF.el("button", { className: "wf-btn", type: "button" }, "Add one");
  button.addEventListener("click", () => { _count.set(_count() + 1); });
  _root.appendChild(button);
  return _root;
}
```

There is no render function that runs again, no tree of descriptions to
compare with the last one, and no component that re-renders. The heading
is a plain `<h1>` made once. The paragraph's text is the one thing that
can change, and it is handed to `WF.el` as a function: that function is
what will run again.

## A signal knows who reads it

`WF.signal(0)` returns a getter. Calling it returns the value, and, if an
effect is running at that moment, adds that effect to the signal's list of
readers. That is the whole trick of dependency tracking: nothing is
declared, because reading is declaring.

The function the paragraph was given becomes an effect over one text node.
Its first run reads `_count()` and `_double()`, so both now list it. When
the button calls `_count.set(1)`, the signal compares the new value with
the old one (`!==`), and, since they differ, runs the effects that read it.
The text node gets its new text. The `<h1>`, the button and the rest of
the page are not touched, because nothing about them read `count`.

An effect listens to exactly what its last run read. Each run takes it off
every signal it was listed on, and the run lists it again. So an effect
whose condition stopped reading a value stops running for that value, and
the set of dependencies is always the true one, not one you wrote down.

## A derived value is an effect that writes a signal

`derived double = count * 2` compiles to `WF.computed`: a signal of its
own, kept current by an effect that reads `count`. When `count` changes,
that effect runs, writes `double`, and the write runs `double`'s readers
in turn. A derived value that works out to the same value as before
(`count > 0` while `count` goes from 3 to 4) stops there: its signal
compares, sees no change, and its readers do not run.

Two consequences follow:

- A derived value is eager. It is worked out when what it reads changes,
  not when something reads it, so reading it costs nothing.
- It cannot be assigned. It has one source, its expression, and the
  compiler refuses an assignment to it (`X01`).

## One change, one ordered run

A change does not redraw at once and then again for every dependent; it
collects every effect it reaches and runs them in one pass, oldest first.
The order matters. A branch is made before what it draws, so when an `if`
closes and a `derived` it guarded changes in the same click, the branch
runs first, takes its contents away, and the effects inside it are
disposed of before they can read a value that no longer makes sense for
them.

An effect that writes what it reads comes back into the same pass. One
that settles does so in a run or two. One that never settles would loop
forever; the compiler refuses the ones it can see (`X02`), and the runtime
stops any other after 100 runs, with a warning under `wf serve`.

## Values are replaced, not mutated

A signal notices a change by comparing identities. If code pushed onto
the list a `state` holds, the list would be the same object, and nothing
would redraw. So WebFluent never lets that happen silently:
`items[i].done = v`, `form.name = v` or `items.sort()` written as a
statement compile to the state given a copy with that place changed.
Reading `items.sort(…)` in an expression sorts a copy. You write the
change the way you think of it, and the signal still sees a new value.

## A branch tracks its condition, and nothing else

An `if` compiles to `WF.when`, an effect that reads only the condition.
The branch's body is drawn with tracking turned off, so a value the body
reads does not make the whole branch draw again; it makes the one node
that reads it update, as anywhere else. The branch draws again only when
the condition flips.

Everything the body made belongs to the branch. Effects, timers from
`every` and `after`, event listeners on the window, handles from `ref:`:
each is created inside a scope, and when the branch leaves, the scope runs
every cleanup it collected. A signal that outlives the branch, such as a
store's state, has the branch's effects taken off its list, so it does not
keep their nodes alive. That is why a page in WebFluent does not leak a
timer when you navigate away from it: the page is a scope too.

## A keyed list moves rows instead of rebuilding them

This page has a list, a count worked out from it, and a branch.

```wf
type Todo { id: String, title: String, done: Bool = false }

page Todos(path: "/", title: "Todos", description: "A list that keeps its rows.") {
    state todos: [Todo] = [Todo(id: "1", title: "Milk"), Todo(id: "2", title: "Bread")]
    state draft = ""
    derived left = todos.filter(item => !item.done).length

    Heading("Todos").h1
    Text("{left} left")
    Input(bind: draft, label: "New todo")
    Button("Add") { on click { todos = todos.concat([Todo(id: uuid(), title: draft)])  draft = "" } }
    for todo in todos by todo.id {
        Checkbox(bind: todo.done, label: todo.title)
    }
    if left == 0 {
        Text("All done.")
    }
}
```

`for … by todo.id` compiles to `WF.each` with a key. Each row is drawn
once and filed under its key. When the list changes, the runtime walks the
new list: a key it has seen keeps its nodes and is moved into place if it
moved; a new key is drawn; a key that is gone is disposed of and removed.
A checkbox that has focus keeps it while rows are added or removed around it, because
it is the same element.

A keyed `for` is also *live*. Its body reads the item through an accessor,
a small signal of its own, so a new value under the same key reaches the
row's nodes in place: the label's text node updates, and nothing else in
the row is redrawn. Without `by`, the runtime has no way to tell one row
from another, and rebuilds the list whole on every change; the compiler
warns when that would lose state (`U09`).

## A refresh keeps what did not change

Data from a server makes the same promise harder to keep, because every
response is new objects. A resource that asks again on an interval would
otherwise hand the list a fresh array of fresh rows every 30 seconds.

```wf
type Server { id: String, name: String, status: String }

page Servers(path: "/", title: "Servers", description: "Every server, kept current.") {
    resource servers: [Server] = fetch("/api/servers", on: .interval(30.seconds))
    Heading("Servers").h1
    match servers {
        loading { Text("Loading…") }
        error(e) { Text(e.message) }
        ready(list) {
            for s in list by s.id { Text("{s.name}: {s.status}") }
        }
    }
}
```

Before the answer to a refresh is stored, the runtime lays it over what
was there: a row with the same `id` that is equal to the old one is
replaced by the old object, and a list in which nothing changed is the old
list. Then the ordinary machinery does the rest. The `ready` arm stays
drawn, since the arm is still `ready`. The list, handed the same array,
has nothing to do. A server whose status changed gives a new object under
the same key, and its row's text node updates. An answer that changed
nothing redraws nothing.

## What it costs

Fine-grained reactivity is not free. Its costs are the reasons for most
of the rules you meet in the book:

- **A value read once is read once.** A `state` that starts from `now` or
  `viewport` takes the value it had when the page opened and never moves
  (`X06`). Only a `derived`, or text that reads the value, follows it.
- **Identity is the change.** A list changed in place behind the
  compiler's back, by your own script, is not a change. Hand a script an
  action, and let the action assign.
- **Keys are your job.** The runtime cannot guess which row is which. A
  `by` that names what identifies the item, not its position, is what
  lets a row keep its focus and its state.

In return, a click costs what it changes and nothing more, a page needs no
memoization to stay fast, and what you see in the source, the place a
value is read, is the place the page updates.

For the language side of this, read
[State and reactivity](../learn/07-state-and-reactivity.md) and
[Control flow](../learn/09-control-flow.md). The functions named here are
listed in the [runtime API](../reference/runtime-api.md).
