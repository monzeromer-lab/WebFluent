//! Where a page may send its reader, what the router takes from a `?p=`
//! address, how a translation splices a value, and how a kept value is
//! named and brought forward.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load({ origin = "https://site.example", path = "/", search = "" } = {}) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  window.location.pathname = path;
  window.location.search = search;
  window.location.origin = origin;
  Object.defineProperty(window.location, "href", {
    configurable: true,
    get() { return origin + this.pathname + this.search + this.hash; },
    set(v) { window.__assigned = v; },
  });
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", `${fullRuntime()}\nreturn WF;`);
  const WF = fn(window, document, Node, Element, DocumentFragment);
  return { WF, window, document };
}

// ─── S13: navigate() in a static build ──

test("navigate() in a static build goes to the site's own pages", () => {
  const { WF, window } = load();
  WF.setSsgMode(true);
  WF.navigate("/next");
  assert.equal(window.__assigned, "/next");
  WF.navigate("/a?b=1#c");
  assert.equal(window.__assigned, "/a?b=1#c");
});

test("navigate() in a static build refuses to leave for another origin", () => {
  for (const to of ["//evil.example", "https://evil.example/x", "/\\evil.example", "\\\\evil.example", " //evil.example", "http://site.example.evil.example/"]) {
    const { WF, window } = load();
    WF.setSsgMode(true);
    WF.navigate(to);
    assert.equal(window.__assigned, undefined, `stayed on the site for ${JSON.stringify(to)}`);
  }
});

test("navigate() with a base path stays under it, and an absolute address of the site itself is allowed", () => {
  const { WF, window } = load();
  WF.setSsgMode(true);
  WF.setBasePath("/docs");
  WF.navigate("/guide");
  assert.equal(window.__assigned, "/docs/guide");
  const { WF: W2, window: w2 } = load();
  W2.setSsgMode(true);
  W2.navigate("https://site.example/here");
  assert.equal(w2.__assigned, "https://site.example/here");
});

test("navigate() through the router refuses another origin instead of throwing", () => {
  const { WF, window, document } = load();
  const container = document.createElement("main");
  document.body.appendChild(container);
  WF.router([{ path: "/", render: () => WF.el("p", {}, ["home"]) }, { path: "/b", render: () => WF.el("p", {}, ["b"]) }], container);
  WF.navigate("//evil.example/");
  assert.equal(window.location.pathname, "/");
  WF.navigate("/b");
  assert.equal(window.location.pathname, "/b");
});

// ─── S28: ?p= from a 404 page ──

test("a ?p= that is not a path of this site is ignored, and the router still starts", () => {
  for (const p of ["//evil.example", "/\\evil.example", "https://evil.example", "evil", "/%0d"]) {
    const { WF, window, document } = load({ search: "?p=" + encodeURIComponent(p) });
    const container = document.createElement("main");
    document.body.appendChild(container);
    let replaced = null;
    window.history.replaceState = (_s, _t, url) => {
      if (/^\/\/|^\/\\|^[a-z]+:/i.test(String(url))) throw new Error("SecurityError");
      replaced = url;
      window._setUrl(url);
    };
    WF.router([{ path: "/", render: () => WF.el("p", {}, ["home"]) }, { path: "*", render: () => WF.el("p", {}, ["missing"]) }], container);
    assert.ok(container.textContent.includes("home") || container.textContent.includes("missing"), `drew a page for ${p}`);
    if (p !== "/%0d") assert.equal(replaced, null, `${p} was not taken as the address`);
  }
});

test("a ?p= that is a path of this site is still taken", () => {
  const { WF, window, document } = load({ search: "?p=" + encodeURIComponent("/b?x=1") });
  const container = document.createElement("main");
  document.body.appendChild(container);
  WF.router([{ path: "/", render: () => WF.el("p", {}, ["home"]) }, { path: "/b", render: () => WF.el("p", {}, ["bee"]) }], container);
  assert.equal(window.location.pathname, "/b");
  assert.ok(container.textContent.includes("bee"));
});

// ─── S32: translations splice values as written ──

test("a value spliced into a translation is put in as it is, `$` patterns and all", () => {
  const { WF } = load();
  const i18n = WF.locales("en", { en: { greeting: "Hello, {name}!", two: "{a} and {a}" } });
  assert.equal(i18n.t("greeting", { name: "$&" }), "Hello, $&!");
  assert.equal(i18n.t("greeting", { name: "$`$'$$" }), "Hello, $`$'$$!");
  assert.equal(i18n.t("greeting", { name: "Ada" }), "Hello, Ada!");
  assert.equal(i18n.t("two", { a: "x" }), "x and x");
});

// ─── S31: a stored envelope whose version is not a number ──

test("a stored value whose version is not a number is not taken unmigrated", () => {
  for (const v of ['"2"', "null", '"x"', "{}", "2.5", "-1", "0", "true"]) {
    const { WF, window } = load();
    window.localStorage.setItem("wf:items", `{"wf:v":${v},"wf:d":"old"}`);
    const s = WF.persist("items", "fresh", { version: 3, migrate: { 2: (x) => x + "2", 3: (x) => x + "3" } });
    assert.equal(s(), "fresh", `version ${v}`);
  }
  // A real version is still brought forward.
  const { WF, window } = load();
  window.localStorage.setItem("wf:items", `{"wf:v":1,"wf:d":"old"}`);
  const s = WF.persist("items", "fresh", { version: 3, migrate: { 2: (x) => x + "2", 3: (x) => x + "3" } });
  assert.equal(s(), "old23");
});

// ─── S26: a site under a base path keeps its own values ──

test("a site under a base path keeps its values under a name of its own", () => {
  const { WF, window } = load();
  WF.setBasePath("/shop");
  const s = WF.persist("cart", []);
  s.set([1]);
  assert.equal(window.localStorage.getItem("wf:/shop:cart"), "[1]");
  assert.equal(window.localStorage.getItem("wf:cart"), null, "the origin's other site is not touched");
  // A site at the root keeps the names it always had.
  const { WF: W2, window: w2 } = load();
  const t = W2.persist("cart", []);
  t.set([2]);
  assert.equal(w2.localStorage.getItem("wf:cart"), "[2]");
});

test("a value kept under the old name is read once, and kept under the new one", () => {
  const { WF, window } = load();
  window.localStorage.setItem("wf:cart", "[7]");
  WF.setBasePath("/shop");
  const s = WF.persist("cart", []);
  assert.deepEqual(s(), [7], "what a returning reader had is still there");
  assert.equal(window.localStorage.getItem("wf:/shop:cart"), "[7]");
  // From here the new name is the one read.
  window.localStorage.setItem("wf:cart", "[9]");
  const again = WF.persist("cart", []);
  assert.deepEqual(again(), [7]);
});

test("a keyed value under a base path is named by both", () => {
  const { WF, window } = load();
  WF.setBasePath("/shop");
  const id = WF.signal("p1");
  const s = WF.persist("open", false, { key: () => id() });
  s.set(true);
  assert.equal(window.localStorage.getItem("wf:/shop:open:p1"), "true");
});
