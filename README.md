# WebFluent

A web-first programming language that compiles to HTML, CSS, JavaScript, and PDF. Build single-page applications, static sites, and documents with built-in components, a design system, reactivity, routing, i18n, and animations — no frameworks, no dependencies, just clean output.

**[Documentation](https://monzeromer-lab.github.io/WebFluent)** · **[Getting Started](https://monzeromer-lab.github.io/WebFluent/getting-started)**

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

**Internationalization (i18n)**
- JSON translation files per locale
- `t("key")` function with interpolation
- Reactive locale switching: `setLocale("ar")`
- Automatic RTL support for Arabic, Hebrew, Farsi, Urdu
- All translations baked into the build output

**Static Site Generation (SSG)**
- Pre-render pages to HTML at build time
- Instant content visibility — no blank white screen
- One HTML file per route with JavaScript hydration
- Enable with one config flag: `"ssg": true`

**PDF Generation**
- Generate PDF documents from `.wf` source files
- Raw PDF 1.7 output — zero external dependencies
- Document components: Document, Section, Paragraph, Header, Footer, PageBreak
- Tables, code blocks, lists, headings, alerts, blockquotes, badges, progress bars
- 14 standard PDF base fonts with WinAnsiEncoding
- Automatic page breaks with repeated headers/footers
- Compile-time rejection of interactive elements (Button, Input, etc.)
- Configurable page size (A4, Letter, Legal, A3, A5) and margins

**Accessibility**
- 12 compile-time lint rules
- Checks for missing alt text, labels, headings, table headers
- Warnings printed during build — never blocks compilation

**Developer Experience**
- Zero-config start: `init` → `build` → `serve`
- Dev server with SPA route fallback
- Scaffolding: `generate page|component|store`
- Clear error messages with file:line:column
- Three starter templates: SPA, static site, and PDF document
- Cross-platform packaging: `.deb`, `.msi`, `.tar.gz`, `.zip`
- Task runner integration with `just`

## Quick Start

### Install

**Linux (Debian/Ubuntu):**
```bash
sudo dpkg -i webfluent_0.2.0-alpha-1_amd64.deb
```

**From source** (requires [Rust](https://rustup.rs)):
```bash
git clone https://github.com/monzeromer-lab/WebFluent.git
cd WebFluent
cargo build --release
# Binary is at target/release/wf
```

**Install script:**
```bash
# Linux
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex
```

### Create a Project

```bash
# Interactive SPA (dashboard, forms, routing)
wf init my-app --template spa

# Static site with SSG + i18n (blog, marketing)
wf init my-site --template static

# PDF document (reports, invoices, docs)
wf init my-report --template pdf
```

### Build & Serve

```bash
cd my-app
wf build
wf serve     # opens http://localhost:3000
```

## Language Overview

### Pages & Routing

```
Page Home (path: "/", title: "Home") {
    Container {
        Heading("Welcome", h1)
        Text("This is the home page.")
    }
}

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
    derived remaining = tasks.filter(t => !t.done).length

    action add(title: String) {
        tasks.push({ id: tasks.length + 1, title: title, done: false })
    }
}
```

### Data Fetching

```
fetch users from "/api/users" {
    loading { Spinner() }
    error (err) { Alert("Failed to load", danger) }
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

// Control flow animations
if showPanel, animate(scaleIn, scaleOut) {
    Card { Text("Animated panel") }
}

// Staggered list animations
for item in items, animate(slideUp, fadeOut, stagger: "50ms") {
    Text(item.name)
}
```

### i18n

```json
// src/translations/en.json
{ "greeting": "Hello, {name}!", "nav.home": "Home" }

// src/translations/ar.json
{ "greeting": "!أهلاً، {name}", "nav.home": "الرئيسية" }
```

```
Text(t("greeting", name: "Monzer"))
Button("العربية") { setLocale("ar") }
```

### PDF Documents

```
Page Report (path: "/", title: "Q1 Report") {
    Document(page_size: "A4") {
        Header {
            Text("Company Inc.", muted, small, right)
        }

        Footer {
            Text("Confidential", muted, small, center)
        }

        Section {
            Heading("Quarterly Report", h1)
            Text("Revenue grew 15% this quarter.")

            Table {
                Thead {
                    Trow { Tcell("Region") Tcell("Revenue") }
                }
                Tbody {
                    Trow { Tcell("North America") Tcell("$2.4M") }
                    Trow { Tcell("Europe") Tcell("$1.8M") }
                }
            }

            PageBreak()

            Heading("Key Highlights", h2)
            List {
                Text("Launched 3 new products")
                Text("Expanded to 5 new markets")
            }
        }
    }
}
```

### Styling

```
// Variant modifiers
Button("Save", primary, large, rounded)
Text("Warning!", danger, bold, uppercase)

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
        "ssg": true,
        "output_type": "spa",
        "pdf": {
            "page_size": "A4",
            "default_font": "Helvetica",
            "output_filename": "report.pdf"
        }
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
wf init <name> [-t spa|static|pdf]      Create a new project
wf build [--dir DIR]                    Compile to HTML/CSS/JS or PDF
wf serve [--dir DIR]                    Start dev server
wf generate page|component|store <name> Scaffold a new file
```

## Build Output

```
# Web (SPA/SSG)                    # PDF
build/                             build/
├── index.html                     └── report.pdf
├── app.js
├── styles.css
└── public/
```

## Architecture

```
.wf source → Lexer → Parser → A11y Linter → Code Generator → HTML + CSS + JS
                                           → PDF Validator  → PDF 1.7
```

The compiler is written in Rust. The generated JavaScript uses a minimal signal-based reactivity runtime with no framework dependencies. PDF output is raw PDF 1.7 bytes with Base14 font metrics — no external crates.

## Documentation

- **[Live Docs](https://monzeromer-lab.github.io/WebFluent)** — Interactive documentation built with WebFluent itself
- [Language Design](spec/DESIGN.md) — Vision, principles, architecture
- [Language Specification](spec/SPEC.md) — Full syntax reference, all components, grammar
- [Animation System](spec/ANIMATION_SPEC.md) — Declarative animations and transitions
- [Internationalization](spec/I18N_SPEC.md) — Multi-language support and RTL
- [Accessibility Linting](spec/ACCESSIBILITY_SPEC.md) — Compile-time a11y checks
- [Static Site Generation](spec/SSG_SPEC.md) — Pre-rendering and hydration

## License

[GPL-3.0](LICENSE)
