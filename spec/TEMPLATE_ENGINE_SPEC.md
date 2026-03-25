# WebFluent Template Engine Specification

## Overview

WebFluent can be used as a **template engine** from other programming languages (Rust, Node.js, etc.) to render `.wf` templates into HTML strings or PDF bytes. This enables server-side rendering of emails, reports, invoices, documentation, and static pages using WebFluent's component system and design tokens.

## Modes

| Mode | Input | Output | Use Case |
|------|-------|--------|----------|
| **HTML** | `.wf` template + JSON data | HTML string | Emails, server-rendered pages, static sites |
| **PDF** | `.wf` template + JSON data | PDF bytes | Reports, invoices, documents |

## Template Syntax

Templates use standard `.wf` syntax with one addition: **data context variables** are accessed directly by name from the JSON data passed at render time.

```wf
// invoice.wf
Container {
    Heading("Invoice #{number}", h1)
    Text("Date: {date}")
    Text("Bill to: {customer.name}")

    Table {
        Thead { Trow { Tcell("Item") Tcell("Qty") Tcell("Price") } }
        for item in items {
            Trow {
                Tcell(item.name)
                Tcell("{item.quantity}")
                Tcell("${item.price}")
            }
        }
    }

    Divider()
    Text("Total: ${total}", bold, large)
}
```

### Data Context

Data is passed as a JSON object. All top-level keys become variables accessible in the template.

```json
{
    "number": "INV-001",
    "date": "2026-03-25",
    "customer": { "name": "Acme Corp", "email": "billing@acme.com" },
    "items": [
        { "name": "Widget", "quantity": 5, "price": 9.99 },
        { "name": "Gadget", "quantity": 2, "price": 24.99 }
    ],
    "total": 99.93
}
```

### Supported Features

Templates support a **subset** of WebFluent — the static, data-driven parts:

| Feature | Supported | Notes |
|---------|-----------|-------|
| All layout components | Yes | Container, Row, Column, Grid, Stack, Spacer, Divider |
| Typography | Yes | Text, Heading, Code, Blockquote |
| Data display | Yes | Card, Table, List, Badge, Tag, Avatar, Progress |
| Feedback (static) | Yes | Alert (no Toast — requires JS) |
| Media | Yes | Image (src resolved to absolute URL or embedded) |
| PDF components | Yes | Document, Section, Paragraph, Header, Footer, PageBreak |
| `for` loops | Yes | Iterates over data arrays |
| `if`/`else` | Yes | Evaluates conditions against data |
| String interpolation | Yes | `"Hello {name}"` resolved from data |
| Nested property access | Yes | `customer.address.city` |
| Index access | Yes | `items[0].name` |
| Design tokens/themes | Yes | CSS variables applied to HTML output |
| Style blocks | Yes | Inline styles emitted |
| Modifiers | Yes | `bold`, `primary`, `large`, etc. |
| **state** | **No** | Use data context instead |
| **derived/effect** | **No** | Computed at call site, pass result as data |
| **Events (on:click)** | **No** | No JavaScript in templates |
| **Navigation/Router** | **No** | No client-side routing |
| **Stores** | **No** | Pass data directly |
| **fetch** | **No** | Fetch data before rendering, pass as data |
| **Animations** | **No** | No JavaScript for animations |
| **i18n (t())** | **Partial** | Pass pre-translated strings, or use `locale` in data |

### Expressions in Templates

Templates evaluate expressions against the data context:

```wf
// Property access
Text(user.name)
Text(order.items[0].name)

// String interpolation
Text("Hello, {user.name}! You have {items.length} items.")

// Conditionals
if order.status == "shipped" {
    Badge("Shipped", success)
} else if order.status == "pending" {
    Badge("Pending", warning)
}

// Loops
for product in products {
    Card {
        Text(product.name, bold)
        Text("${product.price}")
    }
}

// Nested loops
for category in categories {
    Heading(category.name, h2)
    for item in category.items {
        Text("- {item.name}")
    }
}

// Boolean checks
if user.verified {
    Badge("Verified", success)
}
```

## Rust API

### Installation

```toml
[dependencies]
webfluent = "0.2"
```

### Usage

```rust
use webfluent::Template;
use serde_json::json;

// From string
let template = Template::from_str(r#"
    Container {
        Heading("Hello, {name}!", h1)
        Text("You have {count} messages.")
    }
"#)?;

// From file
let template = Template::from_file("templates/invoice.wf")?;

// Render to HTML string (includes CSS)
let html = template.render_html(&json!({
    "name": "Monzer",
    "count": 5
}))?;

// Render to HTML fragment (no <html> wrapper, just the content)
let fragment = template.render_html_fragment(&json!({
    "name": "Monzer",
    "count": 5
}))?;

// Render to PDF bytes
let pdf_bytes = template.render_pdf(&json!({
    "name": "Monzer",
    "count": 5
}))?;

// With custom theme
let html = template
    .with_theme("dark")
    .with_tokens(&[("color-primary", "#8B5CF6")])
    .render_html(&data)?;
```

### Template Struct

```rust
pub struct Template {
    source: String,
    theme: String,           // default: "default"
    custom_tokens: HashMap<String, String>,
}

impl Template {
    pub fn from_str(source: &str) -> Result<Self>;
    pub fn from_file(path: &str) -> Result<Self>;
    pub fn with_theme(self, theme: &str) -> Self;
    pub fn with_tokens(self, tokens: &[(&str, &str)]) -> Self;
    pub fn render_html(&self, data: &serde_json::Value) -> Result<String>;
    pub fn render_html_fragment(&self, data: &serde_json::Value) -> Result<String>;
    pub fn render_pdf(&self, data: &serde_json::Value) -> Result<Vec<u8>>;
}
```

## Node.js API

### Installation

```bash
npm install @aspect/webfluent
```

### Usage

```javascript
const { Template } = require('@aspect/webfluent');

// From string
const tpl = Template.fromString(`
    Container {
        Heading("Hello, {name}!", h1)
        Text("You have {count} messages.")
    }
`);

// From file
const tpl = Template.fromFile('templates/invoice.wf');

// Render to HTML
const html = tpl.renderHtml({ name: "Monzer", count: 5 });

// Render to HTML fragment
const fragment = tpl.renderHtmlFragment({ name: "Monzer", count: 5 });

// Render to PDF (returns Buffer)
const pdfBuffer = tpl.renderPdf({ name: "Monzer", count: 5 });
fs.writeFileSync('invoice.pdf', pdfBuffer);

// With theme
const html = tpl
    .withTheme('dark')
    .withTokens({ 'color-primary': '#8B5CF6' })
    .renderHtml(data);
```

### Express.js Integration

```javascript
const express = require('express');
const { Template } = require('@aspect/webfluent');

const app = express();

app.get('/invoice/:id', async (req, res) => {
    const invoice = await db.getInvoice(req.params.id);
    const tpl = Template.fromFile('templates/invoice.wf');
    const html = tpl.renderHtml(invoice);
    res.send(html);
});

app.get('/invoice/:id/pdf', async (req, res) => {
    const invoice = await db.getInvoice(req.params.id);
    const tpl = Template.fromFile('templates/invoice.wf');
    const pdf = tpl.renderPdf(invoice);
    res.type('application/pdf').send(pdf);
});
```

## CLI Usage

Templates can also be rendered via the CLI:

```bash
# Render to HTML (stdout)
wf render template.wf --data data.json --format html

# Render to HTML file
wf render template.wf --data data.json --format html -o output.html

# Render to PDF
wf render template.wf --data data.json --format pdf -o report.pdf

# Pipe JSON data from stdin
echo '{"name":"Monzer"}' | wf render template.wf --format html

# With custom theme
wf render template.wf --data data.json --format html --theme dark
```

## HTML Output

The HTML renderer produces a complete HTML document (or fragment) with:

1. **Inline CSS** — design tokens and component styles embedded in `<style>`
2. **Semantic HTML** — proper tags for each component (table, ul, h1-h6, etc.)
3. **CSS classes** — `wf-*` classes for styling (same as SPA output)
4. **No JavaScript** — pure static HTML

### Example Output

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <style>
        :root { --color-primary: #3B82F6; /* ... tokens */ }
        .wf-container { max-width: 1200px; margin: 0 auto; /* ... */ }
        /* ... component styles ... */
    </style>
</head>
<body>
    <div class="wf-container">
        <h1 class="wf-heading wf-heading--h1">Hello, Monzer!</h1>
        <p class="wf-text">You have 5 messages.</p>
    </div>
</body>
</html>
```

## Error Handling

Template errors are reported with source location:

```
Error: Undefined variable 'username' at line 3, column 15
  |
3 |     Text("Hello, {username}!")
  |                    ^^^^^^^^ not found in data context

Hint: Available variables: name, email, count
```

```
Error: Cannot iterate over non-array value at line 7
  |
7 |     for item in total {
  |                 ^^^^^ expected array, got number

Hint: 'total' is a number (99.93). Use 'items' to iterate over the items list.
```
