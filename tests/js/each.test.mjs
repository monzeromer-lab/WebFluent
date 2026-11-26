//! A keyed list replaced, emptied or drawn from empty: what each item made
//! is disposed of, the new items arrive in order and in the right place,
//! what shares the list's parent stays, and an exit still plays — and
//! the old rows stay beside the new ones while it does. A live item's index
//! is an accessor that follows the item's place, whether or not anything
//! read it before it moved.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { runtimeFor } from "./runtime.mjs";

function load({ motion = false } = {}) {
  const dom = makeDom();
  const { window, document } = dom;
  window.matchMedia = () => ({ matches: false, addEventListener() {} });
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "getComputedStyle", "requestAnimationFrame",
    `${runtimeFor(motion ? ["each", "motion"] : ["each"])}\nreturn WF;`,
  )(
    window, document, dom.Node, dom.Element, dom.DocumentFragment,
    (f) => { f(); return 0; }, window.getComputedStyle, (f) => globalThis.setTimeout(f, 1),
  );
  return { WF, document };
}

const settle = () => new Promise((r) => setTimeout(r, 0));
const texts = (host) => host.children.map((c) => c.textContent);

function list(WF, document, rows, config = {}) {
  const host = document.createElement("ul");
  const live = new Set();
  WF.each(host, () => rows(), (row, i, rowOf) => {
    const li = WF.el("li", {}, [() => rowOf().t]);
    live.add(row.id);
    WF.onCleanup(() => live.delete(row.id));
    return li;
  }, { key: (r) => r.id, live: true, ...config });
  return { host, live };
}

const make = (from, n, t = "") => Array.from({ length: n }, (_, i) => ({ id: from + i, t: t + (from + i) }));

test("a list replaced whole disposes of every old item and draws the new ones in order", () => {
  const { WF, document } = load();
  const rows = WF.signal(make(1, 5));
  const { host, live } = list(WF, document, rows);
  assert.deepEqual(texts(host), ["1", "2", "3", "4", "5"]);
  rows.set(make(10, 3));
  assert.deepEqual(texts(host), ["10", "11", "12"]);
  assert.deepEqual([...live].sort((a, b) => a - b), [10, 11, 12], "the old items' scopes are gone");
  // The markers are where they were: the list goes on working.
  rows.set([...rows(), { id: 13, t: "13" }]);
  assert.deepEqual(texts(host), ["10", "11", "12", "13"]);
  rows.set([]);
  assert.deepEqual(texts(host), []);
  assert.equal(live.size, 0);
  assert.equal(host.childNodes.length, 2, "only the list's markers are left");
  rows.set(make(20, 2));
  assert.deepEqual(texts(host), ["20", "21"]);
});

test("emptying a list that shares its parent leaves what is beside it", () => {
  const { WF, document } = load();
  const host = document.createElement("div");
  host.appendChild(WF.el("p", {}, ["before"]));
  const rows = WF.signal(make(1, 3));
  WF.each(host, () => rows(), (row) => WF.el("span", {}, [row.t]), { key: (r) => r.id, live: true });
  host.appendChild(WF.el("p", {}, ["after"]));
  rows.set(make(7, 2));
  assert.deepEqual(texts(host), ["before", "7", "8", "after"]);
  rows.set([]);
  assert.deepEqual(texts(host), ["before", "after"]);
  rows.set(make(1, 1));
  assert.deepEqual(texts(host), ["before", "1", "after"]);
});

test("a list replaced while its old rows play an exit puts the new rows first, and the old go when it ends", async () => {
  const { WF, document } = load({ motion: true });
  const rows = WF.signal(make(1, 2));
  const { host } = list(WF, document, rows, { exit: "fadeOut" });
  rows.set(make(5, 2));
  assert.deepEqual(texts(host), ["5", "6", "1", "2"], "the leaving rows are still there, after the new ones");
  await settle();
  await settle();
  assert.deepEqual(texts(host), ["5", "6"]);
});

test("a row whose element marks an exit plays it when the list is emptied", async () => {
  const { WF, document } = load({ motion: true });
  const rows = WF.signal(make(1, 2));
  const host = document.createElement("ul");
  WF.each(host, () => rows(), (row) => WF.el("li", { "data-wf-exit": "fadeOut" }, [row.t]), { key: (r) => r.id, live: true });
  rows.set([]);
  assert.deepEqual(texts(host), ["1", "2"], "on their way out");
  await settle();
  await settle();
  assert.deepEqual(texts(host), []);
});

test("a live item's index follows its place, read before the move or not", () => {
  const { WF, document } = load();
  const rows = WF.signal(make(1, 3));
  const host = document.createElement("ol");
  const indexOf = new Map();
  WF.each(host, () => rows(), (row, i, rowOf, at) => {
    indexOf.set(row.id, at);
    // The first row reads its index as it is drawn; the rest only later.
    return WF.el("li", {}, row.id === 1 ? [() => `${at()}:${rowOf().t}`] : [() => rowOf().t]);
  }, { key: (r) => r.id, live: true });
  assert.deepEqual(texts(host), ["0:1", "2", "3"]);
  rows.set([rows()[2], rows()[1], rows()[0]]);
  assert.deepEqual(texts(host), ["3", "2", "2:1"], "the row that reads its index is told it moved");
  assert.equal(indexOf.get(3)(), 0, "one that never read it reads where it is now");
  assert.equal(indexOf.get(2)(), 1);
  // Once read inside an effect, it follows the next move too.
  let seen = null;
  WF.effect(() => { seen = indexOf.get(3)(); });
  rows.set([rows()[1], rows()[2], rows()[0]]);
  assert.equal(seen, 2);
});

test("a list with an item that throws still draws the rest when drawn from empty", () => {
  const { WF, document } = load();
  const rows = WF.signal([]);
  const host = document.createElement("ul");
  const errors = console.error;
  console.error = () => {};
  try {
    WF.each(host, () => rows(), (row) => {
      if (row.id === 2) throw new Error("no");
      return WF.el("li", {}, [row.t]);
    }, { key: (r) => r.id, live: true });
    rows.set(make(1, 3));
  } finally {
    console.error = errors;
  }
  assert.deepEqual(texts(host), ["1", "3"]);
});
