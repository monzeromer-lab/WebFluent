//! `WF.worker`, against a fake `Worker` that records what it was posted and
//! answers as a test says: requests and their answers, failures, a newer
//! request superseding an older (and the worker told), timeouts, crashes
//! and restarts, the base path, and a store's connection closing with it.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

/// The runtime with a fake `Worker`: every one made is in `made`, with what
/// it was posted, and `reply`/`crash` stand for the worker's side.
function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const made = [];
  class FakeWorker {
    constructor(url, opts) {
      this.url = url;
      this.opts = opts;
      this.posted = [];
      this.terminated = false;
      made.push(this);
    }
    postMessage(value) { this.posted.push(value); }
    terminate() { this.terminated = true; }
    reply(data) { this.onmessage({ data }); }
    crash(message) { this.onerror({ message, preventDefault() {} }); }
  }
  // Timers are real here: a timeout must be able to fire.
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", "Worker", "setTimeout", "clearTimeout",
    `${fullRuntime()}\nreturn WF;`);
  const WF = fn(window, document, Node, Element, DocumentFragment, FakeWorker, setTimeout, clearTimeout);
  return { WF, made };
}

test("a request is posted with an id, and its answer resolves it", async () => {
  const { WF, made } = load();
  const [handle] = WF.scoped(() => WF.worker("/workers/lang.js", {}));
  assert.equal(handle.state(), "open");
  const answer = handle.request("upper", { text: "a" });
  const [w] = made;
  assert.deepEqual(w.posted[0], { id: 1, method: "upper", params: { text: "a" } });
  assert.equal(handle.pending(), 1);
  w.reply({ id: 1, result: "A" });
  assert.equal(await answer, "A");
  assert.equal(handle.pending(), 0);
});

test("a failure the worker posts rejects, with its message and data", async () => {
  const { WF, made } = load();
  const [handle] = WF.scoped(() => WF.worker("/w.js", {}));
  const asked = handle.request("fail", null);
  made[0].reply({ id: 1, error: { message: "no such word", code: 7 } });
  await assert.rejects(asked, (e) => e.message === "no such word" && e.data.code === 7);
});

test("what comes with no id is a message, and the last of each kind is kept", () => {
  const { WF, made } = load();
  const heard = [];
  const [handle] = WF.scoped(() => WF.worker("/w.js", { onMessage: (m) => heard.push(m) }));
  made[0].reply({ type: "diagnostics", items: [1] });
  made[0].reply({ method: "progress", done: 3 });
  made[0].reply({ id: 99, result: "for nobody" });
  assert.equal(heard.length, 3, "an answer to nothing asked is a message too");
  assert.deepEqual(handle.last("diagnostics").items, [1]);
  assert.equal(handle.last("progress").done, 3);
  assert.equal(handle.last().id, 99);
  assert.equal(handle.last("nothing"), null);
});

test("a newer request supersedes the older one of its method, and the worker is told", async () => {
  const { WF, made } = load();
  const [handle] = WF.scoped(() => WF.worker("/w.js", {}));
  const first = handle.request("complete", { at: 1 }, { supersede: true });
  const other = handle.request("hover", { at: 1 }, { supersede: true });
  const second = handle.request("complete", { at: 2 }, { supersede: true });
  await assert.rejects(first, (e) => e.superseded === true);
  assert.deepEqual(made[0].posted.map((p) => p.cancel ?? p.method), ["complete", "hover", 1, "complete"]);
  made[0].reply({ id: 3, result: "two" });
  made[0].reply({ id: 2, result: "hovered" });
  assert.equal(await second, "two");
  assert.equal(await other, "hovered", "another method is left alone");
});

test("a request not answered in time fails, and is no longer pending", async () => {
  const { WF } = load();
  const [handle] = WF.scoped(() => WF.worker("/w.js", { timeout: 20 }));
  await assert.rejects(handle.request("never", null), (e) => e.timeout === true && /`never` in 20 ms/.test(e.message));
  assert.equal(handle.pending(), 0);
});

test("a crash fails what was out and starts it again, as many times as it may", async () => {
  const { WF, made } = load();
  const restarts = [];
  const errors = [];
  const [handle] = WF.scoped(() => WF.worker("/w.js", { restart: 1, onRestart: (e) => restarts.push(e.message), onError: (e) => errors.push(e.message) }));
  const out = handle.request("slow", null);
  made[0].crash("fell over");
  await assert.rejects(out, /fell over/);
  assert.equal(made.length, 2, "started again");
  assert.ok(made[0].terminated);
  assert.deepEqual(restarts, ["fell over"]);
  assert.equal(handle.state(), "open");
  made[1].crash("again");
  assert.equal(made.length, 2, "no restarts left");
  assert.equal(handle.state(), "error");
  assert.equal(handle.error().message, "again");
  assert.deepEqual(errors, ["again"]);
  await assert.rejects(handle.request("upper", null), /not running/);
});

test("restart() starts a new one; close() stops it, fails what was out, and says how", async () => {
  const { WF, made } = load();
  const [handle] = WF.scoped(() => WF.worker("/w.js", {}));
  const out = handle.request("slow", null);
  handle.restart();
  await assert.rejects(out, /restarted/);
  assert.equal(made.length, 2);
  const out2 = handle.request("slow", null);
  handle.close();
  await assert.rejects(out2, /stopped/);
  assert.equal(handle.state(), "closed");
  assert.deepEqual(handle.closure(), { code: 1000, reason: "stopped" });
  assert.ok(made[1].terminated);
});

test("the scope leaving stops it", () => {
  const { WF, made } = load();
  const [, dispose] = WF.scoped(() => WF.worker("/w.js", {}));
  dispose();
  assert.ok(made[0].terminated);
});

test("a script of the site is asked for under the base path; another origin's is not", () => {
  const { WF, made } = load();
  WF.setBasePath("/Halyard");
  WF.scoped(() => WF.worker("/workers/lang.js", {}));
  WF.scoped(() => WF.worker("https://cdn.example/w.js", {}));
  WF.scoped(() => WF.worker("//cdn.example/w.js", {}));
  assert.deepEqual(made.map((w) => w.url), ["/Halyard/workers/lang.js", "https://cdn.example/w.js", "//cdn.example/w.js"]);
  WF.scoped(() => WF.worker("/m.js", { type: "module" }));
  assert.deepEqual(made[3].opts, { type: "module" });
});

test("a store's worker is made with the store, and closed with a route store's route", () => {
  const { WF, made } = load();
  const Lang = WF.store("Lang", () => ({
    state: { said: "" },
    connections: { lang: (store) => WF.worker("/w.js", { onMessage: (m) => { store.said = m.text; } }) },
  }), { scope: "route" });
  assert.equal(made.length, 0, "nothing until the store is read");
  assert.equal(Lang.lang.state(), "open");
  assert.equal(made.length, 1);
  made[0].reply({ text: "hello" });
  assert.equal(Lang.said, "hello", "its handler writes the store");
  WF.dropRouteStores();
  assert.ok(made[0].terminated, "the route left: the worker stopped");
  assert.equal(Lang.lang.state(), "open", "read again, the store is built again, with a new worker");
  assert.equal(made.length, 2);
});
