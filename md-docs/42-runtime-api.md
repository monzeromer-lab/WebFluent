# 42. Runtime API

<!--
route: guide/runtime
group: reference
blurb: window.WF — the runtime every compiled page calls, usable from a script of your own, the console or a library's callback.
description: The runtime reference, window.WF: signals, elements, motion, routing, data, stores and their inspection, widgets, safety and boot.
-->

`app.js` opens with the runtime, and the page can reach it as `window.WF`.
Compiled code calls it; so may a hand-written script in `public/`, a
library's callback, or you in the browser's console:

```js
WF.navigate("/about");
WF.toast("Saved", "success");
WF.storeSnapshot("Cart");
```

A build keeps only the modules its own code reaches, so a script calling a
module nothing else uses needs `"build": { "runtime": "full" }`
([chapter 32](32-javascript-interop.md)). Most names here are what the
compiler emits for the language's features; the ones worth calling yourself
are marked **●**.

## State — `core`

| Function | Does |
|---|---|
| `signal(value)` ● | A reactive value: call it to read, `.set(v)`, `.update(f)`, `.subscribe(f)` |
| `computed(fn)` ● | A value computed from signals, recomputed when they change |
| `effect(fn)` ● | Runs `fn`, and again when a signal it read changes; a returned function is its cleanup |
| `scoped(fn)`, `onCleanup(fn)` | Ownership: what `fn` creates is disposed of with its scope; `onCleanup` registers a disposal |
| `every(ms, fn)`, `after(ms, fn)`, `listen(target, event, fn)` | A timer or listener tied to the current scope |
| `ref()` | An element handle, filled when the element is made |

## Elements and bodies — `core`, `when`, `each`, `show`, `match`, `slot`

| Function | Does |
|---|---|
| `el(tag, attrs, …children)` | Makes an element; a function attribute or child follows the signals it reads |
| `text(v)`, `props(given, defaults)`, `classes(node, fn)`, `mark(node, attrs)` | A text node, a component's props, live classes, animation marks |
| `onRoot(node, event, fn)` | A handler on a component's root element |
| `when(parent, cond, then, else, anim)` | `if`/`else` |
| `each(parent, list, item, options)` | `for`, keyed or not |
| `show(parent, cond, body, anim)` | `show` |
| `match(parent, key, arg, arms)` | `match` |
| `slot(…)` | A component's slot fill |
| `emit(props, event, …args)` | Fires a component's declared event |
| `onKey(node, combination, fn)`, `keyIs(event, combination)` | `on key("ctrl+k")` |

## Motion — `motion`

| Function | Does |
|---|---|
| `animate(target, name, duration)` ● | Plays an animation now; returns a handle with `play()`, `cancel()`, `finished` |
| `replay(node, name)` ● | Plays an element's own animation again |
| `animateIn`, `animateOut(node, name, duration, delay, easing)` | Enter and exit |
| `expand(node, open, duration, easing)` | Opens or closes to the measured height |
| `countTo(node, from, to, duration, format)`, `counted(…)` | A number that counts |
| `onEnterView(node, name, …)` | Plays when scrolled to |
| `shared(node, name)` | An element carried across a route change |

## Routing — `core`, `router`, `pages`

| Function | Does |
|---|---|
| `navigate(path)` ● | Moves to a route (a page load in a static build) |
| `params()` ● | The current route's parameters |
| `router(routes, container, options)` | The router `Router` compiles to |
| `activeLink(a, href, prefix)` | Keeps a link's `.active` and `aria-current` |
| `page(name, fn)`, `loadPage`, `loadSheet` | Page chunks and their stylesheets |
| `setBasePath(path)`, `setSsgMode(on)` | Set at boot from the config |

## Browser values — `browser`, `core`

`viewport()`, `query()`, `hash()`, `now(every)`, `network()` and `theme()`
read what the language's browser values hold; `setTheme(choice)` ●
switches the theme.

## Data — `net`, `http`, `api`

| Function | Does |
|---|---|
| `resource(url, options)` | `{ state, data, error, items, hasMore, reload, loadMore, invalidate, cancel }` |
| `request(url, options)`, `fetch(url, options)` ● | A request through the engine: retries, timeouts, cache, typed errors; resolves to the parsed body |
| `send(url, options)` | The engine's request itself |
| `invalidate(key)` | Drops a cached answer |
| `optimistic(holder, change)`, `attempt(fn)` | A change shown now and rolled back if `fn` throws |
| `beacon(url, data)` ● | A post that survives unloading |
| `api(definition)` | A service `api` compiles to |
| `ws(url, options)`, `sse(url, options)`, `broadcast(name, options)` | A socket, a stream, a channel |
| `rtc(options)` | A peer — module `peer` |

## Stores — `store`, `keep`

| Function | Does |
|---|---|
| `store(name, define, options)` | A store, built on first read |
| `dropRouteStores()` | Drops the `.route` stores, on a route change |
| `watchStores(fn)` ● | Calls `fn` with every action and the state either side of it; returns the function that stops it |
| `storeSnapshot(name?)` ● | What every store — or one — holds |
| `restoreStore(name, values)` ● | Puts a store back to what it held |
| `persist(…)` | A `persist` value's storage, versions and migrations |

Nothing is recorded while nothing is watching.

## Widgets — `toast`, `overlay`, `announce`, `field`, `tooltip`, `carousel`, `drawer`, `form`

| Function | Does |
|---|---|
| `toast(message, tone, ms)` ● | Shows a toast: `"success"`, `"danger"`, `"warning"`, `"info"`; 3 seconds by default |
| `announce(text)` ● | Says `text` to screen readers, politely |
| `dialog`, `popup`, `tabs`, `menu`, `drawer`, `tooltip`, `carousel`, `field` | What the matching built-ins compile to |
| `form(options)`, `validate(…)` | A `Form` handle and its rules |

## Safety — `core`, `sanitize`

| Function | Does |
|---|---|
| `safeUrl(url)` ● | The URL where a browser may follow it, `""` where it may not |
| `sanitize(html)` ● | Markup through the allow-list |
| `jsonAttr(raw)` | An attribute holding JSON, as the prop it stands for |

Both `safeUrl` and `sanitize` have a build-time twin, held to one table of
cases.

## Text — `text`, `format`

`markdown(text)` ● renders the language's Markdown to HTML;
`highlight(code, lang)` ● colours `wf`, `json`, `bash` or `css`;
`format(value, style, option)` ● and `ago(date)` ● format in the locale.

## Values — `helpers`, `scalars`

The list and string helpers (`sortBy`, `groupBy`, `unique`, `take`,
`first`, `last`, `capitalize`, `truncate`, `dedent`, `lines`, `words`,
`range`, `removeAt`) and the built-in types' methods (`plus`, `minus`,
`until`, `year` … `inZone`, `money`, `times`, `convert`, `host`, `path`,
`domain`, `mix`, `lighten`, `darken`, `alpha`, `contrast`, `preview`,
`uuid`, `dateOf`, `timeOf`, `urlWith`, `urlQuery`), and `caseOf(v)` and
`payload(v)` for an enum case — each a function of the value it works on.

## Everything else

| Module | Functions |
|---|---|
| `i18n` | `locales(default, tables)` → `WF.i18n`, with `t`, `locale`, `dir`, `setLocale` |
| `offline` | `offline(options)`, `update()` — the service worker and the update flow |
| `head` | `head(tags)` — a page's own head tags |
| `host` | `attach(node, mount, update, cleanup)` — what `Host` compiles to |
| `picture` | `picture(…)` — an `image`'s `<picture>` |
| `hydrate` | `hydrate(fn, container)` — takes over a pre-rendered page |
| `core` | `mount(fn, container)`, `mainOf(root)` — boot |
| `debug` | `__debug`, `__reg` — for WebFluent Studio |

## Next

[Grammar](43-grammar.md).
