# WebFluent

A web-first programming language that compiles to HTML, CSS, and JavaScript. Build fully functional single-page applications and static sites with built-in components, a design system, reactivity, routing, i18n, and animations — no frameworks, no dependencies, just clean web output.

```
Page Home (path: "/") {
    Container {
        Heading("Hello, WebFluent!", h1, fadeIn)
        Text("Build for the web. Nothing else.")

        Button("Get Started", primary, large) {
            navigate("/docs")
        }
    }
}
```

## Features

**Language**
- Declarative syntax — no XML, no JSX, no templates
- Signal-based reactivity — fine-grained DOM updates, no virtual DOM
- Client-side routing with SPA navigation
- Stores for shared state across pages
- Control flow: `if`/`else`, `for` loops, `show`/hide
- Data fetching with built-in loading/error/success states
- String interpolation: `"Hello, {name}!"`

**Components**
- 50+ built-in components across 8 categories
- Layout: Container, Row, Column, Grid, Stack, Spacer, Divider
- Navigation: Navbar, Sidebar, Breadcrumb, Link, Menu, Tabs
- Data Display: Card, Table, List, Badge, Avatar, Tooltip, Tag
- Data Input: Input, Select, Checkbox, Radio, Switch, Slider, DatePicker, Form
- Feedback: Alert, Toast, Modal, Dialog, Spinner, Progress, Skeleton
- Actions: Button, IconButton, ButtonGroup, Dropdown
- Media: Image, Video, Icon, Carousel
- Typography: Text, Heading, Code, Blockquote

**Design System**
- Design tokens for colors, spacing, typography, radii, shadows
- 4 built-in themes: default, dark, minimal, brutalist
- Variant modifiers: `primary`, `large`, `rounded`, `elevated`, `bold`, `center`, ...
- Custom style blocks on any component
- Responsive grid with breakpoint modifiers
- One-line theme switching via config

**Animation**
- 12 built-in animations: fadeIn, slideUp, scaleIn, bounce, shake, pulse, ...
- Mount animations as modifiers: `Card(elevated, fadeIn)`
- Enter/exit animations on control flow: `if visible, animate(fadeIn, fadeOut) { ... }`
- Staggered list animations: `for item in list, animate(slideUp, stagger: "50ms") { ... }`
- Transition blocks for CSS property transitions
- Animation design tokens for consistent timing

**Internationalization (i18n)**
- JSON translation files per locale
- `t("key")` function with interpolation: `t("greeting", name: user.name)`
- Reactive locale switching: `setLocale("ar")`
- Automatic RTL support for Arabic, Hebrew, Farsi, Urdu
- All translations baked into the build output

**Static Site Generation (SSG)**
- Pre-render pages to HTML at build time
- Instant content visibility — no blank white screen
- One HTML file per route
- JavaScript hydration for interactivity
- Works with i18n (default locale pre-rendered)
- Enable with one config flag: `"ssg": true`

**Accessibility**
- 12 compile-time lint rules
- Checks for missing alt text, labels, headings, table headers
- Warnings printed during build — never blocks compilation
- Catches issues before they reach users

**Developer Experience**
- Zero-config start: `init` → `build` → `serve`
- Dev server with SPA route fallback
- Scaffolding: `generate page|component|store`
- Clear error messages with file:line:column
- Two starter templates: interactive SPA and static site

## Quick Start

### Install

Build from source (requires [Rust](https://rustup.rs)):

```bash
git clone https://github.com/user/webfluent.git
cd webfluent
cargo build --release
```

The binary is at `target/release/webfluent`.

### Create a Project

```bash
# Interactive SPA (dashboard, task manager, etc.)
webfluent init my-app --template spa

# Static site with SSG + i18n (blog, marketing, etc.)
webfluent init my-site --template static
```

### Build & Serve

```bash
cd my-app
webfluent build
webfluent serve
```

Open `http://localhost:3000`.

## Language Overview

### Pages & Routing

```
Page Home (path: "/", title: "Home") {
    Container {
        Heading("Welcome", h1)
        Text("This is the home page.")
    }
}

Page About (path: "/about", title: "About") {
    Container {
        Heading("About Us", h1)
    }
}
```

```
App {
    Navbar {
        Navbar.Brand { Text("My App", heading) }
        Navbar.Links {
            Link(to: "/") { Text("Home") }
            Link(to: "/about") { Text("About") }
        }
    }

    Router {
        Route(path: "/", page: Home)
        Route(path: "/about", page: About)
    }
}
```

### State & Reactivity

```
Page Counter (path: "/counter") {
    state count = 0

    Container {
        Text("Count: {count}")
        Button("+1", primary) { count = count + 1 }
        Button("-1") { count = count - 1 }
    }
}
```

### Components

```
Component UserCard (name: String, role: String, active: Bool = true) {
    Card(elevated) {
        Row(align: center, gap: md) {
            Avatar(initials: "U", primary)
            Stack {
                Text(name, bold)
                Text(role, muted)
            }
            if active {
                Badge("Active", success)
            }
        }
    }
}

// Usage
UserCard(name: "Monzer", role: "Developer")
```

### Stores

```
Store TaskStore {
    state tasks = []
    state filter = "all"

    derived remaining = tasks.filter(t => !t.done).length

    action add(title: String) {
        tasks.push({ id: tasks.length + 1, title: title, done: false })
    }

    action toggle(id: Number) {
        state task = tasks.filter(t => t.id == id)[0]
        task.done = !task.done
    }
}
```

```
Page Tasks (path: "/tasks") {
    use TaskStore

    state newTask = ""

    Container {
        Input(text, bind: newTask, placeholder: "New task...")
        Button("Add", primary) {
            TaskStore.add(newTask)
            newTask = ""
        }

        for task in TaskStore.tasks, animate(slideUp, fadeOut, stagger: "50ms") {
            Checkbox(checked: task.done, label: task.title) {
                on:change { TaskStore.toggle(task.id) }
            }
        }
    }
}
```

### Forms

```
Page Settings (path: "/settings") {
    state username = ""
    state theme = "light"
    state notifications = true
    state fontSize = 16

    Form {
        Input(text, bind: username, label: "Username", required: true)
        Select(bind: theme, label: "Theme") {
            Option("light", "Light")
            Option("dark", "Dark")
        }
        Switch(bind: notifications, label: "Email Notifications")
        Slider(bind: fontSize, min: 12, max: 24, label: "Font Size")
        Button("Save", primary) { log("saved") }

        on:submit { log("form submitted") }
    }
}
```

### Data Fetching

```
fetch users from "/api/users" {
    loading {
        Spinner()
    }
    error (err) {
        Alert("Failed to load", danger)
    }
    success {
        for user in users {
            UserCard(name: user.name, role: user.role)
        }
    }
}
```

### Animations

```
// Mount animations
Card(elevated, fadeIn) { ... }
Heading("Title", h1, slideUp, slow)

// Control flow animations
if showPanel, animate(scaleIn, scaleOut) {
    Card { Text("Animated panel") }
}

for item in items, animate(slideUp, fadeOut, stagger: "50ms") {
    Text(item.name)
}

// Transition block
Button("Hover me") {
    transition {
        background 200ms ease
        transform 150ms spring
    }
}
```

### i18n

```json
// src/translations/en.json
{
    "greeting": "Hello, {name}!",
    "nav.home": "Home"
}
```

```json
// src/translations/ar.json
{
    "greeting": "!أهلاً، {name}",
    "nav.home": "الرئيسية"
}
```

```
Text(t("greeting", name: "Monzer"))
Button("العربية") { setLocale("ar") }
```

### Styling

```
// Variant modifiers
Button("Save", primary, large, rounded)
Text("Warning!", danger, bold, uppercase)
Card(elevated, outlined) { ... }

// Style blocks
Button("Custom") {
    style {
        background: "#8B5CF6"
        padding: xl
        radius: lg
    }
}
```

## Project Configuration

```json
{
    "name": "My App",
    "theme": {
        "name": "default",
        "tokens": {
            "color-primary": "#8B5CF6",
            "font-family": "Poppins, sans-serif"
        }
    },
    "build": {
        "output": "./build",
        "ssg": true
    },
    "dev": {
        "port": 3000
    },
    "i18n": {
        "defaultLocale": "en",
        "locales": ["en", "ar"],
        "dir": "src/translations"
    }
}
```

## CLI

```
webfluent init <name> [--template spa|static]   Create a new project
webfluent build [--dir DIR]                     Compile to HTML + CSS + JS
webfluent serve [--dir DIR]                     Start dev server
webfluent generate page|component|store <name>  Scaffold a new file
```

## Build Output

```
build/
├── index.html      # SPA shell or pre-rendered page (SSG)
├── app.js          # Reactive runtime + all compiled pages
├── styles.css      # Design tokens + component styles
└── public/         # Copied static assets
```

## Architecture

```
.wf source → Lexer → Parser → A11y Linter → Code Generator → HTML + CSS + JS
```

The compiler is written in Rust (~7,100 lines). The generated JavaScript uses a minimal signal-based reactivity runtime (~560 lines) with no framework dependencies. CSS is generated from design tokens as custom properties.

## Documentation

- [Language Design](docs/DESIGN.md) — Vision, principles, architecture
- [Language Specification](docs/SPEC.md) — Full syntax reference, all components, grammar
- [Animation System](docs/ANIMATION_SPEC.md) — Declarative animations and transitions
- [Internationalization](docs/I18N_SPEC.md) — Multi-language support and RTL
- [Accessibility Linting](docs/ACCESSIBILITY_SPEC.md) — Compile-time a11y checks
- [Static Site Generation](docs/SSG_SPEC.md) — Pre-rendering and hydration

## License

[GPL-3.0](LICENSE)
