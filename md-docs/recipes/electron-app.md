---
kind: "recipe"
route: "/recipes/electron-app"
group: "desktop"
description: "Make a WebFluent project an Electron desktop app: serve the build from its own origin, call Node from a page through a preload script, and package it."
outputs: ["desktop"]
from: ["/docs/guide/desktop#prepare-the-build-for-a-desktop", "/docs/guide/desktop#make-an-electron-app"]
---
# Make a desktop app with Electron

This recipe ships a WebFluent project as an Electron app: the build runs
in a Chromium shipped inside the app, and a page calls a Node.js side for
what a browser cannot do — open a folder dialog, read a file on disk,
start a process. Nothing in the language changes; a page reaches Node
through a plain script under `src/`, with one typed function per thing it
asks for ([Call your own JavaScript](call-your-own-script.md)). For a
native side in Rust and the system's own web view instead, see
[Make a desktop or phone app with Tauri](tauri-app.md).

You need a WebFluent project that builds with `wf build`, and Node.js
with npm.

These are the files of an Electron app, beside `src/`. First, the build
settings in `webfluent.app.json`:

```json
{
    "build": { "compress": false },
    "meta": { "sitemap": false }
}
```

Install Electron and electron-builder in the project's folder:

```bash
npm init -y
npm install --save-dev electron electron-builder
```

Point `package.json` at the main process, and add the scripts that run and
package the app:

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

The main process opens the window, serves the build to it, and answers
the page's calls:

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

The preload script hands the page exactly what it may ask for:

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

And one script under `src/` gives the pages typed names for each call:

```js
// src/native.js — what electron/preload.js exposed, under the names the pages call.
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

## How it works

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

A single-page build and a static build both work
([Static sites and SPAs](../about/static-and-spa.md)), and addresses stay as they
are. The main process answers a path the build has no file for with
`index.html`, so reloading the window on `/settings` shows the settings
page.

An app is often used with no network, and a font named in `meta.fonts`
comes from another origin. To make a font always available, put the file in
`public/` and declare it with `@font-face` in a `.css` file under `src/`
([Styling](../learn/13-styling.md)).

### Set up Electron

1. `npm init -y` writes a `package.json`, and `npm install` adds Electron
   and electron-builder to it as development dependencies.
2. Electron's two files go in a folder called `electron/`. Do **not**
   put them under `src/`: every `.js` file under `src/` is linked into the
   pages, and these files run in Node, outside the pages.
3. `"main"` in `package.json` points Electron at the main process, and
   `npm start` runs `electron .`.
4. The main process opens the window and serves the build to it. A
   `file://` address would break the build's absolute addresses (there,
   `/app.js` is the root of the disk), so the build is served from an
   `app://` origin of its own. `protocol.handle("app", serve)` answers each
   request with the file of `build/` it names, the folder's `index.html`,
   or — for a path with no extension, a route the router draws —
   `build/index.html`. Packaged, the window loads `app://localhost/`;
   while you work, `http://localhost:3000/`.

### Call Node from a page

A page has no Node in it. The *preload script* runs beside the page and
hands it exactly what it may ask for: one function per channel, never
`ipcRenderer` itself. Each `ipcMain.handle` in the main process answers
one channel: `greet` returns a value, `start-sync` reports its progress as
`sync-progress` events, and `pick-folder` opens the system's folder
dialog. `onEvent` lets a page hear only the events the preload script
lists.

`src/native.js` takes the place of the three scripts of the Tauri recipe.
It keeps their names, so the `Hello`, `Sync` and `Library` pages of
[Make a desktop or phone app with Tauri](tauri-app.md#call-a-rust-command)
run in Electron without a change. `inApp()` is false in a browser tab —
`wf serve` opened in a browser, `wf test`, `wf verify` — and each function
answers there instead of throwing, so every page still loads.

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
With electron-builder, `npm run dist` does both: its `dist` script runs
`wf build`, then `electron-builder`, and `build.files` names the two
folders the package holds.

## Troubleshooting

| What you see | What to check |
|---|---|
| Every native call fails with `This needs the desktop app.` | The page is open in a browser tab, not the app |
| A packaged Electron app shows an empty window | The package is missing `build/`, or `wf build` did not run before packaging |
| The app's text falls back to another font offline | The font comes from `meta.fonts`; put the file in `public/` and declare it with `@font-face` |
