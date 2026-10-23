//! The rules the language keeps about what reaches the page.
//!
//! Two of them have a build-time twin, and the twins are what these hold
//! together: a `javascript:` URL is refused the same way whether it was
//! written in the source or arrived at run time, and markup is sanitised,
//! and Markdown drawn, to exactly the same bytes in the static paint as in
//! the browser.
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

test("Markdown gives the same bytes as the compiler's", () => {
  const { WF } = loadRuntime();
  const cases = JSON.parse(
    readFileSync(new URL("../markdown-cases.json", import.meta.url), "utf8"),
  );
  for (const [text, options, expected] of cases) {
    WF.setBasePath(options.base || "");
    const opts = options.inert ? { inert: true } : undefined;
    assert.equal(WF.markdown(text, opts), expected, `markdown(${JSON.stringify(text)}, ${JSON.stringify(options)})`);
  }
  WF.setBasePath("");
});

test("a Markdown link or image a browser would run is drawn as its words", () => {
  const { WF } = loadRuntime();
  // `[x](javascript:alert(1))` used to be a live link.
  assert.equal(WF.markdown("[x](javascript:alert(1))"), "<p>x)</p>\n");
  // Whitespace before the URL is no URL at all; a control character a
  // browser strips does not hide the scheme behind it.
  for (const md of ["[x]( JavaScript:alert(1))", "[x](\u001fJavaScript:alert(1))", "[x](JAVASCRIPT:alert(1))"]) {
    const html = WF.markdown(md);
    assert.ok(!html.includes("<a") && !html.includes("href"), `${JSON.stringify(md)} → ${html}`);
    assert.ok(html.includes("x"), "the words stay");
  }
  assert.equal(WF.markdown("![pic](javascript:alert(1))"), "<p>pic)</p>\n");
  assert.equal(WF.markdown("[x](vbscript:msgbox(1)) ![d](data:image/png;base64,AA==)"), "<p>x) d</p>\n");
  assert.equal(
    WF.markdown("[docs](https://example.com/docs) ![i](https://example.com/i.png)"),
    "<p><a href=\"https://example.com/docs\">docs</a> <img src=\"https://example.com/i.png\" alt=\"i\"></p>\n",
  );
  // A site-relative link still takes the base path; a refused one has none to take.
  WF.setBasePath("/site/");
  assert.equal(WF.markdown("[a](/docs) [b](javascript:x)"), "<p><a href=\"/site/docs\">a</a> b</p>\n");
  WF.setBasePath("");
});

test("inert Markdown shows text somebody else wrote and nothing in it runs", () => {
  const { WF } = loadRuntime();
  const inert = (md) => WF.markdown(md, { inert: true });
  assert.equal(inert("Returns &lt;color&gt;"), "<p>Returns &lt;color&gt;</p>\n", "shown as <color>");
  for (const md of ["<script>alert(1)</script>", "&lt;script&gt;alert(1)&lt;/script&gt;", "&#60;script&#62;alert(1)&#60;/script&#62;"]) {
    const html = inert(md);
    assert.ok(!html.includes("<script"), `${md} → ${html}`);
    assert.equal(html, "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>\n");
  }
  const linked = inert("See ![logo](https://x.y/l.png) the [guide](https://x.y/g), [run](javascript:x) and [![b](/b.svg)](/ci).");
  assert.ok(!linked.includes("<img"), linked);
  assert.ok(!linked.includes("<a"), linked);
  assert.equal(linked, "<p>See the guide, run and .</p>\n");
  assert.equal(inert("_Widely_ used"), "<p><em>Widely</em> used</p>\n");
  assert.equal(inert("```ts\nconst a: Array&lt;T&gt; = [];\n```"), "<pre><code class=\"language-ts\">const a: Array&amp;lt;T&amp;gt; = [];\n</code></pre>\n", "a fence is a pre, its entities as written");
  assert.equal(inert("`&amp;` is &amp;"), "<p><code>&amp;amp;</code> is &amp;</p>\n", "an entity in code is as written");
  // Plain Markdown keeps today's: an entity is shown as it was written.
  assert.equal(WF.markdown("&lt;color&gt;"), "<p>&amp;lt;color&amp;gt;</p>\n");
});

test("a backslash before punctuation draws the character and nothing more", () => {
  const { WF } = loadRuntime();
  const punctuation = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
  const shown = (c) => c.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  for (const c of punctuation) {
    for (const opts of [undefined, { inert: true }]) {
      assert.equal(WF.markdown("\\" + c, opts), "<p>" + shown(c) + "</p>\n", `\\${c}`);
      assert.equal(WF.markdown("a \\" + c + " b", opts), "<p>a " + shown(c) + " b</p>\n", `a \\${c} b`);
    }
  }
  // An escaped mark at a line's start makes no block of it.
  assert.equal(
    WF.markdown("\\# not a heading\n\\- not a list\n1\\. nor this\n\\> not a quote\n\\---"),
    "<p># not a heading\n- not a list\n1. nor this\n&gt; not a quote\n---</p>\n",
  );
  // No link, picture or emphasis is made of an escaped character.
  assert.equal(WF.markdown("\\[not a link\\](https://x.y) !\\[nor a picture](/p.png)"), "<p>[not a link](https://x.y) ![nor a picture](/p.png)</p>\n");
  assert.equal(WF.markdown("[a\\](b) [c](d)"), "<p>[a](b) <a href=\"d\">c</a></p>\n", "nor carries a link's words into the next");
  assert.equal(WF.markdown("[a\\]b](/c)"), "<p><a href=\"/c\">a]b</a></p>\n");
  assert.equal(WF.markdown("\\*not em\\* *a \\* b* **c \\*\\* d** _e\\_f_"), "<p>*not em* <em>a * b</em> <strong>c ** d</strong> <em>e_f</em></p>\n");
  assert.equal(WF.markdown("\\\\*a*"), "<p>\\<em>a</em></p>\n", "`\\\\` is one backslash");
});

test("an escaped character in a Markdown URL is checked as the character it is", () => {
  const { WF } = loadRuntime();
  // `javascript\:` is `javascript:`, refused as it is.
  for (const md of ["[x](javascript\\:alert(1))", "![x](javascript\\:alert(1))", "[x](JavaScript\\:alert\\(1\\))"]) {
    const html = WF.markdown(md);
    assert.ok(!html.includes("<a") && !html.includes("<img"), `${md} → ${html}`);
  }
  assert.equal(WF.markdown("[x](javascript\\:alert\\(1\\))"), "<p>x</p>\n");
  WF.setBasePath("/site");
  assert.equal(
    WF.markdown("[a](\\/docs) [b](/\\/b.c) [c](/\\*x\\*) [d](/e\\(1\\))"),
    "<p><a href=\"/site/docs\">a</a> <a href=\"//b.c\">b</a> <a href=\"/site/*x*\">c</a> <a href=\"/site/e(1)\">d</a></p>\n",
  );
  WF.setBasePath("");
  assert.equal(
    WF.markdown("\\<b\\> ![a\\\"b](/i.png) [c](/x\\\"onclick=alert(1))"),
    "<p>&lt;b&gt; <img src=\"/i.png\" alt=\"a&quot;b\"> <a href=\"/x&quot;onclick=alert(1\">c</a>)</p>\n",
    "no markup comes through an escaped character",
  );
});

test("a backslash before anything else, ending a block, or in code is shown", () => {
  const { WF } = loadRuntime();
  assert.equal(WF.markdown("\\a C:\\Users\\me \\ x \\1"), "<p>\\a C:\\Users\\me \\ x \\1</p>\n");
  assert.equal(WF.markdown("a\\\nb\\"), "<p>a<br>\nb\\</p>\n", "it breaks a line, and is shown at a paragraph's end");
  assert.equal(WF.markdown("a\\\\\nb"), "<p>a\\\nb</p>\n", "an escaped one breaks nothing");
  assert.equal(WF.markdown("# Title\\\n\n- item\\"), "<h1>Title\\</h1>\n<ul>\n<li>item\\</li>\n</ul>\n");
  assert.equal(WF.markdown("`a\\*b` `C:\\` \\`not code`"), "<p><code>a\\*b</code> <code>C:\\</code> `not code`</p>\n");
  assert.equal(WF.markdown("`a\\\nb`"), "<p><code>a\\\nb</code></p>\n");
  assert.equal(WF.markdown("```c\\+\\+\n\\*a\\* \\\\\n```"), "<pre><code class=\"language-c++\">\\*a\\* \\\\\n</code></pre>\n");
  const inert = (md) => WF.markdown(md, { inert: true });
  assert.equal(inert("\\&lt; &lt\\; &lt;"), "<p>&amp;lt; &amp;lt; &lt;</p>\n", "an escaped character begins no entity");
});

test("a language server's escaped documentation reads as it was meant", () => {
  const { WF } = loadRuntime();
  // vscode-css-languageservice's hover for `border-radius`: a backslash
  // before each of \ ` * _ { } [ ] ( ) # + - . !, and < > as entities.
  const hover = "Defines the radii of the outer border edge\\.\n\n![Baseline icon](data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMTgi) _Widely available across major browsers (Baseline since 2015)_\n\nSyntax: &lt;length\\-percentage \\[0,∞\\]&gt;\\{1,4\\} \\[ / &lt;length\\-percentage \\[0,∞\\]&gt;\\{1,4\\} \\]?\n\n[MDN Reference](https://developer.mozilla.org/docs/Web/CSS/Reference/Properties/border-radius)";
  const el = WF.el("div", { className: "wf-markdown", markdown: { text: hover, inert: true } });
  assert.equal(
    el.innerHTML,
    "<p>Defines the radii of the outer border edge.</p>\n<p><em>Widely available across major browsers (Baseline since 2015)</em></p>\n<p>Syntax: &lt;length-percentage [0,∞]&gt;{1,4} [ / &lt;length-percentage [0,∞]&gt;{1,4} ]?</p>\n<p>MDN Reference</p>\n",
  );
  assert.ok(!el.innerHTML.includes("\\"), "no backslash is left to read");
});

test("the markdown attribute takes the inert form, and follows its text", () => {
  const { WF } = loadRuntime();
  const docs = WF.signal("[a](https://a.b) &lt;T&gt;");
  const el = WF.el("div", { className: "wf-markdown", markdown: { text: () => docs(), inert: true } });
  assert.equal(el.innerHTML, "<p>a &lt;T&gt;</p>\n");
  docs.set("![p](/p.png) **b**");
  assert.equal(el.innerHTML, "<p><strong>b</strong></p>\n", "the render follows the text");
  const still = WF.el("div", { markdown: { text: "[a](/a)", inert: true } });
  assert.equal(still.innerHTML, "<p>a</p>\n");
  const on = WF.signal(true);
  const both = WF.el("div", { markdown: { text: "[a](/a)", inert: () => on() } });
  assert.equal(both.innerHTML, "<p>a</p>\n");
  on.set(false);
  assert.equal(both.innerHTML, "<p><a href=\"/a\">a</a></p>\n", "and the flag, where it reads state");
  // The plain form is as it was.
  assert.equal(WF.el("div", { markdown: "[a](/a)" }).innerHTML, "<p><a href=\"/a\">a</a></p>\n");
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
