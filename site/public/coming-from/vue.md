# WebFluent for Vue developers

> Vue ideas in WebFluent: refs, computed, watchers, props, slots, emits, v-model, Pinia, routing, v-if and v-for, side by side.

The page: https://webfluent.monzeromer.dev/coming-from/vue

Vue's single-file components, refs, computed values, slots and `v-model` all have a close match in WebFluent. The difference to hold on to is that WebFluent is a compiler with typed markup: there is no `.value`, no template language beside the script, and every prop, slot and event is checked at build time.

## The map

| Concept | Vue | WebFluent |
|---|---|---|
| A value that changes | `const n = ref(0)` | `state n = 0` |
| Changing it | `n.value++` | `n = n + 1` |
| A value worked out from others | `const total = computed(() => sum(items.value))` | `derived total = items.sum()` |
| A side effect | `watchEffect((onCleanup) => { start(); onCleanup(stop) })` | `effect { start()  cleanup { stop() } }` |
| A handle on an element | `<input ref="box"> … box.value.focus()` | `Input(bind: q, ref: box) … box.focus()` |
| A component | `a .vue file with <script setup> and <template>` | `component Card(_ title: String) { … }` |
| Props | `defineProps({ title: String })` | `component Card(_ title: String, tone: Tone = .calm)` |
| The default slot | `<slot />` | `children` |
| A named slot | `<slot name="trailing" />` | `slot trailing, filled as trailing { … }` |
| A scoped slot | `<slot :item="item" />` | `slot row(item: Todo), filled as row(todo) { … }` |
| An event | `defineEmits(["change"]), emit("change", n)` | `event change(value: Number), emit change(n)` |
| v-model | `<input v-model="q">` | `Input(bind: q, label: "Search")` |
| Shared state | `provide/inject, Pinia` | `store Cart { … }, read as Cart.items after use Cart` |
| A route | `vue-router { path: "/users/:id", component: User }` | `page User(path: "/users/:id", id: String) { … }` |
| v-if / v-else | `<Panel v-if="open" /><p v-else>…</p>` | `if open { Panel } else { Text("…") }` |
| v-show | `<Panel v-show="open" />` | `show open { Panel }` |
| v-for | `<li v-for="i in items" :key="i.id">{{ i.name }}</li>` | `for i in items by i.id { Text(i.name) }` |
| Class bindings | `:class="{ done: todo.done }"` | `class: { "done": todo.done }` |
| Transitions | `<Transition name="fade">` | `.fadeIn, exit: .fadeOut` |
| Fetching | `<Suspense> with an async setup, or VueUse's useFetch("/api/users")` | `resource users: [User] = http.get("/api/users"), read with match users { … }` |
| A test | `Vitest with Vue Test Utils: mount, trigger("click"), text()` | `test "…" { click "Save"  expect "Saved" }` |

## Things that work differently

**One file, one language.** A component is declared in `.wf` with its state, its markup and its handlers together; there is no `<template>`, no `<script>` block and no `.value` to remember. A component's body runs once, and only what reads a changed value updates.

**The markup is typed.** The compiler knows the props, flags, events and slots of every built-in and of every component you declare. A misspelled prop, an unknown flag (`Button("Save").primray`) or a handler for an event the component does not fire stops the build, with the fix suggested.

**Typed without a second language.** There is no `lang="ts"` and no `vue-tsc`: a `.wf` file is typed by the compiler that builds it. The checker infers what it can and asks you to write the rest: a state whose first value says nothing (`state items: [Todo] = []`), an action that returns (`-> Number`), a parameter. There is no `any`: JSON whose shape is open is `Json`, matched or decoded (`User.decode(raw)`) before it is read, and no setting turns the checks off.

**Pages own their routes.** There is no router file. Each `page` names its `path:`, and the compiler builds the router from every page.

**No imports.** Every file under `src/` is part of one program, so a component declared anywhere is usable everywhere, and two declarations of one name are an error.

**Styles are on the element.** A `style { }` block is CSS for that element, with design tokens (`$surface`) and nesting (`&:hover`); there is no `<style scoped>`, and what a selector over many elements needs goes in a `.css` file under `src/`.

**Using it inside a Vue app.** Publish components as custom elements and use their tags in a template ([Publish components as custom elements](https://webfluent.monzeromer.dev/recipes/publish-custom-elements.md)).

## A component, ported

The same counter, in Vue and in WebFluent. It starts at a number the caller passes, counts up on a click, shows a badge when the count is even, and tells the caller about each change.

```vue
<script setup>
import { ref, computed } from "vue";
const props = defineProps({ start: { type: Number, default: 0 } });
const emit = defineEmits(["change"]);
const n = ref(props.start);
const even = computed(() => n.value % 2 === 0);
function inc() { n.value++; emit("change", n.value); }
</script>

<template>
  <div class="row">
    <button @click="inc">+</button>
    <span>{{ n }}</span>
    <span v-if="even" class="badge">even</span>
  </div>
</template>
```

```wf
page Scores(path: "/", title: "Scores", description: "A counter that reports each change.") {
    state last = 0
    Heading("Scores").h1
    Counter(start: 2) { on change(value) { last = value } }
    Text("Last change: {last}")
}

component Counter(start: Number = 0) {
    event change(value: Number)
    state n = start
    derived even = n % 2 == 0
    Row(gap: .sm, align: .center) {
        Button("+") { on click { n = n + 1  emit change(n) } }
        Text("{n}")
        if even { Badge("even").info }
    }
}
```

- **No `.value`.** A state is read and assigned by its name.
- **`derived` replaces `computed`**, and knows what it reads.
- **The event is declared** where the component is, and checked where it is handled.

## Data, from a service

VueUse's `useFetch` hands back separate refs for the data, the error and whether a request is running, and the type of the data is whatever you assert:

```vue
<script setup lang="ts">
import { useFetch } from "@vueuse/core";
const { data, error, isFetching } = useFetch("/api/people").json<Person[]>();
</script>

<template>
  <p v-if="isFetching">Loading…</p>
  <p v-else-if="error">{{ error }}</p>
  <p v-else v-for="p in data" :key="p.id">{{ p.name }}</p>
</template>
```

In WebFluent the service is described once, in an `api` block, and a page reads it with a `resource`:

```wf
type Person { id: String, name: String }
type NewPerson { name: String }

api Backend(base: "/api") {
    get people() at "people" -> [Person]!
    post addPerson(body: NewPerson) at "people" -> Person!
        invalidates [people]
}

page People(path: "/", title: "People", description: "The team.") {
    state said = ""
    resource people = Backend.people()
    Heading("People").h1
    match people {
        .loading { Spinner }
        .failed(e) { Alert(e.message).danger }
        .loaded(list) { for p in list by p.id { Text(p.name) } }
    }
    Button("Add Grace") {
        on click {
            match await Backend.addPerson(body: NewPerson(name: "Grace")) {
                .ok(p) { said = "Added {p.name}" }
                .err(e) { said = e.message }
            }
        }
    }
    Text(said)
}
```

- **The contract is written once.** `-> [Person]!` says what the endpoint answers and that it may fail; what arrives is checked against it, and a server that breaks it fails as `.decode`. A service with an OpenAPI spec gets this block written for it by `wf api add`.
- **A read is one value.** A `match` that leaves out `.failed` does not build, so the page always says what happens when the request fails.
- **A call answers with a result.** `.ok(p)` or `.err(e)`, never a throw; `e` is an `HttpError` (offline, a timeout, a status) with a `.message`.
- **`invalidates [people]`** on the write asks the list again once the write succeeds, with no refetch written by hand.
- **A test needs no server.** A `fake` answers the endpoints, held to the same contract ([Testing](https://webfluent.monzeromer.dev/learn/16-testing.md)).
