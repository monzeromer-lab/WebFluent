# 4. Coming from React, Vue or Svelte

<!--
route: guide/coming-from
group: start
blurb: What you already know, mapped to what is here — and the few places WebFluent works differently.
description: A map from React, Vue, Svelte and HTMX concepts to WebFluent: state, effects, components, props, slots, context, routing, fetching and styling.
-->

If you have built with a JavaScript framework, most of WebFluent will feel
familiar: components with props, reactive state, a router, a store. This
chapter maps those ideas across, then lists what is genuinely different.

## The map

| You know | In WebFluent | Chapter |
|---|---|---|
| `useState`, `ref()`, `let x` (Svelte) | `state count = 0` | [State](08-state-and-reactivity.md) |
| `useMemo`, `computed()`, `$:` / `$derived` | `derived total = …` | [State](08-state-and-reactivity.md#derived) |
| `useEffect`, `watchEffect`, `$effect` | `effect { … cleanup { … } }` | [State](08-state-and-reactivity.md#effect) |
| `useRef` on an element | `Input(ref: box)`, then `box.focus()` | [State](08-state-and-reactivity.md#element-handles) |
| a component function / SFC | `component Name(props) { … }` | [Components](11-components.md) |
| props | typed props: `component Card(_ title: String, tone: Tone = .calm)` | [Components](11-components.md) |
| `children`, default slot | `slot` + `children` | [Components](11-components.md#slots) |
| named slots, render props | `slot trailing`, scoped slots `slot row(item: T)` | [Components](11-components.md#slots) |
| callback props, `$emit`, `createEventDispatcher` | `event change(value: Number)` + `emit change(n)` | [Components](11-components.md#events) |
| `v-model`, `bind:value` | `Input(bind: name)` | [Forms](16-forms.md) |
| context, provide/inject, a global store | `store Cart { … }`, read as `Cart.items` | [Stores](12-stores.md) |
| Redux / Pinia / Zustand | a `store` with `action`s; `wf serve` shows every action | [Stores](12-stores.md) |
| React Router / Vue Router / SvelteKit routes | `page Name(path: "/users/:id")` — pages own their routes | [Pages](06-pages-and-routing.md) |
| layouts | `page P(layout: Shell)` | [Pages](06-pages-and-routing.md#layouts) |
| route guards | `page P(guard: Auth.loggedIn, redirect: "/login")` | [Pages](06-pages-and-routing.md#guards) |
| `{cond && <X/>}`, `v-if`, `{#if}` | `if cond { X }` | [Control flow](10-control-flow.md) |
| `.map()` in JSX, `v-for`, `{#each}` | `for item in items by item.id { … }` | [Control flow](10-control-flow.md#for) |
| TanStack Query / SWR | `resource` and `api` with cache, retry, dedupe | [Data](17-data.md) |
| TypeScript | a built-in, gradual type checker; types are optional | [Types](13-types.md) |
| CSS modules / scoped styles | `style { }` on the element, with tokens and nesting | [Styling](15-styling.md) |
| Tailwind utility classes | flags (`.primary.lg`), layout props, design tokens | [Styling](15-styling.md) |
| Framer Motion / transitions | `animate:`, `exit:`, `stagger:`, route transitions | [Motion](20-motion.md) |
| react-i18next / vue-i18n | `t("key", { name: x })`, `setLocale("ar")` | [i18n](21-i18n.md) |
| Next.js static export / Astro | `build.ssg: true` | [Static or SPA](26-static-and-spa.md) |
| Testing Library + Playwright | `test "…" { … click "Save"  expect "Saved" }` | [Testing](24-testing.md) |
| ESLint a11y plugin | accessibility checks on every build | [Accessibility](22-accessibility.md) |

## The same component, three ways

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

What changed: no hooks and no dependency arrays — `derived` knows what it
read. No setter — you assign. No `className` — `Badge` is designed, and
`.info` is a flag the compiler checks. The event is declared, so a caller
handling `on chnage` is an error, not a handler that never runs.

## What is genuinely different

**It is a compiler, not a library.** There is no virtual DOM and no
re-render of a component. Each text, attribute or style that reads a state
subscribes to it and updates alone. A component's body runs once, when it is
placed.

**The markup is typed.** A built-in's props, flags, events and slots are
known to the compiler, and so are yours. `Button("Save").primray` does not
build. There is no "any prop goes through to the DOM" — an unknown named
argument becomes an HTML attribute with a warning, unless it is `aria-*`,
`data-*` or a global attribute.

**Pages own their routes.** There is no route table or file-system routing:
each `page` names its `path:` and the router is built from them, ordered by
specificity.

**No imports.** Every file under `src/` is one program. A component declared
anywhere is usable everywhere; two declarations with one name are an error.

**Styling lives on the element.** A `style { }` block is real CSS scoped to
that element; `$token` reads a design token. There is no separate CSS-in-JS
runtime — static rules are compiled into the stylesheet.

**The build is the site.** Pre-rendering, code splitting, precompression,
image resizing, a sitemap, Open Graph, a CSP — all from `wf build`, with no
plugins to choose.

**Safe by construction.** Text is always text; the only way to inject markup
is `Unsafe.Html(…)`, which the build flags every time. Inline handlers and
`javascript:` URLs do not compile. See [Security](23-security.md).

## Things you might look for

- **npm packages.** There is no package manager for WebFluent code. A
  JavaScript library is used through `external` — a typed declaration of what
  you call — and `Host`, which gives it a DOM node with a lifetime.
  [JavaScript interop](32-javascript-interop.md) shows a chart library end
  to end.
- **Server components / SSR per request.** Not here: a WebFluent site is
  static files plus a browser runtime, served by any host, talking to your
  API. For HTML generated on a server — emails, invoices, fragments — see
  [Server rendering](34-server-rendering.md).
- **Using WebFluent inside an existing React or Vue app.** Publish components
  as custom elements and place the tags:
  [Custom elements](32-javascript-interop.md#publishing-your-components-as-custom-elements).
- **Coming from HTMX.** The closest thing to `hx-get` is a `resource` whose
  URL reads state, and to `hx-post` an action that `await`s an `api`
  endpoint. The page stays static HTML until its script arrives, then updates
  in place.

## Next

[Language basics](05-language-basics.md).
