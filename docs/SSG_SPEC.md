# WebFluent Static Site Generation (SSG) Specification

> Version 1.0 — Draft
> Author: Monzer Omer
> Date: 2026-03-24

---

## Overview

By default, WebFluent builds a **Single-Page Application** — one `index.html` with an empty `<div id="app"></div>` that gets populated entirely by JavaScript. This works great for interactive apps, but means users see a blank white screen until JS loads.

**Static Site Generation (SSG)** pre-renders each page's HTML at build time. The browser shows content immediately, then JavaScript **hydrates** the page to add interactivity. Best of both worlds: fast initial paint + full SPA behavior.

---

## 1. Enabling SSG

Add `"ssg": true` to the build config in `webfluent.app.json`:

```json
{
    "build": {
        "output": "./build",
        "ssg": true
    }
}
```

That's it. The next `webfluent build` will generate per-page HTML files with pre-rendered content.

---

## 2. Build Output

### SPA mode (default)

```
build/
├── index.html          ← Single shell with empty <div id="app">
├── app.js              ← Full runtime + all pages
└── styles.css
```

### SSG mode

```
build/
├── index.html          ← Pre-rendered "/" page
├── about/
│   └── index.html      ← Pre-rendered "/about" page
├── settings/
│   └── index.html      ← Pre-rendered "/settings" page
├── app.js              ← Hydration runtime + all pages
└── styles.css
```

Each page gets its own HTML file at its route path. The HTML contains the full page markup, so content is visible without JavaScript.

---

## 3. What Gets Pre-Rendered

### Static content (rendered at build time)

| Element | Pre-rendered as |
|---------|----------------|
| `Container`, `Row`, `Column`, etc. | `<div class="wf-container">...</div>` |
| `Text("Hello")` | `<p class="wf-text">Hello</p>` |
| `Heading("Title", h1)` | `<h1 class="wf-heading">Title</h1>` |
| `Button("Save", primary)` | `<button class="wf-btn wf-btn--primary">Save</button>` |
| `Card(elevated) { ... }` | `<div class="wf-card wf-card--elevated">...</div>` |
| `Image(src: "...", alt: "...")` | `<img class="wf-image" src="..." alt="...">` |
| `Input(text, placeholder: "...")` | `<input class="wf-input" type="text" placeholder="...">` |
| `Navbar { ... }` | `<nav class="wf-navbar">...</nav>` |
| `t("key")` (i18n) | Default locale text rendered statically |
| Nested static components | Full HTML tree |

### Dynamic content (skipped, handled by JS)

| Element | SSG behavior |
|---------|-------------|
| `state count = 0` | Skipped (initialized by JS) |
| `Text(count)` | Renders empty `<p class="wf-text"></p>` |
| `if condition { ... }` | Renders empty placeholder `<!--wf-if-->` |
| `for item in items { ... }` | Renders empty placeholder `<!--wf-for-->` |
| `show condition { ... }` | Renders content hidden (`display:none`) |
| `fetch ... { ... }` | Renders `loading` block if present, else empty |
| Event handlers | Attached during hydration |
| `navigate()` | Works after hydration |

---

## 4. Hydration

After the browser loads the pre-rendered HTML, `app.js` runs and **hydrates** the page:

1. Finds the existing DOM (doesn't destroy it)
2. Initializes reactive signals with their default values
3. Attaches event listeners to existing elements
4. Fills in dynamic content (state-dependent text, conditionals, loops)
5. SPA routing takes over for subsequent navigation

The user sees content immediately. Interactivity appears as soon as JS loads — typically within milliseconds.

### Hydration vs. Full Render

| | SPA (default) | SSG |
|---|---|---|
| Initial HTML | Empty `<div id="app">` | Full pre-rendered content |
| Time to content | After JS loads + executes | Immediate |
| JS behavior | Builds entire DOM | Hydrates existing DOM |
| Subsequent navigation | Client-side routing | Client-side routing (same) |
| Output files | 1 HTML file | 1 HTML file per page |

---

## 5. Generated HTML Structure

Each pre-rendered page looks like:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Page Title</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div id="app">
        <!-- Pre-rendered page content -->
        <nav class="wf-navbar">
            <div class="wf-navbar__brand">
                <p class="wf-text wf-text--heading">My App</p>
            </div>
            ...
        </nav>
        <div class="wf-container">
            <h1 class="wf-heading">Welcome</h1>
            <p class="wf-text">This content is visible immediately.</p>
        </div>
    </div>
    <script src="/app.js"></script>
</body>
</html>
```

Note: CSS and JS paths use absolute paths (`/styles.css`, `/app.js`) so they work from any route depth.

---

## 6. Dynamic Routes

Pages with dynamic segments (e.g., `/user/:id`) cannot be pre-rendered without knowing the possible values. These pages fall back to SPA behavior — they get the base HTML template and render client-side.

In the future, a data source (JSON file, API) could provide the list of dynamic route values for pre-rendering.

---

## 7. i18n + SSG

When both i18n and SSG are enabled, pages are pre-rendered using the **default locale**. The JS hydration switches to the user's preferred locale on load.

---

## 8. Example

### Source

```wf
Page Home (path: "/", title: "Home") {
    Container {
        Heading("Welcome to My App", h1)
        Text("This loads instantly with SSG.")

        Spacer()

        Card(elevated) {
            Text("Static content pre-rendered at build time.", muted)
        }
    }
}
```

### Config

```json
{
    "build": { "ssg": true }
}
```

### Output: `build/index.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Home</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div id="app">
        <div class="wf-container">
            <h1 class="wf-heading">Welcome to My App</h1>
            <p class="wf-text">This loads instantly with SSG.</p>
            <div class="wf-spacer"></div>
            <div class="wf-card wf-card--elevated">
                <p class="wf-text wf-text--muted">Static content pre-rendered at build time.</p>
            </div>
        </div>
    </div>
    <script src="/app.js"></script>
</body>
</html>
```

This HTML is viewable with JavaScript disabled — content appears instantly.
