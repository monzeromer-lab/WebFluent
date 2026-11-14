# Call WF from a script

> Call the WebFluent runtime, window.WF, from a script of your own: show a toast, navigate, or read a store, and keep the call in the build.

The page: https://webfluent.monzeromer.dev/recipes/call-wf-from-a-script

The runtime is `window.WF`. A script under `src/`, the console, or a
library's callback can call it. This page has a button that copies the
page's address, and the script it calls says so with the runtime's toast:

**Needs:** the script `src/share.js`, shown below.

```wf
page Article(path: "/article", title: "Slow roasting", description: "Why we take eleven minutes over it.") {
    Heading("Slow roasting").h1
    Text("Eleven minutes, and why it matters.")
    Button("Copy the link") { on click { shareLink() } }
}
```

The script is a plain file under `src/`
([Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)):

```js
// src/share.js
/**
 * Copy this page's address, and say so in a toast.
 */
async function shareLink() {
  try {
    await navigator.clipboard.writeText(location.href);
    WF.toast("Link copied", "success");
  } catch (e) {
    WF.toast("Could not copy the link", "danger");
  }
}
```

## How it works

- `WF` is on `window` once `app.js` has run, so any script may call it
  while the page is open. `WF.toast(message, tone, ms)` shows a toast in
  a tone (`"info"` by default) for `ms` milliseconds (3000 by default).
- Other calls a script makes the same way: `WF.navigate("/about")` moves
  to another route, and `WF.storeSnapshot("Cart")` reads what a store
  holds. The [runtime reference](https://webfluent.monzeromer.dev/reference/runtime.md) lists every
  function.
- A build keeps only the runtime modules its code reaches, and it reads
  your scripts under `src/` for that too: the `WF.toast` written in
  `src/share.js` keeps the toast module in the build, though no `.wf` code
  uses it.

> [!WARNING]
> A call the build cannot read is not counted: a script in `public/` or
> from another origin, a `WF[name]` built at run time, or the console. If
> it reaches a module nothing else uses, set `"build": { "runtime": "full" }`,
> or the call fails in the browser.

A script from another origin must be listed in `meta.scripts`, which widens
the policy for it. The build's CSP check refuses an inline `<script>`
([Security](https://webfluent.monzeromer.dev/about/security.md#the-content-security-policy)).

## Troubleshooting

| What you see | What to do |
|---|---|
| `WF.toast is not a function` (or another runtime function) | The build left the module out: write the call in a script under `src/`, or set `"build": { "runtime": "full" }` |
| `WF is not defined` | The script ran before `app.js`: call `WF` from an event, a callback or a `wf:render` listener, not at the script's top level |
