//! `persist` with a key that follows state, and a storage the project
//! gives itself (`in: .app` → `window.wfStorage.app`).
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", `${fullRuntime()}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment), window };
}

/// A storage of the project's, as a desktop app's store would be: a map,
/// answering at once or (with `slow`) later, and telling watchers of a
/// write another window made.
function area({ slow = false } = {}) {
  const data = new Map();
  const watchers = new Map();
  const pending = [];
  return {
    data,
    pending,
    get(name) {
      const value = data.has(name) ? data.get(name) : null;
      if (!slow) return value;
      return new Promise((resolve) => pending.push(() => resolve(value)));
    },
    set(name, text) { data.set(name, text); },
    watch(name, fn) {
      watchers.set(name, fn);
      return () => watchers.delete(name);
    },
    // Another window wrote it.
    elsewhere(name, text) {
      data.set(name, text);
      const fn = watchers.get(name);
      if (fn) fn(text);
    },
    answer: async () => { for (const go of pending.splice(0)) go(); await Promise.resolve(); await Promise.resolve(); },
  };
}

test("a key that follows state keeps one value per key", () => {
  const { WF, window } = load();
  const project = WF.signal("halyard");
  const side = WF.persist("Layout.side", 240, { key: () => project() });
  assert.equal(side(), 240);
  side.set(300);
  assert.equal(window.localStorage.getItem("wf:Layout.side:halyard"), "300");
  project.set("docs");
  assert.equal(side(), 240, "a project with nothing kept has the initial value");
  side.set(180);
  project.set("halyard");
  assert.equal(side(), 300, "and each keeps its own");
  assert.equal(window.localStorage.getItem("wf:Layout.side:docs"), "180");
});

test("in: .app keeps it where the project says, and hears another window", () => {
  const { WF, window } = load();
  const app = area();
  window.wfStorage = { app };
  app.data.set("wf:Ui.mode", JSON.stringify("design"));
  const mode = WF.persist("Ui.mode", "code", { in: "app" });
  assert.equal(mode(), "design", "read back from the project's storage");
  mode.set("agent");
  assert.equal(app.data.get("wf:Ui.mode"), "\"agent\"");
  assert.equal(window.localStorage.getItem("wf:Ui.mode"), null, "not the browser's");
  app.elsewhere("wf:Ui.mode", JSON.stringify("code"));
  assert.equal(mode(), "code", "another window's write is taken");
  assert.equal(app.data.get("wf:Ui.mode"), "\"code\"");
});

test("a storage that answers later: the initial value until then, and a write wins", async () => {
  const { WF, window } = load();
  const app = area({ slow: true });
  window.wfStorage = { app };
  app.data.set("wf:Ui.size", "16");
  const size = WF.persist("Ui.size", 14, { in: "app", version: 0 });
  assert.equal(size(), 14, "nothing yet");
  await app.answer();
  assert.equal(size(), 16, "then what was kept");

  app.data.set("wf:Ui.zoom", "2");
  const zoom = WF.persist("Ui.zoom", 1, { in: "app" });
  zoom.set(3);
  await app.answer();
  assert.equal(zoom(), 3, "a write made before the answer is not undone by it");
});

test("a key that moves on before the answer does not take the old key's", async () => {
  const { WF, window } = load();
  const app = area({ slow: true });
  window.wfStorage = { app };
  app.data.set("wf:L.side:a", "300");
  app.data.set("wf:L.side:b", "200");
  const project = WF.signal("a");
  const side = WF.persist("L.side", 240, { in: "app", key: () => project() });
  project.set("b");
  await app.answer();
  assert.equal(side(), 200, "b's, not a's");
});

test("a storage the project has not given falls back to localStorage", () => {
  const { WF, window } = load();
  const kept = WF.persist("Ui.theme", "light", { in: "app" });
  kept.set("dark");
  assert.equal(window.localStorage.getItem("wf:Ui.theme"), "\"dark\"");
});

test("a store's key may read the store, and another store", () => {
  const { WF, window } = load();
  const Project = WF.store("Project", () => ({ state: { id: "halyard" } }));
  const Layout = WF.store("Layout", () => ({
    state: { side: 240, mode: "code" },
    persist: { side: { key: () => Project.id }, mode: { key: (store) => store.side } },
  }));
  Layout.side = 320;
  assert.equal(window.localStorage.getItem("wf:Layout.side:halyard"), "320");
  Project.id = "docs";
  assert.equal(Layout.side, 240, "another project, another value");
  Layout.mode = "design";
  assert.equal(window.localStorage.getItem("wf:Layout.mode:240"), "\"design\"");
});

test("a store's key may read a state declared after it", () => {
  const { WF, window } = load();
  const Layout = WF.store("Layout", () => ({
    state: { side: 240, project: "halyard" },
    persist: { side: { key: (store) => store.project } },
  }));
  Layout.side = 300;
  assert.equal(window.localStorage.getItem("wf:Layout.side:halyard"), "300");
  Layout.project = "docs";
  assert.equal(Layout.side, 240, "it follows the later state");
});

test("no key yet is nowhere: the value lives in memory, and nothing is written", () => {
  const { WF, window } = load();
  const project = WF.signal(null);
  const side = WF.persist("Layout.side", 240, { key: () => project() });
  side.set(300);
  assert.equal(side(), 300, "kept in memory");
  assert.equal(window.localStorage.length ?? Object.keys(window.localStorage).length, 0, "and not under `null`");
  assert.equal(window.localStorage.getItem("wf:Layout.side:null"), null);
  project.set("halyard");
  assert.equal(side(), 240, "a project opened has its own");
});
