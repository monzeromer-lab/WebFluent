---
chapter: 46
kind: "reference"
route: "/reference/runtime"
group: "reference"
blurb: "window.WF — the runtime every compiled page calls, usable from a script of your own, the console or a library's callback."
description: "The runtime reference, window.WF: signals, elements, motion, routing, data, stores and their inspection, widgets, safety, text and boot, module by module."
---
# 46. Runtime API

This chapter is the reference for `window.WF`, the runtime every compiled
page calls. Look up a function here when a script of your own, a library's
callback or the browser's console needs to do what a page does — navigate,
show a toast, read a store.

`app.js` opens with the runtime, and the page reaches it as `window.WF`.
This plain script, in `src/`, moves to a route, shows a toast and logs what
a store holds:

```js
// src/support.js
function openHelp() {
  WF.navigate("/about");
  WF.toast("Saved", "success");
  console.log(WF.storeSnapshot("Cart"));
}
```

## Calling the runtime

These rules apply to every function in this chapter:

- **Who calls it.** The compiled pages call the runtime. So may a project
  script under `src/` or in `meta.scripts`, a library's callback, or you in
  the browser's console.
- **Which functions to call.** Most names here are what the compiler emits
  for a language feature, listed so you can read the output. The ones
  meant for your own code are marked **●**.
- **Modules.** The runtime is a set of feature modules. Each table names
  the module a function belongs to, and a build ships only the modules its
  own code reaches. A script that calls a module nothing else uses needs
  `"build": { "runtime": "full" }`
  ([JavaScript interop](../recipes/javascript-interop.md)).

## Events the runtime dispatches

`wf:render` is dispatched on `document` once a drawing lands: the first
mount, the takeover of a pre-rendered page, and every route change. Its
`detail` is `{ route, params }`. A script that finds elements itself
listens for it:

```js
// src/highlight-tables.js
document.addEventListener("wf:render", (e) => {
  for (const t of document.querySelectorAll("table.sortable")) t.dataset.route = e.detail.route;
});
```

See also: [JavaScript interop](../recipes/javascript-interop.md#listen-for-wf-render-in-a-script-that-finds-elements-itself).

## State

| Function | Module | What it does |
|---|---|---|
| `signal(value)` ● | `core` | A reactive value: call it to read it; `.set(v)`, `.update(f)` and `.subscribe(f)` change and watch it. |
| `computed(fn)` ● | `core` | A value computed from signals, recomputed when they change. |
| `effect(fn)` ● | `core` | Runs `fn`, and again when a signal it read changes. A function `fn` returns is its cleanup, run before the next run and when the effect is disposed. An effect that changes what it reads on every run is stopped after 100 nested runs (with a warning under `wf serve`). |
| `scoped(fn)` | `core` | Runs `fn` in a new scope: what it creates is disposed of with the scope. |
| `onCleanup(fn)` | `core` | Registers `fn` to run when the current scope is disposed. |
| `every(ms, fn)`, `after(ms, fn)` | `core` | A repeating or one-shot timer that stops with the current scope. |
| `listen(target, event, fn, options)` | `core` | An event listener removed with the current scope. |
| `ref()` | `core` | An element handle, filled when the element is made; used as the element itself (`name.focus()`, `name.value`). |

## Elements

| Function | Module | What it does |
|---|---|---|
| `el(tag, attrs, …children)` | `core` | Makes an element. A function given as an attribute or a child follows the signals it reads. |
| `text(fn)` | `core` | A text node that follows what `fn` reads. |
| `props(given, defaults)` | `core` | A component's props, with defaults filled in. |
| `classes(node, fn)` | `core` | Live classes: the classes `fn` named last time come off, the ones it names now go on. |
| `mark(node, attrs)` | `core` | Puts `data-wf-exit`, `data-wf-delay`, `data-wf-duration` and `data-wf-animate` on a component's root, for the bodies below to play. |
| `rootAttrs(node, attrs)` | `core` | Writes the attributes a component call passes onto the component's root element. |
| `onRoot(node, event, fn)` | `core` | A handler on a component's root element. |
| `bound(control)` | `core` | What a bound control holds, typed: a number field's number (`null` when empty), a select's option value as it was written, otherwise the text. |
| `emit(props, event, …args)` | `core` | Fires a component's declared event. |

## Bodies

What the control-flow statements and slots compile to.

| Function | Module | What it does |
|---|---|---|
| `when(parent, cond, then, else, anim)` | `when` | `if` / `else`. |
| `held(read)` | `when` | What an `if let` binding reads: the value now, or the last one while the branch leaves. |
| `each(parent, list, item, options)` | `each` | `for`, keyed or not. Keyed items keep their nodes across inserts, removals and moves, and with `live` across a new value too. |
| `show(parent, cond, body, anim)` | `show` | `show`. |
| `match(parent, key, arg, arms)` | `match` | `match`. An arm is called with the value and an accessor of it, and drawn again when the arm changes. |
| `slot(parent, values, render)` | `slot` | A component's slot, drawing the caller's fill. |
| `windowed(node, listFn, itemFn, options)` | `windowed` | A `VirtualList`'s keyed `for`. |

## Keys, focus and gestures

| Function | Module | What it does |
|---|---|---|
| `onKey(node, combination, fn)` | `core` | `on key("mod+k")`. |
| `keyIs(event, combination)` | `core` | `keyIs(e, "mod+k")`: whether a key event is that combination. |
| `keyLabel(combination)` | `core` | `keyLabel("mod+k")`: how this system writes the combination (`⌘K`, `Ctrl+K`). |
| `keyAria(combination)` | `core` | The combination as `aria-keyshortcuts` writes it, for a drawn `shortcut:`. |
| `trapFocus(node, read)` | `layer` | `trapFocus: open`. |
| `gesture(node, name, fn)` | `gesture` | `on longpress(p)`, `on swipe(s)`, `on receive(d)`, `on dragmove(d)`. |
| `draggable(node, value, options)`, `dropTarget(node, options)` | `gesture` | `drag:`, `dragGroup:`, `dropValue:`, `dropAxis:`, `dropInside:`. |
| `splitter(node, read, write, options)` | `gesture` | `Splitter(bind:, min:, max:, reset:)`. |

## Motion

| Function | Module | What it does |
|---|---|---|
| `animate(target, name, duration)` ● | `motion` | Plays an animation now, and returns a handle with `play()`, `cancel()` and `finished`. |
| `replay(node, name, duration)` ● | `motion` | Plays an element's animation again from the start (`replayAnimation` in `.wf`). |
| `animateIn(node, name, duration, delay, easing)`, `animateOut(…)` | `motion` | Enter and exit animations. |
| `expand(node, open, duration, easing)` | `motion` | Opens or closes a box to its measured height. |
| `countTo(node, from, to, duration, format)`, `counted(node, valueFn, duration, format)` | `motion` | A number that counts to its new value. |
| `onEnterView(node, name, duration, delay, easing)` | `motion` | Plays an animation when the element is scrolled into view. |
| `shared(node, name)` | `motion` | An element carried across a route change. |

## Routing and pages

| Function | Module | What it does |
|---|---|---|
| `navigate(path)` ● | `core` | Moves to a route (a page load in a static build). |
| `params()` ● | `router` | The current route's parameters. |
| `router(routes, container, options)` | `router` | The router `Router` compiles to; `options.transition` is `fade` or `slide`. |
| `activeLink(a, href, prefix)` | `router` | Keeps a link's `.active` class and `aria-current`. |
| `page(name, fn)`, `loadPage(name, cb)`, `loadSheet(name, cb)` | `pages` | Page chunks and their stylesheets. |
| `setBasePath(path)`, `setSsgMode(on)` | `core` | Set at boot from the config. |

## Browser values

| Function | Module | What it does |
|---|---|---|
| `viewport()`, `query()`, `hash()`, `network()` | `browser` | What the language's `viewport`, `query`, `hash` and `network` hold. |
| `now(options)` | `browser` | `now`, kept current every minute, or as often as `options.every` asks. |
| `theme()` | `core` | The theme the reader chose. |
| `setTheme(choice)` ● | `core` | Switches the theme: `"dark"`, `"light"` or `"system"`, kept across visits. |

## Data and connections

| Function | Module | What it does |
|---|---|---|
| `resource(url, options)` | `net` | A resource: `{ state, data, error, refreshing, pending, items, hasMore, reload, loadMore, invalidate, cancel }`. |
| `request(url, options)`, `fetch(url, options)` ● | `net` | A request through the engine — retries, timeouts, cache, typed errors — that resolves to the parsed body. `await fetch(…)` in an action compiles to `request`. |
| `optimistic(holder, change)` | `net` | Shows a change at once, rolled back if the action throws. |
| `attempt(fn)` | `net` | Runs an action so that a throw rolls back its optimistic changes. |
| `send(url, options)` | `http` | The engine's request itself. |
| `invalidate(prefix)` | `http` | Drops cached answers: every one with no argument, or each key equal to or starting with `prefix`. |
| `beacon(url, data)` ● | `http` | A post that survives the page unloading. |
| `api(definition)` | `api` | The service an `api` declaration compiles to. |
| `ws(url, options)`, `sse(url, options)`, `broadcast(name, options)` | `api` | A socket, a stream of events, a channel between tabs. |
| `rtc(options)` | `peer` | A peer: a WebRTC data channel. |
| `worker(url, options)` | `worker` | A `worker`: a script of the site run beside the page. |

## Stores

| Function | Module | What it does |
|---|---|---|
| `store(name, define, options)` | `store` | A store, built the first time something reads it. |
| `dropRouteStores()` | `store` | Drops the `scope: .route` stores, on a route change. |
| `watchStores(fn)` ● | `store` | Calls `fn` with every action and the state on each side of it, and returns the function that stops it. Nothing is recorded while nothing is watching. |
| `storeSnapshot(name?)` ● | `store` | What every store, or one, holds. |
| `restoreStore(name, values)` ● | `store` | Puts a store back to what it held. |
| `persist(base, initial, policy)` | `keep` | A `persist` value's storage, versions and migrations. |

## Widgets and layers

| Function | Module | What it does |
|---|---|---|
| `toast(message, tone, ms)` ● | `toast` | Shows a toast in tone `"success"`, `"danger"`, `"warning"` or `"info"` (the default), for `ms` milliseconds (3000 by default). |
| `announce(text)` ● | `announce` | Says `text` to screen readers, politely. |
| `dialog(el, read, write)`, `tabs(nav, active)` | `overlay` | What `Modal` and `Dialog`, and `Tabs`, compile to. |
| `popup(root, trigger, open)` | `overlay` | Closes a popup on Escape or a click outside, and returns focus to its trigger. |
| `layer(el, read, write, options)` | `layer` | A layer drawn in the browser's top layer at an anchor or a point: what `Popover` compiles to. |
| `menu`, `submenu`, `tooltip` | `layer` | What `Menu`, `Menu.Sub` and `Tooltip` compile to. |
| `placeAt(anchor, size, viewport, placement, offset)` | `layer` | Where a layer of `size` goes against `anchor`, flipped and slid to stay on screen. |
| `uid(prefix)` | `layer` | An id no other element of the page has. |
| `drawer(panel, toggle, scrim)` | `drawer` | Wires an off-canvas panel to its toggle button: a `Sidebar`'s or a `Navbar`'s links on a narrow screen. |
| `carousel(root, options)` | `carousel` | What `Carousel` compiles to. |
| `field(control, options)`, `choiceError(label, check)` | `field` | A control's label, hint and error; a `Checkbox` or `Switch`'s rule message. |
| `form(options)`, `validate(read, rules, options)` | `form` | A `Form` handle and a `validate` block's rules. |

## Safety

| Function | Module | What it does |
|---|---|---|
| `safeUrl(url)` ● | `core` | The URL where a browser may follow it, `""` where it may not. |
| `safePicture(url)` | `core` | As `safeUrl`, and also a picture carried in its address (`data:image/…`), for an image's `src`. |
| `allowSchemes(schemes)` | `core` | Adds an app's own schemes (from `meta.frame`) to those `safeUrl` allows; called once at boot. |
| `sanitize(html)` ● | `sanitize` | Markup through the allow-list. |
| `jsonAttr(raw)` | `core` | An attribute holding JSON, as the prop it stands for. |

`safeUrl` and `sanitize` each have a build-time twin, and both twins are
held to one shared table of cases.

## Text and formatting

| Function | Module | What it does |
|---|---|---|
| `markdown(text)` ● | `text` | Renders the language's Markdown to HTML. |
| `highlight(code, lang)` ● | `text` | Colours `wf`, `json`, `bash` or `css` with `wf-tok-*` spans. |
| `format(value, style, option)` ● | `format` | Formats a value in the page's locale, as `format` in `.wf`. |
| `ago(date)` ● | `format` | A relative time in the page's locale: `3 minutes ago`. |
| `locales(default, tables)` | `i18n` | Sets up `WF.i18n`, with `t`, `locale`, `dir` and `setLocale`. |

## Values

Each of these is a function of the value it works on.

| Functions | Module | What they are |
|---|---|---|
| `sortBy`, `groupBy`, `unique`, `take`, `first`, `last`, `capitalize`, `truncate`, `dedent`, `lines`, `words`, `range`, `removeAt` | `helpers` | The list and string helpers, and `a..b` ranges. |
| `setIn(target, path, value)`, `mutated(list, method, args)` | `helpers` | A state changed in place — `form.name = v`, `items.sort()` — as a copy with that place changed. |
| `year`, `month`, `day`, `weekday`, `hour`, `minute`, `second`, `plus`, `minus`, `isBefore`, `isAfter`, `isSame`, `until`, `startOfDay`, `startOfWeek`, `startOfMonth`, `endOfDay`, `inZone`, `native` | `scalars` | The methods of `Date`, `Time` and `DateTime`. |
| `days`, `hours`, `minutes`, `seconds`, `ms` | `scalars` | The methods of `Duration`. |
| `money`, `times`, `convert` | `scalars` | `Money`. |
| `host`, `path`, `domain`, `urlWith`, `urlQuery` | `scalars` | `Url` and `Email`; `.with()` and `.query()`. |
| `mix`, `lighten`, `darken`, `alpha`, `contrast` | `scalars` | `Color`. |
| `preview`, `uuid`, `dateOf`, `timeOf` | `scalars` | A `File`'s preview, `uuid()`, `.date()` and `.time()`. |
| `caseOf(v)`, `payload(v)` | `core` | An enum value's case and its payload. |

## Boot and interop

| Function | Module | What it does |
|---|---|---|
| `mount(fn, container)` | `core` | Draws the app into `container`. |
| `mainOf(root)` | `core` | The `<main>` the pages draw into: `root` itself, the `<main>` inside it, or a new one. |
| `live(node)` | `core` | Marks `node` as held by something, so a takeover keeps the drawn node rather than the painted one. |
| `hydrate(fn, container)`, `hydrating(root)` | `hydrate` | Draws beside a pre-rendered page and takes it over in place. |
| `attach(node, mount, update, cleanup)` | `host` | What `mount:`, `update:` and `cleanup:` compile to, on `Host` or any element; a component's fragment is attached at its root element. |
| `portal(box, read)` | `host` | What `Portal` compiles to (below). |
| `head(tags)` | `head` | A page's own `head { }` tags. |
| `picture(asset, options)` | `picture` | An `image`'s `<picture>`. |
| `offline(options)`, `update()`, `clearQueued()` | `offline` | The service worker, the update flow, and forgetting the writes kept offline. |
| `loadAfter(scripts)` | `later` | Loads the `meta.scripts` entries marked `"load": "after"` once the page has painted. |
| `__debug`, `__reg` | `debug` | For WebFluent Studio. |

`portal(box, read)` keeps `box` in the element, or `ref:` handle, that
`read()` names. It reads `read()` again when what it reads changes and once
the drawing is over, keeps the box aside while `read()` is `null`, and takes
it away with its scope. It returns what runs the lifetimes its block asked
for while it was drawn.

The `icons` module has no functions: it is the table of glyphs, holding
only the icons the program names.

## Next

[Grammar](grammar.md) — every statement and expression of the language.
