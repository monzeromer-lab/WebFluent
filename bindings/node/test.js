// `npm test`: the binding against a real `wf` (on PATH, or at WF_BIN).
"use strict";

const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { Template } = require("./index");

const SHOP = `
page Test(path: "/", title: "Test") {
    Container {
        Heading("Hello, {name}!").h1
        for item in items {
            Card.elevated {
                Card.Body {
                    Text(item.name).bold
                    Text("Price: {format(item.price, .currency)}")
                }
            }
        }
        if showBadge {
            Badge("Active").success
        }
    }
}
`;
const DATA = { name: "World", items: [{ name: "Widget", price: 9.99 }, { name: "Gadget", price: 24.99 }], showBadge: true };

test("a fragment interpolates, loops and branches", () => {
  const html = Template.fromString(SHOP).renderHtmlFragment(DATA);
  assert.match(html, /Hello, World!/);
  assert.match(html, /Widget/);
  assert.match(html, /Gadget/);
  assert.match(html, /\$9\.99/);
  assert.match(html, /Active/);
  assert.doesNotMatch(html, /<!DOCTYPE/);
});

test("a document carries its styles", () => {
  const html = Template.fromString(SHOP).renderHtml({ ...DATA, showBadge: false });
  assert.match(html, /<!DOCTYPE html>/);
  assert.match(html, /<style>/);
  assert.doesNotMatch(html, /Active/);
});

test("a PDF is a PDF", () => {
  const pdf = Template.fromString(SHOP).renderPdf(DATA);
  assert.ok(Buffer.isBuffer(pdf));
  assert.equal(pdf.toString("ascii", 0, 5), "%PDF-");
});

test("a deck is a PDF", () => {
  const deck = Template.fromString(`
page Deck(path: "/", title: "Deck") {
    Presentation {
        TitleSlide("{title}", subtitle: "A test")
        Slide { Heading("One").h1 }
    }
}
`).renderSlides({ title: "Q1" });
  assert.equal(deck.toString("ascii", 0, 5), "%PDF-");
});

test("withTheme picks one of the template's themes", () => {
  const tpl = `
theme Day { color-primary: #111111 }
theme Night { color-primary: #EEEEEE }
page P(path: "/", title: "T") { Button("Go").primary }
`;
  assert.match(Template.fromString(tpl).withTheme("Night").renderHtml({}), /#EEEEEE/i);
  assert.match(Template.fromString(tpl, { theme: "Day" }).renderHtml({}), /#111111/i);
});

test("withTokens reaches the stylesheet", () => {
  const html = Template.fromString(`page P(path: "/", title: "T") { Button("Go").primary }`)
    .withTokens({ "color-primary": "#8B5CF6" })
    .renderHtml({});
  assert.match(html, /--color-primary:\s*#8B5CF6/i);
});

test("fromFile reads a file, and an indented one", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-node-"));
  try {
    fs.writeFileSync(path.join(dir, "a.wf"), `page P(path: "/", title: "T") { Text("from {who}") }`);
    fs.writeFileSync(path.join(dir, "b.wfx"), `page P(path: "/", title: "T")\n    Text("indented {who}")\n`);
    assert.match(Template.fromFile(path.join(dir, "a.wf")).renderHtmlFragment({ who: "a file" }), /from a file/);
    assert.match(Template.fromFile(path.join(dir, "b.wfx")).renderHtmlFragment({ who: "too" }), /indented too/);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("a template that does not compile throws what wf said", () => {
  assert.throws(
    () => Template.fromString(`page P(path: "/", title: "T") { Button("x").huge }`).renderHtml({}),
    (e) => e instanceof Error && /huge/.test(e.message) && !/Command failed/.test(e.message),
  );
});

test("a directory is one template, and a page is picked by name", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-node-dir-"));
  fs.writeFileSync(path.join(dir, "parts.wf"), "component Amount(_ v: Number) { Text(format(v, .currency)).bold }\n");
  fs.writeFileSync(
    path.join(dir, "docs.wf"),
    'page Invoice(path: "/i", title: "Invoice #{n}") { Text("invoice")  Amount(total) }\n' +
      'page Receipt(path: "/r", title: "Receipt") { Text("receipt") }\n'
  );
  try {
    const tpl = Template.fromDir(dir);
    const invoice = tpl.page("Invoice").renderHtmlFragment({ n: 1, total: 12.5 });
    assert.match(invoice, /\$12\.50/);
    assert.doesNotMatch(invoice, /receipt/);
    const doc = tpl.page("Invoice").withLang("ar").renderHtml({ n: 7, total: 1 });
    assert.match(doc, /<html lang="ar" dir="rtl">/);
    assert.match(doc, /<title>Invoice #7<\/title>/);
    assert.throws(() => tpl.page("Refund").renderHtml({}), /no page called `Refund`/);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("the async methods render without blocking, and fail the same way", async () => {
  const tpl = Template.fromString(SHOP);
  const [html, pdf] = await Promise.all([tpl.renderHtmlFragmentAsync(DATA), tpl.renderPdfAsync(DATA)]);
  assert.match(html, /Hello, World!/);
  assert.equal(pdf.subarray(0, 5).toString(), "%PDF-");
  await assert.rejects(Template.fromString('page P(path: "/") { Butt0n("x") }').renderHtmlAsync({}), /Butt0n/);
});

test("a URL from the data cannot run script", () => {
  const html = Template.fromString('page P(path: "/") { Link("Pay", to: url) }').renderHtmlFragment({
    url: "javascript:alert(1)",
  });
  assert.doesNotMatch(html, /javascript/i);
});
