---
kind: "recipe"
route: "/recipes/electron-app"
group: "ship"
description: "The same project builds for the web and for a desktop."
from: ["/docs/guide/desktop#prepare-the-build-for-a-desktop", "/docs/guide/desktop#make-an-electron-app"]
---
# Make a desktop app with Electron

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
([Static sites and SPAs](../about/static-and-spa.md)), and addresses stay as they
are. Both shells in this chapter answer a path the build has no file for
with `index.html`, so reloading the window on `/settings` shows the
settings page.

An app is often used with no network, and a font named in `meta.fonts`
comes from another origin. To make a font always available, put the file in
`public/` and declare it with `@font-face` in a `.css` file under `src/`
([Styling](../learn/13-styling.md)).

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
