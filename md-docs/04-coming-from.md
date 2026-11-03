# 4. Coming from React, Vue or Svelte

<!--
route: guide/coming-from
group: start
blurb: What you already know, mapped to what is here — and the few places WebFluent works differently.
description: A map from React, Vue, Svelte and HTMX concepts to WebFluent: state, effects, components, props, slots, context, routing, fetching and styling.
-->

If you have built with a JavaScript framework, most of WebFluent will feel
familiar: components with props, reactive state, a router, a store. This
chapter maps those ideas across, then explains the places where WebFluent
works differently.

The one thing to remember: WebFluent is a compiler with typed markup, not a
library, so a component's body runs once and only the parts that read a
changed value update.

In this chapter:

- [The map](#the-map) from framework concepts to WebFluent
- [One component, side by side](#one-component-side-by-side) in React, Vue,
  Svelte and WebFluent
- [What works differently](#what-works-differently)
- [Things you might look for](#things-you-might-look-for), such as npm
  packages and server rendering

## The map

Find the idea you know in the first column:

| You know | In WebFluent | Chapter |
|---|---|---|
| `useState`, `ref()`, `let x` (Svelte) | `state count = 0` | [State](12-state-and-reactivity.md) |
| `useMemo`, `computed()`, `$:` / `$derived` | `derived total = …` | [State](12-state-and-reactivity.md#derived) |
| `useEffect`, `watchEffect`, `$effect` | `effect { … cleanup { … } }` | [State](12-state-and-reactivity.md#effect) |
| `useRef` on an element | `Input(ref: box)`, then `box.focus()` | [State](12-state-and-reactivity.md#element-handles) |
| a component function / SFC | `component Name(props) { … }` | [Components](15-components.md) |
| props | typed props: `component Card(_ title: String, tone: Tone = .calm)` | [Components](15-components.md) |
| `children`, default slot | `slot` + `children` | [Components](15-components.md#slots) |
| named slots, render props | `slot trailing`, scoped slots `slot row(item: T)` | [Components](15-components.md#slots) |
| callback props, `$emit`, `createEventDispatcher` | `event change(value: Number)` + `emit change(n)` | [Components](15-components.md#events) |
| `v-model`, `bind:value` | `Input(bind: name)` | [Forms](23-forms.md) |
| context, provide/inject, a global store | `store Cart { … }`, read as `Cart.items` | [Stores](16-stores.md) |
| Redux / Pinia / Zustand | a `store` with `action`s; `wf serve` shows every action | [Stores](16-stores.md) |
| React Router / Vue Router / SvelteKit routes | `page Name(path: "/users/:id")` — pages own their routes | [Pages](10-pages-and-routing.md) |
| layouts | `page P(layout: Shell)` | [Pages](10-pages-and-routing.md#layouts) |
| route guards | `page P(guard: Auth.loggedIn, redirect: "/login")` | [Pages](10-pages-and-routing.md#guards) |
| `{cond && <X/>}`, `v-if`, `{#if}` | `if cond { X }` | [Control flow](14-control-flow.md) |
| `.map()` in JSX, `v-for`, `{#each}` | `for item in items by item.id { … }` | [Control flow](14-control-flow.md#for) |
| TanStack Query / SWR | `resource` and `api` with cache, retry, dedupe | [Fetching data](24-data.md) |
| TypeScript | a built-in, gradual type checker; types are optional | [Types](17-types.md) |
| CSS modules / scoped styles | `style { }` on the element, with tokens and nesting | [Styling](19-styling.md) |
| Tailwind utility classes | flags (`.primary.lg`), layout props, design tokens | [Styling](19-styling.md) |
| Framer Motion / transitions | `animate:`, `exit:`, `stagger:`, route transitions | [Motion](20-motion.md) |
| react-i18next / vue-i18n | `t("key", { name: x })`, `setLocale("ar")` | [i18n](28-i18n.md) |
| Next.js static export / Astro | `build.ssg: true` | [Static or SPA](33-static-and-spa.md) |
| Testing Library + Playwright | `test "…" { … click "Save"  expect "Saved" }` | [Testing](29-testing.md) |
| ESLint a11y plugin | accessibility checks on every build | [Accessibility](22-accessibility.md) |

## One component, side by side

Here is the same counter in each framework: it starts at a number the caller
passes, counts up on a click, shows a badge when the count is even, and tells
the caller about each change.

React:

```jsx
function Counter({ start = 0, onChange }) {
  const [n, setN] = useState(start);
  const even = useMemo(() => n % 2 === 0, [n]);
  return (
    <div className="row">
      <button onClick={() => { setN(n + 1); onChange?.(n + 1); }}>+</button>
      <span>{n}</span>
      {even && <span className="badge">even</span>}
    </div>
  );
}
```

Vue:

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

Svelte:

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

WebFluent:

```wf
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

What changed:

- **No hooks and no dependency arrays.** `derived` knows what it read.
- **No setter.** You assign: `n = n + 1`.
- **No `className`.** `Badge` comes designed, and `.info` is a flag the
  compiler checks.
- **The event is declared.** A caller that handles `on chnage` gets error
  `E103` at build time, instead of a handler that never runs.

A page places the counter and handles its event like this:

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

The page shows `2`, the **even** badge and `Last change: 0`. After one
click it shows `3`, no badge, and `Last change: 3`.

## What works differently

**It is a compiler, not a library.** There is no virtual DOM and no
re-render of a component. Each text, attribute or style that reads a state
subscribes to it and updates on its own. A component's body runs once, when
the component is placed.

**The markup is typed.** The compiler knows the props, flags, events and
slots of every built-in, and of every component you declare.
`Button("Save").primray` does not build. Unknown props do not pass through
to the DOM silently:

- On a built-in, a named argument it does not declare is written to the
  element as an HTML attribute, with warning `C04` — unless it is `aria-*`,
  `data-*` or a global attribute such as `id`, `role` or `title`, which draw
  no warning.
- On your own component, a named argument it does not declare is error
  `C02`. The exceptions are the same attribute families (`aria-*`, `data-*`,
  `id`, `role`, `title`, `hidden`), which go to the component's root
  element.

**Pages own their routes.** There is no route table and no file-system
routing. Each `page` names its `path:`, and the compiler builds the router
from them, ordered by specificity, so the order you declare pages in cannot
hide a route.

**No imports.** Every file under `src/` is part of one program. A component
declared anywhere is usable everywhere, and two declarations with one name
are an error.

**Styling lives on the element.** A `style { }` block is real CSS scoped to
that element, and `$token` reads a design token. There is no CSS-in-JS
runtime: the build compiles static rules into the stylesheet.

**The build is the site.** Pre-rendering, code splitting, precompression,
image resizing, a sitemap, Open Graph tags and a Content-Security-Policy all
come from `wf build`, with no plugins to choose.

**Safe by construction.** Text is always text. The only way to put markup
on a page is `Unsafe.Html(…)`, which the build flags every time. Inline
handlers and `javascript:` URLs do not compile. See [Security](30-security.md).

## Things you might look for

- **npm packages.** There is no package manager for WebFluent code. A
  JavaScript library is a URL in `meta.scripts`. Your own JavaScript is a
  plain `.js` file under `src/`, and `.wf` code calls its functions by name.
  `mount:` and `cleanup:` hand any element to a script for as long as the
  element exists. [JavaScript interop](39-javascript-interop.md) shows a
  chart library end to end.
- **Server components, or rendering on every request.** WebFluent does not
  do this. A WebFluent site is static files plus a browser runtime, served
  by any host and talking to your API. For HTML generated on a server, such
  as emails, invoices and fragments, see
  [Server rendering](38-server-rendering.md).
- **WebFluent inside an existing React or Vue app.** Publish components as
  custom elements and place their tags:
  [Custom elements](39-javascript-interop.md#publish-your-components-as-custom-elements).
- **HTMX.** The closest thing to `hx-get` is a `resource` whose URL reads
  state, and the closest thing to `hx-post` is an action that `await`s an
  `api` endpoint. The page stays static HTML until its script arrives, then
  updates in place.

## Next

[Tutorial: Todos](05-tutorial-todos.md): build a todo list kept in the reader's browser — types, a store that remembers, a component, a filter in the address and tests that click.
