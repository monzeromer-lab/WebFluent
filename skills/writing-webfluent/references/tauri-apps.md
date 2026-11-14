# Apps with Tauri: desktop and phone

One WebFluent project becomes an app on Linux, macOS and Windows, and on
Android and iOS, by putting its `build/` folder inside a Tauri 2 shell. The
language doesn't change.
- **The bridge.** A page reaches Rust through **one plain script**,
  `src/native.js`. It has one JSDoc-typed function per Rust command, so the
  compiler checks every call.
- **Platforms.** The same pages, stores and bridge serve every platform. What
  differs is the window: a title bar and menus on a desktop; safe areas, touch
  and the back gesture on a phone.

This file collects what WebFluent Studio, itself a WebFluent app in Tauri,
learned the hard way, plus what Tauri 2.12's own sources say about phones. The
recipe "Make a desktop or phone app with Tauri" (`md-docs/recipes/tauri-app.md`, or
`/recipes/tauri-app` on the site) is the step-by-step version; Electron has its
own, `/recipes/electron-app`.

## Contents

- [The shape of the project](#the-shape-of-the-project)
- [webfluent.app.json for an app](#webfluentappjson-for-an-app)
- [src-tauri/tauri.conf.json](#src-tauritauriconfjson)
- [The bridge: src/native.js](#the-bridge-srcnativejs)
- [Rust commands](#rust-commands)
- [Calling Rust from a store](#calling-rust-from-a-store)
- [Keeping settings](#keeping-settings-persist--in-app-)
- [Capabilities](#capabilities)
- [One shell for every platform](#one-shell-for-every-platform)
- [Desktop: window chrome, menus, several windows](#desktop-window-chrome-menus-several-windows)
- [Phone: Android and iOS](#phone-android-and-ios)
- [Theme, language, fonts](#theme-language-fonts)
- [Files, custom protocols, previews, libraries](#files-custom-protocols-previews-libraries)
- [Testing the app](#testing-the-app)
- [Packaging and release](#packaging-and-release)
- [Pitfalls](#pitfalls)
- [Checklist](#checklist-before-you-call-it-done)

## The shape of the project

```
my-app/
├── webfluent.app.json
├── src/
│   ├── App.wf             # app { WindowShell { Router } } — chrome per platform
│   ├── native.js          # the bridge: every Rust command, typed
│   ├── shell.css          # title bar, safe areas, touch sizes
│   ├── types.wf           # the records Rust sends, as `type`s
│   ├── stores/            # one store per area; actions await the bridge
│   ├── pages/
│   └── components/
├── public/fonts/          # fonts ship inside the app
├── tests/                 # `wf test` works: the bridge degrades in a browser
└── src-tauri/             # Rust; WebFluent never reads it
    ├── tauri.conf.json    # + tauri.android.conf.json, tauri.ios.conf.json, tauri.macos.conf.json
    ├── capabilities/      # desktop.json, mobile.json
    ├── gen/android/       # `cargo tauri android init`: commit it
    ├── gen/apple/         # `cargo tauri ios init` (on a Mac): commit it
    └── src/lib.rs         # the app; main.rs only calls it on desktop
```

Keep every hand-written script for Tauri in `src/native.js`, or in a few files
named by area (`src/native-files.js`). Anything else belongs in `.wf`. Every
`.js` file under `src/` is linked into every page, so Node or Rust-side files
must never sit there.

## webfluent.app.json for an app

```json
{
  "name": "Notes",
  "theme": { "name": "Day", "dark": "Night" },
  "build": { "ssg": false, "csp": true, "compress": false },
  "meta": {
    "title": "Notes",
    "lang": "en",
    "sitemap": false,
    "connect": ["ipc:", "http://ipc.localhost"]
  }
}
```

- **`ssg: false`.** Use a single-page build. A pre-render makes each rebuild
  slower and gains nothing in an app.
- **`compress: false`.** Tauri embeds the folder, so `.gz` twins would ship
  every file twice, which matters most on a phone.
- **`sitemap: false`.** Nothing crawls an app. Leave `offline`, `site_url` and
  `base_path` out.
- **`csp: true` and `meta.connect`.**
  - WebFluent writes the page's security policy itself.
  - Tauri sends every `invoke` as a request, to `ipc://localhost` on Linux,
    macOS and iOS, and to `http://ipc.localhost` on Windows and Android.
  - Name both in `meta.connect`, the first by its scheme alone. Without them,
    every call is blocked by the policy, on some platforms and not others.
- **Additions as you need them:**
  - `"frame": ["myproto:", "http://myproto.localhost"]` for a `Frame` filled
    from a custom protocol.
  - `"wasm": true` for WebAssembly.
  - `"style_elements": true` when a library such as CodeMirror injects
    `<style>`.
- **Fonts.** `meta.fonts` loads from another origin and fails offline. Put the
  font files in `public/fonts/` and declare them with `@font-face` in a `.css`
  file under `src/`.

## src-tauri/tauri.conf.json

**With hot reload** (the guide's way): Tauri runs `wf serve`, and a save
reloads the window. On a phone too.

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Notes",
  "identifier": "com.example.notes",
  "version": "1.0.0",
  "build": {
    "frontendDist": "../build",
    "devUrl": "http://localhost:3000",
    "beforeDevCommand": "wf serve",
    "beforeBuildCommand": "wf build"
  },
  "app": {
    "withGlobalTauri": true,
    "security": { "csp": null },
    "windows": [{ "title": "Notes", "width": 1200, "height": 800, "minWidth": 360, "decorations": false }]
  },
  "bundle": { "active": true, "targets": "all", "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"] }
}
```

- **`withGlobalTauri: true`** puts the whole API, and every plugin's, on
  `window.__TAURI__`. A plain script then needs no `import` and no npm.
- **`security.csp: null`** leaves the policy to WebFluent. Every page already
  carries one, and two policies would intersect.
- **`identifier`** must be your own reverse-domain name. `cargo tauri build`
  refuses `com.tauri.dev`. On Android it becomes the package name, so choose
  it before the first release: it can't change after.
- **`devUrl`** must match `dev.port` (3000 unless you changed it).
  - On a phone, `cargo tauri android dev` / `ios dev` replaces `localhost`
    in it with your machine's network address.
  - `wf serve` listens on this machine only by default; run it as
    `wf serve --host 0.0.0.0` (in `beforeDevCommand` too) so the phone can
    reach it. If it still can't, a firewall is blocking the port.
- **`decorations: false`** gives you your own title bar on the desktop
  (below). A phone has no window decorations, and the setting is ignored
  there.
- **Platform files.** `tauri.android.conf.json`, `tauri.ios.conf.json`,
  `tauri.macos.conf.json` and the rest are merged over this file on their
  platform, so a phone-only or Mac-only setting goes there.
- **Pinning the compiler.** A team that pins its compiler runs a script that
  builds with that `wf`, as Studio does with `bash ../build.sh`. Use it for
  both commands and drop `devUrl`.

## The bridge: src/native.js

This is the whole pattern:
- one `callNative` that turns Tauri's rejection into an `Error` with a `code`;
- a typed wrapper per command;
- an answer for every wrapper outside the app, so `wf serve` in a browser,
  `wf test` and `wf verify` still load every page;
- the few platform facts the pages branch on.

```js
// src/native.js
// The one place the pages reach Tauri. One function per command, typed
// with JSDoc, so `wf check` holds every call to its arguments.

/**
 * Whether the page runs inside the app, not in a browser tab.
 * @returns {boolean}
 */
function inApp() {
  return "__TAURI__" in window;
}

/**
 * The system the app runs on: "android", "ios", "mac", "windows" or "linux".
 * @returns {string}
 */
function platformName() {
  const ua = navigator.userAgent;
  if (/Android/.test(ua)) return "android";
  if (/iPhone|iPad|iPod/.test(ua) || (ua.includes("Mac") && navigator.maxTouchPoints > 1)) return "ios";
  return ua.includes("Mac") ? "mac" : ua.includes("Windows") ? "windows" : "linux";
}

/**
 * Whether the app runs on a phone or tablet: no window to move, a back gesture.
 * @returns {boolean}
 */
function onPhone() {
  const p = platformName();
  return p === "android" || p === "ios";
}

// On a phone the page is drawn edge to edge, under the status and
// navigation bars; `viewport-fit=cover` lets CSS read their size with
// env(safe-area-inset-*). Set before the first page draws.
if (onPhone()) {
  document.querySelector('meta[name="viewport"]')
    ?.setAttribute("content", "width=device-width, initial-scale=1, viewport-fit=cover");
}

/**
 * Call a Rust command. Its `Err` arrives as an `Error` with a `code`.
 * @param {string} command
 * @param {object} [args]
 * @returns {Promise<any>}
 */
function callNative(command, args) {
  if (!inApp()) {
    return Promise.reject(Object.assign(new Error("Open this in the app."), { code: "app.outside" }));
  }
  return window.__TAURI__.core.invoke(command, args).catch((err) => {
    const failure = err && typeof err === "object" && typeof err.message === "string" ? err : null;
    const e = new Error(failure ? failure.message : String(err));
    e.code = failure ? failure.code : "app.unknown";
    throw e;
  });
}

/**
 * Hear an event Rust emits to this window, until the returned function runs.
 * @param {string} name
 * @param {(payload: any) => void} heard
 * @returns {() => void}
 */
function hear(name, heard) {
  if (!inApp()) return () => {};
  const listening = window.__TAURI__.webviewWindow
    .getCurrentWebviewWindow()
    .listen(name, (e) => heard(e.payload));
  return () => { listening.then((stop) => stop()); };
}

/**
 * While `active`, take Android's back gesture, until the returned function
 * runs. While it is taken, back no longer goes back by itself: `heard` decides.
 * @param {boolean} active
 * @param {(canGoBack: boolean) => void} heard
 * @returns {() => void}
 */
function takeBack(active, heard) {
  if (!active || !inApp() || platformName() !== "android") return () => {};
  const listening = window.__TAURI__.app.onBackButtonPress((e) => heard(e.canGoBack));
  return () => { listening.then((l) => l.unregister()); };
}

/** Go back one route, as the back gesture does when nothing takes it. */
function goBack() {
  history.back();
}

/**
 * The notes in the library folder.
 * @returns {Promise<any[]>}
 */
function listNotes() {
  if (!inApp()) return Promise.resolve([]);
  return callNative("notes_list");
}

/**
 * Write one note; resolves with its path.
 * @param {string} title
 * @param {string} body
 * @returns {Promise<string>}
 */
function saveNote(title, body) {
  return callNative("notes_save", { title, body });
}

/**
 * Ask for a folder; `null` when the reader cancels.
 * @returns {Promise<?string>}
 */
function pickFolder() {
  if (!inApp()) return Promise.resolve(null);
  return callNative("library_pick");
}

/** Minimise the window. */
function minimizeWindow() {
  if (inApp()) window.__TAURI__.window.getCurrentWindow().minimize();
}

/** Maximise the window, or put it back. */
function toggleMaximizeWindow() {
  if (inApp()) window.__TAURI__.window.getCurrentWindow().toggleMaximize();
}

/** Close the window. */
function closeWindow() {
  if (inApp()) window.__TAURI__.window.getCurrentWindow().close();
}

// `persist … { in: .app }` keeps its values through the store plugin.
window.wfStorage = {
  app: inApp()
    ? {
        get: (name) => callNative("store_get", { name }),
        set: (name, text) => { callNative("store_set", { name, text }).catch((e) => console.error(e.message)); },
        remove: (name) => { callNative("store_remove", { name }).catch((e) => console.error(e.message)); },
      }
    : {
        get: (name) => localStorage.getItem(name),
        set: (name, text) => localStorage.setItem(name, text),
        remove: (name) => localStorage.removeItem(name),
      },
};
```

The rules:
- **Arguments are an object in camelCase.** Tauri maps `{ noteId }` to the
  Rust parameter `note_id`.
- **The JSDoc is the type.**
  - `@param {string}` is `String` and `@returns {Promise<string>}` is what
    `await` gives. `?string` is `String?`.
  - A wrapper with no JSDoc is `Any` and checks nothing. Write the comment.
- **Names.** A Rust command is `area_verb` in snake case (`notes_save`), and
  its wrapper is camelCase (`saveNote`).
- **Plugins too.** A plugin's command goes through the same wrapper:
  `callNative("plugin:clipboard-manager|read_text")`.
- **Don't clash.** A wrapper may not reuse a name the language or browser
  owns (`fetch`, `format`, `log`, `navigator`) or a name the project declares.
  That is `E110`.
- **Platform facts are functions.** The pages ask `onPhone()` or
  `platformName()`. They never read the user agent themselves.

## Rust commands

Return a `Result` whose error is a small serialisable record with a **stable
code** and a sentence a person can read. Make anything slow `async`.

```rust
// src-tauri/src/lib.rs
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize)]
pub struct Failure { code: &'static str, message: String }
type Answer<T> = Result<T, Failure>;

#[derive(Serialize)]
pub struct Note { path: String, title: String, modified: String } // ISO 8601: a DateTime

#[tauri::command]
async fn notes_list(app: AppHandle) -> Answer<Vec<Note>> {
    let dir = library(&app).ok_or(Failure { code: "library.none", message: "Choose a folder first.".into() })?;
    read_notes(&dir).map_err(|e| Failure { code: "library.read", message: e.to_string() })
}

#[tauri::command]
async fn notes_save(app: AppHandle, title: String, body: String) -> Answer<String> {
    let path = write_note(&app, &title, &body)
        .map_err(|e| Failure { code: "note.write", message: e.to_string() })?;
    let _ = app.emit("notes:changed", &path);            // event names: `area:what`, never a dot
    Ok(path)
}

#[tauri::command]
async fn library_pick(app: AppHandle) -> Answer<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app.dialog().file().set_title("Choose a folder").blocking_pick_folder();
    Ok(picked.and_then(|p| p.into_path().ok()).map(|p| p.display().to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build());

    // What only a desktop has: a window remembered, one instance, a menu bar.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_window_state::Builder::default().build());

    builder
        .invoke_handler(tauri::generate_handler![notes_list, notes_save, library_pick, store_get, store_set, store_remove])
        .run(tauri::generate_context!())
        .expect("the app failed to start");
}
```

```rust
// src-tauri/src/main.rs — the desktop's entry; a phone calls run() itself
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() { notes_lib::run() }
```

```toml
# src-tauri/Cargo.toml
[lib]
name = "notes_lib"
crate-type = ["staticlib", "cdylib", "rlib"]   # the phone needs the first two
```

- **`#[cfg(desktop)]` and `#[cfg(mobile)]`** pick what each kind of platform
  registers: menus, window state, a global shortcut or an updater on the
  desktop; haptics or a barcode scanner on the phone.
- **Dates.** A date is sent as an ISO 8601 string, and the `.wf` `type`
  declares it as a `DateTime`. Money is `{ amount, currency }` in minor units.
  Records arrive as plain JSON, so declare a `type` for each one Rust sends.
- **Dialogs in Rust.** A dialog opened in Rust needs no `dialog:` permission
  for the page.
- **Plugins differ by platform.** Check each one's supported platforms
  before you rely on it on a phone; picking a folder, in particular, is a
  desktop idea.
- **Long work.** Spawn a thread and `emit` its progress. For a stream, use a
  `tauri::ipc::Channel`. A phone may suspend the app mid-task, so make long
  work resumable.
- **Errors.** The page branches on `e.code` and shows `e.message`. A finding a
  person must act on is an answer (`{ ok: false, … }`), not an `Err`.

## Calling Rust from a store

A store owns the calls, the loading and failure state, and what the page shows.
Pages call the store's actions and never call the bridge directly.

```wf
/// One note, as `notes_list` returns it.
type Note {
    path: String
    title: String
    modified: DateTime
}

/// The notes on disk. Every call to Rust goes through `src/native.js`.
store Notes {
    state items: [Note] = []
    state loading = false
    state failure = ""
    /// Where the notes live, kept with the app's own settings.
    persist folder = "" { in: .app }

    derived count = items.length

    action load() {
        loading = true
        failure = ""
        try {
            items = await listNotes()
        } catch e {
            failure = e.message
        }
        loading = false
    }

    action save(title: String, body: String) {
        try {
            await saveNote(title, body)
            await load()
        } catch e {
            failure = e.message
        }
    }

    action choose() {
        let picked = await pickFolder()
        if picked != null {
            folder = picked
            await load()
        }
    }

    /// Rust says a file changed on disk.
    action changed(_path: String) {
        load()
    }
}
```

The page:
- runs the store's set-up once;
- hears Rust for exactly as long as it shows (an `effect` with a `cleanup`);
- takes a keyboard shortcut;
- tells a reader in a browser tab what they're missing.

```wf
page Library(path: "/", title: "Notes", description: "The notes in your library folder.") {
    use Notes
    state draftTitle = ""
    state draftBody = ""

    // Set-up: read the folder once, and hear Rust while the page shows.
    Notes.load()
    effect {
        let stop = hear("notes:changed", Notes.changed)
        cleanup { stop() }
    }
    on key("mod+s") { Notes.save(draftTitle, draftBody) }

    Heading("Notes").h1
    if !inApp() {
        Alert("Open this in the app to read your notes from disk.").info
    }
    Row(gap: .sm, align: .center).stacks {
        Text(if Notes.folder == "" { "No folder chosen." } else { Notes.folder }).muted
        Button("Choose a folder") { on click { Notes.choose() } }
    }
    if Notes.failure != "" {
        Alert(Notes.failure).danger
    }
    Input(bind: draftTitle, label: "Title")
    Textarea(bind: draftBody, label: "Note", rows: 6)
    Button("Save", disabled: draftTitle == "").primary.full { on click { Notes.save(draftTitle, draftBody) } }

    if Notes.loading {
        Spinner.sm
    } else if Notes.count == 0 {
        Text("No notes yet.").muted
    } else {
        for note in Notes.items by note.path {
            Row(gap: .sm, justify: .between) {
                Text(note.title)
                Text(ago(note.modified)).muted.sm
            }
        }
    }
}
```

There are two ways to tie a listener to a lifetime:
- an `effect { … cleanup { … } }` at the top of a page or component (above);
- `mount:` and `cleanup:` on an element:
  `Stack(mount: (n) => hear("sync:progress", setDone), cleanup: (stop) => stop()) { … }`.

Either way, a page that leaves stops hearing. Never call `listen` from a bare
script at load time, because nothing would ever stop it.

## Keeping settings: `persist { in: .app }`

`persist x = … { in: .app }` reads and writes through `window.wfStorage.app`,
which the bridge above defines.
- **Inside the app** it's backed by `tauri-plugin-store` and kept in the app's
  own data folder, so it survives updates. It's shared by every window.
- **In a browser tab** it falls back to `localStorage`.

`get` may return a promise. On the Rust side, three small commands do the work:

```rust
use tauri_plugin_store::StoreExt;

#[tauri::command]
fn store_get(app: AppHandle, name: String) -> Answer<Option<String>> {
    let store = app.store("settings.json").map_err(|e| Failure { code: "store.open", message: e.to_string() })?;
    Ok(store.get(&name).map(|v| v.to_string()))
}

#[tauri::command]
fn store_set(app: AppHandle, name: String, text: String) -> Answer<()> {
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| Failure { code: "store.text", message: e.to_string() })?;
    let store = app.store("settings.json").map_err(|e| Failure { code: "store.open", message: e.to_string() })?;
    store.set(name, value);
    Ok(())
}

#[tauri::command]
fn store_remove(app: AppHandle, name: String) -> Answer<()> {
    let store = app.store("settings.json").map_err(|e| Failure { code: "store.open", message: e.to_string() })?;
    store.delete(&name);
    Ok(())
}
```

- **Several windows (desktop).** Have `store_set` emit `store:changed` to
  every window except the writer (`app.emit_filter(…, |t| label != writer)`),
  and give `wfStorage.app` a `watch(name, heard)` that listens **as this
  window**, or each window hears its own writes.
- **Flushing on a phone.** A phone may kill a backgrounded app without
  warning. Save the store as soon as a value changes; don't wait for exit.
- **`key:`.** A persisted value can be kept per project with `key:`:
  `persist sideOpen = true { key: project  in: .app }`.
- **Secrets.** Secrets (tokens, API keys) never go in `persist`. Keep them in
  the OS keychain (desktop: the `keyring` crate; phone: the platform's secure
  storage, through a plugin) behind commands that never return the secret to
  the page.

## Capabilities

Capabilities grant the page **Tauri core and plugin** commands. The project's
own commands aren't covered by them: any webview may call those. Give the
desktop and the phone a file each, by `platforms`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "desktop",
  "platforms": ["linux", "macOS", "windows"],
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-minimize",
    "core:window:allow-toggle-maximize",
    "core:window:allow-close",
    "core:window:allow-start-dragging",
    "store:default"
  ]
}
```

```json
{
  "$schema": "../gen/schemas/mobile-schema.json",
  "identifier": "mobile",
  "platforms": ["android", "iOS"],
  "windows": ["main"],
  "permissions": ["core:default", "store:default", "haptics:default"]
}
```

- **Adding a plugin.** `cargo tauri add dialog` (or `store`, `opener`,
  `clipboard-manager`, `fs`, `notification`, `haptics`, `biometric`,
  `barcode-scanner`, `geolocation`) adds the crate, the `.plugin(…)` line and
  its default permission. A plugin's call fails until its permission is
  granted.
- **The back gesture.** `core:default` includes what `takeBack` (Tauri's
  `onBackButtonPress`) needs (`core:app:allow-register-listener`).
- **Phone permissions.** A phone plugin may also need an Android permission
  in `gen/android/app/src/main/AndroidManifest.xml`, or an iOS usage string in
  `Info.plist` (the camera for a barcode, location). The plugin's page names
  them.
- **Pages you didn't write.** If the app shows pages it didn't write (a
  preview, a user's site), keep those webviews out of the capability. Also
  reject invokes from them in the invoke handler: check the calling webview's
  label and call `invoke.resolver.reject(…)`.

## One shell for every platform

`app` draws the chrome once and asks the bridge which platform it's on:
- **On a desktop:** a title bar of its own, and room for menus.
- **On a phone:** padding for the status bar, the notch and the gesture bar,
  and nothing to drag.

```wf
/// The window's own title bar, on a desktop: the app draws it, so
/// `decorations` is off. Every empty part carries `data-tauri-drag-region`
/// to move the window.
component TitleBar(_ title: String) {
    derived onMac = platformName() == "mac"
    Row(class: "titlebar", align: .center, data-tauri-drag-region: true) {
        Text(title, class: "titlebar__title", data-tauri-drag-region: true).bold
        Row(class: "titlebar__fill", data-tauri-drag-region: true) { }
        if !onMac {
            Row(class: "titlebar__buttons", gap: .xs) {
                IconButton(icon: "window-minimize", label: "Minimise") { on click { minimizeWindow() } }
                IconButton(icon: "window-maximize", label: "Maximise") { on click { toggleMaximizeWindow() } }
                IconButton(icon: "close", label: "Close") { on click { closeWindow() } }
            }
        }
    }
}

/// Every page's frame: a title bar on a desktop, the safe areas on a phone.
component WindowShell(_ title: String) {
    slot
    derived phone = onPhone()
    Stack(class: { "shell": true, "shell--phone": phone }) {
        if !phone {
            TitleBar(title)
        }
        Stack(class: "shell__body") {
            children
        }
    }
}

app {
    WindowShell("Notes") {
        Router
    }
}
```

```css
/* src/shell.css */
.shell { min-height: 100vh; }
.titlebar { height: 36px; user-select: none; background: var(--color-surface); padding-inline: var(--spacing-sm); }
.titlebar__fill { flex: 1; align-self: stretch; }
/* A phone draws edge to edge: keep the content out from under the bars. */
.shell--phone .shell__body {
  padding-top: env(safe-area-inset-top);
  padding-bottom: env(safe-area-inset-bottom);
  padding-left: env(safe-area-inset-left);
  padding-right: env(safe-area-inset-right);
}
/* Fingers are wider than pointers. */
.shell--phone .wf-btn { min-height: 44px; }
```

## Desktop: window chrome, menus, several windows

- **The title bar.**
  - `data-tauri-drag-region` must sit on the bar **and** on every empty
    filler inside it, because a child without it doesn't drag.
  - The buttons need the four `core:window:allow-*` permissions above.
  - On macOS, keep the system's traffic lights (`"titleBarStyle": "Overlay"`
    in `tauri.macos.conf.json`) and draw no buttons of your own.
- **One table of commands.** Keep a single table of what the app can do
  (id, label, keys such as `mod+s`).
  - A page-level `on keydown(e)` runs the one whose `keyIs(e, keys)` matches,
    and menus, a command palette and the help are all drawn from that table.
  - `keyLabel(keys)` writes the keys the way this system shows them.
  - `on key("mod+s") { … }` at the top of a page listens while the page
    shows. `mod` is ⌘ on a Mac and Ctrl elsewhere. `keyIs` matches by key
    position, so shortcuts work on an Arabic layout too.
- **The macOS menu bar.**
  - Build it in Rust (`#[cfg(desktop)]`) from the same table, turning `mod+s`
    into `CmdOrCtrl+S`.
  - Have it emit `menu:run` with the command's id to the focused window, and
    let the page run it.
  - On Linux and Windows, an in-page `Menu` usually reads better than a
    native bar.
- **Several windows.**
  - One single-page build serves every window. Each window loads
    `index.html`, and the page asks which window it is in
    (`getCurrentWebviewWindow().label`, wrapped in the bridge) and branches.
  - In Rust, `window.emit_to(label, …)` sends to one window and
    `app.emit_filter(…)` to all but one.
- **Avoid a flash.** Start the window hidden (`"visible": false`). Call a
  `window_show` command once the first `wfStorage.app.get` calls have
  resolved, and keep a timeout fallback in Rust. This avoids a flash of
  default sizes.

## Phone: Android and iOS

### Set up

- **Android.**
  - Install Android Studio with an SDK, the NDK and a JDK, and set
    `ANDROID_HOME` and `NDK_HOME`.
  - Add the Rust targets:
    `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`.
- **iOS.**
  - Needs a Mac with Xcode, and an Apple developer account for a device or
    the App Store.
  - Add the Rust targets:
    `rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios`.
- **Generate the native projects.** `cargo tauri android init` writes
  `src-tauri/gen/android/`, and `cargo tauri ios init` writes
  `src-tauri/gen/apple/`. Commit them: they are where the manifest, the
  icons, the signing and `MainActivity.kt` live.

### Run while you work

```bash
cargo tauri android dev            # an emulator, or the phone on adb
cargo tauri android dev --host     # a phone on the network: devUrl becomes your machine's address
cargo tauri ios dev                # the simulator, or a device (on a Mac)
cargo tauri android dev --open     # in Android Studio
```

`beforeDevCommand` runs `wf serve`. A save rebuilds and reloads the app on the
phone, and a failed build shows its error over the page, as in a browser.

**Debug the page.**
- **Android:** `chrome://inspect` in a desktop Chrome lists the app's WebView
  in a debug build, with its console and elements. `adb logcat` has the Rust
  side.
- **iOS:** Safari's Develop menu shows the simulator's or device's WebView.

### What's different on a phone

- **There is no window.** There's no title bar, no drag region, no minimise
  and no menu bar, and multiple windows are a desktop idea. The shell above
  draws none of them on a phone.
- **Edge to edge.**
  - Tauri's Android template calls `enableEdgeToEdge()` (in
    `gen/android/app/src/main/java/…/MainActivity.kt`), and Android 15 and
    later draw apps edge to edge regardless. The page draws under the status
    bar and the gesture bar. On an iPhone, the notch and the home indicator
    do the same.
  - The bridge sets `viewport-fit=cover`, and `.shell--phone .shell__body`
    pads by `env(safe-area-inset-*)`.
  - Check on the oldest Android you support: if the insets read 0 there, pad
    the top and bottom by a fixed amount for Android, or report the real
    insets from Kotlin into CSS variables.
- **The back gesture.**
  - On Android, back goes back in the WebView's history. WebFluent's router
    keeps that history, so back returns to the previous route, and on the
    first route it leaves the app. That's usually right with no code.
  - When something is open (a `Modal`, a drawer, a sheet), back should close
    it first. Take the gesture while it's open with `takeBack`, as in the
    example after this list.
  - iOS has no back button: give every screen a visible way back
    (`Breadcrumb`, a back `IconButton`), and `on swipe(s)` for the edge swipe
    if you want it.

An inbox whose filter sheet closes on back. The effect runs again when
`sheetOpen` changes, and its `cleanup` gives the gesture back:

```wf
page Inbox(path: "/inbox", title: "Inbox", description: "Messages, with a sheet that back closes.") {
    state sheetOpen = false

    action closeSheet(_canGoBack: Bool) {
        sheetOpen = false
    }

    // While the sheet is open, Android's back closes it instead of leaving.
    effect {
        let stop = takeBack(sheetOpen, closeSheet)
        cleanup { stop() }
    }

    Heading("Inbox").h1
    Button("Filters") { on click { sheetOpen = true } }
    Modal(visible: sheetOpen, title: "Filters") {
        on close { sheetOpen = false }
        Text("Only unread")
    }
}
```

More that differs on a phone:

- **Touch, not a pointer.**
  - Make targets at least 44 px.
  - Nothing should only appear on hover. `Tooltip` is a desktop affordance,
    so the label must also be visible or spoken (`IconButton`'s `label:`).
  - Use `on longpress(p)` for a context menu, and `on swipe(s)` for a swipe
    action.
  - `drag:` and `on receive` work with a finger, which picks a row up after
    it is held still.
- **Layout.**
  - Start from a phone's width: `Grid(columns: { base: 1, md: 3 })`,
    `Row(…).stacks`, a bottom bar instead of a sidebar
    (`if viewport.md { Sidebar … } else { BottomTabs … }`, which is behaviour,
    so `viewport` is right here).
  - `wf serve` in a desktop browser's device mode shows the layout before you
    run on a phone.
- **The soft keyboard.**
  - The input flags pick the right keyboard: `.email`, `.tel`, `.number`,
    `.url`, `.search`.
  - The keyboard covers the bottom of the screen, so don't pin a form's only
    submit button there.
  - `press "Enter"` in a field submits its `Form`, on a phone too.
- **The app goes to the background.** The OS may stop it at any time. Persist
  what the reader typed as they type (`persist draft = ""`), not on exit.
  `document.visibilityState` and a page-level `on visibilitychange` tell you
  when it comes back, so refresh then (`resource.reload()`).
- **Network.**
  - `network.online` and `network.saveData` matter more on a phone. Show a
    note when offline.
  - An `api` can retry on `.network`, and `on: .reconnect` asks again when the
    connection returns.
- **Platform features.** Use phone-only plugins (`haptics`, `biometric`,
  `barcode-scanner`, `geolocation`, `nfc`) through the bridge, each with a
  browser and desktop fallback. Register them under `#[cfg(mobile)]`, and
  grant them only in `mobile.json`.

## Theme, language, fonts

- **Theme.** `effect { setTheme(Settings.appearance) }` with `"system"`,
  `"light"` or `"dark"`. `"system"` follows the OS through
  `prefers-color-scheme`, on desktops and phones alike. Declare both themes
  and name them in `theme.name`/`theme.dark`.
- **Language.** `effect { setLocale(Settings.language) }`. Arabic flips the
  whole app right to left. Use logical CSS properties
  (`margin-inline-start`) so the chrome flips with it. The default can follow
  the device:
  `derived deviceLanguage = if (navigator.language ?? "").startsWith("ar") { "ar" } else { "en" }`.
- **Fonts.** Ship them in `public/fonts/`. A phone may be offline on first
  launch.

## Files, custom protocols, previews, libraries

- **Files dropped on a desktop window.** In Rust, match
  `WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. })` in
  `on_window_event` and emit `files:dropped`. A phone gets files through a
  picker or a share intent instead.
- **A custom protocol** (`register_uri_scheme_protocol("myproto", …)`) is
  addressed in two forms:
  - `myproto://localhost/…` on Linux, macOS and iOS;
  - `http://myproto.localhost/…` on Windows and Android.

  List both in `meta.frame` (or `meta.connect`).
- **Showing a page in a `Frame`.**
  `Frame(src: url, title: "Preview", sandbox: "allow-scripts") { on message(m) { … } }`
  hears only what that page posts. A page that should have no IPC must not be
  served from the app's origin.
- **Redirects.** A webview doesn't follow a redirect from a custom scheme.
  Serve a page that replaces its own address instead.
- **Libraries.**
  - Build a library (CodeMirror, a chart library) once into
    `public/vendor/x.js` and list it in `meta.scripts` with
    `"globals": ["Lib"]`.
  - Wrap it in a typed function, and mount it with
    `Host(tag: "div", mount: (n) => make(n), cleanup: (h) => h.destroy())`.
  - Draw popups inside its nodes with `Portal(into: node) { … }`.
  - Heavy work goes in a `worker` (`spawn("/workers/x.js")`).

## Testing the app

- **`wf test` and `wf verify`.** They run in a browser, where `inApp()` is
  false. A wrapper that answers outside the app keeps every page testable.
  Test what the page shows in that state, and test the logic with data seeded
  through `test "…"(data: { Notes: { items: […] } })`.
- **Phone layouts.** Test them with `wf serve` in a browser's device mode, and
  on an emulator or simulator with `cargo tauri android dev` / `ios dev`.
- **Rust commands.** Unit-test them against Tauri's mock runtime (the `test`
  feature of the `tauri` crate).
- **End-to-end on Linux.** Run under `xvfb-run` with its own `XDG_*` folders
  and `GDK_BACKEND=x11`. Never drive your own desktop session, because
  synthetic keys go to whatever window has focus.

## Packaging and release

- **Desktop.**
  - `cargo tauri build` runs `beforeBuildCommand` (`wf build`) and writes the
    installers under `src-tauri/target/release/bundle/`.
  - In CI, build the UI once, then use `tauri-apps/tauri-action` for each OS.
    If the UI is already built, add a `tauri.ci.conf.json` holding
    `{"build":{"beforeBuildCommand":""}}`.
  - macOS: `--target universal-apple-darwin`.
  - Linux: needs `libwebkit2gtk-4.1-dev libsoup-3.0-dev
    libjavascriptcoregtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`.
  - Keep a WiX `upgradeCode` stable across releases.
- **Android.**
  - `cargo tauri android build --apk` for a file to install, and
    `cargo tauri android build --aab` for Google Play. `--target aarch64`
    builds one architecture, and `--split-per-abi` writes one per
    architecture.
  - Release builds must be signed: make a keystore with `keytool`, reference
    it from a `keystore.properties` that is **not** committed, and sign in
    `gen/android/app/build.gradle.kts`.
  - Raise the version in `tauri.conf.json` for every upload; the Android
    version code is derived from it.
  - For Android 15's 16 KB pages, link with `-Wl,-z,max-page-size=16384` (in
    `.cargo/config.toml` for the android targets).
- **iOS.**
  - `cargo tauri ios build` on a Mac.
  - Set the signing team (`bundle.iOS.developmentTeam`, or in Xcode through
    `cargo tauri ios dev --open`), then upload the archive with Xcode or
    `xcrun altool`.
- **One version everywhere.** `version` in `tauri.conf.json` is the app's
  version on every platform. Keep `webfluent.app.json`'s `version` the same.

## Pitfalls

| What you see | Why, and the fix |
|---|---|
| Every call fails with "Open this in the app." | The page is in a browser tab, or `withGlobalTauri` is off |
| A call is blocked by the security policy on Windows or Android only | `meta.connect` lacks `http://ipc.localhost` (and `ipc:` for the others) |
| A plugin call is refused | Its permission is missing from the capability for that platform |
| The window shows nothing in `cargo tauri dev` | `devUrl` doesn't match `dev.port`, or `wf serve` failed: run `wf check` |
| The phone shows a connection error in dev | The phone can't reach your machine: same network, `--host`, and the firewall open on the port |
| Content sits under the status bar or the gesture bar | Edge to edge: `viewport-fit=cover` and `env(safe-area-inset-*)` padding on the phone shell |
| Back leaves the app with a dialog open | Take the gesture with `takeBack(open, close)` in an effect while it's open |
| Back does nothing at all | The gesture is still taken: the effect's `cleanup` must give it back |
| A window hears its own events | It listened app-wide; listen with `getCurrentWebviewWindow().listen`, and emit with `emit_to`/`emit_filter` |
| An event never arrives | Its name has a `.`; Tauri refuses it. Use `area:what` |
| A library's injected `<style>` is refused | Set `meta.style_elements: true` |
| WebAssembly fails to compile | Set `meta.wasm: true` |
| Console code fails with an `eval` error | The policy forbids `eval`; run debug code from Rust with the webview's own script call |
| The font changes offline | It came from `meta.fonts`; ship it in `public/fonts/` with `@font-face` |
| On Linux, an `<iframe>` freezes the app while it loads | WebKitGTK runs frames on the app's thread; show heavy pages in a second webview instead |
| A shortcut breaks on an Arabic layout | A library matched `e.key`; WebFluent's `keyIs` matches by position |
| Settings typed on a phone are lost | The OS killed the app in the background; persist as the reader types |

## Checklist before you call it done

- [ ] `wf check` is clean, and every bridge function has JSDoc.
- [ ] Every wrapper answers outside the app, so `wf test` and `wf verify` pass
      in a browser.
- [ ] Every listener stops with its page (`cleanup`), and event names are
      `area:what`.
- [ ] `meta.connect` (and `meta.frame`, if needed) names both IPC origins.
- [ ] Capabilities are split by platform, and grant only what the pages use.
      Your own commands refuse webviews that aren't yours.
- [ ] On a phone: safe areas padded, back closes what's open first, targets
      are 44 px, nothing hides behind hover, and the layout works at 360 px
      wide.
- [ ] Secrets stay in Rust and the platform's keychain, never in `persist`.
- [ ] `cargo tauri build`, and `cargo tauri android build` / `ios build` for
      the phones you ship, succeed with your own `identifier` and signing.
