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
