//! What a long-lived page lets go of: an effect, a computed, a branch or a
//! list item that has been disposed of must not stay reachable from a signal
//! that outlives it, and an effect must stop listening to what its last run
//! no longer read.
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

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment",
    `${fullRuntime()}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment);
  return { WF, window, document };
}

async function collect() {
  gc();
  await new Promise((r) => setTimeout(r, 20));
  gc();
}

test("a disposed effect is not kept alive by a signal that outlives it", async () => {
  const { WF } = load();
  const s = WF.signal(0);
  const refs = [];
  for (let i = 0; i < 200; i++) {
    const [, dispose] = WF.scoped(() => {
      const big = { payload: new Array(1000).fill(i) };
      refs.push(new WeakRef(big));
      WF.effect(() => { s(); big.payload.length; });
    });
    dispose();
  }
  await collect();
  assert.equal(refs.filter((r) => r.deref()).length, 0, "every disposed effect's closure is collectable");
  // The signal still works for the effects that are alive.
  let runs = 0;
  WF.effect(() => { s(); runs++; });
  s.set(1);
  assert.equal(runs, 2);
});

test("a disposed computed is not kept alive by the signal it read", async () => {
  const { WF } = load();
  const s = WF.signal(0);
  const refs = [];
  for (let i = 0; i < 200; i++) {
    const [, dispose] = WF.scoped(() => {
      const big = { p: new Array(1000).fill(1) };
      refs.push(new WeakRef(big));
      const c = WF.computed(() => s() + big.p.length);
      WF.effect(() => c());
    });
    dispose();
  }
  await collect();
  assert.equal(refs.filter((r) => r.deref()).length, 0);
});

test("an effect stops listening to a signal its last run did not read", () => {
  const { WF } = load();
  const cond = WF.signal(true);
  const a = WF.signal(1);
  const b = WF.signal(2);
  let runs = 0;
  WF.effect(() => { runs++; return cond() ? a() : b(); });
  assert.equal(runs, 1);
  a.set(10);
  assert.equal(runs, 2, "a is read");
  cond.set(false);
  assert.equal(runs, 3);
  a.set(11);
  assert.equal(runs, 3, "a was dropped by the branch, so it no longer runs the effect");
  b.set(3);
  assert.equal(runs, 4, "b is read now");
  cond.set(true);
  assert.equal(runs, 5);
  b.set(4);
  assert.equal(runs, 5, "b was dropped again");
  a.set(12);
  assert.equal(runs, 6);
});

test("an effect that reads one signal twice runs once per change", () => {
  const { WF } = load();
  const s = WF.signal(0);
  let runs = 0;
  WF.effect(() => { runs++; s(); s(); });
  s.set(1);
  s.set(2);
  assert.equal(runs, 3);
});

test("an effect that throws keeps listening to what it read before", () => {
  const { WF } = load();
  const s = WF.signal(0);
  const t = WF.signal(0);
  let runs = 0;
  WF.effect(() => {
    runs++;
    s();
    if (runs === 2) throw new Error("once");
    t();
  });
  assert.throws(() => s.set(1), /once/);
  assert.equal(runs, 2);
  // `t` was not reached by the run that threw; it still runs the effect.
  t.set(1);
  assert.equal(runs, 3);
  s.set(2);
  assert.equal(runs, 4);
});

test("an effect that writes what it reads still settles", () => {
  const { WF } = load();
  const s = WF.signal(0);
  WF.effect(() => { if (s() < 5) s.set(s() + 1); });
  assert.equal(s(), 5);
});

test("a disposed effect is dropped from a signal it is being scheduled by", () => {
  const { WF } = load();
  const s = WF.signal(0);
  let inner = 0;
  let disposeInner = null;
  // The first effect disposes of the second while the change runs.
  WF.effect(() => { if (s() === 1 && disposeInner) disposeInner(); });
  const [, d] = WF.scoped(() => WF.effect(() => { s(); inner++; }));
  disposeInner = d;
  s.set(1);
  assert.equal(inner, 1, "disposed before its turn, it does not run");
  s.set(2);
  assert.equal(inner, 1);
});

test("a keyed list replaced many times keeps no old item alive", async () => {
  const { WF, document } = load();
  const items = WF.signal([]);
  const shared = WF.signal("x");
  const refs = [];
  const host = document.createElement("div");
  WF.each(host, () => items(), (it) => {
    const big = { p: new Array(500).fill(0) };
    refs.push(new WeakRef(big));
    const node = document.createElement("span");
    WF.effect(() => { node.textContent = shared() + big.p.length; });
    return node;
  }, { key: (it) => it.id });
  for (let round = 0; round < 50; round++) {
    items.set([{ id: "a" + round }, { id: "b" + round }]);
  }
  await collect();
  const alive = refs.filter((r) => r.deref()).length;
  assert.ok(alive <= 2, `only the items on screen are alive, got ${alive}`);
});
