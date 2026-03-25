# WebFluent — Agent Role File

You are an expert WebFluent developer. WebFluent is a web-first programming language that compiles to HTML, CSS, JavaScript, and PDF. You write `.wf` source files and configure projects via `webfluent.app.json`.

## Project Structure

```
project/
├── webfluent.app.json        # Config: theme, build, i18n, meta
├── src/
│   ├── App.wf                # Root: navbar, router, footer
│   ├── pages/                # One .wf per page
│   ├── components/           # Reusable components
│   ├── stores/               # Shared state stores
│   └── translations/         # i18n JSON files (en.json, ar.json)
├── public/                   # Static assets (images, fonts)
└── build/                    # Compiled output
```

## CLI

```bash
wf init <name> -t spa|static|pdf   # Create project
wf build [-d DIR]                  # Compile
wf serve [-d DIR]                  # Dev server (localhost:3000)
wf generate page|component|store <name>
```

## Core Syntax

### Pages

```wf
Page Home (path: "/", title: "Home") {
    state count = 0

    Container {
        Heading("Welcome", h1)
        Text("Count: {count}")
        Button("+1", primary) { count = count + 1 }
    }
}
```

- `path` — URL route. Supports dynamic segments: `/user/:id` (access via `params.id`)
- `title` — Browser tab title

### Components

```wf
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

- Prop types: `String`, `Number`, `Bool`, `List`, `Map`
- Optional props: `avatar?: String`
- Default values: `active: Bool = true`

### App (Router + Layout)

```wf
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
        Route(path: "/user/:id", page: UserProfile)
        Route(path: "*", page: NotFound)
    }

    Footer
}
```

### Stores (Shared State)

```wf
Store CartStore {
    state items = []
    state total = 0

    derived count = items.length

    action add(product: Map) {
        items = items.concat([product])
        total = total + product.price
    }

    action clear() {
        items = []
        total = 0
    }
}

// Usage in any page/component:
Page Shop (path: "/shop") {
    use CartStore
    Text("Cart: {CartStore.count} items")
    Button("Clear") { CartStore.clear() }
}
```

### State & Reactivity

```wf
state count = 0                      // Signal — reactive variable
derived double = count * 2           // Computed — auto-updates
effect { log(count) }                // Side effect — runs on change
```

- UI elements that reference state variables auto-update when values change
- String interpolation: `"Hello, {name}!"` — reactive in UI elements

### Events

```wf
Button("Click") {
    on:click { doSomething() }
}
Input(text, bind: query) {
    on:input { search(query) }
}
Form {
    on:submit { saveData() }
}
```

Events: `on:click`, `on:input`, `on:change`, `on:submit`, `on:focus`, `on:blur`, `on:keydown`, `on:keyup`, `on:mouseenter`, `on:mouseleave`

The event object is available as `event` (e.g., `event.currentTarget`, `event.target`).

### Control Flow

```wf
// Conditional
if isLoggedIn {
    Text("Welcome!")
} else {
    Button("Log In") { navigate("/login") }
}

// Loop
for item in items {
    Card { Text(item.name) }
}

// Loop with index
for item, index in items {
    Text("{index}. {item.name}")
}

// Show/hide (keeps in DOM, toggles visibility)
show isVisible {
    Modal { Text("Content") }
}
```

### Data Fetching

```wf
fetch users from "/api/users" {
    loading { Spinner() }
    error (err) { Alert("Failed: {err.message}", danger) }
    success {
        for user in users {
            Text(user.name)
        }
    }
}
```

Options: `(method: "POST", body: { key: value }, headers: { "Authorization": token })`

### Navigation

```wf
navigate("/path")                    // Programmatic
Link(to: "/about") { Text("About") } // Declarative
```

## Built-in Components

### Layout

| Component | Usage |
|-----------|-------|
| `Container` | `Container { ... }` — centered max-width wrapper |
| `Row` | `Row(gap: md, align: center, justify: between) { ... }` — horizontal flex |
| `Column` | `Column(span: 6) { ... }` — 12-column grid child |
| `Grid` | `Grid(columns: 3, gap: md) { ... }` — CSS grid |
| `Stack` | `Stack(gap: md) { ... }` — vertical flex |
| `Spacer` | `Spacer()` or `Spacer(sm)` `Spacer(xl)` — vertical space |
| `Divider` | `Divider()` — horizontal line |

### Navigation

| Component | Usage |
|-----------|-------|
| `Navbar` | `Navbar { Navbar.Brand { ... } Navbar.Links { ... } Navbar.Actions { ... } }` |
| `Sidebar` | `Sidebar { Sidebar.Item(to: "/") { Text("Home") } }` |
| `Link` | `Link(to: "/path") { Text("Label") }` |
| `Tabs` | `Tabs { TabPage("Tab 1") { ... } TabPage("Tab 2") { ... } }` |
| `Breadcrumb` | `Breadcrumb { Breadcrumb.Item(to: "/") { Text("Home") } }` |
| `Menu` | `Menu(trigger: "Options") { Menu.Item { ... } }` |

### Data Display

| Component | Usage |
|-----------|-------|
| `Card` | `Card(elevated) { Card.Header { ... } Card.Body { ... } Card.Footer { ... } }` |
| `Table` | `Table { Thead { Trow { Tcell("Col") } } Tbody { Trow { Tcell("Val") } } }` |
| `List` | `List { Text("Item 1") Text("Item 2") }` — `List(ordered)` for numbered |
| `Badge` | `Badge("Label", primary)` — variants: primary, success, danger, warning, info |
| `Tag` | `Tag("JavaScript")` |
| `Avatar` | `Avatar(initials: "MO", primary)` or `Avatar(src: "/img.png")` |
| `Tooltip` | `Tooltip(text: "Help") { Button("?") }` |

### Form & Input

| Component | Usage |
|-----------|-------|
| `Input` | `Input(text, bind: var, placeholder: "...", label: "Name")` |
| `Select` | `Select(bind: var, label: "Choose") { Option("val1", "Label 1") }` |
| `Checkbox` | `Checkbox(bind: var, label: "Agree")` |
| `Radio` | `Radio(bind: var, value: "opt1", label: "Option 1")` |
| `Switch` | `Switch(bind: var, label: "Enable")` |
| `Slider` | `Slider(bind: var, min: 0, max: 100, label: "Volume")` |
| `Form` | `Form { ... on:submit { save() } }` |

Input types (first positional arg): `text`, `email`, `password`, `number`, `search`, `tel`, `url`, `date`, `time`, `color`

### Feedback

| Component | Usage |
|-----------|-------|
| `Alert` | `Alert("Message", success)` — variants: success, danger, warning, info |
| `Toast` | `Toast("Saved!", success)` — temporary notification |
| `Modal` | `Modal(visible: showModal, title: "Title") { ... Modal.Footer { ... } }` |
| `Dialog` | `Dialog(visible: show, title: "Confirm") { ... }` |
| `Spinner` | `Spinner()` or `Spinner(large, primary)` |
| `Progress` | `Progress(value: 75, max: 100)` |
| `Skeleton` | `Skeleton(height: "20px", width: "200px")` |

### Actions

| Component | Usage |
|-----------|-------|
| `Button` | `Button("Label", primary, large)` — click handler: `Button("Save") { doSave() }` |
| `IconButton` | `IconButton(icon: "close", label: "Close")` |
| `ButtonGroup` | `ButtonGroup { Button("A") Button("B") }` |
| `Dropdown` | `Dropdown(label: "Actions") { Dropdown.Item { ... } }` |

Button variants: `primary`, `secondary`, `success`, `danger`, `warning`, `info`
Button modifiers: `small`, `large`, `full`, `rounded`, `pill`, `outlined`

### Media

| Component | Usage |
|-----------|-------|
| `Image` | `Image(src: "/photo.jpg", alt: "Description")` |
| `Video` | `Video(src: "/video.mp4", controls: true)` |
| `Icon` | `Icon("home")` |
| `Carousel` | `Carousel { Carousel.Slide { Image(src: "...") } }` |

### Typography

| Component | Usage |
|-----------|-------|
| `Text` | `Text("Hello", bold, muted, center)` |
| `Heading` | `Heading("Title", h1)` — levels: h1, h2, h3, h4, h5, h6 |
| `Code` | `Code("const x = 1", block)` — `block` for multi-line |
| `Blockquote` | `Blockquote { Text("Quote text") }` |

Text modifiers: `bold`, `italic`, `underline`, `uppercase`, `lowercase`, `center`, `right`, `muted`, `small`, `large`, `primary`, `danger`, `success`, `warning`, `info`

## Modifiers Reference

Modifiers are positional keyword arguments that apply CSS classes:

**Size**: `small`, `medium`, `large`
**Color**: `primary`, `secondary`, `success`, `danger`, `warning`, `info`
**Shape**: `rounded`, `pill`, `square`
**Elevation**: `flat`, `elevated`, `outlined`
**Width**: `full`, `fit`
**Text**: `bold`, `italic`, `underline`, `uppercase`, `lowercase`, `center`, `right`, `muted`, `heading`
**Animation**: `fadeIn`, `fadeOut`, `slideUp`, `slideDown`, `slideLeft`, `slideRight`, `scaleIn`, `scaleOut`, `bounce`, `shake`, `pulse`, `spin`
**Speed**: `fast` (150ms), `slow` (500ms)

## Animation

```wf
// Mount animation (plays once on appear)
Card(elevated, fadeIn) { ... }
Heading("Title", h1, slideUp, slow)

// Conditional enter/exit
if showPanel, animate(scaleIn, fadeOut) {
    Card { Text("Animated panel") }
}

// List with stagger
for item in items, animate(slideUp, fadeOut, stagger: "50ms") {
    Text(item.name)
}

// Replay on hover (via event)
Card(outlined, fadeIn) {
    on:mouseenter { replayAnimation(event.currentTarget, "fadeIn") }
}

// Transition block
Button("Hover") {
    transition {
        background 200ms ease
        transform 150ms spring
    }
}
```

## Styling

### Style Blocks

```wf
Card {
    style {
        background: "#f0f0f0"
        border-radius: "1rem"
        padding: "2rem"
        box-shadow: "0 2px 8px rgba(0,0,0,0.1)"
    }
}
```

Style properties use CSS names (hyphenated). Values are strings or numbers.

### Design Tokens (in webfluent.app.json)

```json
{
    "theme": {
        "name": "default",
        "tokens": {
            "color-primary": "#3B82F6",
            "color-secondary": "#8B5CF6",
            "font-family": "Inter, sans-serif",
            "radius-md": "0.5rem"
        }
    }
}
```

Built-in themes: `default`, `dark`, `minimal`, `brutalist`

## i18n (Internationalization)

### Translation files

```json
// src/translations/en.json
{ "nav.home": "Home", "greeting": "Hello, {name}!" }

// src/translations/ar.json
{ "nav.home": "الرئيسية", "greeting": "أهلاً، {name}!" }
```

### Config

```json
{
    "i18n": {
        "defaultLocale": "en",
        "locales": ["en", "ar"],
        "dir": "src/translations"
    }
}
```

### Usage

```wf
Text(t("nav.home"))                    // Translated text
Text(t("greeting", name: user.name))   // With interpolation
Button("EN") { setLocale("en") }       // Switch locale
Button("AR") { setLocale("ar") }       // Auto-RTL for Arabic
```

RTL locales (automatic `dir="rtl"`): `ar`, `he`, `fa`, `ur`

## SSG (Static Site Generation)

```json
{ "build": { "ssg": true, "base_path": "/my-site" } }
```

- Pre-renders each page to its own `index.html`
- JavaScript hydrates for interactivity
- Static content visible immediately (no blank screen)
- Dynamic content (state, if/for, fetch) handled by JS after hydration

## PDF Output

```json
{
    "build": {
        "output_type": "pdf",
        "pdf": {
            "page_size": "A4",
            "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
            "default_font": "Helvetica",
            "default_font_size": 12,
            "output_filename": "report.pdf"
        }
    }
}
```

### PDF-specific components

| Component | Purpose |
|-----------|---------|
| `Document(page_size: "A4")` | Root document wrapper |
| `Section` | Groups content with spacing |
| `Paragraph` | Block of text with paragraph spacing |
| `Header` | Content repeated at top of every page |
| `Footer` | Content repeated at bottom of every page |
| `PageBreak()` | Forces a new page |

Supported in PDF: `Text`, `Heading`, `Table`, `List`, `Code`, `Blockquote`, `Divider`, `Alert`, `Badge`, `Progress`, `Card`, `Image`, `Spacer`, `Container`, `Row`, `Stack`, `Grid`

**Rejected in PDF** (compile error): `Button`, `Input`, `Select`, `Checkbox`, `Switch`, `Form`, `Modal`, `Dialog`, `Toast`, `Router`, `Navbar`, `Sidebar`, `Tabs`, `Video`, `Carousel`, and all event handlers.

Page sizes: `A4`, `A3`, `A5`, `Letter`, `Legal`
Fonts: Helvetica, Helvetica-Bold, Times-Roman, Times-Bold, Courier, Courier-Bold (all Base14)

### PDF example

```wf
Page Report (path: "/", title: "Report") {
    Document(page_size: "A4") {
        Header {
            Text("Company Inc.", muted, small, right)
        }
        Footer {
            Text("Confidential", muted, small, center)
        }
        Section {
            Heading("Q1 Report", h1)
            Text("Revenue grew 15% this quarter.")

            Table {
                Thead { Trow { Tcell("Region") Tcell("Revenue") } }
                Tbody { Trow { Tcell("North America") Tcell("$2.4M") } }
            }

            PageBreak()
            Heading("Highlights", h2)
            List {
                Text("Launched 3 new products")
                Text("Expanded to 5 markets")
            }
        }
    }
}
```

## Escaping Braces

In strings, `{` starts interpolation. To use literal braces (e.g., in code blocks), escape with `\{` and `\}`:

```wf
Code("function() \{ return 42; \}", block)
```

## Configuration Reference (webfluent.app.json)

```json
{
    "name": "My App",
    "version": "1.0.0",
    "author": "Name",
    "theme": {
        "name": "default",
        "mode": "light",
        "tokens": {}
    },
    "build": {
        "output": "./build",
        "minify": true,
        "ssg": false,
        "base_path": "",
        "output_type": "spa",
        "pdf": {
            "page_size": "A4",
            "margins": { "top": 72, "bottom": 72, "left": 72, "right": 72 },
            "default_font": "Helvetica",
            "default_font_size": 12,
            "output_filename": null
        }
    },
    "dev": { "port": 3000 },
    "meta": {
        "title": "",
        "description": "",
        "favicon": "",
        "lang": "en"
    },
    "i18n": {
        "defaultLocale": "en",
        "locales": ["en"],
        "dir": "src/translations"
    }
}
```

## Template Engine (Server-Side Rendering)

WebFluent can be used as a **template engine** from Rust or Node.js to render `.wf` templates to HTML or PDF with JSON data.

### CLI

```bash
# Render to HTML
wf render template.wf --data data.json --format html -o output.html

# Render to HTML fragment (no <html> wrapper)
wf render template.wf --data data.json --format fragment

# Render to PDF
wf render template.wf --data data.json --format pdf -o report.pdf

# Pipe JSON from stdin
echo '{"name":"Monzer"}' | wf render template.wf --format html

# With theme
wf render template.wf --data data.json --format html --theme dark
```

### Rust API

```rust
use webfluent::Template;
use serde_json::json;

let tpl = Template::from_str("Container { Heading(\"Hello, {name}!\", h1) }")?;
// or: Template::from_file("templates/invoice.wf")?;

let html = tpl.render_html(&json!({"name": "World"}))?;           // Full HTML doc
let frag = tpl.render_html_fragment(&json!({"name": "World"}))?;  // Fragment only
let pdf  = tpl.render_pdf(&json!({"name": "World"}))?;            // Vec<u8>

// With theme
let html = tpl.with_theme("dark")
    .with_tokens(&[("color-primary", "#8B5CF6")])
    .render_html(&data)?;
```

### Node.js API

```javascript
const { Template } = require('@aspect/webfluent');

const tpl = Template.fromString('Container { Heading("Hello, {name}!", h1) }');
// or: Template.fromFile('templates/invoice.wf');

const html = tpl.renderHtml({ name: "World" });           // Full HTML string
const frag = tpl.renderHtmlFragment({ name: "World" });   // Fragment string
const pdf  = tpl.renderPdf({ name: "World" });             // Buffer

// With theme
const html = tpl.withTheme('dark')
    .withTokens({ 'color-primary': '#8B5CF6' })
    .renderHtml(data);
```

### Template Data Context

Data is a JSON object. Top-level keys become template variables:

```wf
// template.wf
Page Invoice (path: "/", title: "Invoice") {
    Container {
        Heading("Invoice #{number}", h1)
        Text("Customer: {customer.name}")

        for item in items {
            Card { Text(item.name, bold) Text("${item.price}") }
        }

        if paid { Badge("PAID", success) }
        else    { Badge("UNPAID", danger) }
    }
}
```

```json
{
    "number": "INV-001",
    "customer": { "name": "Acme Corp" },
    "items": [{ "name": "Widget", "price": 9.99 }],
    "paid": true
}
```

**Supported in templates**: all layout, typography, data display components, `for` loops, `if/else`, string interpolation, style blocks, modifiers, themes.

**Not supported**: `state`, `derived`, `effect`, events (`on:click`), navigation, stores, animations, `fetch`.

## Key Rules

1. **Every page needs a path**: `Page Name (path: "/route") { ... }`
2. **State is reactive**: any UI referencing a state variable auto-updates
3. **Modifiers are positional**: `Button("Label", primary, large)` — order doesn't matter
4. **Named args use colon**: `Input(text, bind: myVar, placeholder: "...")`
5. **Braces for children/body**: `Card { Card.Body { Text("content") } }`
6. **Sub-components use dot**: `Card.Header`, `Card.Body`, `Card.Footer`, `Navbar.Brand`, `Navbar.Links`
7. **Event handlers**: `on:click { ... }` inside a component's block
8. **String interpolation is reactive**: `Text("Count: {count}")` updates when `count` changes
9. **Imports via `use`**: `use StoreName` to access shared stores
10. **No semicolons needed**: statements are newline-separated
