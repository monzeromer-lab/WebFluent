//! Reaching other people's code, and being reachable from it.
//!
//! `Host` is the node with a lifetime — the `ref` + `effect` + `cleanup`
//! people write by hand, with the cleanup impossible to forget. A
//! published custom element is the other direction: the shared interface
//! between frameworks is the platform, so what React, Vue, Svelte and a
//! plain page all understand is a tag.
//!
//! Run: node --test tests/js/interop.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync, mkdirSync, readFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function loadRuntime() {
  const dom = makeDom();
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    `${fullRuntime()}\nreturn WF;`,
  );
  return { WF: fn(dom.window, dom.document, dom.Node, dom.Element, dom.DocumentFragment, setTimeout), ...dom };
}

test("a Host hands the node over once, keeps it in step, and gives it back", () => {
  const { WF, document } = loadRuntime();
  const log = [];
  const node = document.createElement("canvas");
  const data = WF.signal(1);

  const [, dispose] = WF.scoped(() => {
    WF.attach(
      node,
      (el) => { log.push(["mount", el.tagName]); return { id: 7 }; },
      (handle) => { log.push(["update", handle.id, data()]); },
      (handle) => { log.push(["cleanup", handle.id]); },
    );
  });
  assert.deepEqual(log, [["mount", "CANVAS"], ["update", 7, 1]]);

  // The update runs again on the state it read, and the mount does not.
  data.set(2);
  assert.deepEqual(log.at(-1), ["update", 7, 2]);
  assert.equal(log.filter(([what]) => what === "mount").length, 1);

  // And the cleanup is the scope's, so it cannot be forgotten.
  dispose();
  assert.deepEqual(log.at(-1), ["cleanup", 7]);
  data.set(3);
  assert.equal(log.at(-1)[0], "cleanup", "nothing runs after it has gone");
});

test("a library that throws takes itself out; the page stays", () => {
  const { WF, document } = loadRuntime();
  const errors = [];
  const before = console.error;
  console.error = (...a) => errors.push(a[0]);
  try {
    const handle = WF.attach(document.createElement("div"), () => { throw new Error("no"); });
    assert.equal(handle, undefined);
  } finally {
    console.error = before;
  }
  assert.ok(errors.some((m) => String(m).includes("mount failed")));
});

test("an attribute holding JSON is the prop it stands for", () => {
  const { WF } = loadRuntime();
  assert.deepEqual(WF.jsonAttr('{"count":3}'), { count: 3 });
  assert.deepEqual(WF.jsonAttr("[1,2]"), [1, 2]);
  assert.equal(WF.jsonAttr(null), undefined);
  // Text that is not JSON is text, which is what a page that wrote a
  // plain string meant.
  assert.equal(WF.jsonAttr("plain"), "plain");
});

/// Build a project of custom elements with the compiler, and run what it
/// wrote against the fake DOM — the only honest way to test the output is
/// to use it.
function buildElements() {
  const wf = ["target/release/wf", "target/debug/wf"].find((p) => existsSync(p));
  if (!wf) return null;
  const dir = mkdtempSync(join(tmpdir(), "wf-elements-"));
  mkdirSync(join(dir, "src"));
  writeFileSync(
    join(dir, "webfluent.app.json"),
    JSON.stringify({
      name: "Widgets",
      build: { output: "./build", output_type: "elements", elements: ["PriceTag"], minify: false },
    }),
  );
  writeFileSync(
    join(dir, "src/App.wf"),
    `component PriceTag(_ label: String, amount: Number, sale: Bool = false) {
    event picked(label: String)
    Row {
        on click { emit picked(label) }
        Text(label).bold
        Text("{amount}")
        if sale { Badge("Sale").danger }
    }
}
`,
  );
  execFileSync(wf, ["build", "-d", dir], { stdio: "pipe" });
  return readFileSync(join(dir, "build/elements.js"), "utf8");
}

test("a published component is a tag any page can place", () => {
  const bundle = buildElements();
  if (!bundle) return; // no compiler built: the Rust tests cover the rest
  const dom = makeDom();
  const setTimeout = (fn) => { fn(); return 0; };
  new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "HTMLElement", "customElements", "CustomEvent",
    bundle,
  )(
    dom.window, dom.document, dom.Node, dom.Element, dom.DocumentFragment, setTimeout,
    dom.HTMLElement, dom.customElements, dom.CustomEvent,
  );

  assert.ok(dom.customElements.get("price-tag"), "the tag is registered");
  const el = dom.makeCustom("price-tag", { label: "Pro", amount: "29", sale: "" });
  const text = el.textContent;
  assert.ok(text.includes("Pro"), text);
  assert.ok(text.includes("29"), text);
  assert.ok(text.includes("Sale"), "a bare attribute is true, as HTML reads one");

  // An attribute change rebuilds it; `sale="false"` is false, so a
  // framework that writes the string still works.
  el.setAttribute("label", "Team");
  el.setAttribute("sale", "false");
  el.attributeChangedCallback();
  assert.ok(el.textContent.includes("Team"));
  assert.ok(!el.textContent.includes("Sale"));

  // An event the component declares is a DOM event the host page hears.
  let heard = null;
  el.addEventListener("picked", (e) => { heard = e.detail; });
  el.children[0].dispatchEvent({ type: "click" });
  assert.equal(heard, "Team");
});
