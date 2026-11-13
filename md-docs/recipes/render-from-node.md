---
kind: "recipe"
route: "/recipes/render-from-node"
group: "interop"
description: "The Node binding is a small wrapper around `wf render`."
from: ["/docs/guide/server-rendering#render-from-node"]
---
# Render templates from Node

The Node binding is a small wrapper around `wf render`. Install it with
npm:

```bash
npm install webfluent
```

The binding needs `wf` itself installed. It looks for it at `WF_BIN`, then
on `PATH`, then in `~/.webfluent/bin` and `~/.cargo/bin` (and, outside
Windows, `/usr/local/bin` and `/usr/bin`). Its version is the compiler's.

This Express app renders an invoice as HTML and as a PDF:

```js
const { Template } = require("webfluent");

const templates = Template.fromDir("templates");          // or fromFile, fromString
const invoice = templates.page("Invoice");

app.get("/invoices/:id", async (req, res) => {
  const data = await loadInvoice(req.params.id);
  res.send(await invoice.renderHtmlAsync(data));
});
app.get("/invoices/:id.pdf", async (req, res) => {
  res.type("application/pdf").send(await invoice.renderPdfAsync(await loadInvoice(req.params.id)));
});
```

Every render has a synchronous form (`renderHtml`, `renderHtmlFragment`,
`renderPdf`, `renderSlides`) and an `…Async` form that does not block the
event loop. Use the `…Async` forms in a server. `withTheme`, `withTokens`
and `withLang` set what the Rust API sets.

A render runs `wf` once and takes a few milliseconds. The data goes over
standard input, so nothing is written to disk.
