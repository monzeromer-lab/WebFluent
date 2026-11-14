# Make a desktop or phone app with Tauri

> Make a WebFluent project a Tauri app for the desktop or a phone: set it up, run it while you work, call Rust from a page, hear its events, package it.

The page: https://webfluent.monzeromer.dev/recipes/tauri-app

This recipe ships a WebFluent project as a Tauri app: the build runs in
the system's own web view, and a page calls a Rust side for what a browser
cannot do — read a file on disk, open a folder dialog, show a
notification. Nothing in the language changes; a page reaches Rust through
a plain script under `src/`, with one typed function per command
([Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)). For a native side in
JavaScript on Node.js instead, see
[Make a desktop app with Electron](https://webfluent.monzeromer.dev/recipes/electron-app.md).

You need a WebFluent project that builds with `wf build`, Rust, and the
system libraries listed in Tauri's
[prerequisites](https://tauri.app/start/prerequisites/).

These are the settings a Tauri app needs. In `webfluent.app.json`:

```json
{
    "build": { "compress": false },
    "meta": { "sitemap": false, "connect": ["ipc:", "http://ipc.localhost"] }
}
```

The commands that make the Rust side and run the app:

```bash
cargo install tauri-cli --version "^2" --locked
cargo tauri init --app-name notes --window-title Notes \
    --frontend-dist ../build --dev-url http://localhost:3000 \
    --before-dev-command "wf serve" --before-build-command "wf build"
cargo tauri dev
```

And two keys in the `src-tauri/tauri.conf.json` that `cargo tauri init`
writes:

```json
{
    "identifier": "com.example.notes",
    "app": { "withGlobalTauri": true }
}
```

## How it works

The steps below take these settings in order, then add what a page calls:
a Rust command, an event, a plugin.

### Prepare the build for a desktop

The same project builds for the web and for a desktop. Two keys in
`webfluent.app.json`, and one to leave out, drop what only a website needs:

- **`build.compress: false`** — the shell packs the folder into the app, so
  a `.gz` file beside every file would ship every file twice.
- **`meta.sitemap: false`** — writes no `sitemap.xml` or `robots.txt`;
  nothing crawls an app. You can leave `meta.site_url` and the link-preview
  tags unset too.
- **Leave `offline` out** — the files are already on the machine, so a
  service worker has nothing to keep.

`meta.connect` is for the security policy; see
[Allow Tauri requests in the security policy](#allow-tauri-requests-in-the-security-policy).

A single-page build and a static build both work
([Static sites and SPAs](https://webfluent.monzeromer.dev/about/static-or-single-page.md)), and addresses stay as they
are. Tauri answers a path the build has no file for with `index.html`, so
reloading the window on `/settings` shows the settings page.

An app is often used with no network, and a font named in `meta.fonts`
comes from another origin. To make a font always available, put the file in
`public/` and declare it with `@font-face` in a `.css` file under `src/`
([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md)).

### Set up Tauri

1. `cargo install tauri-cli` installs the Tauri CLI, once per machine.
2. `cargo tauri init`, run in the project's folder, writes `src-tauri/`
   beside `src/`: the Rust side, its `Cargo.toml` and `tauri.conf.json`.
   Its flags point Tauri at `build/` for the app, at `wf serve` on port
   3000 while you work, and at the `wf` commands that make each.
   WebFluent never reads `src-tauri/`, and Tauri runs the two `wf`
   commands in the project's folder.
3. Two keys in `src-tauri/tauri.conf.json` change:
   - **`identifier`** — a reverse-domain name of your own. `cargo tauri
     build` refuses the default, `com.tauri.dev`.
   - **`withGlobalTauri`** — puts Tauri's JavaScript API on
     `window.__TAURI__`, every plugin's API included, so a plain script uses
     it with no `import` and no npm.

Leave `app.security.csp` at `null`: every page already carries its own
policy (see [Allow Tauri requests in the security policy](#allow-tauri-requests-in-the-security-policy)).

### Run the Tauri app while you work

`cargo tauri dev` runs `wf serve` and opens the app's window on it. Saving
a `.wf` file rebuilds and reloads the window, and a build that fails shows
its error over the page, as it does in a browser ([CLI](https://webfluent.monzeromer.dev/reference/cli.md)).
Saving a Rust file rebuilds the app. If `dev.port` is not `3000`, change
`devUrl` to match.

To build the installers, run:

```bash
cargo tauri build
```

This command runs `wf build`, embeds `build/` in the binary, and writes the
installers under `src-tauri/target/release/bundle/`.

### Call a Rust command

A *command* is a Rust function a page may call. Write it in
`src-tauri/src/lib.rs` and list it in `run()`. This file has two commands:
`greet` returns a value, and `start_sync` returns at once and reports its
progress as events:

```rust
use tauri::Emitter;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[tauri::command]
fn start_sync(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        for done in [25, 50, 75, 100] {
            // … a quarter of the work …
            let _ = app.emit("sync-progress", done);
        }
    });
}

// in run(), before .run(…):
//     .invoke_handler(tauri::generate_handler![greet, start_sync])
```

A page calls a command through a script that names it. Write one function
per command, typed with JSDoc, so `.wf` code calls `greet(name)` like any
other function and the checker holds every call to its types:

```js
// src/native.js
/**
 * Whether the page is inside the app, rather than a browser tab.
 * @returns {boolean}
 */
function inApp() {
  return "__TAURI__" in window;
}

/**
 * Call a Rust command. What its `Err` holds arrives as an `Error`.
 * @param {string} command
 * @param {object} [args]
 * @returns {Promise<any>}
 */
function invoke(command, args) {
  if (!inApp()) return Promise.reject(new Error("This needs the desktop app."));
  return window.__TAURI__.core.invoke(command, args).catch((err) => {
    throw new Error(typeof err === "string" ? err : JSON.stringify(err));
  });
}

/**
 * @param {string} name
 * @returns {Promise<string>}
 */
function greet(name) {
  return invoke("greet", { name });
}

/** Start syncing; how far it has got arrives as `sync-progress`. */
function startSync() {
  return invoke("start_sync");
}
```

Three rules hold for every command:

- **Arguments are named.** A command's parameters are the keys of the
  object, in camelCase: a Rust `note_id` is `{ noteId }`.
- **A result is a promise.** A command that returns `Result<T, E>` resolves
  with the `T`. Its `Err` rejects, and `invoke` turns it into an `Error`,
  so a page's `catch e` reads `e.message`.
- **`inApp()` is false in a browser tab** — `wf serve` opened in a browser,
  `wf test`, `wf verify` — where there is no native side. Each function
  answers there instead of throwing, so every page still loads.

The page awaits the function in an action and shows the reply:

```wf
page Hello(path: "/", title: "Hello", description: "Says hello from Rust.") {
    state name = ""
    state reply = ""

    action ask() {
        try {
            reply = await greet(name)
        } catch e {
            reply = e.message
        }
    }

    Heading("Hello").h1
    if !inApp() {
        Alert("Open this in the app to talk to Rust.").info
    }
    Input(bind: name, label: "Your name")
    Button("Greet").primary { on click { ask() } }
    Text(reply)
}
```

In the app, typing `Ada` and pressing **Greet** shows `Hello, Ada!`. In a
browser tab, the page shows the alert, and **Greet** shows `This needs the
desktop app.`

### Listen for events from Rust

`start_sync` returns at once and reports as it goes, with `app.emit(…)`. A
page hears an event for as long as it shows, so tie the listener to the
page's lifetime: `mount:` starts it, and `cleanup:` stops it when the page,
branch or item leaves. First, wrap Tauri's `listen` in a script:

```js
// src/events.js
/**
 * Hear an event the native side emits, until the function it returns is called.
 * @param {string} name
 * @param {(payload: any) => void} handler
 * @returns {() => void}
 */
function onEvent(name, handler) {
  if (!inApp()) return () => {};
  const listening = window.__TAURI__.event.listen(name, (e) => handler(e.payload));
  return () => { listening.then((stop) => stop()); };
}
```

Then hand the listener an action. This page shows a progress bar that
fills as the events arrive:

```wf
page Sync(path: "/sync", title: "Sync", description: "How far the sync has got.") {
    state done = 0
    state problem = ""

    action setDone(n: Number) { done = n }
    action start() {
        try {
            await startSync()
        } catch e {
            problem = e.message
        }
    }

    Stack(gap: .md,
          mount: (n) => onEvent("sync-progress", setDone),
          cleanup: (stop) => stop()) {
        Heading("Sync").h1
        Progress(value: done, max: 100)
        Text("{done}%")
        Button("Sync now") { on click { start() } }
        Text(problem)
    }
}
```

Pressing **Sync now** moves the bar through 25, 50, 75 and 100%.

### Use a Tauri plugin

A dialog, the file system, notifications, the clipboard, opening a link in
the browser: each is a Tauri plugin. To add one, run the CLI. It adds the
crate, the line in `run()`, and the plugin's default permission in
`src-tauri/capabilities/default.json`:

```bash
cargo tauri add dialog
```

With `withGlobalTauri` on, the plugin's API is on
`window.__TAURI__.dialog`. Wrap what a page uses, as with a command:

```js
// src/dialogs.js
/**
 * Ask for a folder; `null` when the reader cancels.
 * @returns {Promise<?string>}
 */
function pickFolder() {
  if (!inApp()) return Promise.resolve(null);
  return window.__TAURI__.dialog.open({ directory: true });
}
```

This page asks for a folder and shows the one the reader picked:

```wf
page Library(path: "/library", title: "Library", description: "Where the notes are kept.") {
    state folder = ""

    action choose() {
        let picked = await pickFolder()
        if picked != null {
            folder = picked
        }
    }

    Heading("Library").h1
    Button("Choose a folder") { on click { choose() } }
    if folder != "" {
        Text("Notes are kept in {folder}.")
    } else {
        Text("No folder yet.").muted
    }
}
```

A plugin's call fails until its permission is granted. `cargo tauri add`
grants the default set. Name anything beyond it — for example, a path the
`fs` plugin may read — in the same capabilities file.

### Allow Tauri requests in the security policy

With `build.csp` on — as it is for a project `wf init` makes — every page
says where it may send requests ([Security](https://webfluent.monzeromer.dev/about/security.md)). Tauri sends
each call as a request: to `http://ipc.localhost` on Windows and Android,
and to `ipc://localhost` on macOS, Linux and iOS. Name both in
`meta.connect`, the second as its scheme alone, which is how a policy names
an app's own scheme:

```json
{ "meta": { "connect": ["ipc:", "http://ipc.localhost"] } }
```

Other settings follow the same pattern:

- **A protocol of your own** (`register_uri_scheme_protocol`) is named the
  same way. For a frame the app fills from it, write `"frame": ["myapp:",
  "http://myapp.localhost"]`; a frame's `src` may then use that scheme.
- **WebAssembly**: a page that compiles it adds `"wasm": true`.
- **Style elements**: a page whose scripts write `<style>` elements adds
  `"style_elements": true`.

## Variations

### Run it on a phone

The same project runs on Android and iOS. Make the phone's side once, then
run it while you work (`ios` in place of `android`, on a Mac):

```bash
cargo tauri android init
cargo tauri android dev
```

- **The dev server.** `wf serve` listens to this machine only. To let the
  phone reach it, make `beforeDevCommand` `wf serve --host 0.0.0.0` and
  point `devUrl` at the machine's address on the network
  ([CLI](https://webfluent.monzeromer.dev/reference/cli.md)).
- **No window chrome.** A phone has no title bar: draw one only on a
  desktop.
- **Edge to edge.** The page is drawn under the phone's own bars, so pad
  the app's frame by `env(safe-area-inset-*)`, once a script sets
  `viewport-fit=cover` on the viewport tag.
- **Back.** Android's back gesture goes back through the router's history.
  To close an open sheet first, take the gesture with Tauri's
  `app.onBackButtonPress` from the same script, in an effect that gives it
  back.
- **Plugins.** Capabilities are split by `"platforms"`, and a phone-only
  plugin (haptics, a barcode scanner) is registered under `#[cfg(mobile)]`.

## Troubleshooting

| What you see | What to check |
|---|---|
| Every native call fails with `This needs the desktop app.` | The page is open in a browser tab, not the app — or `withGlobalTauri` is off, so `window.__TAURI__` is missing |
| A Tauri call is blocked by the security policy | `meta.connect` names both `ipc:` and `http://ipc.localhost` |
| A Tauri plugin's call fails | Its permission is not granted in `src-tauri/capabilities/default.json` |
| `cargo tauri build` refuses to run | `identifier` in `tauri.conf.json` is still `com.tauri.dev` |
| The Tauri window shows nothing while you work | `devUrl` does not match `dev.port` |
| The app's text falls back to another font offline | The font comes from `meta.fonts`; put the file in `public/` and declare it with `@font-face` |
