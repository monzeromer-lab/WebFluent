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

test("a write made offline is kept, without the credentials its hook added", async () => {
  let token = "alice";
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: false });
  const B = WF.api({
    name: "B", base: "/api",
    onRequest: (r) => { r.headers.Authorization = "Bearer " + token; return r; },
    endpoints: { save: { method: "POST", path: "items" } },
  });
  const result = await B.save({ body: { a: 1 } });
  assert.equal(result, undefined, "the call resolves with nothing: it is kept");
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
    endpoints: { save: { method: "POST", path: "items" } },
  });
  await B.save({ body: { a: 1 } });
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
    WF.send("/api/items", { method: "POST", body: { a: 1 }, headers: { Authorization: "Bearer alice" } }),
    (e) => e.kind === "offline",
  );
  await assert.rejects(
    WF.send("/api/items", { method: "POST", body: { a: 1 }, headers: { "X-Api-Key": "k" } }),
    (e) => e.kind === "offline",
  );
  await settle();
  assert.equal(stored(indexedDB).length, 0);
});

test("a write another origin refused while the browser is online is not kept and does not succeed", async () => {
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: true });
  await assert.rejects(
    WF.send("https://elsewhere.example/api", { method: "POST", body: { a: 1 } }),
    (e) => e.kind === "offline",
  );
  await assert.rejects(
    WF.send("//elsewhere.example/api", { method: "POST", body: { a: 1 } }),
    (e) => e.kind === "offline",
  );
  await settle();
  assert.equal(stored(indexedDB).length, 0);
});

test("a write to this site's own server that could not be reached is kept", async () => {
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: true });
  assert.equal(await WF.send("/api/items", { method: "POST", body: { a: 1 } }), undefined);
  assert.equal(await WF.send("https://site.example/api/more", { method: "POST", body: "x" }), undefined);
  await settle();
  assert.equal(stored(indexedDB).length, 2);
});

test("a site under a base path keeps its writes in a database of its own, and can clear them", async () => {
  const { WF, indexedDB } = load({ fetchImpl: async () => { throw new TypeError("Failed to fetch"); }, online: false, base: "/shop" });
  await WF.send("/shop/api/items", { method: "POST", body: "x" });
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
  await WF.send("/api/items", { method: "POST", body: "x", headers: { "X-Trace": "1" } });
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
