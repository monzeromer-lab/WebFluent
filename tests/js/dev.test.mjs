//! What the runtime does about a mistake (`spec/DIAGNOSTICS_PLAN.md`, Part
//! E): one list item that throws, a route nothing answers, an effect that
//! feeds itself, a response of the wrong shape, an empty path parameter —
//! shown on the page under `wf serve`, and kept from breaking the rest of
//! it everywhere.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

/// The runtime, under `wf serve` when `dev`, with a `fetch` the test writes.
function load({ dev = false, fetchImpl = async () => { throw new Error("no fetch"); } } = {}) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  if (dev) window.__WF_DEV__ = true;
  const setTimeout = (fn) => globalThis.setTimeout(fn, 0);
  const clearTimeout = (id) => globalThis.clearTimeout(id);
  const said = { warn: [], error: [] };
  const console = {
    log() {},
    warn: (...a) => said.warn.push(a.join(" ")),
    error: (...a) => said.error.push(a.map(String).join(" ")),
  };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "fetch", "navigator", "AbortController", "console",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(
    window, document, Node, Element, DocumentFragment, setTimeout, clearTimeout,
    fetchImpl, { onLine: true }, globalThis.AbortController, console,
  );
  return { WF, document, said };
}

const settle = () => new Promise((done) => globalThis.setTimeout(done, 0));

function response(body) {
  return {
    ok: true,
    status: 200,
    headers: { forEach() {} },
    text: async () => JSON.stringify(body),
  };
}

for (const dev of [true, false]) {
  test(`one item that throws leaves the rest of the list (${dev ? "dev" : "deployed"})`, () => {
    const { WF, document, said } = load({ dev });
    const parent = document.createElement("ul");
    const items = WF.signal([{ t: "a" }, null, { t: "c" }]);
    WF.each(parent, () => items(), (item) => WF.el("li", {}, [item.t]));
    const shown = parent.children.map((n) => n.textContent);
    assert.equal(shown[0], "a");
    assert.equal(shown[shown.length - 1], "c");
    if (dev) assert.match(shown[1], /Item 1 could not be drawn/);
    else assert.equal(parent.children.length, 2);
    assert.equal(said.error.length, 1, said.error.join("\n"));
  });
}

test("a route nothing answers says so under wf serve", () => {
  const { WF, document, said } = load({ dev: true });
  const container = document.createElement("main");
  WF.router([{ path: "/", render: () => WF.el("p", {}, ["home"]) }], container);
  WF.navigate("/nowhere");
  assert.match(container.textContent, /404 — no page has the route \/nowhere/);
  assert.ok(said.warn.some((w) => w.includes("/nowhere")), said.warn.join("\n"));
});

test("an effect that feeds itself is stopped, and said to", () => {
  const { WF, said } = load({ dev: true });
  const n = WF.signal(0);
  let runs = 0;
  WF.effect(() => { runs++; n.set(n() + 1); });
  assert.ok(runs <= 101, `${runs} runs`);
  assert.ok(said.warn.some((w) => w.includes("never settles")), said.warn.join("\n"));
});

test("a response is held to its declared shape under wf serve", async () => {
  const fetchImpl = async () => response([{ id: "1", name: "Ada" }, { id: "2" }]);
  const shape = ["l", ["r", "User", { id: "s", name: "s" }]];
  const { WF } = load({ dev: true, fetchImpl });
  const error = await WF.send("/api/users", { shape }).then(() => null, (e) => e);
  assert.ok(error, "the wrong shape is an error");
  assert.equal(error.kind, "parse");
  assert.match(error.message, /\$\[1\]\.name is missing/);
  // A deployed page trusts the server.
  const { WF: deployed } = load({ fetchImpl });
  assert.equal((await deployed.send("/api/users", { shape })).length, 2);
});

test("an empty path parameter does not ask for the collection", async () => {
  const asked = [];
  const fetchImpl = async (url) => { asked.push(String(url)); return response({ id: "1" }); };
  const { WF } = load({ fetchImpl });
  const Backend = WF.api({ name: "Backend", base: "/api", endpoints: { user: { method: "GET", path: "users/:id" } } });
  const error = await Backend.user({ id: "" }).then(() => null, (e) => e);
  assert.equal(error.kind, "aborted");
  assert.deepEqual(asked, []);
  const r = WF.resource(Backend.user, { call: true, args: () => ({ id: "" }) });
  await settle();
  assert.equal(r.state(), "loading", "a resource over it stays loading");
  assert.deepEqual(asked, []);
  await Backend.user({ id: "7" });
  assert.deepEqual(asked, ["/api/users/7"]);
});
