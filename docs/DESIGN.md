# WebFluent Language Design Document

> Version 0.1.0 — Draft
> Author: Monzer Omer
> Date: 2026-03-24

---

## 1. Vision

WebFluent is a **web-first programming language** designed to build fully functional single-page applications (SPAs) with minimal boilerplate. It compiles exclusively to **HTML + CSS + JavaScript** — no server runtime, no virtual DOM library dependency at runtime, just clean web output.

The language ships with a **built-in component library** that covers the most common UI patterns on the web. Every built-in component has a **default design** out of the box, and the entire design system can be swapped or customized through a single configuration point.

### Core Principles

1. **Web-Only** — WebFluent targets the browser. It compiles to standard HTML, CSS, and JS. No native, no desktop, no mobile. The web is the platform.

2. **Batteries Included** — Common components (Navbar, Card, Modal, Form, etc.) are built into the language. You don't import a UI library — the language *is* the UI library.

3. **Design System First** — Every built-in component renders with a coherent default design. Switching themes is a one-line config change. Custom design systems can override any token (colors, spacing, typography, radii, shadows).

4. **Simplicity Over Flexibility** — The language favors a single clear way to do things. Fewer concepts, less ceremony, faster results.

5. **Readable Syntax** — Code should read like a description of what the page looks like. Curly braces for nesting, parentheses for attributes, no XML/HTML tags.

---

## 2. Language Goals

| Goal | Description |
|------|-------------|
| **Build SPAs** | Client-side routing, state management, reactivity — all built in |
| **Compile to Web** | Output is static HTML + CSS + JS files. No runtime framework dependency |
| **Built-in Components** | Ship 20+ production-ready components with default styling |
| **Swappable Design System** | Change the entire look and feel via a theme configuration |
| **Fast Compilation** | Instant feedback during development with hot reload |
| **Zero Config Start** | `webfluent init` → write code → `webfluent serve` → working app |
| **Small Output** | Generated JS should be minimal — only include what's used (tree-shaking) |

---

## 3. Architecture Overview

```
┌─────────────┐     ┌─────────┐     ┌──────────┐     ┌───────────────┐
│  .wf files  │────▶│  Lexer  │────▶│  Parser  │────▶│   Compiler    │
│  (source)   │     │(tokens) │     │  (AST)   │     │(HTML+CSS+JS)  │
└─────────────┘     └─────────┘     └──────────┘     └───────────────┘
                                                            │
                                          ┌─────────────────┼──────────────────┐
                                          ▼                 ▼                  ▼
                                     index.html         styles.css         app.js
                                   (structure)        (design system)    (reactivity,
                                                                        routing, state)
```

### 3.1 Compilation Pipeline

1. **Lexer** — Tokenizes `.wf` source files into a stream of tokens (keywords, identifiers, literals, operators, punctuation).

2. **Parser** — Transforms tokens into an Abstract Syntax Tree (AST). Validates syntax, resolves component references, and builds a tree of pages, components, state, events, and expressions.

3. **Analyzer** — (New stage) Performs semantic analysis: type checking, state dependency tracking, unused variable detection, component prop validation.

4. **Compiler** — Walks the AST and emits:
   - **HTML** — Semantic markup for each page
   - **CSS** — Design system tokens + component styles + user overrides
   - **JS** — Reactive runtime, event handlers, routing, state management

### 3.2 Runtime Model

The generated JavaScript uses a **fine-grained reactivity system** (similar to Solid.js signals). When state changes, only the specific DOM nodes that depend on that state are updated — no virtual DOM diffing.

```
State Change → Signal Update → Subscriber Notified → DOM Patch (surgical)
```

This keeps the runtime small and performance high.

---

## 4. File Structure

A WebFluent project has this structure:

```
my-app/
├── webfluent.app.json          # Project config (theme, port, etc.)
├── src/
│   ├── App.wf                  # Root application file (routes, global state)
│   ├── pages/
│   │   ├── Home.wf             # Page files
│   │   ├── About.wf
│   │   └── Settings.wf
│   ├── components/
│   │   ├── Header.wf           # Reusable components
│   │   └── UserCard.wf
│   ├── stores/
│   │   └── auth.wf             # Shared state stores
│   └── theme/
│       └── overrides.wf        # Design system overrides
├── public/
│   ├── images/
│   └── fonts/
└── build/                      # Compiled output (generated)
    ├── index.html
    ├── app.js
    └── styles.css
```

---

## 5. Core Language Concepts

### 5.1 Pages

Pages are top-level route targets. Each page defines a URL path and contains the UI tree for that route.

```
Page Home (path: "/") {
    Text("Welcome to WebFluent")
}

Page About (path: "/about") {
    Text("About us")
}
```

### 5.2 Components

Reusable UI blocks. Components accept props and can have internal state.

```
Component UserCard (name: String, role: String) {
    Card {
        Text(name, heading)
        Text(role, subtitle)
    }
}
```

### 5.3 State

Reactive state declarations. When state changes, the UI updates automatically.

```
Page Counter (path: "/counter") {
    state count = 0

    Row {
        Button("- Decrease") { count = count - 1 }
        Text(count)
        Button("+ Increase") { count = count + 1 }
    }
}
```

### 5.4 Events

Event handlers are declared inline using `on:<event>` or via action blocks on interactive elements.

```
Button("Submit") {
    on:click {
        submitForm()
    }
}

Input(text, placeholder: "Search...") {
    on:input {
        searchQuery = value
    }
}
```

### 5.5 Control Flow

Conditional rendering and loops are first-class constructs.

```
// Conditional
if isLoggedIn {
    Text("Welcome back!")
} else {
    Button("Log In") { navigate("/login") }
}

// Loop
for user in users {
    UserCard(name: user.name, role: user.role)
}

// Show/Hide (keeps element in DOM, toggles visibility)
show isVisible {
    Dialog("Settings") {
        Text("Your settings here")
    }
}
```

### 5.6 Data Fetching

Built-in async data loading with automatic loading/error states.

```
fetch users from "/api/users" {
    loading {
        Spinner()
    }
    error (err) {
        Alert("Failed to load users", danger)
    }
    success {
        for user in users {
            UserCard(name: user.name, role: user.role)
        }
    }
}
```

### 5.7 Routing

SPA routing is declared in the App file. Navigation is handled client-side.

```
App {
    Router {
        Route(path: "/", page: Home)
        Route(path: "/about", page: About)
        Route(path: "/user/:id", page: UserProfile)
        Route(path: "*", page: NotFound)
    }
}
```

Navigation between pages:

```
Link(to: "/about") {
    Text("Go to About")
}

// Programmatic navigation
Button("Go Home") {
    navigate("/")
}
```

### 5.8 Stores (Shared State)

For state that is shared across pages/components:

```
Store AuthStore {
    state user = null
    state token = ""

    action login(email: String, password: String) {
        fetch result from "/api/login" (method: "POST", body: { email, password }) {
            success {
                user = result.user
                token = result.token
            }
        }
    }

    action logout() {
        user = null
        token = ""
        navigate("/login")
    }
}
```

Using a store:

```
Page Profile (path: "/profile") {
    use AuthStore

    Text(AuthStore.user.name, heading)
    Button("Log Out") { AuthStore.logout() }
}
```

---

## 6. Design System Architecture

### 6.1 Design Tokens

The design system is built on **tokens** — named values for colors, spacing, typography, radii, and shadows. Every built-in component references tokens, never hard-coded values.

```
// webfluent.app.json
{
    "theme": {
        "name": "default",
        "tokens": {
            "color-primary": "#3B82F6",
            "color-secondary": "#64748B",
            "color-success": "#22C55E",
            "color-danger": "#EF4444",
            "color-warning": "#F59E0B",
            "color-background": "#FFFFFF",
            "color-surface": "#F8FAFC",
            "color-text": "#0F172A",
            "color-text-muted": "#64748B",

            "font-family": "Inter, system-ui, sans-serif",
            "font-size-sm": "0.875rem",
            "font-size-base": "1rem",
            "font-size-lg": "1.25rem",
            "font-size-xl": "1.5rem",
            "font-size-2xl": "2rem",

            "spacing-xs": "0.25rem",
            "spacing-sm": "0.5rem",
            "spacing-md": "1rem",
            "spacing-lg": "1.5rem",
            "spacing-xl": "2rem",

            "radius-sm": "0.25rem",
            "radius-md": "0.5rem",
            "radius-lg": "1rem",
            "radius-full": "9999px",

            "shadow-sm": "0 1px 2px rgba(0,0,0,0.05)",
            "shadow-md": "0 4px 6px rgba(0,0,0,0.1)",
            "shadow-lg": "0 10px 15px rgba(0,0,0,0.1)"
        }
    }
}
```

### 6.2 Switching Themes

Changing the entire design is a one-line change:

```json
{ "theme": { "name": "dark" } }
```

Or use a custom theme:

```json
{ "theme": { "name": "custom", "tokens": { "color-primary": "#8B5CF6", ... } } }
```

### 6.3 Built-in Themes

WebFluent ships with these themes:
- `default` — Clean, modern light theme
- `dark` — Dark mode variant
- `minimal` — Ultra-minimal with maximum whitespace
- `brutalist` — Bold, raw, high-contrast

### 6.4 Component-Level Style Overrides

Override styles on any component instance:

```
Button("Submit", primary) {
    style {
        background: "#custom-color"
        radius: lg
        padding: md
    }
}

// Or use variant modifiers
Button("Submit", primary, rounded, large)
Card(elevated, bordered) {
    Text("Content")
}
```

---

## 7. Built-in Component Library

### 7.1 Component Categories

| Category | Components |
|----------|-----------|
| **Layout** | `Container`, `Row`, `Column`, `Grid`, `Stack`, `Spacer`, `Divider` |
| **Navigation** | `Navbar`, `Sidebar`, `Breadcrumb`, `Link`, `Menu`, `Tabs` |
| **Data Display** | `Card`, `Table`, `List`, `Badge`, `Avatar`, `Tooltip`, `Tag` |
| **Data Input** | `Input`, `Select`, `Checkbox`, `Radio`, `Switch`, `Slider`, `DatePicker`, `FileUpload`, `Form` |
| **Feedback** | `Alert`, `Toast`, `Modal`, `Dialog`, `Spinner`, `Progress`, `Skeleton` |
| **Actions** | `Button`, `IconButton`, `ButtonGroup`, `Dropdown` |
| **Media** | `Image`, `Video`, `Icon`, `Carousel` |
| **Typography** | `Text`, `Heading`, `Code`, `Blockquote` |

### 7.2 Component Variants

Every component supports variants through modifier keywords:

```
// Size variants
Button("Click", small)
Button("Click", medium)       // default
Button("Click", large)

// Color variants
Button("Save", primary)
Button("Delete", danger)
Alert("Saved!", success)
Alert("Warning!", warning)

// Style variants
Card(elevated)
Card(outlined)
Card(flat)
```

---

## 8. Compilation Targets

WebFluent compiles to three output files:

### 8.1 HTML Output
- Semantic HTML5 markup
- Proper heading hierarchy
- ARIA attributes on interactive components
- `<template>` elements for conditional content

### 8.2 CSS Output
- CSS custom properties for all design tokens
- Component styles scoped via data attributes
- Responsive breakpoints built in
- Dark mode via `prefers-color-scheme` or class toggle
- Only styles for used components are included

### 8.3 JavaScript Output
- Fine-grained reactivity runtime (signals + effects)
- Client-side router (history API)
- Event delegation for performance
- State management with stores
- Fetch wrapper for data loading
- Tree-shaken — unused features are excluded

---

## 9. Developer Experience

### 9.1 CLI Commands

```bash
webfluent init                          # Create new project
webfluent serve                         # Dev server with hot reload
webfluent build                         # Production build
webfluent generate page <name>          # Generate a new page
webfluent generate component <name>     # Generate a new component
webfluent generate store <name>         # Generate a new store
webfluent theme list                    # List available themes
webfluent theme set <name>             # Switch theme
```

### 9.2 Error Messages

WebFluent provides clear, actionable error messages with source location:

```
Error: Unknown component "UserCrad" at src/pages/Home.wf:15:5
  Did you mean "UserCard"?

Error: State "cont" is not defined at src/pages/Counter.wf:8:12
  Did you mean "count"?

Error: Component "Button" expects 1-2 arguments, got 0 at src/pages/Form.wf:22:9
```

### 9.3 Hot Reload

The dev server watches for file changes and pushes updates to the browser instantly without full page reloads.

---

## 10. Non-Goals

These are explicitly **not** goals for WebFluent:

- **Server-side rendering (SSR)** — WebFluent is client-side only
- **Backend/API development** — It's a frontend language
- **Native mobile apps** — Web only
- **Package manager / ecosystem** — Built-in components cover common needs; no third-party component system initially
- **TypeScript/JavaScript interop** — WebFluent is its own language, not a JS superset

---

## 11. Future Considerations

These may be explored after v1.0:

- **Animation system** — Declarative transitions and animations
- **Internationalization (i18n)** — Built-in multi-language support
- **Accessibility linting** — Compile-time accessibility checks
- **Static site generation** — Pre-render pages at build time
- **Plugin system** — Allow extending the component library
- **IDE extension** — Syntax highlighting, autocomplete, error squiggles
