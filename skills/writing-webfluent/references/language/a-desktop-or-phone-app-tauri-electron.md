### A desktop or phone app (Tauri, Electron)

The `build/` folder is what a Tauri or Electron window shows; nothing in
the language changes. Leave out what only a website needs, and let the
page reach Tauri's IPC, which travels as a request:

```json
{
    "build": { "ssg": false, "compress": false },
    "meta": { "sitemap": false, "connect": ["ipc:", "http://ipc.localhost"] }
}
```

In `src-tauri/tauri.conf.json`: `"frontendDist": "../build"`, `"devUrl":
"http://localhost:3000"`, `"beforeDevCommand": "wf serve"`,
`"beforeBuildCommand": "wf build"`, `"app": { "withGlobalTauri": true,
"security": { "csp": null } }` — the page carries its own policy. A page
reaches Rust through one plain script, `src/native.js`, with one
JSDoc-typed function per command over `window.__TAURI__.core.invoke`; each
answers harmlessly when `"__TAURI__" in window` is false, so `wf serve` in
a browser, `wf test` and `wf verify` still load every page. A store's
actions `await` those functions in a `try`; an event Rust emits is heard
in an `effect { let stop = …  cleanup { stop() } }`, so it stops with its
page; `persist x = … { in: .app }` keeps settings through
`window.wfStorage.app`, which that script backs with the store plugin.
Fonts go in `public/` with `@font-face`, never `meta.fonts`, so the app
works offline. The guide's "Desktop apps" chapter is the step-by-step,
Electron included.

The same project runs on Android and iOS (`cargo tauri android init`,
`cargo tauri android dev`; `ios` on a Mac). `wf serve --host 0.0.0.0`
listens on every interface (by default it is this machine's only), so a
phone reaches the dev server when Tauri points `devUrl` at the machine's
address. A phone has no window chrome: draw a title bar only
on a desktop. Its page is drawn edge to edge, so a shell pads by
`env(safe-area-inset-*)` once the script sets `viewport-fit=cover` on the
viewport tag. Android's back gesture goes back through the router's
history; to close an open sheet first, take it with Tauri's
`app.onBackButtonPress` from the same script, in an effect that gives it
back. Capabilities are split by `"platforms"`, and a phone-only plugin
(haptics, a barcode scanner) is registered under `#[cfg(mobile)]`.

