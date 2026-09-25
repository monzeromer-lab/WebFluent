//! Every runtime module stands up with the modules it declares, and no others.
//!
//! The runtime ships in pieces now: a build carries `each` without `router`,
//! `router` without `carousel`. A module that quietly reached for something
//! outside its declared dependencies used to be invisible — the whole file was
//! always there. This loads each module with exactly its closure and runs it.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { runtimeFor, moduleNames, exportsOf } from "./runtime.mjs";

function run(src) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    `${src}\nreturn WF;`,
  );
  return fn(window, document, Node, Element, DocumentFragment, setTimeout);
}

for (const name of moduleNames) {
  test(`the ${name} module loads with what it declares`, () => {
    const WF = run(runtimeFor(name));
    for (const exported of exportsOf(name)) {
      assert.notEqual(typeof WF[exported], "undefined", `${name} must export ${exported}`);
    }
    // Core is in every build, so its surface is there whatever else is not.
    assert.equal(typeof WF.el, "function");
    assert.equal(typeof WF.signal, "function");
  });
}

test("a module's own work runs without the modules it does not name", () => {
  // `each` with no router, no store, no widgets: a keyed list still renders,
  // and still reconciles. This is the shape of a build that holds a `for` and
  // nothing else — before the split it could only be tested with everything.
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    `${runtimeFor("each")}\nreturn WF;`,
  );
  const WF = fn(window, document, Node, Element, DocumentFragment, (f) => { f(); return 0; });
  assert.equal(typeof WF.router, "undefined", "a list is not a router");
  assert.equal(typeof WF.store, "undefined");

  const items = WF.signal([{ id: "a" }, { id: "b" }]);
  const parent = document.createElement("ul");
  WF.each(parent, () => items(), (item) => WF.el("li", {}, [item.id]), { key: (item) => item.id });
  assert.deepEqual(parent.children.map((li) => li.textContent), ["a", "b"]);
  items.set([{ id: "b" }]);
  assert.deepEqual(parent.children.map((li) => li.textContent), ["b"]);
});
