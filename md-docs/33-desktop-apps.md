# 33. Desktop apps

<!--
route: guide/desktop
group: shipping
blurb: The build is a folder a desktop shell can show. Here is how to make a Tauri or an Electron app of a project, call the native side from a page, and ship it.
description: A WebFluent project as a Tauri or Electron desktop app: setting up, the dev loop, calling Rust or Node from a page, events, plugins and packaging.
-->

A desktop shell is a window that shows a web page, and a native side the
page can ask for what a browser cannot do: a file on disk, a folder dialog,
a notification, a process. `wf build` writes a folder of static files
([chapter 29](29-deploying.md)), and that folder is what a shell shows.

Nothing in the language changes for a desktop app. A page reaches the
native side through a plain script under `src/`
([chapter 32](32-javascript-interop.md)), one typed function per thing it
asks for, and the shell decides what that script can reach.

| | Tauri | Electron |
|---|---|---|
| The native side is written in | Rust | JavaScript, on Node.js |
| The window is | the system's own web view | a Chromium shipped inside the app |
| A page reaches the native side through | `window.__TAURI__` | what your preload script exposes |
| You need | Rust and the Tauri CLI | Node.js |

Both run the same pages, the same runtime and the same security policy.
Choose by the language you want on the native side.

## The build for a desktop

The same project builds for the web and for a desktop. Two settings leave
out what only a website needs:

```json
{
    "build": { "compress": false },
    "meta": { "sitemap": false }
}
```

- **`build.compress: false`** — the shell packs the folder into the app, so
  a `.gz` beside every file would ship every file twice.
- **`meta.sitemap: false`** — no `sitemap.xml` or `robots.txt`; nothing
  crawls an app. `meta.site_url` and the link-preview tags can stay unset
  too.
- **Leave `offline` out** — the files are already on the machine, so there
  is nothing for a service worker to keep.

A single-page build and a static one both work
([chapter 26](26-static-and-spa.md)), and addresses stay as they are: both
shells below answer a path the build has no file for with `index.html`, so
reloading the window on `/settings` shows the settings page.

An app is often used with no network. A font named in `meta.fonts` comes
from another origin; for one that is always there, put the file in
`public/` and declare it with `@font-face` in a `.css` file under `src/`
([chapter 15](15-styling.md)).

## Tauri

### Setting up

Install the Tauri CLI once — and what Tauri needs on your system, listed in
its [prerequisites](https://v2.tauri.app/start/prerequisites/):

```bash
cargo install tauri-cli --version "^2" --locked
```

Then, in the project's folder:

```bash
cargo tauri init --app-name notes --window-title Notes \
    --frontend-dist ../build --dev-url http://localhost:3000 \
    --before-dev-command "wf serve" --before-build-command "wf build"
```

That writes `src-tauri/` beside `src/`: the Rust side, its `Cargo.toml` and
`tauri.conf.json`. WebFluent never reads `src-tauri/`, and Tauri runs the
two `wf` commands in the project's folder. Change two things in
`src-tauri/tauri.conf.json`:

```json
{
    "identifier": "com.example.notes",
    "app": { "withGlobalTauri": true }
}
```

- **`identifier`** — a reverse-domain name of your own. `cargo tauri build`
  refuses the default, `com.tauri.dev`.
- **`withGlobalTauri`** — puts Tauri's JavaScript API on `window.__TAURI__`,
  every plugin's included, so a plain script uses it with no `import` and no
  npm.

Leave `app.security.csp` at `null`: every page already carries its policy
([below](#the-security-policy)).

### Working on it

```bash
cargo tauri dev
```

runs `wf serve` and opens the app's window on it. Saving a `.wf` file
rebuilds and reloads the window, and a build that fails shows its error over
the page, as it does in a browser ([chapter 38](38-cli.md)). Saving a Rust
file rebuilds the app. If `dev.port` is not `3000`, change `devUrl` to match.

```bash
cargo tauri build
```

runs `wf build`, embeds `build/` in the binary, and writes the installers
under `src-tauri/target/release/bundle/`.

### Calling Rust

A command is a Rust function a page may call. Write it in
`src-tauri/src/lib.rs` and list it in `run()`:

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

A page calls a command through a script that names it: one function per
command, typed with JSDoc, so `.wf` code calls `greet(name)` like any other
function and the checker holds every call to its types:

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

- **Arguments are named.** A command's parameters are the keys of the
  object, in camelCase: a Rust `note_id` is `{ noteId }`.
- **A result is a promise.** A command that returns `Result<T, E>` resolves
  with the `T`; its `Err` rejects, and `invoke` makes it an `Error`, so a
  page's `catch e` reads `e.message`.
- **`inApp()` is false in a browser tab** — `wf serve` opened in a browser,
  `wf test`, `wf verify` — where there is no native side. Each function
  answers there instead of throwing, so every page still loads.

The page awaits the function in an action:

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

### Hearing from Rust

`start_sync` above returns at once and reports as it goes, with
`app.emit(…)`. A page hears an event for as long as it shows, so the
listener belongs to the page's lifetime — `mount:` starts it and `cleanup:`
stops it when the page, branch or item leaves:

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

### Plugins

A dialog, the file system, notifications, the clipboard, opening a link in
the browser: each is a Tauri plugin. The CLI adds one — the crate, the line
in `run()`, and its default permission in
`src-tauri/capabilities/default.json`:

```bash
cargo tauri add dialog
```

With `withGlobalTauri` on, its API is on `window.__TAURI__.dialog`. Wrap
what a page uses, as with a command:

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
grants the default set; anything beyond it — a path the `fs` plugin may
read, for one — is named in the same capabilities file.

### The security policy

With `build.csp` on — it is for a project `wf init` makes — every page says
where it may send requests ([chapter 23](23-security.md)). Tauri sends each
call as a request: to `http://ipc.localhost` on Windows and Android, and to
`ipc://localhost` on macOS, Linux and iOS. Name both — the second as its
scheme alone, which is how a policy names an app's own:

```json
{ "meta": { "connect": ["ipc:", "http://ipc.localhost"] } }
```

A protocol of your own (`register_uri_scheme_protocol`) is named the same
way: `"frame": ["myapp:", "http://myapp.localhost"]` for a frame the app
fills from it. A page that compiles WebAssembly adds `"wasm": true`.

## Electron

### Setting up

In the project's folder:

```bash
npm init -y
npm install --save-dev electron
```

Electron's two files go in `electron/`, **not** under `src/`: every `.js`
file under `src/` is linked into the pages, and these run in Node, outside
them. Point `package.json` at the first:

```json
{
    "main": "electron/main.js",
    "scripts": { "start": "electron ." }
}
```

The main process opens the window and serves the build to it. `file://`
would break the build's absolute addresses (`/app.js` is the root of the
disk there), so the build is served from an `app://` origin of its own:

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

### Calling Node

A page has no Node in it. The preload script runs beside the page and hands
it exactly what it may ask for — one function per channel, never
`ipcRenderer` itself:

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
scripts above. It keeps their names, so the `Hello`, `Sync` and `Library`
pages do not change at all:

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

### Working on it

Run the dev server, then the app, in two terminals:

```bash
wf serve
```

```bash
npm start
```

Saving a `.wf` file rebuilds and reloads the window. A change to
`electron/` needs the app restarted.

The page's policy needs nothing added: `app://localhost` is the page's own
origin, as `http://localhost:3000` is while you work.

### Packaging

Electron Forge and electron-builder both package an app from
`package.json`. Whichever you use, run `wf build` first, and make sure the
package holds `electron/` and `build/` — `src/` is not needed at run time.
With electron-builder (`npm install --save-dev electron-builder`):

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

## Next

[PDF documents and slide decks](34-pdf-and-slides.md).
