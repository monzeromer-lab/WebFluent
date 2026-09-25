//! Stores: when one is built, where what it keeps is written, and what it
//! shows of itself.
//!
//! Every store used to be constructed at boot, in the order it was
//! declared, so a `derived` that read a store declared below it read
//! `undefined`. These hold the engine to the opposite: nothing is built
//! until something reads it.
//!
//! Run: node --test tests/js/store.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function loadRuntime() {
  const dom = makeDom();
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(dom.window, dom.document, dom.Node, dom.Element, dom.DocumentFragment, setTimeout);
  return { WF, ...dom };
}

test("a store is built the first time something reads it, and only then", () => {
  const { WF } = loadRuntime();
  let built = 0;
  const S = WF.store("S", () => { built += 1; return { state: { n: 1 } }; });
  assert.equal(built, 0, "declaring one builds nothing");
  assert.equal(S.n, 1);
  assert.equal(built, 1);
  S.n = 2;
  assert.equal(S.n, 2);
  assert.equal(built, 1, "and it is built once");
});

test("a store declared after the one that reads it is still there", () => {
  const { WF } = loadRuntime();
  // `Totals` is declared first and reads `Cart`, which is declared second.
  // Eagerly, this read `undefined`.
  const Totals = WF.store("Totals", () => ({
    derived: { doubled: () => Cart.count * 2 },
  }));
  const Cart = WF.store("Cart", () => ({ state: { count: 21 } }));
  assert.equal(Totals.doubled, 42);
});

test("`eager: true` builds it at boot", () => {
  const { WF } = loadRuntime();
  let built = 0;
  WF.store("S", () => { built += 1; return { state: { n: 1 } }; }, { eager: true });
  assert.equal(built, 1);
});

test("a `.route` store is dropped when the route changes", () => {
  const { WF } = loadRuntime();
  let built = 0;
  const Filters = WF.store("Filters", () => { built += 1; return { state: { q: "" } }; }, { scope: "route" });
  Filters.q = "shoes";
  assert.equal(Filters.q, "shoes");
  assert.equal(built, 1);

  WF.dropRouteStores();
  assert.equal(Filters.q, "", "the page it belonged to has gone, and so has what it held");
  assert.equal(built, 2, "the next read builds it again");
});

test("what a store keeps is written where its policy says", () => {
  const { WF, window } = loadRuntime();
  const S = WF.store("S", () => ({
    state: { a: "", b: "" },
    persist: { a: {}, b: { in: "session" } },
  }));
  S.a = "one";
  S.b = "two";
  assert.equal(window.localStorage.getItem("wf:S.a"), '"one"');
  assert.equal(window.sessionStorage.getItem("wf:S.b"), '"two"');
  assert.equal(window.localStorage.getItem("wf:S.b"), null);
});

test("a `.session` store keeps everything in the tab's own storage", () => {
  const { WF, window } = loadRuntime();
  const S = WF.store("S", () => ({ state: { a: "" }, persist: { a: {} } }), { scope: "session" });
  S.a = "one";
  assert.equal(window.sessionStorage.getItem("wf:S.a"), '"one"');
  assert.equal(window.localStorage.getItem("wf:S.a"), null);
});

test("a value an older build wrote is brought forward one version at a time", () => {
  const { WF, window } = loadRuntime();
  // What version 1 of the site left behind: a bare value, no envelope.
  window.localStorage.setItem("wf:Cart.items", JSON.stringify([{ id: "a", count: 2 }]));
  const Cart = WF.store("Cart", () => ({
    state: { items: [] },
    persist: {
      items: {
        version: 3,
        migrate: {
          2: (old) => old.map((i) => ({ id: i.id, qty: i.count })),
          3: (old) => old.map((i) => ({ ...i, added: null })),
        },
      },
    },
  }));
  assert.deepEqual(Cart.items, [{ id: "a", qty: 2, added: null }]);
  // And what it writes carries the version it is at.
  Cart.items = [{ id: "b", qty: 1, added: null }];
  assert.deepEqual(JSON.parse(window.localStorage.getItem("wf:Cart.items")), {
    "wf:v": 3,
    "wf:d": [{ id: "b", qty: 1, added: null }],
  });
});

test("a value a newer build wrote is left alone", () => {
  const { WF, window } = loadRuntime();
  // The reader has this site open in another tab, on a later deploy.
  window.localStorage.setItem("wf:S.a", JSON.stringify({ "wf:v": 9, "wf:d": "from the future" }));
  const S = WF.store("S", () => ({ state: { a: "initial" }, persist: { a: { version: 2 } } }));
  assert.equal(S.a, "initial", "this build cannot know what version 9 means");
});

test("a gap in the chain falls back rather than handing over the wrong shape", () => {
  const { WF, window } = loadRuntime();
  window.localStorage.setItem("wf:S.a", JSON.stringify({ "wf:v": 1, "wf:d": "old" }));
  const S = WF.store("S", () => ({
    state: { a: "initial" },
    persist: { a: { version: 3, migrate: { 3: (old) => old } } },
  }));
  assert.equal(S.a, "initial");
});

test("another tab's write arrives, and `sync: false` keeps it out", () => {
  const { WF, window } = loadRuntime();
  const S = WF.store("S", () => ({
    state: { shared: "", mine: "" },
    persist: { shared: {}, mine: { sync: false } },
  }));
  assert.equal(S.shared, "");
  window._fire("storage", { key: "wf:S.shared", newValue: JSON.stringify("from the other tab") });
  assert.equal(S.shared, "from the other tab");
  window._fire("storage", { key: "wf:S.mine", newValue: JSON.stringify("ignored") });
  assert.equal(S.mine, "", "a value that says it is this tab's own stays this tab's own");
});

test("storage that is blocked does not stop the store", () => {
  const { WF, window } = loadRuntime();
  window.localStorage.getItem = () => { throw new Error("blocked"); };
  window.localStorage.setItem = () => { throw new Error("blocked"); };
  const S = WF.store("S", () => ({ state: { a: "initial" }, persist: { a: {} } }));
  assert.equal(S.a, "initial");
  S.a = "changed";
  assert.equal(S.a, "changed", "it is simply not written down");
});

test("the devtools see every action, with the state on each side of it", () => {
  const { WF } = loadRuntime();
  const seen = [];
  WF.watchStores((entry) => seen.push(entry));
  const Cart = WF.store("Cart", () => ({
    state: { count: 0 },
    actions: { add: (s, n) => { s.count = s.count + n; } },
  }));
  Cart.add(3);
  const built = seen.find((e) => e.action === null);
  assert.ok(built, "a store reports itself when it is built");
  const call = seen.find((e) => e.action === "add");
  assert.deepEqual(call.args, [3]);
  assert.deepEqual(call.before, { count: 0 });
  assert.deepEqual(call.after, { count: 3 });

  // The tree, and one step of time travel.
  assert.deepEqual(WF.storeSnapshot("Cart"), { count: 3 });
  assert.deepEqual(WF.storeSnapshot(), { Cart: { count: 3 } });
  WF.restoreStore("Cart", call.before);
  assert.equal(Cart.count, 0);
});

test("nothing is recorded while nothing is watching", () => {
  const { WF } = loadRuntime();
  const Cart = WF.store("Cart", () => ({
    state: { count: 0 },
    actions: { add: (s) => { s.count = s.count + 1; } },
  }));
  Cart.add();
  const seen = [];
  const stop = WF.watchStores((e) => seen.push(e));
  Cart.add();
  stop();
  Cart.add();
  assert.equal(seen.length, 1, "one call watched, of three");
  assert.equal(Cart.count, 3);
});
