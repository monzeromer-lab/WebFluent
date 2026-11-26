//! The order a change runs what reads it in: oldest effect first, so a
//! parent runs before what it made; listeners with no age after every
//! effect, in the order they were added; an effect already waiting is
//! not run twice; one that writes what it reads comes back and is
//! stopped after a hundred runs; one that throws does not stop the rest,
//! and the first error is thrown once they have run. And a change that
//! reaches many effects costs about as much per effect as one that
//! reaches a few.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { runtimeFor } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  return new Function(
    "window", "document", "Node", "Element", "DocumentFragment",
    `${runtimeFor([])}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment);
}

test("effects run oldest first, whatever order they started listening in", () => {
  const WF = load();
  const s = WF.signal(0);
  const ran = [];
  const made = [];
  // Made in order a, b, c; run again by hand in reverse, each lists
  // itself again, so the signal lists them c, b, a.
  const gate = WF.signal(false);
  for (const name of ["a", "b", "c"]) {
    made.push(WF.effect(() => { if (gate()) { s(); ran.push(name); } }));
  }
  gate.set(true);
  for (const e of [...made].reverse()) e();
  ran.length = 0;
  s.set(1);
  assert.deepEqual(ran, ["a", "b", "c"]);
});

test("a parent runs before what it made, and what it disposed of does not run", () => {
  const WF = load();
  const s = WF.signal(1);
  const ran = [];
  WF.effect(() => {
    const v = s();
    ran.push("parent " + v);
    // A child made during the parent's run reads the same signal; the
    // parent's next run disposes of it before it could run for the change.
    const [, dispose] = WF.scoped(() => WF.effect(() => ran.push("child " + s())));
    return dispose;
  });
  ran.length = 0;
  s.set(2);
  assert.deepEqual(ran, ["parent 2", "child 2"]);
});

test("an effect made during a run runs at once, and not again for the change that made it", () => {
  const WF = load();
  const a = WF.signal(0);
  const ran = [];
  const late = [];
  WF.effect(() => { a(); ran.push("first"); });
  WF.effect(() => {
    if (a() === 1 && !late.length) late.push(WF.effect(() => { ran.push("made " + a()); }));
    ran.push("second");
  });
  WF.effect(() => { a(); ran.push("third"); });
  ran.length = 0;
  a.set(1);
  assert.deepEqual(ran, ["first", "made 1", "second", "third"]);
});

test("listeners with no age run after every effect, in the order they were added", () => {
  const WF = load();
  const s = WF.signal(0);
  const ran = [];
  s.subscribe(() => ran.push("listener 1"));
  WF.effect(() => { s(); ran.push("effect"); });
  s.subscribe(() => ran.push("listener 2"));
  s.subscribe(() => ran.push("listener 3"));
  ran.length = 0;
  s.set(1);
  assert.deepEqual(ran, ["effect", "listener 1", "listener 2", "listener 3"]);
});

test("an effect two changes reach in one run runs once", () => {
  const WF = load();
  const a = WF.signal(0);
  const b = WF.signal(0);
  let runs = 0;
  WF.effect(() => { a(); b(); runs++; });
  // An older effect writes both: the younger one waits for both changes
  // and runs once for them.
  const go = WF.signal(0);
  WF.effect(() => { if (go()) { a.set(go()); b.set(go()); } });
  runs = 0;
  go.set(1);
  assert.equal(runs, 1);
});

test("an effect that writes what it reads comes back until it settles", () => {
  const WF = load();
  const n = WF.signal(0);
  const seen = [];
  WF.effect(() => { const v = n(); seen.push(v); if (v > 0 && v < 3) n.set(v + 1); });
  seen.length = 0;
  n.set(1);
  assert.deepEqual(seen, [1, 2, 3]);
});

test("an effect that never settles is stopped after a hundred runs, and the rest still run", () => {
  const WF = load();
  const n = WF.signal(0);
  let runs = 0;
  let after = 0;
  WF.effect(() => { const v = n(); runs++; if (v > 0) n.set(v + 1); });
  WF.effect(() => { n(); after++; });
  runs = 0;
  after = 0;
  n.set(1);
  assert.equal(runs, 100);
  assert.ok(after >= 1, "the younger effect ran");
  assert.equal(n() > 100, true);
});

test("one effect that throws does not stop the rest; the first error is thrown after them", () => {
  const WF = load();
  const s = WF.signal(0);
  const ran = [];
  WF.effect(() => { if (s()) { ran.push("a"); throw new Error("first"); } });
  WF.effect(() => { if (s()) { ran.push("b"); throw new Error("second"); } });
  WF.effect(() => { if (s()) ran.push("c"); });
  assert.throws(() => s.set(1), /first/);
  assert.deepEqual(ran, ["a", "b", "c"]);
  // The run is over: the next change starts a new one.
  ran.length = 0;
  assert.throws(() => s.set(2), /first/);
  assert.deepEqual(ran, ["a", "b", "c"]);
});

test("a computed read while its update waits is brought up to date, and not run again", () => {
  const WF = load();
  const s = WF.signal(1);
  const reader = WF.signal(0);
  const seen = [];
  let double = null;
  let computes = 0;
  // The reader is the oldest; the writer next; the computed youngest. A
  // change of `s` runs the writer, which wakes the reader, which reads the
  // computed before its own turn has come.
  WF.effect(() => { if (reader() && double) seen.push(double()); });
  WF.effect(() => { reader.set(s()); });
  double = WF.computed(() => { computes++; return s() * 2; });
  seen.length = 0;
  computes = 0;
  s.set(5);
  assert.deepEqual(seen, [10]);
  assert.equal(computes, 1);
});

test("a change that reaches many effects costs about as much per effect as one that reaches a few", () => {
  const WF = load();
  const time = (count) => {
    const s = WF.signal(0);
    const [, dispose] = WF.scoped(() => {
      for (let i = 0; i < count; i++) WF.effect(() => { s(); });
    });
    const t0 = performance.now();
    for (let i = 1; i <= 5; i++) s.set(i);
    const t = performance.now() - t0;
    dispose();
    return t / count;
  };
  time(2000);
  const few = Math.max(time(1000), 1e-4);
  const many = time(16000);
  // Picking the oldest by a scan of everything waiting made sixteen
  // times the effects cost sixteen times as much each.
  assert.ok(many < few * 6, `per effect: ${few.toFixed(5)} ms for 1000, ${many.toFixed(5)} ms for 16000`);
});
