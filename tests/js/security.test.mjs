//! The rules the language keeps about what reaches the page.
//!
//! Two of them have a build-time twin, and the twins are what these hold
//! together: a `javascript:` URL is refused the same way whether it was
//! written in the source or arrived at run time, and markup is sanitised
//! to exactly the same bytes in the static paint as in the browser.
//!
//! Run: node --test tests/js/security.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function loadRuntime() {
  const dom = makeDom();
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    `${fullRuntime()}\nreturn WF;`,
  );
  return { WF: fn(dom.window, dom.document, dom.Node, dom.Element, dom.DocumentFragment, setTimeout), ...dom };
}

test("the sanitiser gives the same bytes as the compiler's", () => {
  const { WF } = loadRuntime();
  const cases = JSON.parse(
    readFileSync(new URL("../sanitize-cases.json", import.meta.url), "utf8"),
  );
  for (const [html, expected] of cases) {
    assert.equal(WF.sanitize(html), expected, `sanitize(${JSON.stringify(html)})`);
  }
});

test("a scheme a browser would run is not one a page may name", () => {
  const { WF } = loadRuntime();
  for (const hostile of [
    "javascript:alert(1)",
    "JavaScript:alert(1)",
    "  javascript:alert(1)",
    "java\tscript:alert(1)",
    "java\nscript:alert(1)",
    "java\u0000script:alert(1)",
    "vbscript:msgbox(1)",
    "data:text/html,<script>alert(1)</script>",
    "blob:https://example.com/x",
    "file:///etc/passwd",
  ]) {
    assert.equal(WF.safeUrl(hostile), "", hostile);
  }
});

test("everything a page actually links to is passed through", () => {
  const { WF } = loadRuntime();
  for (const fine of [
    "/about", "about", "../up", "#section", "?q=1",
    "https://example.com/a:b", "http://example.com", "HTTPS://EXAMPLE.COM",
    "mailto:ada@example.com", "tel:+441234567890", "sms:+441234567890",
    "//example.com/protocol-relative", "/search?q=time:now",
  ]) {
    assert.equal(WF.safeUrl(fine), fine, fine);
  }
  assert.equal(WF.safeUrl(null), "");
});

test("`Unsafe.Html` puts markup in, and nothing else does", () => {
  const { WF, document } = loadRuntime();
  const el = WF.el("div", { html: "<p>from the CMS</p>" });
  assert.equal(el.innerHTML, "<p>from the CMS</p>");
  // Text is text, wherever it comes from: it goes in as a text node, not
  // as markup, so the tags are shown rather than obeyed.
  const text = WF.el("p", {}, "<b>not bold</b>");
  assert.equal(text.textContent, "<b>not bold</b>");
  assert.equal(text.children.length, 0, "nothing was parsed as markup");
  assert.ok(document);
});

test("a route a browser would run is one the router refuses", () => {
  const { WF, document, window } = loadRuntime();
  const container = document.createElement("div");
  WF.router([{ path: "/", render: () => document.createElement("p") }], container);
  const before = window.location.pathname;
  WF.navigate("javascript:alert(1)");
  assert.equal(window.location.pathname, before, "the page stayed where it was");
});

test("`uuid()` is a version-4 identifier, and a fresh one each time", () => {
  const { WF } = loadRuntime();
  // The DOM harness has no `crypto`, which is the fallback this exercises;
  // a browser that has one returns the platform's own, in the same shape.
  const shape = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
  const seen = new Set();
  for (let i = 0; i < 200; i += 1) {
    const id = WF.uuid();
    assert.match(id, shape);
    seen.add(id);
  }
  assert.equal(seen.size, 200, "every one is its own");
});
