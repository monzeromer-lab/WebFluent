//! The request engine and the connections, held to what a long session and a
//! hostile answer need: a paged list that does not double on refresh, a
//! cache that is bounded and does not serve one user's answer to the next,
//! connections whose messages are bounded, a socket closed while connecting
//! that lets go, and path parameters that cannot climb the path.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

const settle = () => new Promise((done) => globalThis.setTimeout(done, 0));

function response(body, { status = 200, headers = {} } = {}) {
  const map = new Map(Object.entries(headers).map(([k, v]) => [k.toLowerCase(), v]));
  return {
    ok: status >= 200 && status < 300,
    status,
    headers: { forEach: (fn) => map.forEach((v, k) => fn(v, k)) },
    text: async () => (typeof body === "string" ? body : JSON.stringify(body)),
  };
}

function load({ fetchImpl = async () => response({}), online = true, WebSocket = class {}, EventSource = class {} } = {}) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const setTimeout = (fn) => globalThis.setTimeout(fn, 0);
  const clearTimeout = (id) => globalThis.clearTimeout(id);
  const navigator = { onLine: online };
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "fetch", "navigator", "AbortController", "XMLHttpRequest",
    "WebSocket", "EventSource",
    `${fullRuntime()}\nreturn WF;`,
  )(
    window, document, Node, Element, DocumentFragment, setTimeout, clearTimeout,
    fetchImpl, navigator, globalThis.AbortController, class {}, WebSocket, EventSource,
  );
  return { WF, window, navigator };
}

// ─── M8: a paged resource refreshed ──

test("refreshing a paged resource replaces the page it asked for, never appends it again", async () => {
  const PAGES = { 1: ["a", "b"], 2: ["c", "d"], 3: [] };
  const { WF } = load();
  const endpoint = (args) => Promise.resolve(PAGES[args.page].slice());
  const list = WF.resource(endpoint, { call: true, args: () => ({ page: 1 }), paginate: { by: "page" } });
  await settle();
  list.loadMore();
  await settle();
  assert.deepEqual(list.items(), ["a", "b", "c", "d"]);
  for (let i = 0; i < 5; i++) {
    list.reload();
    await settle();
  }
  assert.deepEqual(list.items(), ["a", "b", "c", "d"], "five refreshes, the same items");
  assert.equal(list.hasMore(), true);
});

test("a refreshed page with new contents shows them in its place", async () => {
  const PAGES = { 1: ["a", "b"], 2: ["c"] };
  const { WF } = load();
  const page = WF.signal(1);
  const list = WF.resource((args) => Promise.resolve(PAGES[args.page].slice()), {
    call: true, args: () => ({ page: page() }), paginate: { by: "page", set: (v) => page.set(v) },
  });
  await settle();
  list.loadMore();
  await settle();
  PAGES[2] = ["c", "c2"];
  list.reload();
  await settle();
  assert.deepEqual(list.items(), ["a", "b", "c", "c2"]);
  // Going back to the first page starts over from it, as it did.
  page.set(1);
  await settle();
  assert.deepEqual(list.items(), ["a", "b"]);
});

test("a refresh of the first page alone still reconciles it in place", async () => {
  const { WF } = load();
  let answer = [{ id: 1 }, { id: 2 }];
  const list = WF.resource(() => Promise.resolve(answer), { call: true, args: () => ({ page: 1 }), paginate: { by: "page" } });
  await settle();
  const first = list.items()[0];
  answer = [{ id: 1 }, { id: 2 }, { id: 3 }];
  list.reload();
  await settle();
  assert.equal(list.items().length, 3);
  assert.equal(list.items()[0], first, "an unchanged item is kept as the same object");
});

// ─── M9: the cache is bounded ──

test("the HTTP cache keeps at most its cap of entries, the least recently used going first", async () => {
  const { WF } = load({ fetchImpl: async (url) => response({ url }) });
  const swr = { kind: "swr", ttl: 60000 };
  for (let i = 0; i < 600; i++) await WF.send(`/a/${i}`, { cache: swr });
  assert.ok(WF.send.cacheSize() <= 256, `bounded: ${WF.send.cacheSize()}`);
  // A recent one is still served from the cache.
  let asked = 0;
  const { WF: W2 } = load({ fetchImpl: async (url) => { asked++; return response({ url }); } });
  await W2.send("/keep", { cache: swr });
  for (let i = 0; i < 300; i++) {
    await W2.send(`/b/${i}`, { cache: swr });
    if (i % 50 === 0) await W2.send("/keep", { cache: swr });
  }
  const before = asked;
  await W2.send("/keep", { cache: swr });
  assert.equal(asked, before, "a key read recently stays");
});

test("a request that fails leaves no empty cache entry behind", async () => {
  const { WF } = load({ fetchImpl: async () => response({ no: 1 }, { status: 500 }) });
  for (let i = 0; i < 50; i++) await WF.send(`/f/${i}`, { cache: { kind: "forever" } }).catch(() => {});
  assert.equal(WF.send.cacheSize(), 0);
});

// ─── S34: the cache and credentials ──

test("an answer cached for one Authorization is not served for another", async () => {
  let asked = 0;
  const { WF } = load({ fetchImpl: async (url, init) => { asked++; return response({ who: init.headers.Authorization }); } });
  const cache = { kind: "forever" };
  const a = await WF.send("/me", { cache, headers: { Authorization: "Bearer alice" } });
  const again = await WF.send("/me", { cache, headers: { Authorization: "Bearer alice" } });
  assert.equal(a.who, "Bearer alice");
  assert.equal(again.who, "Bearer alice");
  assert.equal(asked, 1, "the same credentials share the entry");
  const b = await WF.send("/me", { cache, headers: { Authorization: "Bearer bob" } });
  assert.equal(b.who, "Bearer bob");
  const anon = await WF.send("/me", { cache });
  assert.equal(anon.who, undefined);
});

test("an answer cached through a request hook is not served once the hook sends other credentials", async () => {
  let token = "alice";
  let asked = 0;
  const { WF } = load({ fetchImpl: async (url, init) => { asked++; return response({ who: init.headers.Authorization }); } });
  const B = WF.api({
    name: "B", base: "/api", cache: { kind: "swr", ttl: 60000 },
    onRequest: (r) => { r.headers.Authorization = "Bearer " + token; return r; },
    endpoints: { me: { method: "GET", path: "me" }, logout: { method: "POST", path: "logout" } },
  });
  assert.equal((await B.me()).who, "Bearer alice");
  assert.equal((await B.me()).who, "Bearer alice");
  assert.equal(asked, 1);
  token = "bob";
  await B.logout();
  const after = await B.me();
  assert.equal(after.who, "Bearer bob", "the cached answer was alice's");
  // And `invalidate()` clears the whole service's cache, for a logout.
  B.invalidate();
  assert.equal(WF.send.cacheSize(), 0);
});

// ─── S33: path parameters ──

test("a path parameter that is `.` or `..` is not requested", async () => {
  const asked = [];
  const { WF } = load({ fetchImpl: async (url) => { asked.push(url); return response(null, { status: 204 }); } });
  const B = WF.api({ name: "B", base: "/api", endpoints: { removeUser: { method: "DELETE", path: "users/:id" } } });
  for (const id of ["..", "."]) {
    await assert.rejects(B.removeUser({ id }), (e) => e.kind === "aborted", `refused: ${id}`);
  }
  await B.removeUser({ id: "a.b" });
  await B.removeUser({ id: "..a" });
  await B.removeUser({ id: "a/../b" });
  // An escaped dot is escaped again, so it names itself, not the parent.
  await B.removeUser({ id: "%2e%2e" });
  assert.deepEqual(asked, ["/api/users/a.b", "/api/users/..a", "/api/users/a%2F..%2Fb", "/api/users/%252e%252e"]);
});

// ─── M10, M16: connections ──

class FakeSocket {
  static made = [];
  constructor(url) {
    this.url = url;
    this.readyState = 0;
    this.sent = [];
    FakeSocket.made.push(this);
  }
  send(t) { this.sent.push(t); }
  close(code, reason) {
    this.readyState = 3;
    const fire = this.onclose;
    globalThis.setTimeout(() => fire && fire({ code, reason }), 0);
  }
  open() { this.readyState = 1; this.onopen && this.onopen(); }
  message(data) { this.onmessage && this.onmessage({ data }); }
}

test("a socket keeps the last 500 messages by default, and `keep:` sets how many", () => {
  FakeSocket.made = [];
  const { WF } = load({ WebSocket: FakeSocket });
  const chat = WF.ws("wss://x/");
  const s = FakeSocket.made[0];
  s.open();
  for (let i = 0; i < 1200; i++) s.message(JSON.stringify({ i }));
  assert.equal(chat.messages().length, 500);
  assert.equal(chat.messages()[0].i, 700, "the oldest go first");
  assert.equal(chat.messages()[499].i, 1199);
  assert.equal(chat.last().i, 1199);
  const few = WF.ws("wss://y/", { keep: 3 });
  const t = FakeSocket.made[1];
  t.open();
  for (let i = 0; i < 10; i++) t.message(String(i));
  assert.deepEqual(few.messages(), [7, 8, 9]);
});

test("a stream's messages are bounded too", () => {
  const made = [];
  class FakeSource { constructor() { made.push(this); } addEventListener() {} close() {} }
  const { WF } = load({ EventSource: FakeSource });
  const ticks = WF.sse("/events", { keep: 10 });
  for (let i = 0; i < 25; i++) made[0].onmessage({ data: String(i), lastEventId: "" });
  assert.equal(ticks.messages().length, 10);
  assert.equal(ticks.messages()[9].data, 24);
  const plenty = WF.sse("/more");
  for (let i = 0; i < 600; i++) made[1].onmessage({ data: String(i), lastEventId: "" });
  assert.equal(plenty.messages().length, 500);
});

test("a socket closed while it was still connecting lets go of its handlers", async () => {
  FakeSocket.made = [];
  const { WF } = load({ WebSocket: FakeSocket });
  const [chat, dispose] = WF.scoped(() => WF.ws("wss://x/", { onMessage: () => {} }));
  const s = FakeSocket.made[0];
  assert.equal(s.readyState, 0);
  dispose();
  assert.equal(s.onopen, null);
  assert.equal(s.onmessage, null);
  assert.equal(s.onerror, null);
  assert.equal(s.onclose, null);
  assert.equal(chat.state(), "closed");
  assert.deepEqual(chat.closure(), { code: 1000, reason: "scope left" });
  await settle();
  assert.equal(FakeSocket.made.length, 1, "it did not reconnect");
});
