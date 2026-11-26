## Template Engine (Server-Side Rendering)

WebFluent can be used as a **template engine** from Rust or Node.js to render `.wf` templates to HTML or PDF with JSON data.

### CLI

```bash
# Render to HTML
wf render template.wf --data data.json --format html -o output.html

# Render to HTML fragment (no <html> wrapper)
wf render template.wf --data data.json --format html-fragment

# Render to PDF
wf render template.wf --data data.json --format pdf -o report.pdf

# Pipe JSON from stdin
echo '{"name":"Monzer"}' | wf render template.wf --format html

# A directory of templates (components shared), one page of it, in Arabic
wf render templates/ --page Invoice --lang ar --data data.json

# With theme, and a design token over the theme's (repeatable)
wf render template.wf --data data.json --theme Brand --token color-primary=#8B5CF6
```

### Rust API

```toml
webfluent = { version = "7", default-features = false }   # the engine, without the `wf` command's deps
```

```rust
use serde::Serialize;
use webfluent::{PdfConfig, Template};

#[derive(Serialize)]
struct Hello { name: String }

let tpl = Template::from_str(r#"page Hello(path: "/", title: "Hi {name}") { Heading("Hello, {name}!").h1 }"#)?;
// or: Template::from_file("invoice.wf")?, Template::from_files(&[..])?,
//     Template::from_dir("templates")?, Template::from_sources(&[("a.wf", SRC)])?

let data = Hello { name: "World".into() };                 // any `Serialize`, or serde_json::Value
let html   = tpl.render_html(&data)?;                      // whole document: <title>, <html lang>, CSS
let frag   = tpl.render_html_fragment(&data)?;             // markup only
let (css, body) = tpl.render_html_parts(&data)?;           // CSS to serve at its own URL
let pdf    = tpl.render_pdf(&data)?;                       // Vec<u8>
let slides = tpl.render_slides(&data)?;                    // Vec<u8> (PDF deck)

let templates = Template::from_dir("templates")?;          // components shared across files
let receipt = templates.page("Receipt")?                   // one page of several; pages() lists them
    .with_theme("Brand")
    .with_tokens(&[("color-primary", "#8B5CF6")])
    .with_lang("ar")
    .with_pdf(PdfConfig { page_size: "Letter".into(), ..PdfConfig::default() })
    .render_pdf(&data)?;
```

A `Template` parses and checks once, is `Send + Sync` and cheap to clone,
so a server builds it at start-up and renders from any thread. Values from
the data are escaped, and a URL from the data a browser would run
(`javascript:`, `data:`) is dropped from `href`/`src`. `const`s and `data`
files are in scope; components expand in HTML, PDF and slides alike.

### Node.js API

```javascript
// npm install webfluent — a wrapper around `wf render`; needs `wf` installed
const { Template } = require('webfluent');

const tpl = Template.fromString('page Hello(path: "/") { Heading("Hello, {name}!").h1 }');
// or: Template.fromFile('invoice.wf'), Template.fromDir('templates')

const html = tpl.renderHtml({ name: "World" });           // Full HTML string
const frag = tpl.renderHtmlFragment({ name: "World" });   // Fragment string
const pdf  = tpl.renderPdf({ name: "World" });             // Buffer

// In a server, the …Async forms do not block the event loop
const page = Template.fromDir('templates').page('Invoice').withLang('en');
const out = await page.renderHtmlAsync(data);             // renderPdfAsync → Promise<Buffer>
```

### In a browser

The same compiler builds for WebAssembly (`crates/wf-wasm`): a page
compiles a project with nothing sent to a server — sites and apps, every
check `wf check` makes, pictures sized to WebP, PDFs and slide decks.
`crates/wf-wasm/build.sh` writes two builds and `wf.js` loads what a
project needs: the compiler on the first call (about 1.9 MB gzipped), the
one with the PDF engine and its fonts only for a config whose
`output_type` is `"pdf"` or `"slides"`.

```javascript
import { compile, preview, previewConfig, registerFont } from "./wf.js";

const out = await compile(
  { "src/App.wf": app, "src/pages/Home.wf": home, "public/hero.png": pngBytes },
  previewConfig({ name: "Playground" }),
);
out.diagnostics;            // as `wf check --format json` lists them
out.files["index.html"];    // text; out.binary holds pictures and PDFs
frame.src = await preview(out);   // every page, chunk and picture, served
```

A browser has no fonts of its own to lend a document: a PDF's text in a
script the built-in faces lack needs one registered (`registerFont(bytes)`);
Arabic is fetched when a document has it. The Rust side is
`webfluent::memory::compile_files`, which builds a map of path → contents
natively too.

### Template data

Data is a JSON object. The page says what it takes, as its parameters:
every render holds the data to them first, and data that does not fit is
refused with the place it goes wrong — `T33: the data is not what
`Invoice` takes: $.items[0].price is missing` — never drawn as nothing. A
name the page reads that is neither a parameter nor declared is `T13`. A
page that declares no parameters reads the data's top-level keys by name,
unchecked.

A template page may live in a project's `src/` beside its other pages:
a parameter its route has no `:name` for is data only a render hands
over, so `wf build` leaves the page out — no route, no file, no page of a
PDF — while `wf check` holds it to its types, and `wf render . --page
Invoice` renders it with the project's components, types and theme,
and its `.css` files under `src/` as a build bundles them.

```wf
// template.wf
type Customer { name: String }
type Item { name: String, price: Number }

page Invoice(path: "/", title: "Invoice",
             number: String, customer: Customer, items: [Item], paid: Bool) {
    Container {
        Heading("Invoice #{number}").h1
        Text("Customer: {customer.name}")

        for item in items {
            Card { Text(item.name).bold Text("${item.price}") }
        }

        if paid { Badge("PAID").success }
        else    { Badge("UNPAID").danger }
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

**Supported in templates**: all layout, typography, data display components, `for` loops, `if/else`, string interpolation, style blocks, flags, themes. A `state` is its first value and a `derived` value what that works out to (data handed to the render wins over either) — so a page may work its figures out once and show them anywhere.

A `resource` has nothing to show at render time, so it paints its
`.loading` arm — unless the data gives it a value by its name: `{ "users":
{ "loaded": [ … ] } }` paints `.loaded`, `{ "users": { "failed": { … } } }`
`.failed`, and its members (`users.value`, `users.isLoaded`) read the same.
A declared type holds what `loaded` gives (`T33`, `users.loaded[0].name is
missing`).

**Not supported**: `effect`, handlers (`on click`), navigation, stores, animations, a `resource`'s request — nothing runs after the render.

A render is bounded: 100 levels of nesting (`E120`), ten million steps and
256 MB of markup (`E121`). A `style { }` value from the data is one value —
a piece with `;`, a brace, a backslash or `url(…)` leaves its declaration
out — and a `class:` from the data is escaped.

<!-- core -->
