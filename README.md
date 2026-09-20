[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/monzeromer-lab/WebFluent)

# WebFluent

A web-first programming language that compiles to HTML, CSS, JavaScript, and PDF. Build single-page applications, static sites, and documents with built-in components, a design system, reactivity, routing, i18n, and animations — no frameworks, no dependencies, just clean output.

**[Documentation](https://monzeromer-lab.github.io/WebFluent)** · **[Getting Started](https://monzeromer-lab.github.io/WebFluent/getting-started)**

```wf
page Home(path: "/") {
    Container {
        Heading("Hello, WebFluent!").h1.fadeIn
        Text("Build for the web. Nothing else.")

        Button("Get Started").primary.lg {
            on click {
                navigate("/docs")
            }
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
- Control flow: `if`/`else`, `if let`, keyed `for … by`, `show`/hide, `match`
- Data fetching as a `resource`, matched on `loading`, `error(e)` and `ready(v)`
- Typed props, records and enums, checked at compile time; `Any` where you write none
- Components with declared events (`event`/`emit`) and named slots
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
- Themes written in the language: `theme Brand { color-primary: #0F766E }`
- Flags on any element: `.primary`, `.lg`, `.rounded`, `.elevated`, `.bold`, `.center`, ...
- Style blocks of raw CSS on any element, with `$token` and `{expr}` splices and nested `&:hover` rules
- Responsive grid with breakpoint modifiers
- Four example themes to copy and edit in `examples/themes/`
- SEO by default: canonical URLs, Open Graph and Twitter cards, JSON-LD, `sitemap.xml` and `robots.txt`
- Accessibility built in: native `<dialog>` modals, ARIA-wired tabs and menus, live regions, a skip link, and compile-time contrast checks

**Animation**
- 12 built-in animations: fadeIn, slideUp, scaleIn, bounce, shake, pulse, ...
- Mount animations as flags: `Card.elevated.fadeIn`
- Enter and exit animations on any element: `Card(exit: .fadeOut).fadeIn`
- Staggered list animations: `for item in list by item.id { Card(stagger: "50ms").slideUp { ... } }`
- Keyed lists that move items instead of rebuilding them; page transitions: `Router(transition: .fade)`
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
- Clear error messages with file:line:column, and a type checker that says what fits where
- A language server (hover, completion, go to definition, rename, extract component) for Zed and VS Code
- `wf migrate` rewrites a WebFluent 2 project into the current grammar
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

```wf
page Home(path: "/", title: "Home") {
    Container {
        Heading("Welcome").h1
        Text("This is the home page.")
    }
}

app {
    Navbar {
        Navbar.Brand { Text("My App").heading }
        Navbar.Links {
            Link(to: "/") { Text("Home") }
            Link(to: "/about") { Text("About") }
        }
    }

    Router
}
```

### State & Reactivity

```wf
page Counter(path: "/counter") {
    state count = 0

    Container {
        Text("Count: {count}")
        Button("+1").primary { on click { count = count + 1 } }
        Button("-1") { on click { count = count - 1 } }
    }
}
```

### Components

```wf
component UserCard(_ name: String, role: String, active: Bool = true) {
    Card.elevated {
        Row(align: .center, gap: .md) {
            Avatar(initials: "U").primary
            Stack {
                Text(name).bold
                Text(role).muted
            }
            if active {
                Badge("Active").success
            }
        }
    }
}

// Usage
UserCard("Monzer", role: "Developer")
```

### Stores

```wf
store TaskStore {
    state tasks = []
    derived remaining = tasks.filter(t => !t.done).length

    action add(title: String) {
        tasks.push({ id: tasks.length + 1, title: title, done: false })
    }
}
```

### Data Fetching

```wf
resource users = fetch("/api/users")
match users {
    loading { Spinner }
    error (err) { Alert("Failed to load").danger }
    ready(users) {
        for user in users {
            UserCard(name: user.name, role: user.role)
        }
    }
}
```

### Animations

```wf
// Mount animations
Card.elevated.fadeIn { ... }

// Enter and exit, on the element that comes and goes
if showPanel {
    Card(exit: .scaleOut).scaleIn { Text("Animated panel") }
}

// Staggered list animations; a keyed list moves items instead of rebuilding them
for item in items by item.id {
    Text(item.name, exit: .fadeOut, stagger: "50ms").slideUp
}
```

### i18n

```json
// src/translations/en.json
{ "greeting": "Hello, {name}!", "nav.home": "Home" }

// src/translations/ar.json
{ "greeting": "!أهلاً، {name}", "nav.home": "الرئيسية" }
```

```wf
Text(t("greeting", { name: "Monzer" }))
Button("العربية") { on click { setLocale("ar") } }
```

### PDF Documents

```wf
page Report(path: "/", title: "Q1 Report") {
    Document(page_size: "A4") {
        Header {
            Text("Company Inc.").muted.sm.right
        }

        Footer {
            Text("Confidential").muted.sm.center
        }

        Section {
            Heading("Quarterly Report").h1
            Text("Revenue grew 15% this quarter.")

            Table {
                Table.Head {
                    Table.Row { Table.Cell("Region") Table.Cell("Revenue") }
                }
                Table.Body {
                    Table.Row { Table.Cell("North America") Table.Cell("$2.4M") }
                    Table.Row { Table.Cell("Europe") Table.Cell("$1.8M") }
                }
            }

            PageBreak

            Heading("Key Highlights").h2
            List {
                Text("Launched 3 new products")
                Text("Expanded to 5 new markets")
            }
        }
    }
}
```

### Styling

```wf
// Flags: a boolean prop, or a case of one of the element's enum props
Button("Save").primary.lg.rounded
Text("Warning!").danger.bold.uppercase

// Style blocks: raw CSS, `$token` for a design token, `{expr}` to read state
Button("Custom") {
    style {
        background: #8B5CF6
        padding: $xl
        border-radius: $lg
        &:hover { background: $primary }
    }
}

// Your own stylesheets: any .css under src/ is bundled into styles.css
Card(class: "feature feature--first")
```

### Braces or indentation

The same grammar reads two layouts. A `.wf` file writes its blocks in
braces; a `.wfx` file writes them by indentation, and `wf fmt --to wfx`
switches a project from one to the other without changing what it builds:

```wfx
page Home(path: "/", title: "Home")
    state count = 0
    Container
        Heading("Welcome").h1
        Text("Count: {count}")
        Button("+1").primary
            on click
                count = count + 1
```

## Project Configuration

```json
{
    "name": "My App",
    "theme": {
        "tokens": {
            "color-primary": "#8B5CF6"
        }
    },
    "meta": {
        "site_url": "https://example.com",
        "site_name": "My App"
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
wf fmt [path] [--check]                 Format the sources; --check fails when one would change
wf test [path] [--update]               Run the test "…" { } declarations under tests/
wf docs [-d DIR] [-o OUT]               Write a component gallery for the built-ins and the project
wf fmt --to wfx|wf [path]               Switch between braces (.wf) and indentation (.wfx)
wf migrate [path] [--check] [--wfx]     Rewrite a WebFluent 2 project as WebFluent 3
```

## Build Output

```
# Web (SPA/SSG)                    # PDF
build/                             build/
├── index.html                     └── report.pdf
├── app.js          runtime, stores, components
├── pages/*.js      one chunk per page, loaded on demand
├── pages/*.css     the style rules only that page reaches
├── styles.css      the rules the project uses, and its own .css files
├── *.gz            every text output, precompressed
└── …               public/ copied to the root
```

## Architecture

```
.wf source → Lexer → Parser → A11y Linter → Code Generator → HTML + CSS + JS
                                           → PDF Validator  → PDF 1.7
```

The compiler is written in Rust. The generated JavaScript uses a minimal signal-based reactivity runtime with no framework dependencies. PDF output is raw PDF 1.7 bytes with Base14 font metrics — no external crates.

## Editor Support

| Editor | Where | What you get |
|--------|-------|--------------|
| [Zed](editors/zed) | `editors/zed` — install with `zed: install dev extension` | Tree-sitter highlighting, outline, brackets, indentation, snippets, and `wf-lsp` (found on `PATH` or downloaded from the latest release), for `.wf` and `.wfx` |
| [VS Code](editors/vscode) | `editors/vscode` | TextMate highlighting and `wf-lsp` |

Both editors talk to the same language server, `crates/wf-lsp`, for
diagnostics, completions, hover documentation, go to definition, rename
across the project, document symbols, quick fixes and `Extract component`. `cargo install --path crates/wf-lsp` puts it on your
`PATH`. The Tree-sitter grammars the Zed extension uses live in
`editors/tree-sitter-webfluent` (`.wf`) and `editors/tree-sitter-webfluentx`
(`.wfx`, generated from the same `grammar.js`).

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
