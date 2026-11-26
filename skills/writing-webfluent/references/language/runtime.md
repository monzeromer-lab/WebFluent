## Runtime

The runtime is the JavaScript library the compiled pages call, exposed as `window.WF`.

`app.js` opens with the runtime, reachable as `window.WF` from the console,
a `script`, or the studio. The compiled pages call it; a hand-written script
may too. Its surface:

| Group | Functions |
|---|---|
| State | `signal(v)` → getter with `.set`/`.update`/`.subscribe`; `effect(fn)`; `computed(fn)` |
| Elements | `el(tag, attrs, …children)` — a thunk attribute or child follows state; `text(v)`; `props(given, defaults)`; `onRoot(node, event, fn)`; `classes(node, fn)`; `bound(control)` — what a bound control holds, typed |
| Bodies | `when(parent, cond, then, else, anim)`, and `held(read)` — what an `if let` binding reads: the value now, or the last one while the branch leaves; `each(parent, list, item, {key, index, live, enter, exit, …})` — keyed items keep their nodes across inserts, removals and moves, and with `live` across a new value too, handed to `item(value, index, itemOf, indexOf)` as accessors; `show(parent, cond, body, anim)`; `match(parent, key, arg, arms)` — an arm is called with the value and an accessor of it, and drawn again when the arm changes |
| Motion | `animate(target, name, duration)` → a handle with `play`/`cancel`/`finished`, playing when it is made; `replay(node, name)`; `animateIn`/`animateOut(node, name, duration, delay, easing)`; `expand(node, open, duration, easing)` to the measured height; `countTo(node, from, to, duration, format)` and `counted(node, valueFn, duration, format)`; `onEnterView(node, name, …)`; `shared(node, name)`; `mark(node, attrs)` puts `data-wf-exit`/`data-wf-delay`/`data-wf-duration`/`data-wf-animate` on a component's root — what `when`, `each`, `show` and `match` play before removing an element, and what times its enter animation |
| Routing | `router(routes, container, {transition, duration})` — `fade` or `slide` plays the old page out and the new one in; `navigate(path)`; `params()`; `activeLink(a, href, prefix)`; `page(name, fn)`; `loadPage`; `loadSheet` |
| Data | `resource(url, options)` → `{read, value, isLoading, isLoaded, isFailed, isRefreshing, error, pending, reload, invalidate, cancel}`, `read()` its one value — `"loading"`, `["failed", e]`, `["loaded", value, refreshing, stale]`; `request(url, options)` → the parsed body; `httpCall(method, url, options)` → a promise of a result (what `http.get(…)` and the other verbs compile to); `store(name, define, options)` — a proxy that builds `define()` on first read; `dropRouteStores()`; `emit(props, event, …args)`; `locales(default, tables)` → `WF.i18n` |
| Stores, seen | `watchStores(fn)` → every action with the state on each side of it, and the function that stops it; `storeSnapshot(name?)`; `restoreStore(name, values)`. Nothing is recorded while nothing is watching |
| Widgets | `toast(message, { tone, duration, action, onAction, closable })` → `{ close, update }`; `toastPromise(promise, { loading, done, failed })`; `dialog(el, read, write, { dismiss, initial })`; `ask(message, { detail, yes, no, danger })` → a promise of the answer; `sheet(el, { side, snap })`; `popup`; `tabs`; `drawer`; `announce(text, "assertive")`; `carousel`; `tooltip`; `menu`; `field` |
| Safety | `safeUrl(url)` → the URL where a browser may follow it, `""` where it may not; `sanitize(html)` → the allow-listed markup. Both have a build-time twin, pinned by a shared case table |
| Text | `markdown(text)` → HTML; `highlight(code, lang)` → HTML with `wf-tok-*` spans (`wf`, `json`, `bash`, `css`), the twin of `codegen::highlight` |
| Interop | `attach(node, mount, update, cleanup)` — what `mount:`/`update:`/`cleanup:` compile to on any element (a component's fragment is attached at its root); `portal(box, read)` — what `Portal` compiles to: `box` kept in the node `read()` names (read again once the drawing is over), aside on `null`, gone with its scope, returning what runs the lifetimes its block held; `wf:render` on `document` after each drawing; `jsonAttr(raw)` — an attribute holding JSON, as the prop it stands for |
| Boot | `mount(fn, container)`; `hydrate(fn, container)` and `hydrating(root)` — draw beside a pre-rendered page and take it over in place; `live(node)` — something holds `node`, so the takeover keeps the drawn one; `mainOf(root)`; `setBasePath`; `setSsgMode`; `__debug`, `__reg` (the studio's) |
| Scopes | `onCleanup(fn)` — run when what owns it leaves; `attempt(fn)` — an action that rolls itself back on a throw |
| Calls | `sameCall(pending, args, run, concurrent)` — what an async action's call compiles to: the call already running with those arguments, or a new one; `busy(el, handler)` — a click or submit handler that marks `el` busy while the calls it started run; `debounce(fn, ms)`, `throttle(fn, ms)` |
| Keys | `onKey(node, combination, fn)`; `keyIs(event, combination)`; `keyLabel(combination)`, `keyAria(combination)` — what `on key("mod+k")`, `keyIs` and `keyLabel` compile to |
| Focus | `trapFocus(node, read)` — what `trapFocus:` compiles to |
| Long lists | `windowed(node, listFn, itemFn, options)` — what a `VirtualList`'s `for` compiles to |
| Gestures | `gesture(node, name, fn)`, `draggable(node, value, options)`, `dropTarget(node, options)`, `splitter(node, read, write, options)` — what `on longpress`, `on swipe`, `drag:`, `on receive` and `Splitter` compile to |
| Values | `caseOf(v)`, `payload(v)` read an enum case and its payload, `readOf(v)` a read's one value; `dateOf`, `timeOf`, `urlWith`, `urlQuery` are the scalar helpers behind `.date()`, `.time()`, `.with()` and `.query()` |

