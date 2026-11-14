//! Calls under way and how often a handler runs, without a browser: a call
//! with the arguments of one still running is that call, a `concurrent`
//! store action runs every call, a button is busy while the work its click
//! started runs, and `debounce`/`throttle` run a handler as often as they
//! say. The same in Chrome, with a real page, is `tests/browser/busy.mjs`.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const timers = new Map();
  let ids = 0;
  let now = 0;
  const setTimeout = (fn, ms) => { const id = ++ids; timers.set(id, { fn, at: now + (Number(ms) || 0) }); return id; };
  const clearTimeout = (id) => { timers.delete(id); };
  const DateNow = { now: () => now };
  // Move the clock on, firing every timer due on the way.
  const advance = (ms) => {
    const end = now + ms;
    for (;;) {
      const due = [...timers].filter(([, t]) => t.at <= end).sort((a, b) => a[1].at - b[1].at)[0];
      if (!due) break;
      timers.delete(due[0]);
      now = due[1].at;
      due[1].fn();
    }
    now = end;
  };
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout", "clearTimeout", "Date",
    `${fullRuntime()}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment, setTimeout, clearTimeout, { ...DateNow, prototype: Date.prototype });
  return { WF, document, advance };
}

const later = () => {
  let done;
  const p = new Promise((r) => { done = r; });
  return { p, done };
};

test("a store action called again with the same arguments while it runs is that call", async () => {
  const { WF } = load();
  let runs = 0;
  const gate = later();
  const Api = WF.store("Api", () => ({ actions: { save: async () => { runs += 1; await gate.p; return runs; } } }));
  const a = Api.save();
  const b = Api.save();
  assert.equal(a, b, "the same promise");
  gate.done();
  assert.equal(await a, 1);
  assert.equal(runs, 1);
  await Api.save();
  assert.equal(runs, 2, "once it has ended, a call runs again");
});

test("a concurrent store action runs every call", async () => {
  const { WF } = load();
  let runs = 0;
  const Api = WF.store("Api2", () => ({ actions: { poll: async () => { runs += 1; } }, concurrent: ["poll"] }));
  await Promise.all([Api.poll(), Api.poll()]);
  assert.equal(runs, 2);
});

test("a button is busy while the work its click started runs, and takes no press meanwhile", async () => {
  const { WF, document } = load();
  let runs = 0;
  const gate = later();
  const Api = WF.store("Api3", () => ({ actions: { save: async () => { runs += 1; await gate.p; } } }));
  const button = document.createElement("button");
  let clicks = 0;
  const handler = WF.busy(button, () => { clicks += 1; Api.save(); });
  handler({ type: "click" });
  assert.equal(button.getAttribute("aria-busy"), "true");
  assert.equal(button.getAttribute("aria-disabled"), "true");
  handler({ type: "click" });
  assert.equal(clicks, 1, "the second press does nothing");
  gate.done();
  await new Promise((r) => globalThis.setTimeout(r, 0));
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(runs, 1);
});

test("a handler that starts no work leaves the button as it is", () => {
  const { WF, document } = load();
  const button = document.createElement("button");
  let clicks = 0;
  const handler = WF.busy(button, () => { clicks += 1; });
  handler({ type: "click" });
  handler({ type: "click" });
  assert.equal(clicks, 2);
  assert.equal(button.getAttribute("aria-busy"), null);
});

test("debounce runs once the events stop, with the last of them", () => {
  const { WF, advance } = load();
  const heard = [];
  const f = WF.debounce((v) => heard.push(v), 300);
  f("l"); advance(100); f("la"); advance(100); f("lamp");
  advance(299);
  assert.deepEqual(heard, []);
  advance(1);
  assert.deepEqual(heard, ["lamp"]);
});

test("throttle runs the first at once, at most once a while, and keeps the last", () => {
  const { WF, advance } = load();
  const heard = [];
  const f = WF.throttle((v) => heard.push(v), 100);
  for (let i = 1; i <= 10; i++) { f(i); advance(20); }
  advance(200);
  assert.equal(heard[0], 1);
  assert.equal(heard[heard.length - 1], 10, "where it stopped is heard");
  assert.ok(heard.length <= 4, JSON.stringify(heard));
});
