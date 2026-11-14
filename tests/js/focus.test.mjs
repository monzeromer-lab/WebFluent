//! `roving`: one Tab stop for a group, the arrow keys to move inside it —
//! the other way round across a right-to-left box — Home and End, the
//! first letters of a name, a disabled item passed over, and a key held
//! with a modifier left to the browser.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load({ rtl = false } = {}) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  if (rtl) document.documentElement.setAttribute("dir", "rtl");
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", "setTimeout", `${fullRuntime()}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment, setTimeout), document };
}

function group(WF, document, names, options = {}, attrs = () => ({})) {
  const items = names.map((n, i) => WF.el("button", attrs(i), [n]));
  const box = WF.el("div", { role: "toolbar" }, items);
  document.body.appendChild(box);
  WF.roving(box, options);
  // A key goes to what has the focus, and on up to the group.
  const press = (k, mods = {}) => {
    const e = { type: "keydown", key: k, preventDefault() { this.prevented = true; }, stopPropagation() {}, ...mods };
    box.dispatchEvent(e);
    return e;
  };
  const focused = () => items.indexOf(document.activeElement);
  const stops = () => items.map((n) => n.getAttribute("tabindex"));
  return { items, box, press, focused, stops };
}

test("one item is the group's Tab stop, the first unless one is marked chosen", () => {
  const { WF, document } = load();
  const plain = group(WF, document, ["Bold", "Italic", "Underline"]);
  assert.deepEqual(plain.stops(), ["0", "-1", "-1"]);
  const chosen = group(WF, document, ["Left", "Centre", "Right"], {}, (i) => ({ "aria-pressed": i === 1 ? "true" : "false" }));
  assert.deepEqual(chosen.stops(), ["-1", "0", "-1"], "the pressed one");
});

test("the arrow keys move, and wrap; Home and End go to the ends", () => {
  const { WF, document } = load();
  const g = group(WF, document, ["Bold", "Italic", "Underline"]);
  g.items[0].focus();
  g.press("ArrowRight");
  assert.equal(g.focused(), 1);
  assert.deepEqual(g.stops(), ["-1", "0", "-1"], "the stop moves with the focus");
  g.press("End");
  assert.equal(g.focused(), 2);
  g.press("ArrowRight");
  assert.equal(g.focused(), 0, "past the end is the start");
  g.press("ArrowLeft");
  assert.equal(g.focused(), 2);
  g.press("Home");
  assert.equal(g.focused(), 0);
  g.press("ArrowDown");
  assert.equal(g.focused(), 0, "a row does not move up and down");
});

test("across a right-to-left box the next item is to the left", () => {
  const { WF, document } = load({ rtl: true });
  const g = group(WF, document, ["أ", "ب", "ج"]);
  g.items[0].focus();
  g.press("ArrowLeft");
  assert.equal(g.focused(), 1);
  g.press("ArrowRight");
  assert.equal(g.focused(), 0);
});

test("a key held with a modifier is the browser's", () => {
  const { WF, document } = load();
  const g = group(WF, document, ["Bold", "Italic"]);
  g.items[0].focus();
  const e = g.press("ArrowRight", { altKey: true });
  assert.equal(g.focused(), 0);
  assert.ok(!e.prevented);
});

test("a disabled item is passed over", () => {
  const { WF, document } = load();
  const g = group(WF, document, ["Cut", "Copy", "Paste"], {}, (i) => (i === 1 ? { disabled: true } : {}));
  g.items[0].focus();
  g.press("ArrowRight");
  assert.equal(g.focused(), 2);
});

test("with typeahead a letter goes to the next item it starts", () => {
  const { WF, document } = load();
  const g = group(WF, document, ["Apple", "Banana", "Blueberry", "Cherry"], { typeahead: true });
  g.items[0].focus();
  g.press("b");
  assert.equal(g.focused(), 1);
  g.press("b");
  assert.equal(g.focused(), 2, "the same letter again goes on to the next");
  g.press("c");
  assert.equal(g.focused(), 3);
});

test("a column moves up and down, not across", () => {
  const { WF, document } = load();
  const g = group(WF, document, ["One", "Two"], { axis: "y" });
  g.items[0].focus();
  g.press("ArrowRight");
  assert.equal(g.focused(), 0);
  g.press("ArrowDown");
  assert.equal(g.focused(), 1);
});

test("a group of tabs selects as the focus moves, unless it opens by hand", () => {
  const { WF, document } = load();
  for (const manual of [false, true]) {
    const tabs = ["One", "Two"].map((t) => WF.el("button", { role: "tab" }, [t]));
    const nav = WF.el("div", { role: "tablist" }, tabs);
    document.body.appendChild(nav);
    const open = WF.signal(0);
    WF.tabs(nav, () => open(), (i) => open.set(i), { manual });
    tabs[0].focus();
    nav.dispatchEvent({ type: "keydown", key: "ArrowRight", preventDefault() {}, stopPropagation() {} });
    assert.equal(document.activeElement, tabs[1]);
    assert.equal(open(), manual ? 0 : 1, manual ? "by hand: the focus moves alone" : "automatic: it opens");
  }
});
