//! The transport: the cache a `GET` is held in, as RFC 9111 describes a
//! private cache — `max-age`, `no-store`, `no-cache`, `stale-while-
//! revalidate`, `Vary`, revalidation by `ETag` — and nothing kept without
//! caching headers; a service's middleware, in order, around each request;
//! an `idempotent` endpoint's key, the same on every try; and the default
//! timeout. Middleware against a real server, and three `401`s that share
//! one refresh, are `tests/browser/transport.mjs`.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

/// The runtime with a `fetch` that answers from `answer(url, init, n)`
/// and records every request; a clock the test moves.
function load(answer) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const asked = [];
  const clock = { now: 1_000_000 };
  const RealDate = globalThis.Date;
  class FakeDate extends RealDate {
    static now() { return clock.now; }
  }
  const fetchImpl = async (url, init) => {
    asked.push({ url: String(url), init });
    return answer(String(url), init, asked.length);
  };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "fetch", "navigator", "AbortController", "console", "Date",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(
    window, document, Node, Element, DocumentFragment,
    (f, ms) => globalThis.setTimeout(f, Math.min(ms || 0, 5)), globalThis.clearTimeout,
    fetchImpl, { onLine: true }, globalThis.AbortController,
    { log() {}, warn() {}, error() {} }, FakeDate,
  );
  return { WF, asked, clock };
}

/// An answer with headers.
function reply(body, headers = {}, status = 200) {
  const lower = Object.fromEntries(Object.entries(headers).map(([k, v]) => [k.toLowerCase(), v]));
  return {
    ok: status >= 200 && status < 300,
    status,
    headers: { forEach(f) { for (const [k, v] of Object.entries(lower)) f(v, k); } },
    text: async () => (status === 304 ? "" : JSON.stringify(body)),
  };
}

const settle = () => new Promise((done) => globalThis.setTimeout(done, 10));

test("with no caching headers nothing is kept", async () => {
  const { WF, asked } = load(() => reply({ n: 1 }));
  await WF.send("/a", {});
  await WF.send("/a", {});
  assert.equal(asked.length, 2);
});

test("max-age keeps an answer fresh for as long as it says", async () => {
  const { WF, asked, clock } = load((url, init, n) => reply({ n }, { "Cache-Control": "max-age=60" }));
  assert.deepEqual(await WF.send("/a", {}), { n: 1 });
  clock.now += 30_000;
  assert.deepEqual(await WF.send("/a", {}), { n: 1 }, "fresh: served from the cache");
  assert.equal(asked.length, 1);
  clock.now += 31_000;
  assert.deepEqual(await WF.send("/a", {}), { n: 2 }, "stale: asked again");
  assert.equal(asked.length, 2);
});

test("an Age the server sends is taken off the freshness", async () => {
  const { WF, asked, clock } = load(() => reply({}, { "Cache-Control": "max-age=60", Age: "50" }));
  await WF.send("/a", {});
  clock.now += 20_000;
  await WF.send("/a", {});
  assert.equal(asked.length, 2, "only ten seconds were left");
});

test("no-store is never kept", async () => {
  const { WF, asked } = load(() => reply({}, { "Cache-Control": "no-store, max-age=60" }));
  await WF.send("/a", {});
  await WF.send("/a", {});
  assert.equal(asked.length, 2);
});

test("no-cache is kept and revalidated before every use", async () => {
  const { WF, asked } = load((url, init, n) =>
    n === 1 ? reply({ v: 1 }, { "Cache-Control": "no-cache", ETag: '"a"' }) : reply(null, { ETag: '"a"' }, 304));
  assert.deepEqual(await WF.send("/a", {}), { v: 1 });
  assert.deepEqual(await WF.send("/a", {}), { v: 1 }, "304: what is held is current");
  assert.equal(asked.length, 2);
  assert.equal(asked[1].init.headers["If-None-Match"], '"a"', "asked with its ETag");
});

test("a stale answer is revalidated with its ETag, and a 304 keeps it", async () => {
  const { WF, asked, clock } = load((url, init, n) =>
    n === 1 ? reply({ v: 1 }, { "Cache-Control": "max-age=10", ETag: '"x"' }) : reply(null, { "Cache-Control": "max-age=10" }, 304));
  await WF.send("/a", {});
  clock.now += 11_000;
  assert.deepEqual(await WF.send("/a", {}), { v: 1 });
  assert.equal(asked[1].init.headers["If-None-Match"], '"x"');
  clock.now += 5_000;
  await WF.send("/a", {});
  assert.equal(asked.length, 2, "the 304 made it fresh again");
});

test("stale-while-revalidate serves what is held and asks behind it", async () => {
  const { WF, asked, clock } = load((url, init, n) =>
    reply({ n }, { "Cache-Control": "max-age=10, stale-while-revalidate=60" }));
  await WF.send("/a", {});
  clock.now += 20_000;
  assert.deepEqual(await WF.send("/a", {}), { n: 1 }, "stale, within the window: served at once");
  await settle();
  assert.equal(asked.length, 2, "and asked again behind it");
  assert.deepEqual(await WF.send("/a", {}), { n: 2 }, "the new answer is the one held");
  clock.now += 200_000;
  assert.deepEqual(await WF.send("/a", {}), { n: 3 }, "past the window: asked before it is served");
});

test("Vary keeps answers to different request headers apart", async () => {
  const { WF, asked } = load((url, init, n) =>
    reply({ lang: init.headers["Accept-Language"], n }, { "Cache-Control": "max-age=60", Vary: "Accept-Language" }));
  await WF.send("/a", { headers: { "Accept-Language": "en" } });
  assert.deepEqual(await WF.send("/a", { headers: { "Accept-Language": "en" } }), { lang: "en", n: 1 });
  assert.equal(asked.length, 1);
  assert.equal((await WF.send("/a", { headers: { "Accept-Language": "ar" } })).lang, "ar");
  assert.equal(asked.length, 2, "another language is another answer");
});

test("Vary: * is never reused", async () => {
  const { WF, asked } = load(() => reply({}, { "Cache-Control": "max-age=60", Vary: "*" }));
  await WF.send("/a", {});
  await WF.send("/a", {});
  assert.equal(asked.length, 2);
});

test("an answer is kept apart by the credentials it was asked with", async () => {
  const { WF, asked } = load((url, init) => reply({ who: init.headers.Authorization }, { "Cache-Control": "private, max-age=60" }));
  await WF.send("/me", { headers: { Authorization: "Bearer a" } });
  assert.equal((await WF.send("/me", { headers: { Authorization: "Bearer b" } })).who, "Bearer b");
  assert.equal(asked.length, 2);
});

test("a contract's override wins over what the server says", async () => {
  const { WF, asked } = load(() => reply({}, { "Cache-Control": "no-store" }));
  await WF.send("/a", { cache: { kind: "maxAge", ttl: 60_000 } });
  await WF.send("/a", { cache: { kind: "maxAge", ttl: 60_000 } });
  assert.equal(asked.length, 1);
});

test("middleware runs in the order written, each around the next", async () => {
  const { WF, asked } = load(() => reply({ ok: true }));
  const order = [];
  const step = (name) => async (req, next) => {
    order.push(`${name} in`);
    const answer = await next({ ...req, headers: { ...req.headers, [`X-${name}`]: "1" } });
    order.push(`${name} out`);
    return answer;
  };
  const Backend = WF.api({ name: "B", base: "/api", middleware: [step("a"), step("b")], endpoints: { ping: { method: "GET", path: "ping" } } });
  assert.deepEqual(await Backend.ping({}), { ok: true });
  assert.deepEqual(order, ["a in", "b in", "b out", "a out"]);
  assert.equal(asked[0].init.headers["X-a"], "1");
  assert.equal(asked[0].init.headers["X-b"], "1");
});

test("refreshOn401 sends the request again through what follows it, bearer reading the new token", async () => {
  let token = "old";
  const { WF, asked } = load((url, init, n) => (n === 1 ? reply({}, {}, 401) : reply({ ok: n })));
  const Backend = WF.api({
    name: "B", base: "/api",
    middleware: [WF.mw.refreshOn401(async () => { token = "new"; }), WF.mw.bearer(() => token)],
    endpoints: { me: { method: "GET", path: "me" } },
  });
  assert.deepEqual(await Backend.me({}), { ok: 2 });
  assert.equal(asked[0].init.headers.Authorization, "Bearer old");
  assert.equal(asked[1].init.headers.Authorization, "Bearer new", "the retry carries the refreshed token");
});

test("a middleware may answer for the network, and a failure it returns is the call's", async () => {
  const { WF, asked } = load(() => reply({}));
  const offline = async () => ["err", ["offline"]];
  const Backend = WF.api({ name: "B", base: "/api", middleware: [offline], endpoints: { me: { method: "GET", path: "me" } } });
  const error = await Backend.me({}).then(() => null, (e) => e);
  assert.equal(error && error.kind, "offline");
  assert.equal(asked.length, 0, "nothing was sent");
});

test("an idempotent write carries one key on every try, and is retried", async () => {
  const { WF, asked } = load((url, init, n) => (n < 3 ? reply({}, {}, 503) : reply({ made: true })));
  const Backend = WF.api({
    name: "B", base: "/api", retry: { times: 3, delay: 1 },
    endpoints: {
      create: { method: "POST", path: "orders", idempotent: true },
      plain: { method: "POST", path: "notes" },
    },
  });
  assert.deepEqual(await Backend.create({ body: { n: 1 } }), { made: true });
  assert.equal(asked.length, 3, "retried, since it is safe to");
  const keys = asked.map((a) => a.init.headers["Idempotency-Key"]);
  assert.ok(keys[0] && keys.every((k) => k === keys[0]), "one key for the call");
  const again = await Backend.create({ body: { n: 2 } });
  assert.ok(again);
  assert.notEqual(asked[3].init.headers["Idempotency-Key"], keys[0], "another call, another key");
});

test("a write that is not idempotent is not retried", async () => {
  const { WF, asked } = load(() => reply({}, {}, 503));
  const Backend = WF.api({ name: "B", base: "/api", retry: { times: 3, delay: 1 }, endpoints: { plain: { method: "POST", path: "notes" } } });
  await Backend.plain({ body: {} }).then(() => null, (e) => e);
  assert.equal(asked.length, 1);
  assert.equal(asked[0].init.headers["Idempotency-Key"], undefined);
});
