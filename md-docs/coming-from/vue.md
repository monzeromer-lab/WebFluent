---
kind: "coming-from"
route: "/coming-from/vue"
framework: "Vue"
language: "vue"
description: "Vue ideas in WebFluent: refs, computed, watchers, props, slots, emits, v-model, Pinia, routing, v-if and v-for, side by side."
---
# WebFluent for Vue developers

Vue's single-file components, refs, computed values, slots and `v-model`
all have a close match in WebFluent. The difference to hold on to is that
WebFluent is a compiler with typed markup: there is no `.value`, no
template language beside the script, and every prop, slot and event is
checked at build time.

## The map

| Concept | Vue | WebFluent |
|---|---|---|
| A value that changes | `const n = ref(0)` | `state n = 0` |
| Changing it | `n.value++` | `n = n + 1` |
| A value worked out from others | `const total = computed(() => sum(items.value))` | `derived total = items.sum()` |
| A side effect | `watchEffect((onCleanup) => { start(); onCleanup(stop) })` | `effect { start()  cleanup { stop() } }` |
| A handle on an element | `<input ref="box">` … `box.value.focus()` | `Input(bind: q, ref: box)` … `box.focus()` |
| A component | a `.vue` file with `<script setup>` and `<template>` | `component Card(_ title: String) { … }` |
| Props | `defineProps({ title: String })` | `component Card(_ title: String, tone: Tone = .calm)` |
| The default slot | `<slot />` | `children` |
| A named slot | `<slot name="trailing" />` | `slot trailing`, filled as `trailing { … }` |
| A scoped slot | `<slot :item="item" />` | `slot row(item: Todo)`, filled as `row(todo) { … }` |
| An event | `defineEmits(["change"])`, `emit("change", n)` | `event change(value: Number)`, `emit change(n)` |
| `v-model` | `<input v-model="q">` | `Input(bind: q, label: "Search")` |
| Shared state | `provide`/`inject`, Pinia | `store Cart { … }`, read as `Cart.items` after `use Cart` |
| A route | vue-router `{ path: "/users/:id", component: User }` | `page User(path: "/users/:id", id: String) { … }` |
| `v-if` / `v-else` | `<Panel v-if="open" /><p v-else>…</p>` | `if open { Panel } else { Text("…") }` |
| `v-show` | `<Panel v-show="open" />` | `show open { Panel }` |
| `v-for` | `<li v-for="i in items" :key="i.id">{{ i.name }}</li>` | `for i in items by i.id { Text(i.name) }` |
| Class bindings | `:class="{ done: todo.done }"` | `class: { "done": todo.done }` |
| Transitions | `<Transition name="fade">` | `.fadeIn`, `exit: .fadeOut` |

## Things that work differently

**One file, one language.** A component is declared in `.wf` with its
state, its markup and its handlers together; there is no `<template>`, no
`<script>` block and no `.value` to remember. A component's body runs once,
and only what reads a changed value updates.

**The markup is typed.** The compiler knows the props, flags, events and
slots of every built-in and of every component you declare. A misspelled
prop, an unknown flag (`Button("Save").primray`) or a handler for an event
the component does not fire stops the build, with the fix suggested.

**Pages own their routes.** There is no router file. Each `page` names its
`path:`, and the compiler builds the router from every page.

**No imports.** Every file under `src/` is part of one program, so a
component declared anywhere is usable everywhere, and two declarations of
one name are an error.

**Styles are on the element.** A `style { }` block is CSS for that element,
with design tokens (`$surface`) and nesting (`&:hover`); there is no
`<style scoped>`, and what a selector over many elements needs goes in a
`.css` file under `src/`.

**Using it inside a Vue app.** Publish components as custom elements and
use their tags in a template
([Publish components as custom elements](../recipes/publish-custom-elements.md)).

## A component, ported

The same counter, in Vue and in WebFluent. It starts at a number the caller
passes, counts up on a click, shows a badge when the count is even, and
tells the caller about each change.

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
- **The event is declared** where the component is, and checked where it is
  handled.
