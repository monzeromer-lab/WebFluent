//! `WF.windowed`, a `VirtualList`'s `for`, without a layout engine: rows of
//! a fixed height in a box of a set height, so the window is arithmetic.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", `${fullRuntime()}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment), document, window };
}

/// A 320 px box of 32 px rows over `list`, each row a `p` saying its item;
/// `drawn` counts how many times a row was made, `gone` how many let go.
function list(WF, document, items, options = {}) {
  const box = document.createElement("div");
  box.clientHeight = 320;
  box.scrollTop = 0;
  document.body.appendChild(box);
  const source = WF.signal(items);
  const stats = { drawn: 0, gone: 0 };
  const [, dispose] = WF.scoped(() =>
    WF.windowed(box, () => source(), (item, index, itemOf, indexOf) => {
      stats.drawn++;
      WF.onCleanup(() => stats.gone++);
      const p = WF.el("p", {}, [() => `${itemOf()}@${indexOf()}`]);
      return p;
    }, { key: (x) => x, live: true, rowHeight: 32, overscan: 2, ...options }));
  const rows = () => [...box.children].filter((n) => n.tagName === "P").map((n) => n.textContent);
  const scroll = (y) => {
    box.scrollTop = y;
    box.dispatchEvent({ type: "scroll" });
  };
  return { box, source, stats, rows, scroll, dispose };
}

const numbers = (n) => Array.from({ length: n }, (_, i) => `r${i}`);

test("a window of rows is drawn, and spacers stand for the rest", () => {
  const { WF, document } = load();
  const l = list(WF, document, numbers(1000));
  // 320 / 32 = 10 in view, one more for a partial row, two overscan below.
  assert.deepEqual(l.rows(), numbers(13).map((r, i) => `${r}@${i}`));
  const [top, bottom] = [l.box.children[0], l.box.children[l.box.children.length - 1]];
  assert.equal(top.style.height, "0px");
  assert.equal(bottom.style.height, `${(1000 - 13) * 32}px`);
});

test("a scroll draws what it reaches and lets go of what it left", () => {
  const { WF, document } = load();
  const l = list(WF, document, numbers(1000));
  l.scroll(32 * 500);
  assert.equal(l.rows()[0], "r498@498", "two of overscan above");
  assert.equal(l.rows().length, 2 + 11 + 2);
  assert.equal(l.box.children[0].style.height, `${498 * 32}px`);
  assert.equal(l.stats.gone, 13, "the first window's rows were let go, and their effects with them");
});

test("a row in the window keeps its node across a scroll and a change", () => {
  const { WF, document } = load();
  const l = list(WF, document, numbers(100));
  const r5 = [...l.box.children].find((n) => n.textContent === "r5@5");
  l.scroll(32 * 3);
  assert.ok(r5.isConnected !== false && l.box.contains(r5), "r5 stayed");
  // An item put in front moves every row one down: the same nodes, told
  // their new index in place.
  const made = l.stats.drawn;
  l.source.set(["new", ...numbers(100)]);
  assert.ok(l.box.contains(r5));
  assert.equal(r5.textContent, "r5@6");
  assert.ok(l.stats.drawn - made <= 1, "at most the row that came into the window is new");
});

test("a list that shrinks under the reader is looked at from its end", () => {
  const { WF, document } = load();
  const l = list(WF, document, numbers(1000));
  l.scroll(32 * 900);
  l.source.set(numbers(20));
  const shown = l.rows();
  assert.ok(shown.includes("r19@19"), shown.join(" "));
  assert.ok(shown.length <= 20);
});

test("reveal scrolls the row with that key into view", () => {
  const { WF, document } = load();
  const box = document.createElement("div");
  box.clientHeight = 320;
  box.scrollTop = 0;
  document.body.appendChild(box);
  const wanted = WF.signal(null);
  WF.scoped(() => WF.windowed(box, () => numbers(1000), (item) => WF.el("p", {}, [item]), {
    key: (x) => x, live: true, rowHeight: 32, reveal: () => wanted(),
  }));
  wanted.set("r700");
  assert.equal(box.scrollTop, 701 * 32 - 320, "its bottom at the box's bottom");
  wanted.set("r650");
  assert.equal(box.scrollTop, 650 * 32, "its top at the box's top");
  const scrolled = box.scrollTop;
  wanted.set("r655");
  assert.equal(box.scrollTop, scrolled, "already in view: nothing moves");
  wanted.set("nothing");
  assert.equal(box.scrollTop, scrolled, "a key not in the list moves nothing");
});

test("the list leaving lets go of every row", () => {
  const { WF, document } = load();
  const l = list(WF, document, numbers(50));
  const drawn = l.stats.drawn;
  l.dispose();
  assert.equal(l.stats.gone, drawn);
  assert.equal(l.rows().length, 0);
});

/// The runtime with a ResizeObserver and frames it can wait for: `seen(el)`
/// is the observer telling what it saw of `el`, after layout and before
/// paint; `frame()` runs what waited for the next frame.
function loadObserved() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const observers = [];
  class ResizeObserver {
    constructor(fn) { this.fn = fn; this.targets = new Set(); observers.push(this); }
    observe(el) { this.targets.add(el); }
    unobserve(el) { this.targets.delete(el); }
    disconnect() { this.targets.clear(); }
  }
  let waiting = new Map();
  let next = 1;
  const requestAnimationFrame = (fn) => { waiting.set(next, fn); return next++; };
  const cancelAnimationFrame = (id) => { waiting.delete(id); };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment",
    "ResizeObserver", "requestAnimationFrame", "cancelAnimationFrame",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(window, document, Node, Element, DocumentFragment, ResizeObserver, requestAnimationFrame, cancelAnimationFrame);
  const seen = (el) => {
    for (const o of observers) if (o.targets.has(el)) o.fn([{ target: el }], o);
  };
  const frame = () => {
    const due = waiting;
    waiting = new Map();
    for (const f of due.values()) f();
  };
  return { WF, document, window, seen, frame, pending: () => waiting.size };
}

test("a box not laid out yet draws only the overscan rows, and the size it is first seen to have is laid out at once", () => {
  const { WF, document, seen, frame, pending } = loadObserved();
  // Built before it is in the page, as a `VirtualList` is: 0 tall.
  const box = document.createElement("div");
  box.clientHeight = 0;
  box.scrollTop = 0;
  const rows = () => [...box.children].filter((n) => n.tagName === "P").map((n) => n.textContent);
  WF.scoped(() => WF.windowed(box, () => numbers(300), (item) => WF.el("p", {}, [item]), {
    key: (x) => x, live: true, rowHeight: 32, overscan: 2,
  }));
  assert.deepEqual(rows(), ["r0", "r1"], "the overscan rows, and not a window's height of them");

  // In the page, 320 px tall: the observer's first word lays it out there
  // and then — before the frame is painted, not a frame later.
  document.body.appendChild(box);
  box.clientHeight = 320;
  seen(box);
  assert.deepEqual(rows(), numbers(13), "ten in view, one partial, two overscan");
  assert.equal(pending(), 0, "nothing was left for the next frame");

  // A later change of size waits for the next frame.
  box.clientHeight = 160;
  seen(box);
  assert.equal(rows().length, 13, "not yet");
  frame();
  assert.deepEqual(rows(), numbers(8), "five in view, one partial, two overscan");

  // Hidden, its rows stay as they are; shown again, it is laid out at once.
  box.clientHeight = 0;
  seen(box);
  frame();
  assert.deepEqual(rows(), numbers(8), "hidden, nothing is drawn or let go");
  box.clientHeight = 320;
  seen(box);
  assert.deepEqual(rows(), numbers(13), "shown again, at once");
  assert.equal(pending(), 0);
});

test("a list without `rowHeight:` lays out the size it is first seen to have at once, too", () => {
  const { WF, document, seen } = loadObserved();
  const box = document.createElement("div");
  box.clientHeight = 0;
  box.scrollTop = 0;
  WF.scoped(() => WF.windowed(box, () => numbers(300), (item) => WF.el("p", {}, [item]), {
    key: (x) => x, live: true, estimate: 40, overscan: 3,
  }));
  const drawn = () => [...box.children].filter((n) => n.tagName === "P").length;
  assert.equal(drawn(), 3, "the overscan rows only");
  document.body.appendChild(box);
  box.clientHeight = 400;
  seen(box);
  // 400 / 40 = 10 in view, one partial, three overscan.
  assert.equal(drawn(), 14);
});

test("a row revealed before the box has a height is in view in the first frame it has one", () => {
  const { WF, document, seen, pending } = loadObserved();
  const box = document.createElement("div");
  box.clientHeight = 0;
  box.scrollTop = 0;
  const picked = WF.signal("r200");
  const rows = () => [...box.children].filter((n) => n.tagName === "P").map((n) => n.textContent);
  WF.scoped(() => WF.windowed(box, () => numbers(300), (item) => WF.el("p", {}, [item]), {
    key: (x) => x, live: true, rowHeight: 32, overscan: 2, reveal: () => picked(),
  }));
  assert.equal(box.scrollTop, 0, "nothing to scroll in yet");
  assert.deepEqual(rows(), ["r0", "r1"]);

  document.body.appendChild(box);
  box.clientHeight = 320;
  seen(box);
  // Its bottom at the box's: 201 × 32 − 320.
  assert.equal(box.scrollTop, 201 * 32 - 320, "scrolled to it as the box got its height");
  assert.ok(rows().includes("r200"), `the row is drawn: ${rows().join(" ")}`);
  assert.equal(rows()[0], "r189", "the rows in view, from two before the first");
  assert.equal(pending(), 0, "in that frame, not the next");

  // With a height, a reveal scrolls at once, as it always did.
  picked.set("r10");
  assert.equal(box.scrollTop, 320, "r10 is above the view: its top at the box's");
  assert.ok(rows().includes("r10"));

  // Hidden, a reveal waits for it to be shown again.
  box.clientHeight = 0;
  seen(box);
  picked.set("r250");
  assert.equal(box.scrollTop, 320, "not while it has no height");
  box.clientHeight = 320;
  seen(box);
  assert.equal(box.scrollTop, 251 * 32 - 320);
  assert.ok(rows().includes("r250"));
});

test("without a ResizeObserver, a box not laid out yet stands on the window's height", () => {
  const { WF, document, window } = load();
  window.innerHeight = 640;
  const box = document.createElement("div");
  box.clientHeight = 0;
  box.scrollTop = 0;
  WF.scoped(() => WF.windowed(box, () => numbers(300), (item) => WF.el("p", {}, [item]), {
    key: (x) => x, live: true, rowHeight: 32, overscan: 2,
  }));
  const rows = [...box.children].filter((n) => n.tagName === "P").map((n) => n.textContent);
  // Nothing would say when the box has a size: 640 / 32 = 20, one partial,
  // two overscan.
  assert.deepEqual(rows, numbers(23));
});
