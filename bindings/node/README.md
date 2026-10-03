# webfluent

Render [WebFluent](https://monzeromer-lab.github.io/WebFluent/) templates to
HTML and PDF from Node.js: an invoice per order, an email, a fragment for a
page you already serve.

```bash
npm install webfluent
```

The package is a thin wrapper around the `wf` compiler, which must be
installed too — it looks on `PATH`, in `~/.webfluent/bin` and `~/.cargo/bin`,
or at `WF_BIN`:

```bash
curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash
```

## Use

```js
const { Template } = require("webfluent");

const tpl = Template.fromFile("templates/invoice.wf");
// or: Template.fromString('page Hello(path: "/") { Heading("Hello, {name}!").h1 }');

const html = tpl.renderHtml(invoice);          // a whole document: <title>, <html lang>, its CSS
const frag = tpl.renderHtmlFragment(invoice);  // the body only
const pdf  = tpl.renderPdf(invoice);           // a Buffer

// One of several `theme` declarations, with tokens on top, in Arabic
const themed = tpl.withTheme("Brand")
  .withTokens({ "color-primary": "#8B5CF6" })
  .withLang("ar")
  .renderHtml(invoice);
```

A directory is one template — every `.wf` and `.wfx` under it, components
shared — and `page(name)` picks the document to render:

```js
const templates = Template.fromDir("templates");
const receipt = templates.page("Receipt");
```

In a server, use the `…Async` forms — `renderHtmlAsync`,
`renderHtmlFragmentAsync`, `renderPdfAsync`, `renderSlidesAsync` — which do
not block the event loop:

```js
app.get("/receipts/:id", async (req, res) => {
  res.send(await receipt.renderHtmlAsync(await load(req.params.id)));
});
```

Each render runs `wf` once (a few milliseconds); the data goes to it over
stdin. Values from the data are escaped, and a URL from the data that a
browser would run (`javascript:`) is dropped.

The data's top-level keys are the template's variables:

```wf
page Invoice(path: "/", title: "Invoice") {
    Heading("Invoice #{number}").h1
    for item in items { Text("{item.name} — {format(item.price, .currency)}") }
    if paid { Badge("Paid").success } else { Badge("Due").warning }
}
```

A template uses the static part of the language — layout, typography, data
display, `for`, `if`, `format`, `style { }`, themes, types and components —
since there is no browser to run state or handlers in. See
[Server rendering](https://monzeromer-lab.github.io/WebFluent/docs/guide/server-rendering)
in the guide.

The package's version is the compiler's it was released with; a newer `wf`
renders the same templates.

MIT licensed.
