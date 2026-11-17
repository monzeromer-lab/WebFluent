//! The writes `offline.sync` keeps for later: only a write that failed for
//! want of a network is kept; a credential is never stored, and the
//! service's own hook adds the credentials of the moment it is sent; a site
//! under a base path keeps its writes apart; and they can be cleared.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

/// Just enough IndexedDB for one object store with an autoincrement key.
function fakeIndexedDB() {
  const dbs = new Map();
  const later = (fn) => globalThis.setTimeout(fn, 0);
  const req = (fn) => {
    const r = {};
    later(() => { r.result = fn(); r.onsuccess && r.onsuccess(); });
    return r;
  };
  return {
    dbs,
    open(name) {
      const r = {};
      later(() => {
        let db = dbs.get(name);
        const fresh = !db;
        if (!db) {
          db = { stores: new Map(), next: 1 };
          dbs.set(name, db);
        }
        const handle = {
          createObjectStore: (s) => { db.stores.set(s, new Map()); },
          transaction: (s) => {
            const tx = {};
            const store = db.stores.get(s);
            tx.objectStore = () => ({
              add: (v) => { const id = db.next++; store.set(id, { ...v, id }); },
              put: (v) => { store.set(v.id, { ...v }); },
              getAll: () => req(() => [...store.values()]),
              count: () => req(() => store.size),
              delete: (id) => { store.delete(id); },
              clear: () => { store.clear(); },
            });
            later(() => later(() => tx.oncomplete && tx.oncomplete()));
            return tx;
          },
          close() {},
        };
        r.result = handle;
        if (fresh && r.onupgradeneeded) r.onupgradeneeded();
        r.onsuccess && r.onsuccess();
      });
      return r;
    },
  };
}

const settle = async (n = 10) => { for (let i = 0; i < n; i++) await new Promise((r) => globalThis.setTimeout(r, 0)); };

function load({ fetchImpl, online = true, base = "", origin = "https://site.example" }) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  window.location = { origin, href: origin + base + "/", pathname: base + "/" };
  const indexedDB = fakeIndexedDB();
  const container = new EventTarget();
  container.controller = {};
  const registration = new EventTarget();
  registration.update = async () => {};
  container.register = async () => registration;
  const navigator = { onLine: online, serviceWorker: container };
  const setTimeout = (fn) => globalThis.setTimeout(fn, 0);
  const setInterval = () => 0;
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout", "setInterval",
    "clearTimeout", "fetch", "navigator", "AbortController", "XMLHttpRequest", "indexedDB", "location",
    `${fullRuntime()}\nreturn WF;`,
  )(
    window, document, Node, Element, DocumentFragment, setTimeout, setInterval,
    globalThis.clearTimeout, (...a) => fetchImpl(...a), navigator, globalThis.AbortController, class {}, indexedDB,
    { reload() {} },
  );
  if (base) WF.setBasePath(base);
  WF.offline({ sync: true });
  return { WF, indexedDB, navigator, window };
}

const stored = (idb, name = "wf-offline") => {
  const db = idb.dbs.get(name);
  return db ? [...db.stores.get("writes").values()] : [];
};

const ok = () => ({ ok: true, status: 200, headers: { forEach() {} }, text: async () => "{}" });

/// A call whose write was kept fails `.offline(queued: true)`: no answer
/// came, and the write waits to be sent.
const kept = (e) => e.kind === "offline" && e.queued === true;

test("a write made offline is kept, without the credentials its hook added", async () => {
  let token = "alice";
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: false });
  const B = WF.api({
    name: "B", base: "/api",
    onRequest: (r) => { r.headers.Authorization = "Bearer " + token; return r; },
    endpoints: { save: { method: "PUT", path: "items" } },
  });
  await assert.rejects(B.save({ body: { a: 1 } }), kept, "the call says it was kept, not answered");
  await settle();
  const [write] = stored(indexedDB);
  assert.ok(write, "kept");
  assert.equal(write.url, "/api/items");
  assert.equal(write.api, "B");
  assert.ok(!JSON.stringify(write).includes("alice"), `no token stored: ${JSON.stringify(write)}`);
  assert.ok(!Object.keys(write.headers).some((k) => /authorization/i.test(k)));
});

test("a kept write is sent with the hook's credentials of the moment it goes", async () => {
  let token = "alice";
  let online = false;
  const sent = [];
  const fetchImpl = async (url, init) => {
    if (!online) throw new TypeError("Failed to fetch");
    sent.push({ url, auth: init.headers && init.headers.Authorization });
    return ok();
  };
  const { WF, indexedDB, navigator, window } = load({ fetchImpl, online: false });
  const B = WF.api({
    name: "B", base: "/api",
    onRequest: (r) => { r.headers.Authorization = "Bearer " + token; return r; },
    endpoints: { save: { method: "PUT", path: "items" } },
  });
  await assert.rejects(B.save({ body: { a: 1 } }), kept);
  await settle();
  token = "bob";
  online = true;
  navigator.onLine = true;
  window._fire("online", {});
  await settle(30);
  assert.deepEqual(sent, [{ url: "/api/items", auth: "Bearer bob" }]);
  assert.equal(stored(indexedDB).length, 0, "gone once sent");
});

test("a write whose call itself carries a credential is not kept: the caller hears the failure", async () => {
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: false });
  await assert.rejects(
    WF.send("/api/items", { method: "PUT", body: { a: 1 }, headers: { Authorization: "Bearer alice" } }),
    (e) => e.kind === "offline",
  );
  await assert.rejects(
    WF.send("/api/items", { method: "PUT", body: { a: 1 }, headers: { "X-Api-Key": "k" } }),
    (e) => e.kind === "offline",
  );
  await settle();
  assert.equal(stored(indexedDB).length, 0);
});

test("a write another origin refused while the browser is online is not kept and does not succeed", async () => {
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: true });
  await assert.rejects(
    WF.send("https://elsewhere.example/api", { method: "PUT", body: { a: 1 } }),
    (e) => e.kind === "offline",
  );
  await assert.rejects(
    WF.send("//elsewhere.example/api", { method: "PUT", body: { a: 1 } }),
    (e) => e.kind === "offline",
  );
  await settle();
  assert.equal(stored(indexedDB).length, 0);
});

test("a write to this site's own server that could not be reached is kept, when sending it twice is safe", async () => {
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: true });
  await assert.rejects(WF.send("/api/items/1", { method: "PUT", body: { a: 1 } }), kept);
  await assert.rejects(WF.send("https://site.example/api/more", { method: "DELETE" }), kept);
  await settle();
  assert.equal(stored(indexedDB).length, 2);
});

test("a POST that may have reached the server is not kept to be sent again", async () => {
  // The browser was online: the request may have arrived and its answer
  // been lost, and a second POST would be a second order.
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: true });
  await assert.rejects(WF.send("/api/orders", { method: "POST", body: { a: 1 } }), (e) => e.kind === "offline" || e.kind === "network");
  await settle();
  assert.equal(stored(indexedDB).length, 0);
});

test("a POST made offline fails rather than being kept", async () => {
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: false });
  await assert.rejects(WF.send("/api/orders", { method: "POST", body: { a: 1 } }), (e) => e.kind === "offline");
  await settle();
  assert.equal(stored(indexedDB).length, 0);
});

test("a site under a base path keeps its writes in a database of its own, and can clear them", async () => {
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: false, base: "/shop" });
  await assert.rejects(WF.send("/shop/api/items", { method: "PUT", body: "x" }), kept);
  await settle();
  assert.equal(stored(indexedDB).length, 0, "nothing in the shared database");
  assert.equal(stored(indexedDB, "wf-offline:/shop").length, 1);
  await WF.clearQueued();
  await settle();
  assert.equal(stored(indexedDB, "wf-offline:/shop").length, 0);
  assert.equal(WF.network().queued, 0);
});

test("a write kept by an older build with a credential is sent without it", async () => {
  const sent = [];
  let online = false;
  const fetchImpl = async (url, init) => {
    if (!online) throw new TypeError("Failed to fetch");
    sent.push({ url, headers: init.headers });
    return ok();
  };
  const { WF, indexedDB, navigator, window } = load({ fetchImpl, online: false });
  // What the old code stored.
  await assert.rejects(WF.send("/api/items", { method: "PUT", body: "x", headers: { "X-Trace": "1" } }), kept);
  await settle();
  const db = indexedDB.dbs.get("wf-offline");
  const writes = db.stores.get("writes");
  for (const w of writes.values()) w.headers = { ...w.headers, Authorization: "Bearer old" };
  online = true;
  navigator.onLine = true;
  window._fire("online", {});
  await settle(30);
  assert.equal(sent.length, 1);
  assert.deepEqual(sent[0].headers, { "X-Trace": "1", "Content-Type": "text/plain;charset=UTF-8" });
});

test("a kept write the server takes makes stale what its endpoint invalidates", async () => {
  const gets = [];
  const fetchImpl = async (url, init) => {
    if ((init.method || "GET") === "GET") gets.push(url);
    if (!navigator.onLine) throw new TypeError("Failed to fetch");
    return ok();
  };
  const { WF, navigator, window } = load({ fetchImpl, online: false });
  const B = WF.api({
    name: "B", base: "/api",
    endpoints: {
      list: { method: "GET", path: "items" },
      save: { method: "PUT", path: "items/{id}", invalidates: () => [["list", null]] },
    },
  });
  await assert.rejects(B.save({ id: "1", body: "x" }), kept);
  await settle();
  // Back online; a read of the list is on the page, asking only when told.
  navigator.onLine = true;
  WF.resource(B.list, { call: true, args: () => ({}), refresh: {} });
  await settle();
  assert.equal(gets.length, 1);
  window._fire("online", {});
  await settle(30);
  assert.equal(WF.network().queued, 0, "sent");
  assert.equal(gets.length, 2, "the list it made stale was asked for again");
});

test("a kept write the server refuses is kept aside, listed in `network.refused`, and not sent again", async () => {
  let online = false;
  let sends = 0;
  const fetchImpl = async () => {
    if (!online) throw new TypeError("Failed to fetch");
    sends += 1;
    return { ok: false, status: 409, headers: { forEach() {} }, text: async () => "" };
  };
  const { WF, indexedDB, navigator, window } = load({ fetchImpl, online: false });
  await assert.rejects(WF.send("/api/items/1", { method: "PUT", body: "x" }), kept);
  await settle();
  online = true;
  navigator.onLine = true;
  window._fire("online", {});
  await settle(30);
  assert.equal(sends, 1);
  assert.equal(WF.network().queued, 0, "no longer waiting");
  const refused = WF.network().refused;
  assert.equal(refused.length, 1);
  assert.equal(refused[0].status, 409);
  assert.equal(refused[0].method, "PUT");
  assert.equal(refused[0].url, "/api/items/1");
  window._fire("online", {});
  await settle(30);
  assert.equal(sends, 1, "a refused write is not sent again");
  WF.network().clearRefused();
  await settle();
  assert.equal(WF.network().refused.length, 0);
  assert.equal(stored(indexedDB).length, 0);
});

test("a kept write is sent through the service's middleware as it is when it goes", async () => {
  let token = "alice";
  let online = false;
  const sent = [];
  const fetchImpl = async (url, init) => {
    if (!online) throw new TypeError("Failed to fetch");
    sent.push({ url, auth: init.headers.Authorization, client: init.headers["X-Client"] });
    return ok();
  };
  const { WF, indexedDB, navigator, window } = load({ fetchImpl, online: false });
  const B = WF.api({
    name: "B", base: "/api",
    middleware: [WF.mw.bearer(() => token), WF.mw.header("X-Client", () => "lab")],
    endpoints: { save: { method: "PUT", path: "items" } },
  });
  await assert.rejects(B.save({ body: { a: 1 } }), kept);
  await settle();
  const [write] = stored(indexedDB);
  assert.ok(write && write.api === "B", `kept, to be made up again: ${JSON.stringify(write)}`);
  assert.ok(!JSON.stringify(write).includes("alice"), "no token stored");
  token = "bob";
  online = true;
  navigator.onLine = true;
  window._fire("online", {});
  await settle(30);
  assert.deepEqual(sent, [{ url: "/api/items", auth: "Bearer bob", client: "lab" }]);
  assert.equal(stored(indexedDB).length, 0, "gone once sent");
});
