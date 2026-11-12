//! `sanitize`'s links that open elsewhere, and Markdown's own markers and
//! deep quotes.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  return new Function("window", "document", "Node", "Element", "DocumentFragment", `${fullRuntime()}\nreturn WF;`)(
    window, document, Node, Element, DocumentFragment,
  );
}
const WF = load();

// ─── S24 ──

test("a link that opens elsewhere never keeps rel=opener, and always has noopener noreferrer", () => {
  const cases = [
    ['<a href="https://e.com" target="_blank" rel="opener">x</a>', '<a href="https://e.com" target="_blank" rel="noopener noreferrer">x</a>'],
    ['<a href="https://e.com" target="_blank" rel="OPENER">x</a>', '<a href="https://e.com" target="_blank" rel="noopener noreferrer">x</a>'],
    ['<a href="https://e.com" target="_blank" rel="nofollow opener">x</a>', '<a href="https://e.com" target="_blank" rel="nofollow noopener noreferrer">x</a>'],
    ['<a href="https://e.com" target="_blank" rel="noopener">x</a>', '<a href="https://e.com" target="_blank" rel="noopener noreferrer">x</a>'],
    ['<a href="https://e.com" target="_blank" rel="">x</a>', '<a href="https://e.com" target="_blank" rel="noopener noreferrer">x</a>'],
    ['<a rel="opener" href="https://e.com" target="_blank">x</a>', '<a rel="noopener noreferrer" href="https://e.com" target="_blank">x</a>'],
    // As it was: no rel is given one.
    ['<a href="https://e.com" target="_blank">x</a>', '<a href="https://e.com" target="_blank" rel="noopener noreferrer">x</a>'],
    // No target: what the author wrote, without `opener`.
    ['<a href="https://e.com" rel="nofollow">x</a>', '<a href="https://e.com" rel="nofollow">x</a>'],
    ['<a href="https://e.com" rel="opener">x</a>', '<a href="https://e.com">x</a>'],
  ];
  for (const [input, want] of cases) assert.equal(WF.sanitize(input), want, input);
});

// ─── S47 ──

test("Markdown's own place markers in the text are not read as its own", () => {
  const nul = "\u0000";
  // A code span is held aside as \u00000\u0000; text that spells the same
  // marker must not bring it back a second time.
  const html = WF.markdown("`a` and " + nul + "0" + nul);
  assert.equal((html.match(/<code>/g) || []).length, 1, html);
  assert.ok(!html.includes(nul), "no marker reaches the page");
  // A line break marker likewise.
  const br = WF.markdown("a\u0001\nb");
  assert.ok(!br.includes("<br>"), br);
});

// ─── M3 ──

test("quotes nested deeper than 32 levels are text, and the page is drawn", () => {
  const deep = ">".repeat(5000) + " hi";
  const html = WF.markdown(deep);
  assert.equal((html.match(/<blockquote>/g) || []).length, 32);
  assert.ok(html.includes("hi"));
  // 32 is still quotes all the way down.
  const exact = WF.markdown(">".repeat(32) + " hi");
  assert.equal((exact.match(/<blockquote>/g) || []).length, 32);
  assert.ok(exact.includes("<p>hi</p>"), exact);
  const over = WF.markdown(">".repeat(33) + " hi");
  assert.equal((over.match(/<blockquote>/g) || []).length, 32);
  assert.ok(over.includes("<p>&gt; hi</p>"), over);
  // A shallow quote is as it was.
  assert.equal(WF.markdown("> a\n> b"), "<blockquote>\n<p>a\nb</p>\n</blockquote>\n");
});
