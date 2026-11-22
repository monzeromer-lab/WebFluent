//! A page a test opens answers its endpoints from `window.__wfFakes`, the
//! table a test's fakes compile to: nothing is sent, every call is counted
//! for `expect called`, a fake's `.err` is the endpoint's error as it
//! stands, a write's `invalidates` still asks its reads again, and a call
//! no fake answers fails, naming the endpoint. The fakes against a real
//! page are `wf test tests/fixtures/fakes`.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

/// The runtime on a page whose fakes are `fakes`, with a `fetch` that
/// records anything sent and a console that keeps what it was told.
function load(fakes) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  window.__wfFakes = fakes;
  const sent = [];
  const errors = [];
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "fetch", "navigator", "AbortController", "console",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(
    window, document, Node, Element, DocumentFragment,
    (f, ms) => globalThis.setTimeout(f, Math.min(ms || 0, 5)), globalThis.clearTimeout,
    async (url) => { sent.push(String(url)); throw new Error("sent"); },
    { onLine: true }, globalThis.AbortController,
    { log() {}, warn() {}, error(m) { errors.push(String(m)); } },
  );
  return { WF, window, sent, errors };
}

const service = (WF) => {
  const Backend = WF.api({ name: "Backend", base: "/api", endpoints: {} });
  return WF.group(Backend, "users", {
    endpoints: {
      list: { method: "GET", path: "users", result: {} },
      create: {
        method: "POST", path: "users",
        result: { enum: true, cases: { 422: "unprocessableContent" } },
        invalidates: () => [["list", null]],
      },
      remove: { method: "DELETE", path: "users/{id}", result: { unit: true } },
    },
  });
};

test("a fake's answer is the call's result, and nothing is sent", async () => {
  const { WF, window, sent } = load({
    "Backend.users.list": ($args) => ["ok", [{ id: "1", page: $args.page }]],
  });
  const users = service(WF);
  assert.deepEqual(await users.list({ page: 2 }), ["ok", [{ id: "1", page: 2 }]]);
  assert.deepEqual(sent, []);
  assert.deepEqual(window.__wfCalls, ["Backend.users.list"]);
});

test("a fake's `.err` is the endpoint's error as it stands", async () => {
  const problem = { title: "Invalid" };
  const { WF } = load({
    "Backend.users.create": () => ["err", ["unprocessableContent", problem]],
    "Backend.users.remove": () => ["err", ["status", 500, null]],
  });
  const users = service(WF);
  assert.deepEqual(await users.create({ body: {} }), ["err", ["unprocessableContent", problem]]);
  assert.deepEqual(await users.remove({ id: "1" }), ["err", ["status", 500, null]]);
});

test("a write a fake takes makes its reads stale, as a real one does", async () => {
  const { WF } = load({
    "Backend.users.create": () => ["ok", { id: "2" }],
  });
  const users = service(WF);
  let staled = 0;
  users.list.invalidate = () => { staled += 1; };
  assert.deepEqual(await users.create({ body: {} }), ["ok", { id: "2" }]);
  assert.equal(staled, 1);
});

test("a call no fake answers fails, naming the endpoint", async () => {
  const { WF, sent, errors } = load({});
  const users = service(WF);
  const got = await users.list({});
  assert.equal(got[0], "err");
  assert.equal(got[1][0], "transport");
  assert.match(errors.join("\n"), /No fake answers Backend\.users\.list/);
  assert.deepEqual(sent, [], "and it is not sent to find out");
});

test("a call counted before a navigate that loads another page is still counted after it", async () => {
  // The tab's storage outlives the page; each `load` is a page of its own.
  const kept = new Map();
  globalThis.sessionStorage = {
    getItem: (k) => (kept.has(k) ? kept.get(k) : null),
    setItem: (k, v) => kept.set(k, String(v)),
  };
  try {
    const answers = { "Backend.users.list": () => ["ok", []] };
    const first = load(answers);
    await service(first.WF).list({});
    const second = load(answers);
    assert.equal(second.window.__wfCalls, undefined, "a new page");
    await service(second.WF).list({ page: 2 });
    assert.deepEqual(second.window.__wfCalls, ["Backend.users.list", "Backend.users.list"]);
  } finally {
    delete globalThis.sessionStorage;
  }
});
