//! A store's `resource`: made when the store is, outside whatever page
//! first read it, and abandoned with the store — a `.route` store's when its
//! route leaves. Its derived values follow it for as long as the store
//! lives.
//!
//! Run: node --test tests/js/store_resource.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function response(body, { status = 200 } = {}) {
  return {
    ok: status >= 200 && status < 300,
    status,
    headers: { forEach: () => {} },
    text: async () => JSON.stringify(body),
  };
}

/// The runtime, with a `fetch` that records each request and answers when
/// the test says: `asked` holds `{ url, signal, answer(body), fail(status) }`.
function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const asked = [];
  const fetchImpl = (url, init) => new Promise((resolve, reject) => {
    const signal = init && init.signal;
    if (signal) signal.addEventListener("abort", () => reject(Object.assign(new Error("aborted"), { name: "AbortError" })));
    asked.push({
      url: String(url),
      signal,
      answer: (body) => resolve(response(body)),
      fail: (status) => resolve(response({ message: "no" }, { status })),
    });
  });
  const setTimeout = (fn, ms) => globalThis.setTimeout(fn, ms);
  const clearTimeout = (id) => globalThis.clearTimeout(id);
  const navigator = { onLine: true };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "fetch", "navigator", "AbortController", "XMLHttpRequest",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(
    window, document, Node, Element, DocumentFragment, setTimeout, clearTimeout,
    fetchImpl, navigator, globalThis.AbortController, class {},
  );
  return { WF, asked, window };
}

const settle = () => new Promise((done) => globalThis.setTimeout(done, 0));

/// `store Feed { state page = 1  resource rows = fetch("/rows/{page}")
/// derived count = rows.data?.length ?? 0  action next() { page = page + 1 } }`,
/// as the compiler emits it.
function feed(WF, options) {
  return WF.store("Feed", () => ({
    state: { page: 1 },
    derived: { count: (store) => (store.rows.data()?.length ?? 0) },
    actions: {
      next: (store) => { store.page = store.page + 1; },
      again: (store) => { store.rows.reload(); },
    },
    resources: {
      rows: (store) => WF.resource(() => `/rows/${encodeURIComponent(store.page)}`, null),
    },
  }), options || { scope: "app" });
}

test("nothing is asked until the store is read, and then once", async () => {
  const { WF, asked } = load();
  const Feed = feed(WF);
  assert.equal(asked.length, 0, "declaring the store asks nothing");
  assert.equal(Feed.rows.state(), "loading");
  assert.equal(asked.length, 1);
  assert.equal(asked[0].url, "/rows/1");
  assert.equal(Feed.count, 0);
  asked[0].answer([{ name: "a" }, { name: "b" }]);
  await settle();
  assert.equal(Feed.rows.state(), "ready");
  assert.deepEqual(Feed.rows.data(), [{ name: "a" }, { name: "b" }]);
  assert.equal(Feed.count, 2, "a derived value follows what it brought");
  assert.equal(asked.length, 1, "reading it again asks nothing more");
});

test("an address that reads the store's state asks again when it changes", async () => {
  const { WF, asked } = load();
  const Feed = feed(WF);
  Feed.rows.state();
  asked[0].answer([{ name: "a" }]);
  await settle();
  Feed.next();
  assert.equal(asked.length, 2);
  assert.equal(asked[1].url, "/rows/2");
  assert.equal(Feed.rows.state(), "loading", "another request is a first load");
  asked[1].answer([{ name: "c" }, { name: "d" }, { name: "e" }]);
  await settle();
  assert.equal(Feed.count, 3);
});

test("another address drops the last answer while it loads, and after it fails", async () => {
  const { WF, asked } = load();
  const Feed = feed(WF);
  Feed.rows.state();
  asked[0].answer([{ name: "a" }, { name: "b" }]);
  await settle();
  assert.equal(Feed.count, 2);
  Feed.next();
  assert.equal(Feed.rows.state(), "loading");
  assert.equal(Feed.rows.data(), null, "page 1's rows are not page 2's");
  assert.equal(Feed.count, 0, "and a derived value over them says so");
  asked[1].fail(404);
  await settle();
  assert.equal(Feed.rows.state(), "error");
  assert.equal(Feed.rows.data(), null);
  assert.equal(Feed.count, 0);
  Feed.again();
  assert.equal(Feed.rows.state(), "loading", "asking again after a failure is still a first load");
  asked[2].answer([{ name: "c" }]);
  await settle();
  assert.equal(Feed.count, 1);
});

test("it outlives the page that first read it", async () => {
  const { WF, asked } = load();
  const Feed = feed(WF);
  // A page reads the store while it is drawn, and then leaves.
  const [, leave] = WF.scoped(() => {
    WF.effect(() => { Feed.rows.state(); Feed.count; });
  });
  assert.equal(asked.length, 1);
  leave();
  assert.equal(asked[0].signal.aborted, false, "leaving did not abandon the store's request");
  asked[0].answer([{ name: "a" }]);
  await settle();
  assert.equal(Feed.rows.state(), "ready");
  assert.equal(Feed.count, 1, "its derived value still follows it");
  Feed.next();
  assert.equal(asked.length, 2, "and its address still follows the store");
  asked[1].answer([{ name: "b" }, { name: "c" }]);
  await settle();
  assert.equal(Feed.count, 2);
});

test("the page that first read it is not made to follow its insides", async () => {
  const { WF, asked } = load();
  const Feed = feed(WF);
  let runs = 0;
  WF.scoped(() => {
    WF.effect(() => { runs += 1; Feed.rows; });
  });
  assert.equal(runs, 1);
  asked[0].answer([{ name: "a" }]);
  await settle();
  Feed.next();
  assert.equal(runs, 1, "it read the handle, not the state the store's request reads");
});

test("an action reloads it, keeping what it has while it asks", async () => {
  const { WF, asked } = load();
  const Feed = feed(WF);
  Feed.rows.state();
  asked[0].answer([{ name: "a" }]);
  await settle();
  Feed.again();
  assert.equal(asked.length, 2);
  assert.equal(Feed.rows.state(), "ready");
  assert.equal(Feed.rows.refreshing(), true);
  asked[1].answer([{ name: "a" }, { name: "b" }]);
  await settle();
  assert.equal(Feed.rows.refreshing(), false);
  assert.equal(Feed.count, 2);
});

test("a failure is the resource's error, and the store carries on", async () => {
  const { WF, asked } = load();
  const Feed = feed(WF);
  Feed.rows.state();
  asked[0].fail(500);
  await settle();
  assert.equal(Feed.rows.state(), "error");
  assert.ok(Feed.rows.error(), "its error is there to show");
  assert.equal(Feed.count, 0);
});

test("a `.route` store's request is abandoned when its route leaves, and made again after", async () => {
  const { WF, asked } = load();
  const Feed = feed(WF, { scope: "route" });
  const first = Feed.rows;
  assert.equal(first.state(), "loading");
  WF.dropRouteStores();
  assert.equal(asked[0].signal.aborted, true, "what was out is abandoned");
  asked[0].answer([{ name: "late" }]);
  await settle();
  assert.equal(first.state(), "loading", "an answer to an abandoned request never lands");

  assert.equal(Feed.rows.state(), "loading", "read again, the store is built again");
  assert.equal(asked.length, 2);
  assert.notEqual(Feed.rows, first);
  asked[1].answer([{ name: "x" }]);
  await settle();
  assert.equal(Feed.count, 1);
});

test("a dropped store's resource no longer follows its state", async () => {
  const { WF, asked } = load();
  const Feed = WF.store("Feed", () => ({
    state: { page: 1 },
    actions: { next: (store) => { store.page = store.page + 1; } },
    resources: {
      rows: (store) => WF.resource(() => `/rows/${store.page}`, { on: { focus: true } }),
    },
  }), { scope: "route" });
  Feed.rows.state();
  // Hold the old instance's action, as a callback left behind would.
  const next = Feed.next;
  WF.dropRouteStores();
  next();
  assert.equal(asked.length, 1, "the old store's address is not followed");
});

test("`on: .focus` stops listening with the store", async () => {
  const { WF, asked, window } = load();
  const Feed = WF.store("Feed", () => ({
    resources: { rows: () => WF.resource("/rows", { on: { focus: true } }) },
  }), { scope: "route" });
  Feed.rows.state();
  asked[0].answer([]);
  await settle();
  window._fire("focus", { type: "focus" });
  assert.equal(asked.length, 2, "focus asks again while the store lives");
  asked[1].answer([]);
  await settle();
  WF.dropRouteStores();
  window._fire("focus", { type: "focus" });
  assert.equal(asked.length, 2, "and not after it is dropped");
});

test("the handle is the store's, not one of its states", () => {
  const { WF } = load();
  const Feed = feed(WF);
  assert.ok(typeof Feed.rows.reload === "function");
  assert.deepEqual(WF.storeSnapshot("Feed"), { page: 1 }, "a snapshot holds state, not requests");
  assert.ok(!Object.keys(Feed).includes("rows"));
});
