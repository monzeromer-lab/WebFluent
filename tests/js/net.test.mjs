//! What the request engine does: the timeout, the retries, the cache, the
//! dedupe, the abort, and the errors a page can actually match on.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

/// The runtime, with a `fetch` the test writes.
function load(fetchImpl, { online = true } = {}) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  // A wait the engine asks for happens at once, so a retry is not a wait.
  const setTimeout = (fn) => globalThis.setTimeout(fn, 0);
  const clearTimeout = (id) => globalThis.clearTimeout(id);
  const navigator = { onLine: online };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "fetch", "navigator", "AbortController", "XMLHttpRequest",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(
    window, document, Node, Element, DocumentFragment, setTimeout, clearTimeout,
    fetchImpl, navigator, globalThis.AbortController, class {},
  );
  return { WF };
}

/// Every microtask the engine has queued, run.
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

test("a request comes back as its body, and a failure as an error a page can match", async () => {
  const { WF } = load(async () => response({ name: "Ada" }));
  assert.deepEqual(await WF.send("/api/user", {}), { name: "Ada" });

  const { WF: failing } = load(async () => response({ messages: ["too short"] }, { status: 422 }));
  const error = await failing.send("/api/user", {}).then(() => null, (e) => e);
  // It reads as an enum — `match e { .status(code, body) }` — and as a record.
  assert.equal(error[0], "status");
  assert.equal(error[1], 422);
  assert.deepEqual(error[2], { messages: ["too short"] });
  assert.equal(error.kind, "status");
  assert.equal(error.status, 422);
  assert.deepEqual(error.body, { messages: ["too short"] }, "the body is decoded, not discarded");
  assert.match(error.message, /422/);
});

test("offline is its own error, not a network failure", async () => {
  const { WF } = load(async () => response({}), { online: false });
  const error = await WF.send("/api/x", {}).then(() => null, (e) => e);
  assert.equal(error.kind, "offline");
  assert.equal(error[0], "offline");
});

test("a failed request is tried again, and the tries stop where the policy says", async () => {
  let calls = 0;
  const { WF } = load(async () => {
    calls += 1;
    return calls < 3 ? response("boom", { status: 503 }) : response({ ok: true });
  });
  assert.deepEqual(await WF.send("/api/x", { retry: { times: 3, delay: 1 } }), { ok: true });
  assert.equal(calls, 3, "two failures, then the answer");

  // A 4xx is not retried: asking again cannot change the answer.
  let asked = 0;
  const { WF: once } = load(async () => { asked += 1; return response("no", { status: 404 }); });
  await once.send("/api/x", { retry: { times: 3, delay: 1 } }).catch(() => {});
  assert.equal(asked, 1);
});

test("two readers of one address make one request, and a cached answer is not asked for again", async () => {
  let calls = 0;
  const { WF } = load(async () => { calls += 1; return response({ n: calls }); });
  const cache = { kind: "swr", ttl: 60000 };
  const [a, b] = await Promise.all([
    WF.send("/api/rows", { cache }),
    WF.send("/api/rows", { cache }),
  ]);
  assert.deepEqual(a, b);
  assert.equal(calls, 1, "the second reader joined the first request");

  assert.deepEqual(await WF.send("/api/rows", { cache }), { n: 1 });
  assert.equal(calls, 1, "and the answer still stands");

  // Until it is dropped.
  WF.invalidate("GET /api/rows ");
  await WF.send("/api/rows", { cache });
  assert.equal(calls, 2);
});

test("what the server says has not changed is served from what is held", async () => {
  let calls = 0;
  const { WF } = load(async (url, init) => {
    calls += 1;
    if (calls === 1) return response({ n: 1 }, { headers: { ETag: "abc" } });
    assert.equal(init.headers["If-None-Match"], "abc", "the tag goes back out");
    return response("", { status: 304 });
  });
  const cache = { kind: "swr", ttl: 0 };
  assert.deepEqual(await WF.send("/api/rows", { cache }), { n: 1 });
  assert.deepEqual(await WF.send("/api/rows", { cache }), { n: 1 });
  assert.equal(calls, 2, "asked again, and told nothing changed");
});

test("a header that reads state is read at the moment of the request", async () => {
  let sent = null;
  const { WF } = load(async (url, init) => { sent = init.headers; return response({}); });
  const token = WF.signal("one");
  await WF.send("/api/x", { headers: { Authorization: () => `Bearer ${token()}` } });
  assert.equal(sent.Authorization, "Bearer one");
  token.set("two");
  await WF.send("/api/x", { headers: { Authorization: () => `Bearer ${token()}` } });
  assert.equal(sent.Authorization, "Bearer two");
});

test("a service is its endpoints, addressed and cached by name", async () => {
  const seen = [];
  const { WF } = load(async (url) => { seen.push(url); return response([{ id: "a" }]); });
  const Backend = WF.api({
    name: "Backend",
    base: "/api/v1",
    headers: { Authorization: () => "Bearer t" },
    endpoints: {
      users: { method: "GET", path: "users" },
      user: { method: "GET", path: "users/:id" },
      createUser: { method: "POST", path: "users" },
    },
  });
  await Backend.users({ page: 2, q: "ada" });
  assert.equal(seen[0], "/api/v1/users?page=2&q=ada", "what is not in the path is the query");
  await Backend.user({ id: "42" });
  assert.equal(seen[1], "/api/v1/users/42", "a path parameter is in the path");
  await Backend.createUser({ body: { name: "Ada" } });
  assert.equal(seen[2], "/api/v1/users");
  assert.equal(typeof Backend.users.invalidate, "function");
  assert.equal(typeof Backend.users.prefetch, "function");
});

test("a resource over an endpoint follows its arguments and abandons what it supersedes", async () => {
  const asked = [];
  const { WF } = load(async (url) => { asked.push(url); return response({ url }); });
  const Backend = WF.api({ name: "B", base: "/api", endpoints: { rows: { method: "GET", path: "rows" } } });
  const page = WF.signal(1);
  const rows = WF.resource(Backend.rows, { call: true, args: () => ({ page: page() }) });
  await settle();
  assert.equal(rows.state(), "ready");
  assert.deepEqual(rows.data(), { url: "/api/rows?page=1" });
  page.set(2);
  await settle();
  assert.deepEqual(asked, ["/api/rows?page=1", "/api/rows?page=2"], "the page changing asks again");
});

test("the network is a value a page can read", () => {
  const { WF } = load(async () => response({}));
  const net = WF.network();
  assert.equal(net.online, true);
  assert.equal(typeof net.effectiveType, "string");
  assert.equal(typeof net.saveData, "boolean");
});
