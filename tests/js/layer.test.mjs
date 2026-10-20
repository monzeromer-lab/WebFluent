//! Layers: where a popover, a menu or a tooltip goes against its anchor,
//! and the one stack Escape and a press outside close the top of.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function loadRuntime() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  window.innerWidth = 1000;
  window.innerHeight = 600;
  const src = fullRuntime();
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", "setTimeout", `${src}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment, setTimeout), document, window };
}

const viewport = { width: 1000, height: 600 };
const rect = (left, top, width, height) => ({ left, top, right: left + width, bottom: top + height });

test("a layer sits on its side of the anchor, aligned as asked", () => {
  const { WF } = loadRuntime();
  const anchor = rect(100, 100, 80, 30);
  const size = { width: 200, height: 120 };
  assert.deepEqual(WF.placeAt(anchor, size, viewport, "bottom-start", 6), { x: 100, y: 136, placement: "bottom-start" });
  assert.deepEqual(WF.placeAt(anchor, size, viewport, "bottom-end", 6), { x: 8, y: 136, placement: "bottom-end" }, "slid back on screen");
  assert.deepEqual(WF.placeAt(anchor, size, viewport, "right-start", 6), { x: 186, y: 100, placement: "right-start" });
  assert.deepEqual(WF.placeAt(rect(400, 100, 80, 30), size, viewport, "bottom", 6), { x: 340, y: 136, placement: "bottom-center" });
  assert.deepEqual(WF.placeAt(anchor, size, viewport, "rightStart", 6), { x: 186, y: 100, placement: "right-start" }, "as a `.case` writes it");
});

test("with no room on its side it flips, and stays inside the viewport", () => {
  const { WF } = loadRuntime();
  const size = { width: 200, height: 120 };
  // Near the bottom: below has 600-570-6-8 = 16 px, above has room.
  const low = rect(100, 540, 80, 30);
  assert.deepEqual(WF.placeAt(low, size, viewport, "bottom-start", 6), { x: 100, y: 414, placement: "top-start" });
  // Near the right edge: a right-side layer flips to the left.
  const edge = rect(900, 100, 60, 30);
  assert.equal(WF.placeAt(edge, size, viewport, "right-start", 6).placement, "left-start");
  // At a point — a context menu where the pointer asked for it.
  const point = { left: 990, right: 990, top: 20, bottom: 20 };
  assert.deepEqual(WF.placeAt(point, size, viewport, "bottom-start", 0), { x: 792, y: 20, placement: "bottom-start" });
});

test("Escape closes the newest layer only, and the state that opened it follows", () => {
  const { WF, document } = loadRuntime();
  const button = document.createElement("button");
  document.body.appendChild(button);
  button.focus();
  const outer = document.createElement("div");
  const inner = document.createElement("div");
  const item = document.createElement("li");
  item.setAttribute("role", "menuitem");
  outer.appendChild(item);
  document.body.appendChild(outer);
  document.body.appendChild(inner);
  const first = WF.signal(true);
  const second = WF.signal(true);
  const closed = [];
  WF.layer(outer, first, undefined, { anchor: button, onClose: (why) => closed.push(["outer", why]) });
  assert.equal(outer.hasAttribute("data-open"), true);
  assert.equal(document.activeElement, item, "a menu's first item takes the focus");
  WF.layer(inner, second, undefined, { anchor: button, focus: false, onClose: (why) => closed.push(["inner", why]) });

  document.dispatchEvent({ type: "keydown", key: "Escape", preventDefault() {}, stopPropagation() {} });
  assert.equal(second(), false, "the newest closed");
  assert.equal(first(), true, "the one under it did not");
  assert.deepEqual(closed, [["inner", "escape"]]);
  assert.equal(inner.hasAttribute("data-open"), false);

  document.dispatchEvent({ type: "keydown", key: "Escape", preventDefault() {}, stopPropagation() {} });
  assert.equal(first(), false);
  assert.equal(document.activeElement, button, "focus is back where it was");
});

test("a press outside closes the top layer; a press on its anchor is the anchor's", () => {
  const { WF, document } = loadRuntime();
  const anchor = document.createElement("button");
  const pop = document.createElement("div");
  const elsewhere = document.createElement("p");
  document.body.appendChild(anchor);
  document.body.appendChild(pop);
  document.body.appendChild(elsewhere);
  const open = WF.signal(true);
  WF.layer(pop, open, undefined, { anchor, focus: false });
  document.dispatchEvent({ type: "pointerdown", target: anchor });
  assert.equal(open(), true, "the anchor's own click toggles it");
  document.dispatchEvent({ type: "pointerdown", target: pop });
  assert.equal(open(), true, "inside is inside");
  document.dispatchEvent({ type: "pointerdown", target: elsewhere });
  assert.equal(open(), false);
});
