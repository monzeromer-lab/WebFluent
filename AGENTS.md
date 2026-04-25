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
├── public/                   # Static assets → copied to build root
└── build/                    # Compiled output
```

Files in `public/` are copied to the **root** of the build output (not nested under `public/`). Example: `public/logo.png` → `build/logo.png`.

## CLI

```bash
wf init <name> -t spa|static|pdf|slides   # Create project
wf build [-d DIR]                         # Compile
wf serve [-d DIR]                         # Dev server (localhost:3000)
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

The Router can be placed at any nesting depth inside the App. The codegen recursively finds it.

```wf
// Simple: Router at top level
App {
    Navbar { ... }
    Router {
        Route(path: "/", page: Home)
        Route(path: "/about", page: About)
        Route(path: "*", page: NotFound)
    }
    Footer
}
```

```wf
// Sidebar layout: Router nested inside Row > Container
App {
    Row {
        style { min-height: "100vh" }

        Sidebar {
            style {
                width: "220px"
                position: "fixed"
                top: "0"
                left: "0"
                bottom: "0"
                background: "#1A1A19"
            }
            Sidebar.Header { Text("My App", bold) }
            Sidebar.Item(to: "/", icon: "home") { Text("Home") }
            Sidebar.Item(to: "/about", icon: "info") { Text("About") }
        }

        Container {
            style {
                margin-left: "220px"
                flex: "1"
            }

            Router {
                Route(path: "/", page: Home)
                Route(path: "/about", page: About)
            }
        }
    }
}
```

```wf
// Navbar + Sidebar + Router
App {
    NavBar

    Row {
        DocSidebar

        Router {
            Route(path: "/", page: Home)
            Route(path: "/docs", page: Docs)
        }
    }

    SiteFooter
}
```

The Router can be wrapped in `Row`, `Container`, `Stack`, or any layout component. Sibling elements (Sidebar, Navbar) inside the same wrapper are emitted alongside the router element.

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

    action getHeaders() {
        h = {}
        h["Authorization"] = "Bearer " + accessToken
        return h
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

### Return Statements

Actions can return values using `return`:

```wf
Store AuthStore {
    state accessToken = ""

    action getHeaders() {
        h = {}
        h["Authorization"] = "Bearer " + accessToken
        return h
    }
}
```

### Browser Globals

Standard browser APIs are available without any prefix — they compile directly to their JavaScript equivalents:

```wf
// Storage
localStorage.setItem("token", tok)
sessionStorage.getItem("key")

// Window & Document
window.open("https://example.com")
document.title = "New Title"

// Console
console.log("debug info")

// JSON
data = JSON.parse(responseText)
text = JSON.stringify(obj)

// Timers
setTimeout(callback, 1000)

// Other globals: Math, Date, Array, Object, Promise, Error,
// parseInt, parseFloat, encodeURIComponent, atob, btoa, fetch,
// alert, confirm, prompt, RegExp, Map, Set, etc.
```

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

Map literals support **quoted string keys** for headers and special field names:

```wf
fetch data from "/api/users" (
    method: "POST",
    headers: { "Content-Type": "application/json", "X-Api-Key": apiKey },
    body: { action: "create", token: sessionToken }
) {
    success { Text("Done") }
}
```

Reserved words (`action`, `token`, `error`, `state`, etc.) work as map keys.

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
| `Sidebar` | `Sidebar { Sidebar.Header { ... } Sidebar.Item(to: "/", icon: "home") { ... } Sidebar.Divider() }` |
| `Link` | `Link(to: "/path") { Text("Label") }` |
| `Tabs` | `Tabs { TabPage("Tab 1") { ... } TabPage("Tab 2") { ... } }` |
| `Breadcrumb` | `Breadcrumb { Breadcrumb.Item(to: "/") { Text("Home") } Breadcrumb.Item { Text("Current") } }` |
| `Menu` | `Menu(trigger: "Options") { Menu.Item { ... } }` |

### Data Display

| Component | Usage |
|-----------|-------|
| `Card` | `Card(elevated) { Card.Header { ... } Card.Body { ... } Card.Footer { ... } }` |
| `Table` | `Table { Thead { Trow { Tcell("Col") } } Tbody { Trow { Tcell("Val") } } }` |
| `List` | `List { Text("Item 1") Text("Item 2") }` — `List(ordered)` for numbered |
| `Badge` | `Badge("Label", primary)` — variants: primary, success, danger, warning, info |
| `Tag` | `Tag("JavaScript")` |
| `Avatar` | `Avatar(src: "/photo.jpg", alt: "User")` or `Avatar(initials: "MO", primary)` — modifiers: small, large, primary |
| `Tooltip` | `Tooltip(text: "Click to save") { Button("Save", primary) }` — wraps children, shows text on hover |

### Form & Input

| Component | Usage |
|-----------|-------|
| `Input` | `Input(text, bind: var, placeholder: "...", label: "Name")` |
| `Select` | `Select(bind: var, label: "Choose") { Option("val1", "Label 1") }` |
| `Checkbox` | `Checkbox(bind: var, label: "Agree")` |
| `Radio` | `Radio(bind: var, value: "opt1", label: "Option 1")` |
| `Switch` | `Switch(bind: var, label: "Enable")` |
| `Slider` | `Slider(bind: volume, min: 0, max: 100, step: 1, label: "Volume")` — range input with reactive value |
| `DatePicker` | `DatePicker(bind: selectedDate, label: "Start Date", min: "2026-01-01")` — date input |
| `FileUpload` | `FileUpload(accept: "image/*", label: "Upload Photo") { on:change { handleFile(event) } }` — modifiers: multiple |
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
| `Skeleton` | `Skeleton(height: "20px", width: "200px")` or `Skeleton(circle, size: "48px")` — modifiers: circle |

### Actions

| Component | Usage |
|-----------|-------|
| `Button` | `Button("Label", primary, large)` — click handler: `Button("Save") { doSave() }` |
| `IconButton` | `IconButton(icon: "close", label: "Close")` or `IconButton(icon: "edit", label: "Edit", primary) { editItem() }` — modifiers: small, large, primary, danger |
| `ButtonGroup` | `ButtonGroup { Button("A") Button("B") }` |
| `Dropdown` | `Dropdown(label: "Actions") { Dropdown.Item { ... } }` |

Button variants: `primary`, `secondary`, `success`, `danger`, `warning`, `info`
Button modifiers: `small`, `large`, `full`, `rounded`, `pill`, `outlined`

### Media

| Component | Usage |
|-----------|-------|
| `Image` | `Image(src: "/photo.jpg", alt: "Description")` |
| `Video` | `Video(src: "/video.mp4", controls: true)` |
| `Icon` | `Icon("home")` or `Icon("search", large, primary)` — 30 built-in SVG icons rendered inline |
| `Carousel` | `Carousel(autoplay: true, interval: 5000) { Carousel.Slide { Image(src: "...") } }` — slide track with dots and autoplay |

### Typography

| Component | Usage |
|-----------|-------|
| `Text` | `Text("Hello", bold, muted, center)` |
| `Heading` | `Heading("Title", h1)` — levels: h1, h2, h3, h4, h5, h6 |
| `Code` | `Code("const x = 1", block)` — `block` for multi-line |
| `Blockquote` | `Blockquote { Text("Quote text") }` |

Text modifiers: `bold`, `italic`, `underline`, `uppercase`, `lowercase`, `center`, `right`, `muted`, `small`, `large`, `primary`, `danger`, `success`, `warning`, `info`

### Component Details

#### Sidebar

Structural codegen with sub-components for app navigation:

```wf
Sidebar {
    Sidebar.Header { Text("My App", heading) }
    Sidebar.Item(to: "/", icon: "home") { Text("Home") }
    Sidebar.Item(to: "/settings", icon: "settings") { Text("Settings") }
    Sidebar.Divider()
    Sidebar.Item(to: "/logout", icon: "logout") { Text("Logout") }
}
```

Sub-components: `Sidebar.Header`, `Sidebar.Item`, `Sidebar.Divider`

#### Breadcrumb

Navigation breadcrumb with proper separator rendering:

```wf
Breadcrumb {
    Breadcrumb.Item(to: "/") { Text("Home") }
    Breadcrumb.Item(to: "/docs") { Text("Docs") }
    Breadcrumb.Item { Text("Components") }
}
```

The last item (without `to:`) renders as the current page (no link).

#### Tooltip

Wraps children, shows text on hover:

```wf
Tooltip(text: "Click to save") {
    Button("Save", primary)
}
```

#### Avatar

Displays an image or initials:

```wf
Avatar(src: "/photo.jpg", alt: "User")
Avatar(initials: "MO", primary)
Avatar(initials: "J", large)
```

Modifiers: `small`, `large`, `primary`

#### Skeleton

Loading placeholder:

```wf
Skeleton(height: "20px", width: "200px")
Skeleton(circle, size: "48px")
```

Modifiers: `circle`

#### Carousel

Slide track with dots and autoplay:

```wf
Carousel(autoplay: true, interval: 5000) {
    Carousel.Slide { Image(src: "/1.jpg", alt: "Slide 1") }
    Carousel.Slide { Image(src: "/2.jpg", alt: "Slide 2") }
}
```

#### IconButton

Icon-only button with aria-label:

```wf
IconButton(icon: "close", label: "Close")
IconButton(icon: "edit", label: "Edit", primary) { editItem() }
```

Modifiers: `small`, `large`, `primary`, `danger`

#### Slider

Range input with reactive value:

```wf
Slider(bind: volume, min: 0, max: 100, step: 1, label: "Volume")
```

#### DatePicker

Date input:

```wf
DatePicker(bind: selectedDate, label: "Start Date", min: "2026-01-01")
```

#### FileUpload

Styled file input:

```wf
FileUpload(accept: "image/*", label: "Upload Photo") {
    on:change { handleFile(event) }
}
FileUpload(accept: ".pdf,.doc", multiple, label: "Documents")
```

### Icon System

WebFluent includes 30 built-in SVG icons, rendered inline. Available icons:

`home`, `menu`, `search`, `close`, `user`, `settings`, `check`, `plus`, `minus`, `edit`, `trash`, `star`, `heart`, `mail`, `bell`, `download`, `upload`, `eye`, `link`, `calendar`, `filter`, `info`, `warning`, `arrow-left`, `arrow-right`, `chevron-down`, `chevron-right`, `chevron-left`, `logout`, `copy`

Usage:

```wf
Icon("home")
Icon("search", large, primary)
IconButton(icon: "close", label: "Close")
Sidebar.Item(to: "/", icon: "home") { Text("Home") }
```

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

// Transition block (declarative, per-property)
Button("Hover") {
    transition {
        background 200ms ease
        transform 150ms spring
    }
}

// CSS transition property (in style block)
Button("Hover") {
    style {
        transition: "all 200ms ease"
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
        transition: "all 200ms ease"
    }
}
```

Style properties use CSS names (hyphenated). Values are strings or numbers. All CSS property names work, including `transition`, `animation`, `filter`, etc.

### Style support in PDF and Slides

The PDF and Slides backends honor the same set of style properties on `Slide` (slides only) and on layout containers (`Container`, `Column`, `Stack`, `Grid`, `Card`, `Section`). Anything else emits a `warning[pdf]:` or `warning[slides]: unsupported style property '<name>' on <Component>` (deduped per build).

| Property | Values | Notes |
|----------|--------|-------|
| `background` / `background-color` | `"#hex"`, `"linear-gradient(...)"` | Paint color or PDF axial-shading gradient |
| `padding`, `padding-{top,right,bottom,left}` | `Npt`, `Npx`, `N` | Insets the container's child rendering |
| `border` | `"Npt #hex"` (CSS shorthand, simplified) | Width + color |
| `border-color` | `"#hex"` | |
| `border-width` | `Npt` | |
| `border-radius` | `Npt` | Rounds the bg + border |
| `box-shadow` | `"X Y #hex"` | Offset rect; blur is ignored |
| `width`, `height` | `Npt`, `N%` | Fixed-size or percent of parent |
| `color`, `font-family`, `font-size`, `text-align` | as CSS | On `Text`/`Heading` |

Linear gradient syntax: `linear-gradient(to bottom, #color1, #color2)`, `linear-gradient(45deg, #c1, #c2)`, or named directions `to top|right|bottom|left|top-right|...`. Two color stops only.

The `Slide` element accepts `style { background }` (full-bleed page background). Setting `background` on a `Container` paints the rect under that container's measured box (or its explicit `width`/`height`).

Painting order inside a styled container: shadow → background → border → children. Children paint on top of decoration via content-stream splicing.

### Responsive Styles with @media

Style blocks support `@media` queries for responsive design:

```wf
Sidebar {
    style {
        width: "260px"
        position: "fixed"

        @media (max-width: 768px) {
            display: "none"
        }
    }
}

Heading("Title", h1) {
    style {
        font-size: "3rem"

        @media (max-width: 768px) {
            font-size: "1.5rem"
        }
    }
}
```

`@media` queries are emitted as scoped CSS `<style>` elements. Each element gets a unique class to ensure the query only applies to that element.

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

## Slides Output (PDF Slide Decks)

PDF deck output where **one `Slide` = one PDF page** (no flow pagination).

```json
{
    "build": {
        "output_type": "slides",
        "slides": {
            "size": "16:9",
            "default_font": "Helvetica",
            "default_font_size": 24,
            "margin": 60,
            "show_slide_numbers": true,
            "footer_text": "My Deck",
            "background_color": "#1A1A19",
            "chrome_color": null,
            "output_filename": "deck.pdf"
        }
    }
}
```

`background_color` paints every slide full-bleed with the given color (override per-slide via `Slide { style { background } }`). `chrome_color` overrides the slide-number/footer color; if `null`, it auto-flips between dark and light grey based on the slide's background luminance.

A deck must be wrapped in a `Presentation { ... }` block inside a `Page` body. Slide elements must not appear outside `Presentation` (compile error).

### Slide kinds

| Component | Purpose |
|-----------|---------|
| `Presentation { ... }` | Deck root — children must be slide elements only |
| `Slide { ... }` | Freeform slide; top-aligned content |
| `TitleSlide("Title", "Subtitle")` | Cover slide; title 56pt bold + subtitle 28pt grey, vertical-centered |
| `SectionSlide("Label", primary)` | Full-bleed colored band, white centered label (48pt bold). Color modifiers: `primary`, `success`, `danger`, `warning`, `info` |
| `TwoColumn { Container { ... } Container { ... } }` | Two equal columns with a 24pt gutter — requires exactly 2 `Container` children |
| `ImageSlide(src: "...", caption: "...")` | Image slide with optional caption; `src` is required |

Body components inside a `Slide` (or inside `TwoColumn`'s columns): `Text`, `Heading` (auto-scaled ~2× for slides), `List`, `Container`, `Stack`, `Column`, `Grid`, `Section`, `Spacer`, `Divider`, `if`/`for` (static iteration only).

### Slide sizing (`slides.size`)

- `"16:9"` → 960×540pt (default)
- `"4:3"` → 720×540pt
- `"A4-landscape"` → 841.89×595.28pt
- `"WIDTHxHEIGHT"` (e.g. `"800x600"`) → explicit points

Or override with `slides.width` + `slides.height` (in points).

### Slide chrome (opt-in via config)

- `slides.show_slide_numbers: true` → `n / total` in bottom-right
- `slides.footer_text: "..."` → text in bottom-left

Both render in 11pt grey at the bottom margin.

### Overflow

Content that exceeds the bottom margin is **clipped** and a warning is printed to stderr (`warning[slides]: slide N content overflows; truncated`). The build does **not** fail on overflow.

### Rejected in slides

Same interactive components as PDF (`Button`, `Input`, `Form`, `Modal`, `Router`, `Navbar`, `Video`, etc.) plus PDF document components (`Document`, `Paragraph`, `PageBreak`, `Header`, `Footer`) — slides have their own footer chrome via config.

### Slides example

```wf
Page Deck (path: "/", title: "Q1 Review") {
    Presentation {
        TitleSlide("Q1 Review", "Company Inc. — March 2026")

        Slide {
            Heading("Highlights", h1)
            List {
                Text("Launched 3 new products")
                Text("Expanded to 5 markets")
                Text("Revenue grew 15%")
            }
        }

        TwoColumn {
            Container {
                Heading("Wins", h3)
                Text("New enterprise deals")
            }
            Container {
                Heading("Risks", h3)
                Text("Supply chain delays")
            }
        }

        ImageSlide(src: "chart.png", caption: "Q1 revenue by region")

        SectionSlide("Q2 Plan", primary)

        Slide {
            Heading("Thanks", h1)
            Text("Questions?")
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
        },
        "slides": {
            "size": "16:9",
            "width": null,
            "height": null,
            "default_font": "Helvetica",
            "default_font_size": 24,
            "margin": 60,
            "show_slide_numbers": false,
            "footer_text": null,
            "background_color": null,
            "chrome_color": null,
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

let html   = tpl.render_html(&json!({"name": "World"}))?;           // Full HTML doc
let frag   = tpl.render_html_fragment(&json!({"name": "World"}))?;  // Fragment only
let pdf    = tpl.render_pdf(&json!({"name": "World"}))?;            // Vec<u8>
let slides = tpl.render_slides(&json!({"name": "World"}))?;         // Vec<u8> (PDF deck)

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
11. **`return` in actions**: `return expr` returns a value from store actions
11. **Style blocks support all CSS properties**: including `transition`, `animation`, `filter` — no conflicts with language keywords
12. **`@media` inside style blocks**: responsive styles are scoped to the element — `@media (max-width: 768px) { display: "none" }`
13. **Router nests anywhere**: `Router` can be inside `Row`, `Container`, `Stack`, or any layout wrapper at any depth
14. **Browser globals are not prefixed**: `localStorage`, `window`, `console`, `JSON`, `Math`, `Date`, `setTimeout`, `fetch`, `Promise`, etc. compile as-is
15. **Both `!=` and `!==`**: both inequality operators are supported (both compile to `!==` in JS)
16. **Quoted map keys**: `{ "Content-Type": "application/json" }` — use for HTTP headers and hyphenated keys
17. **Reserved words as map keys**: `{ action: "approve", token: tok }` — all keywords work as map keys
18. **`public/` copies to build root**: files in `public/` land at the root of the output directory, not nested
19. **Slides need a Presentation wrapper**: `Page X { Presentation { Slide { ... } } }` — slide elements outside `Presentation` are a compile error
20. **One Slide = one PDF page**: slides do not flow across pages; overflow is clipped with a stderr warning
