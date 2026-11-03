# 33. Desktop apps

<!--
route: guide/desktop
group: shipping
blurb: Turn a WebFluent project into a Tauri or Electron desktop app, call the native side from a page, and package it.
description: Make a WebFluent project a Tauri or Electron desktop app: set it up, run it while you work, call Rust or Node from a page, hear events, and package it.
-->

This chapter shows you how to ship a WebFluent project as a desktop app with
Tauri or Electron, and how a page asks the native side for what a browser
cannot do: read a file on disk, open a folder dialog, show a notification,
start a process.

A *desktop shell* is a window that shows a web page, plus a native side
that the page can call. `wf build` writes a folder of static files
([Deploying](29-deploying.md)), and that folder is what the shell shows.
Nothing in the language changes for a desktop app. A page reaches the
native side through a plain script under `src/`
([JavaScript interop](32-javascript-interop.md)), with one typed function
per thing it asks for, and the shell decides what that script can reach.

Choose a shell by the language you want on the native side:

| | Tauri | Electron |
|---|---|---|
| The native side is written in | Rust | JavaScript, on Node.js |
| The window is | the system's own web view | a Chromium shipped inside the app |
| A page reaches the native side through | `window.__TAURI__` | what your preload script exposes |
| You need | Rust and the Tauri CLI | Node.js |

Both run the same pages, the same runtime and the same security policy.

## Before you start

You need:

- a WebFluent project that builds with `wf build`;
- for Tauri: Rust, and the system libraries listed in Tauri's
  [prerequisites](https://v2.tauri.app/start/prerequisites/);
- for Electron: Node.js and npm.

## Prepare the build for a desktop

The same project builds for the web and for a desktop. To leave out what
only a website needs, set two keys in `webfluent.app.json`:

```json
{
    "build": { "compress": false },
    "meta": { "sitemap": false }
}
```

- **`build.compress: false`** — the shell packs the folder into the app, so
  a `.gz` file beside every file would ship every file twice.
- **`meta.sitemap: false`** — writes no `sitemap.xml` or `robots.txt`;
  nothing crawls an app. You can leave `meta.site_url` and the link-preview
  tags unset too.
- **Leave `offline` out** — the files are already on the machine, so a
  service worker has nothing to keep.

A single-page build and a static build both work
([Static sites and SPAs](26-static-and-spa.md)), and addresses stay as they
are. Both shells in this chapter answer a path the build has no file for
with `index.html`, so reloading the window on `/settings` shows the
settings page.

An app is often used with no network, and a font named in `meta.fonts`
comes from another origin. To make a font always available, put the file in
`public/` and declare it with `@font-face` in a `.css` file under `src/`
([Styling](15-styling.md)).

## Make a Tauri app

Tauri runs your build in the system's web view and gives the page a Rust
side to call.

### Set up Tauri

1. Install the Tauri CLI once:

   ```bash
   cargo install tauri-cli --version "^2" --locked
   ```

2. In the project's folder, create the Rust side:

   ```bash
   cargo tauri init --app-name notes --window-title Notes \
       --frontend-dist ../build --dev-url http://localhost:3000 \
       --before-dev-command "wf serve" --before-build-command "wf build"
   ```

   This writes `src-tauri/` beside `src/`: the Rust side, its `Cargo.toml`
   and `tauri.conf.json`. WebFluent never reads `src-tauri/`, and Tauri runs
   the two `wf` commands in the project's folder.

3. Change two keys in `src-tauri/tauri.conf.json`:

   ```json
   {
       "identifier": "com.example.notes",
       "app": { "withGlobalTauri": true }
   }
   ```

   - **`identifier`** — a reverse-domain name of your own. `cargo tauri
     build` refuses the default, `com.tauri.dev`.
   - **`withGlobalTauri`** — puts Tauri's JavaScript API on
     `window.__TAURI__`, every plugin's API included, so a plain script uses
     it with no `import` and no npm.

Leave `app.security.csp` at `null`: every page already carries its own
policy (see [Allow Tauri requests in the security policy](#allow-tauri-requests-in-the-security-policy)).

### Run the Tauri app while you work

To work on the app, run:

```bash
cargo tauri dev
```

This command runs `wf serve` and opens the app's window on it. Saving a
`.wf` file rebuilds and reloads the window, and a build that fails shows
its error over the page, as it does in a browser ([CLI](38-cli.md)). Saving
a Rust file rebuilds the app. If `dev.port` is not `3000`, change `devUrl`
to match.

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
says where it may send requests ([Security](23-security.md)). Tauri sends
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

## Make an Electron app

Electron ships its own Chromium and gives the page a Node.js side to call.

### Set up Electron

1. In the project's folder, install Electron:

   ```bash
   npm init -y
   npm install --save-dev electron
   ```

2. Create a folder called `electron/` for Electron's two files. Do **not**
   put them under `src/`: every `.js` file under `src/` is linked into the
   pages, and these files run in Node, outside the pages.

3. Point `package.json` at the main process:

   ```json
   {
       "main": "electron/main.js",
       "scripts": { "start": "electron ." }
   }
   ```

4. Write the main process. It opens the window and serves the build to it.
   A `file://` address would break the build's absolute addresses (there,
   `/app.js` is the root of the disk), so the build is served from an
   `app://` origin of its own:

```js
// electron/main.js
const { app, BrowserWindow, dialog, ipcMain, net, protocol } = require("electron");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

const BUILD = path.join(__dirname, "..", "build");

// To the page, app://localhost is an ordinary origin: absolute paths,
// fetch and its security policy work as they do on a site.
protocol.registerSchemesAsPrivileged([
  { scheme: "app", privileges: { standard: true, secure: true, supportFetchAPI: true } },
]);

const isFile = (p) => fs.statSync(p, { throwIfNoEntry: false })?.isFile() ?? false;

// A file of the build; a route the router draws gets index.html.
function serve(request) {
  const file = path.join(BUILD, decodeURIComponent(new URL(request.url).pathname));
  const found =
    file.startsWith(BUILD + path.sep) &&
    ([file, path.join(file, "index.html")].find(isFile) ??
      (path.extname(file) ? null : path.join(BUILD, "index.html")));
  return found
    ? net.fetch(pathToFileURL(found).href)
    : new Response("Not found", { status: 404 });
}

function openWindow() {
  const win = new BrowserWindow({
    width: 1000,
    height: 700,
    webPreferences: { preload: path.join(__dirname, "preload.js") },
  });
  // Packaged, the window shows the build; while you work, `wf serve`.
  win.loadURL(app.isPackaged ? "app://localhost/" : "http://localhost:3000/");
}

app.whenReady().then(() => {
  protocol.handle("app", serve);

  ipcMain.handle("greet", (_event, name) => `Hello, ${name}!`);
  ipcMain.handle("start-sync", (event) => {
    for (const done of [25, 50, 75, 100]) {
      // … a quarter of the work …
      event.sender.send("sync-progress", done);
    }
  });
  ipcMain.handle("pick-folder", async (event) => {
    const win = BrowserWindow.fromWebContents(event.sender);
    const { canceled, filePaths } = await dialog.showOpenDialog(win, {
      properties: ["openDirectory"],
    });
    return canceled ? null : filePaths[0];
  });

  openWindow();
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) openWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});
```

### Call Node from a page

A page has no Node in it. The *preload script* runs beside the page and
hands it exactly what it may ask for: one function per channel, never
`ipcRenderer` itself.

```js
// electron/preload.js
const { contextBridge, ipcRenderer } = require("electron");

const events = ["sync-progress"];

contextBridge.exposeInMainWorld("native", {
  greet: (name) => ipcRenderer.invoke("greet", name),
  startSync: () => ipcRenderer.invoke("start-sync"),
  pickFolder: () => ipcRenderer.invoke("pick-folder"),
  onEvent: (name, handler) => {
    if (!events.includes(name)) throw new Error(`No event called ${name}`);
    const listener = (_event, payload) => handler(payload);
    ipcRenderer.on(name, listener);
    return () => ipcRenderer.removeListener(name, listener);
  },
});
```

In an Electron app, one `src/native.js` takes the place of the three Tauri
scripts. It keeps their names, so the `Hello`, `Sync` and `Library` pages
from the Tauri sections do not change at all:

```js
// What electron/preload.js exposed, under the names the pages call.
/**
 * Whether the page is inside the app, rather than a browser tab.
 * @returns {boolean}
 */
function inApp() {
  return "native" in window;
}

const outside = () => Promise.reject(new Error("This needs the desktop app."));

/**
 * @param {string} name
 * @returns {Promise<string>}
 */
function greet(name) {
  return inApp() ? window.native.greet(name) : outside();
}

/** Start syncing; how far it has got arrives as `sync-progress`. */
function startSync() {
  return inApp() ? window.native.startSync() : outside();
}

/**
 * Ask for a folder; `null` when the reader cancels.
 * @returns {Promise<?string>}
 */
function pickFolder() {
  return inApp() ? window.native.pickFolder() : Promise.resolve(null);
}

/**
 * Hear an event the native side emits, until the function it returns is called.
 * @param {string} name
 * @param {(payload: any) => void} handler
 * @returns {() => void}
 */
function onEvent(name, handler) {
  return inApp() ? window.native.onEvent(name, handler) : () => {};
}
```

### Run the Electron app while you work

Run the dev server and the app in two terminals. In the first:

```bash
wf serve
```

In the second:

```bash
npm start
```

Saving a `.wf` file rebuilds and reloads the window. A change under
`electron/` needs the app restarted.

The page's security policy needs nothing added: `app://localhost` is the
page's own origin, as `http://localhost:3000` is while you work.

### Package the Electron app

Electron Forge and electron-builder both package an app from
`package.json`. Whichever you use, run `wf build` first, and make sure the
package holds `electron/` and `build/`; `src/` is not needed at run time.
With electron-builder (`npm install --save-dev electron-builder`), the
`dist` script does both:

```json
{
    "main": "electron/main.js",
    "scripts": {
        "start": "electron .",
        "dist": "wf build && electron-builder"
    },
    "build": { "files": ["electron/**", "build/**"] }
}
```

## Troubleshooting

| What you see | What to check |
|---|---|
| Every native call fails with `This needs the desktop app.` | The page is open in a browser tab, not the app — or, with Tauri, `withGlobalTauri` is off, so `window.__TAURI__` is missing |
| A Tauri call is blocked by the security policy | `meta.connect` names both `ipc:` and `http://ipc.localhost` |
| A Tauri plugin's call fails | Its permission is not granted in `src-tauri/capabilities/default.json` |
| `cargo tauri build` refuses to run | `identifier` in `tauri.conf.json` is still `com.tauri.dev` |
| The Tauri window shows nothing while you work | `devUrl` does not match `dev.port` |
| A packaged Electron app shows an empty window | The package is missing `build/`, or `wf build` did not run before packaging |
| The app's text falls back to another font offline | The font comes from `meta.fonts`; put the file in `public/` and declare it with `@font-face` |

## Next

- [PDF documents and slide decks](34-pdf-and-slides.md) — the same project,
  compiled to paper.
- [JavaScript interop](32-javascript-interop.md) — more on plain scripts,
  JSDoc types and `mount:`/`cleanup:`.
