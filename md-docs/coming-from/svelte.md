---
kind: "coming-from"
route: "/coming-from/svelte"
framework: "Svelte"
language: "svelte"
description: "Svelte ideas in WebFluent: $state, $derived, $effect, props, snippets, events, bind:, stores, routes, #if and #each, side by side."
---
# WebFluent for Svelte developers

If you write Svelte, WebFluent will feel close: it is also a compiler, state
is a plain assignment, and only what reads a changed value updates. What
WebFluent adds is a typed markup: every element, prop, flag, event and slot
is checked at build time, and the components come designed.

## The map

| Concept | Svelte | WebFluent |
|---|---|---|
| A value that changes | `let n = $state(0)` | `state n = 0` |
| Changing it | `n += 1` | `n = n + 1` |
| A value worked out from others | `let total = $derived(sum(items))` | `derived total = items.sum()` |
| A side effect | `$effect(() => { start(); return stop })` | `effect { start()  cleanup { stop() } }` |
| A handle on an element | `<input bind:this={box}>` | `Input(bind: q, ref: box)` |
| A component | a `.svelte` file | `component Card(_ title: String) { … }` |
| Props | `let { title, tone = "calm" } = $props()` | `component Card(_ title: String, tone: Tone = .calm)` |
| What the caller puts inside | `{@render children()}` | `children` |
| A snippet the component draws | `{@render row(item)}` | `slot row(item: Todo)`, filled as `row(todo) { … }` |
| An event | a callback prop, `onchange?.(n)` | `event change(value: Number)`, `emit change(n)` |
| Two-way binding | `<input bind:value={q}>` | `Input(bind: q, label: "Search")` |
| Shared state | a store or a `.svelte.js` module | `store Cart { … }`, read as `Cart.items` after `use Cart` |
| A route | SvelteKit `src/routes/users/[id]/+page.svelte` | `page User(path: "/users/:id", id: String) { … }` |
| `{#if}` | `{#if open}<Panel />{:else}<p>…</p>{/if}` | `if open { Panel } else { Text("…") }` |
| `{#each}` | `{#each items as i (i.id)}<li>{i.name}</li>{/each}` | `for i in items by i.id { Text(i.name) }` |
| `{#await}` | `{#await promise}…{:then data}…{:catch e}…{/await}` | `match users { loading { … } ready(list) { … } error(e) { … } }` |
| A class that follows state | `class:done={todo.done}` | `class: { "done": todo.done }` |
| Transitions | `transition:fade` | `.fadeIn`, `exit: .fadeOut` |
| A test | Vitest with Testing Library: `render`, `fireEvent.click`, `getByText` | `test "…" { click "Save"  expect "Saved" }` |

## Things that work differently

**The elements are typed too.** `svelte-check` types your props and the
expressions in your markup; WebFluent also knows every built-in's flags,
events and slots. `Button("Save").primray` does not build, and neither
does a handler for an event the component does not fire (`E103`).

**Elements come designed.** `Button`, `Card`, `Table`, `Modal` and the rest
are built in, with their accessibility work done, and a flag picks a variant
(`.primary`, `.lg`). Your own look is a theme of design tokens and a
`style { }` on the element.

**Pages own their routes.** There is no `src/routes` folder: each `page`
names its `path:`, and the compiler builds the router from every page,
ordered by specificity.

**No imports.** Every file under `src/` is part of one program, so a
component declared anywhere is usable everywhere.

**Fetching is a value.** A `resource` is read with `match`, and repeats when
a state its address reads changes; an `api` block describes a service once,
so every call is typed, cached and cancellable.

**The build is the site.** Pre-rendering, code splitting, image resizing and
a Content-Security-Policy come from `wf build`; there is no server to run.

## A component, ported

The same counter, in Svelte and in WebFluent. It starts at a number the
caller passes, counts up on a click, shows a badge when the count is even,
and tells the caller about each change.

```svelte
<script>
  let { start = 0, onchange } = $props();
  let n = $state(start);
  let even = $derived(n % 2 === 0);
  function inc() { n += 1; onchange?.(n); }
</script>

<div class="row">
  <button onclick={inc}>+</button>
  <span>{n}</span>
  {#if even}<span class="badge">even</span>{/if}
</div>
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

- **The same reactivity**, spelled `state` and `derived`.
- **The event is declared**, so a caller's misspelled handler is a build
  error rather than a prop nobody reads.
- **No `class`.** `Badge` comes designed, and `.info` is checked.
