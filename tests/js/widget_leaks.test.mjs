//! What a widget leaves behind when its page leaves: a carousel's timer and
//! listener, a drawer's or a popup's document listener, the `now` clock, a
//! long list's measured heights, a submenu's place in its menu.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import v8 from "node:v8";
import vm from "node:vm";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

v8.setFlagsFromString("--expose_gc");
const gc = vm.runInNewContext("gc");
async function collect() {
  gc();
  await new Promise((r) => setTimeout(r, 20));
  gc();
}

/// The runtime with timers the test can count and fire.
function load() {
  const dom = makeDom();
  const { window, document, Node, Element, DocumentFragment } = dom;
  const timers = new Map();
  let ids = 0;
  const setInterval = (fn, ms) => { const id = ++ids; timers.set(id, { fn, ms }); return id; };
  const clearInterval = (id) => { timers.delete(id); };
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setInterval", "clearInterval",
    `${fullRuntime()}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment, setInterval, clearInterval);
  const tick = () => { for (const t of [...timers.values()]) t.fn(); };
  return { WF, document, window, timers, tick };
}

const listenersOn = (target, type) => (target._listeners.get(type) || []).length;

function carouselRoot(document, n) {
  const root = document.createElement("div");
  const track = document.createElement("div");
  track.className = "wf-carousel__track";
  for (let i = 0; i < n; i++) {
    const slide = document.createElement("div");
    slide.className = "wf-carousel__slide";
    track.appendChild(slide);
  }
  root.appendChild(track);
  document.body.appendChild(root);
  return root;
}

test("an autoplaying carousel stops its timer and its document listener when its page leaves", () => {
  const { WF, document, timers } = load();
  const before = listenersOn(document, "visibilitychange");
  for (let i = 0; i < 20; i++) {
    const [, dispose] = WF.scoped(() => WF.carousel(carouselRoot(document, 3), { autoplay: true, interval: 1000 }));
    assert.equal(timers.size, 1, "it plays while its page is shown");
    dispose();
    assert.equal(timers.size, 0, "its interval is cleared");
  }
  assert.equal(listenersOn(document, "visibilitychange"), before);
});

test("a carousel that left does not start its timer again on a late pointer event", () => {
  const { WF, document, timers } = load();
  const root = carouselRoot(document, 3);
  const [, dispose] = WF.scoped(() => WF.carousel(root, { autoplay: true }));
  root.dispatchEvent({ type: "mouseenter" });
  assert.equal(timers.size, 0, "a pointer over it pauses it");
  dispose();
  root.dispatchEvent({ type: "mouseleave" });
  assert.equal(timers.size, 0);
});

test("a carousel still plays and pauses while its page is shown", () => {
  const { WF, document, timers, tick } = load();
  const root = carouselRoot(document, 3);
  const [c] = WF.scoped(() => WF.carousel(root, { autoplay: true }));
  tick();
  assert.equal(c.index(), 1);
  root.dispatchEvent({ type: "mouseenter" });
  assert.equal(timers.size, 0);
  root.dispatchEvent({ type: "mouseleave" });
  assert.equal(timers.size, 1);
});

// A drawer is one of the page's layers: Escape reaches it through the one
// listener every layer shares, added once for the page's life, and what
// leaves with a drawer's page is its place among the layers.
test("a drawer's Escape listener leaves with its page, and works while it is there", () => {
  const { WF, document } = load();
  const before = listenersOn(document, "keydown");
  for (let i = 0; i < 10; i++) {
    const panel = document.createElement("aside");
    const toggle = document.createElement("button");
    const [d, dispose] = WF.scoped(() => WF.drawer(panel, toggle, null));
    d.open();
    assert.equal(panel.getAttribute("data-open"), "true");
    document.dispatchEvent({ type: "keydown", key: "Escape" });
    assert.equal(panel.getAttribute("data-open"), "false", "Escape closes it");
    dispose();
  }
  assert.ok(listenersOn(document, "keydown") <= before + 1, "one listener, however many drawers came and went");
});

test("a popup's outside-click listener leaves with its page, and closes it while it is there", () => {
  const { WF, document } = load();
  const before = listenersOn(document, "click");
  for (let i = 0; i < 10; i++) {
    const root = document.createElement("div");
    document.body.appendChild(root);
    const open = WF.signal(true);
    const [, dispose] = WF.scoped(() => WF.popup(root, null, open));
    document.dispatchEvent({ type: "click", target: document.body });
    assert.equal(open(), false, "a click outside closes it");
    dispose();
  }
  assert.equal(listenersOn(document, "click"), before);
});

test("`now` stops its timer when nothing reads it, and slows down when the fast reader leaves", () => {
  const { WF, timers, tick } = load();
  const [, slow] = WF.scoped(() => WF.effect(() => WF.now()));
  assert.equal(timers.size, 1);
  assert.equal([...timers.values()][0].ms, 60000, "a minute by default");
  const [, fast] = WF.scoped(() => WF.effect(() => WF.now({ every: 1000 })));
  assert.equal(timers.size, 1, "one timer, however many read it");
  assert.equal([...timers.values()][0].ms, 1000, "the fastest asked for");
  fast();
  tick();
  assert.equal(timers.size, 1);
  assert.equal([...timers.values()][0].ms, 60000, "back to the slowest still asked for");
  slow();
  tick();
  assert.equal(timers.size, 0, "nothing reads it: it stops");
  // Read again, it starts again.
  const [, again] = WF.scoped(() => WF.effect(() => WF.now({ every: 5000 })));
  assert.equal(timers.size, 1);
  assert.equal([...timers.values()][0].ms, 5000);
  again();
});

test("`now` read outside any effect starts no timer", () => {
  const { WF, timers } = load();
  const v = WF.now();
  assert.match(v, /^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ$/);
  assert.equal(timers.size, 0);
});

test("`now` keeps a reader current", () => {
  const { WF, tick } = load();
  let runs = 0;
  WF.scoped(() => WF.effect(() => { WF.now({ every: 1000 }); runs++; }));
  const realNow = Date.now;
  Date.now = () => realNow() + 5000;
  const RealDate = globalThis.Date;
  try {
    // The tick sets the value it reads from `new Date()`; force a change.
    globalThis.Date = class extends RealDate { constructor(...a) { super(...(a.length ? a : [realNow() + 3600000])); } };
    tick();
  } finally {
    globalThis.Date = RealDate;
    Date.now = realNow;
  }
  assert.equal(runs, 2);
});

test("a measured VirtualList forgets the heights of rows no longer in its list", async () => {
  const { WF, document } = load();
  const box = document.createElement("div");
  box.clientHeight = 320;
  box.scrollTop = 0;
  document.body.appendChild(box);
  const refs = [];
  const make = (round) => Array.from({ length: 5 }, (_, i) => {
    const item = { id: `${round}-${i}` };
    refs.push(new WeakRef(item));
    return item;
  });
  const source = WF.signal(make(0));
  WF.scoped(() => WF.windowed(box, () => source(), (item) => {
    const p = WF.el("p", {}, [item.id]);
    p._height = 50;
    return p;
  }, { key: (x) => x, live: true, estimate: 40, overscan: 2 }));
  for (let round = 1; round < 40; round++) source.set(make(round));
  await collect();
  const alive = refs.filter((r) => r.deref()).length;
  assert.ok(alive <= 5, `only the rows of the list now are remembered, got ${alive}`);
  // What is still in the list is still laid out by what it measured.
  const bottom = box.children[box.children.length - 1];
  assert.equal(bottom.style.height, "0px");
});

test("a submenu that leaves takes itself off its menu", async () => {
  const { WF, document } = load();
  const root = document.createElement("div");
  const list = document.createElement("div");
  root.appendChild(list);
  document.body.appendChild(root);
  const open = WF.signal(true);
  WF.menu(root, null, list, open);
  const refs = [];
  const closed = [];
  for (let i = 0; i < 30; i++) {
    const item = document.createElement("div");
    item.appendChild(document.createElement("span"));
    const sub = document.createElement("div");
    item.appendChild(sub);
    list.appendChild(item);
    refs.push(new WeakRef(item));
    const [, dispose] = WF.scoped(() => WF.submenu(item, sub, list));
    dispose();
    item.remove();
  }
  // One that stays is still closed by a pointer over another item.
  const item = document.createElement("div");
  item.appendChild(document.createElement("span"));
  const sub = document.createElement("div");
  item.appendChild(sub);
  list.appendChild(item);
  const other = document.createElement("div");
  other.setAttribute("role", "menuitem");
  list.appendChild(other);
  WF.scoped(() => WF.submenu(item, sub, list));
  item.dispatchEvent({ type: "click", target: item });
  list.dispatchEvent({ type: "pointerover", target: other });
  assert.equal(item.getAttribute("aria-expanded"), "false");
  await collect();
  const alive = refs.filter((r) => r.deref()).length;
  assert.equal(alive, 0, `every submenu that left is let go, got ${alive}`);
});
