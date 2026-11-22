### State & Reactivity

```wf
state count = 0                      // Signal — reactive variable
state draft: String = ""             // with a declared type
derived double = count * 2           // Computed — auto-updates
effect { log(count) }                // Side effect — runs on change
```

- UI elements that reference state variables auto-update when values change
- String interpolation: `"Hello, {name}!"` — reactive in UI elements
- A call at the top of a page or component — `Store.load(id)` — is set-up
  code, run once when it renders; at the top of `app`, once when the site
  opens (`Session.load()`)

```wf
persist scheme = "light"              // kept in the browser across visits (page, component or store)
persist items: [Item] = [] {          // read back as its type: a kept value that is not one is dropped for the default
    in: .local                        // .local (default) · .session (this tab) · .app: window.wfStorage.app, a project script's { get, set, watch? } (get may return a promise)
    version: 2                        // the version of the shape this build writes
    sync: false                       // don't adopt what another tab or window wrote
    key: listId                       // one stored value per key, followed as state (key: Project.id keeps one per project; a store's may read the store)
    migrate 1 -> 2 { old.map(i => Item(id: i.id, qty: i.count)) }
}
every(1000) { tick = tick + 1 }       // a timer; stops when its page, branch or item leaves
after(3000) { toast = null }          // once
effect { watch(count)  cleanup { unwatch() } }   // run before the next run, and on leaving
Input(bind: q, ref: search)           // a handle on the element: search.focus(), search.value
Text(if viewport.md { "wide" } else { "narrow" })   // viewport.width/height/sm/md/lg/xl, kept current
Text(query.tab ?? "all")              // the URL's ?tab=…, and `hash` its #fragment
```

What a page, a branch, a list item, a match arm or a slot creates — effects,
timers, listeners, handles — is disposed of when it leaves; an async action's
`name.pending` is true while a call of it runs (`Button("Save",
loading: save.pending)`, `Api.sync.pending` on a store). A call with the
arguments of one still running is that call — the work is not done twice,
and both get its answer — so a double click submits once; a call with
other arguments runs beside it, and `action poll() concurrent { … }` runs
every call. A `Button`'s `on click` or a `Form`'s `on submit` that calls an
async action is busy until that call ends (`aria-busy`, a press that does
nothing, and past 300 ms a spinner in place of the label), with nothing
written for it.

<!-- core -->
