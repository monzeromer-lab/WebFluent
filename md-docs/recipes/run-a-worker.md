---
kind: "recipe"
route: "/recipes/run-a-worker"
group: "realtime"
description: "To keep slow work — a language service, a parser, anything that would make typing stutter — off the page's own thread, declare a `worker` with `spawn(…)`."
from: ["/docs/guide/realtime#run-a-script-in-a-worker"]
---
# Run a script in a worker

To keep slow work — a language service, a parser, anything that would make
typing stutter — off the page's own thread, declare a `worker` with
`spawn(…)`. A worker runs a script of your site beside the page and is held
like a socket: owned by what opened it, and stopped when that leaves. A
store holds one for the whole app:

```wf
store Lang {
    state problems: [Map] = []
    state restarts = 0
    worker css = spawn("/workers/css.js", restart: 1, timeout: 5000) {
        on message(m) { if m.type == "diagnostics" { problems = m.items } }
        on restart(e) { restarts = restarts + 1 }
    }
    action complete(text: String, at: Number) {
        return await css.request("complete", { text: text, at: at }, { supersede: true })
    }
    action check(text: String) { css.send({ method: "check", params: { text: text } }) }
}

page Editor(path: "/", title: "Editor", description: "A worker beside the page.") {
    Heading("Editor").h1
    match Lang.css {
        open     { Text(if Lang.css.pending > 0 { "Checking…" } else { "Checked" }) }
        error(e) { Alert("The CSS service stopped: {e.message}").danger }
        else     { Text("Stopped") }
    }
    Text("{Lang.problems.length} problems, {Lang.restarts} restarts")
    Button("Complete") { on click { Lang.complete("a { col", 7) } }
    Button("Check") { on click { Lang.check("a { col") } }
}
```

The worker's side is a plain script under `public/` — the one place,
besides a project script, where you write JavaScript by hand:

```js
// public/workers/css.js
self.onmessage = ({ data }) => {
  const { id, method, params, cancel } = data;
  if (cancel !== undefined) return;                       // a newer request took over
  if (method === "complete") self.postMessage({ id, result: complete(params) });
  if (method === "check") self.postMessage({ type: "diagnostics", items: check(params.text) });
};
```

The page and the worker talk in three ways:

- **Ask and wait: `.request(method, params)`.** It posts
  `{ id, method, params }` and returns a promise of what the worker posts
  back as `{ id, result }`. A reply of `{ id, error }` rejects it, with the
  error's `message` and its `data`. A request not answered within
  `timeout:` fails — 10 seconds when unset, or `{ timeout }` on the
  request. With `{ supersede: true }`, a newer request of the same method
  fails the older one, and the worker is posted `{ cancel: id }`.
- **Tell: `.send(value)`.** It posts the value and expects nothing.
- **Listen: messages with no `id`.** What the worker posts without an `id`
  is a message: `on message(m)` hears it, and `.last(kind)` is the last of
  a kind, by its `type` or its `method`.

`.pending` is how many requests are out. `.state` is `open`, `error` (a
`match`'s `error(e)` arm) or `closed`.

A worker that crashes — an error nothing in it caught — fails every request
that was out and is started again, up to `restart:` times, running
`on restart(e)` each time. Past that it is `error`, and `on error(e)` runs.
`.restart()` starts it again by hand, and `.close()` stops it.

Pass `type: .module` to start a module worker. The script is a file of the
site, so it is asked for under the base path, and a name `public/` does not
have is a `D01`.
