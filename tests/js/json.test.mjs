//! `Json` and dictionaries at run time: `T.decode` accepts a value of the
//! type and names where one that is not goes wrong; a JSON value is read by
//! what it is; a dictionary's methods work on a plain object and leave a
//! receiver with a method of its own to answer for itself.
//!
//! Run: node --test tests/js/json.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function loadRuntime() {
  const dom = makeDom();
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment",
    `${fullRuntime()}\nreturn WF;`,
  );
  return fn(dom.window, dom.document, dom.Node, dom.Element, dom.DocumentFragment);
}

// `type User { name: String, age: Number, tags: [String], note: String? = null }`
const USER = ["r", "User", { name: "s", age: "n", tags: ["l", "s"], note: ["?", "s"] }];

test("decode accepts a value of the type, as it is", () => {
  const WF = loadRuntime();
  const ada = { name: "Ada", age: 36, tags: ["math"] };
  assert.deepEqual(WF.decode(USER, ada), ["ok", ada]);
  assert.deepEqual(WF.decode(USER, { ...ada, note: "first" }), ["ok", { ...ada, note: "first" }]);
});

test("decode names the place a value goes wrong, and what is wrong there", () => {
  const WF = loadRuntime();
  assert.deepEqual(WF.decode(USER, { age: 36, tags: [] }), ["err", { at: "$.name", message: "is missing" }]);
  assert.deepEqual(
    WF.decode(USER, { name: "Ada", age: "36", tags: [] }),
    ["err", { at: "$.age", message: "is \"36\", not a Number" }],
  );
  assert.deepEqual(
    WF.decode(["l", USER], [{ name: "Ada", age: 36, tags: [] }, { name: "Bo", age: 1, tags: [3] }]),
    ["err", { at: "$[1].tags[0]", message: "is 3, not a String" }],
  );
  assert.deepEqual(WF.decode(USER, null), ["err", { at: "$", message: "is null" }]);
});

test("a dictionary decodes key by key, and Json takes any JSON", () => {
  const WF = loadRuntime();
  assert.deepEqual(WF.decode(["d", "n"], { a: 1, b: 2 }), ["ok", { a: 1, b: 2 }]);
  assert.deepEqual(WF.decode(["d", "n"], { a: 1, b: "2" }), ["err", { at: "$[\"b\"]", message: "is \"2\", not a Number" }]);
  assert.deepEqual(WF.decode(["d", "n"], [1]), ["err", { at: "$", message: "is a list, not a dictionary" }]);
  assert.deepEqual(WF.decode("j", { a: [1, "x", null, true] }), ["ok", { a: [1, "x", null, true] }]);
  assert.equal(WF.decode("j", { a: () => 1 })[0], "err");
});

test("a JSON value is read by what it is", () => {
  const WF = loadRuntime();
  const cases = [[{}, "object"], [[], "array"], ["x", "string"], [0, "number"], [false, "bool"], [null, "null"]];
  for (const [value, name] of cases) {
    assert.equal(WF.jsonCase(value), name, JSON.stringify(value));
    assert.equal(WF.jsonPayload(value, name), value);
  }
  assert.equal(WF.jsonPayload("x", "number"), null);
  assert.deepEqual(WF.jsonTagged(3), ["number", 3]);
  assert.equal(WF.jsonTagged(null), "null");
});

test("a dictionary's methods work on a plain object", () => {
  const WF = loadRuntime();
  const d = { ada: 3, bo: 5 };
  assert.deepEqual(WF.dictKeys(d), ["ada", "bo"]);
  assert.deepEqual(WF.dictValues(d), [3, 5]);
  assert.deepEqual(WF.dictEntries(d), [{ key: "ada", value: 3 }, { key: "bo", value: 5 }]);
  assert.equal(WF.dictHas(d, "ada"), true);
  assert.equal(WF.dictHas(d, "toString"), false);
  assert.deepEqual(WF.dictWithout(d, "ada"), { bo: 5 });
  assert.deepEqual(WF.urlWith(d, "cy", 1), { ada: 3, bo: 5, cy: 1 });
  assert.deepEqual(d, { ada: 3, bo: 5 }, "nothing changes the dictionary itself");
});

test("a receiver with a method of the name answers for itself", () => {
  const WF = loadRuntime();
  const m = new Map([["a", 1]]);
  assert.equal(WF.dictHas(m, "a"), true);
  assert.deepEqual([...WF.dictKeys(m)], ["a"]);
  assert.equal(WF.urlWith("https://example.com/a", { query: { q: "x" } }), "https://example.com/a?q=x");
});
