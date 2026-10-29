//! The runtime's helpers on lists and strings — `first`, `take`,
//! `truncate` … — and the file `preview`: each works on what it is for, and
//! leaves a receiver that declares a method of its name to answer for
//! itself, with every argument the call had. `OpenFiles.preview(path)` used
//! to reach the store's action with no arguments at all.
//!
//! Run: node --test tests/js/helpers.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function loadRuntime() {
  const dom = makeDom();
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    `${fullRuntime()}\nreturn WF;`,
  );
  return fn(dom.window, dom.document, dom.Node, dom.Element, dom.DocumentFragment, setTimeout);
}

test("the helpers still work on lists and strings", () => {
  const WF = loadRuntime();
  assert.equal(WF.first(["a", "b"]), "a");
  assert.equal(WF.last(["a", "b"]), "b");
  assert.equal(WF.first([]), null);
  assert.deepEqual(WF.take([1, 2, 3], 2), [1, 2]);
  assert.deepEqual(WF.unique([1, 1, 2]), [1, 2]);
  assert.deepEqual(WF.sortBy([{ n: 2 }, { n: 1 }], (x) => x.n), [{ n: 1 }, { n: 2 }]);
  assert.deepEqual(WF.groupBy(["a", "bb"], (x) => x.length), { 1: ["a"], 2: ["bb"] });
  assert.equal(WF.truncate("hello", 2), "he…");
  assert.equal(WF.capitalize("hi"), "Hi");
  assert.deepEqual(WF.lines("a\r\nb"), ["a", "b"]);
  assert.deepEqual(WF.words(" a  b "), ["a", "b"]);
  assert.equal(WF.dedent("  a\n  b"), "a\nb");
});

test("a receiver that declares the method answers for itself, with every argument", () => {
  const WF = loadRuntime();
  const seen = [];
  const own = {};
  for (const name of [
    "first", "last", "take", "unique", "sortBy", "groupBy",
    "truncate", "capitalize", "lines", "words", "dedent", "preview",
  ]) {
    own[name] = (...args) => { seen.push([name, ...args]); return name; };
  }
  assert.equal(WF.preview(own, "a.wf"), "preview");
  assert.equal(WF.first(own, 3), "first");
  assert.equal(WF.last(own, "diagnostics"), "last");
  assert.equal(WF.take(own, 2, "more"), "take");
  assert.equal(WF.truncate(own, "t", 4), "truncate");
  assert.equal(WF.unique(own), "unique");
  assert.deepEqual(seen, [
    ["preview", "a.wf"],
    ["first", 3],
    ["last", "diagnostics"],
    ["take", 2, "more"],
    ["truncate", "t", 4],
    ["unique"],
  ]);
});

test("a store's action named like a helper gets its arguments", () => {
  const WF = loadRuntime();
  const OpenFiles = WF.store("OpenFiles", () => ({
    state: { x: "" },
    actions: { preview: (store, path) => { store.x = path; } },
  }));
  // Whether the call reaches it directly, as the compiler now emits, or
  // through the runtime's `preview`, as it used to, the path arrives.
  OpenFiles.preview("a.wf");
  assert.equal(OpenFiles.x, "a.wf");
  WF.preview(OpenFiles, "b.wf");
  assert.equal(OpenFiles.x, "b.wf");
});
