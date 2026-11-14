//! The built-ins against the keyboard and direction rules every widget is
//! held to: a key with a modifier is the browser's (Alt+ArrowLeft goes
//! back), the arrow keys follow the page's direction, and a live region is
//! in the document before the first message it carries. What needs a real
//! layout — the top layer, a backdrop, a switch's input — is
//! `tests/browser/standards.mjs`.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load({ rtl = false } = {}) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  if (rtl) document.documentElement.setAttribute("dir", "rtl");
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", "setTimeout", `${fullRuntime()}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment, setTimeout), document };
}

const key = (k, mods = {}) => ({ type: "keydown", key: k, preventDefault() { this.prevented = true; }, ...mods });

function tabStrip(WF) {
  const nav = WF.el("div", { role: "tablist" }, [WF.el("button", {}, ["One"]), WF.el("button", {}, ["Two"]), WF.el("button", {}, ["Three"])]);
  const active = WF.signal(0);
  WF.tabs(nav, active);
  return { nav, active };
}

test("tabs leave a key held with a modifier to the browser", () => {
  const { WF } = load();
  const { nav, active } = tabStrip(WF);
  const back = key("ArrowLeft", { altKey: true });
  nav.dispatchEvent(back);
  assert.equal(active(), 0, "Alt+ArrowLeft is the browser's back, not a move");
  assert.ok(!back.prevented, "and is not taken from it");
  nav.dispatchEvent(key("ArrowRight", { ctrlKey: true }));
  assert.equal(active(), 0);
  nav.dispatchEvent(key("ArrowRight"));
  assert.equal(active(), 1, "a bare arrow still moves");
});

test("across a right-to-left page the next tab is to the left", () => {
  const { WF } = load({ rtl: true });
  const { nav, active } = tabStrip(WF);
  nav.dispatchEvent(key("ArrowLeft"));
  assert.equal(active(), 1, "ArrowLeft goes forward");
  nav.dispatchEvent(key("ArrowRight"));
  assert.equal(active(), 0, "ArrowRight goes back");
  nav.dispatchEvent(key("ArrowDown"));
  assert.equal(active(), 1, "up and down do not turn");
});

function carousel(WF, document) {
  const track = WF.el("div", { className: "wf-carousel__track" }, [
    WF.el("div", { className: "wf-carousel__slide" }, ["one"]),
    WF.el("div", { className: "wf-carousel__slide" }, ["two"]),
  ]);
  const root = WF.el("div", { className: "wf-carousel" }, [track]);
  document.body.appendChild(root);
  return { api: WF.carousel(root, {}), track };
}

test("a carousel's track moves toward the next slide, which is to the left in a right-to-left page", () => {
  {
    const { WF, document } = load();
    const { api, track } = carousel(WF, document);
    api.show(1);
    assert.equal(track.style.transform, "translateX(-100%)");
  }
  const { WF, document } = load({ rtl: true });
  const { api, track } = carousel(WF, document);
  api.show(1);
  assert.equal(track.style.transform, "translateX(100%)", "the track moves right");
});

test("a splitter's arrow keys follow the page's direction, and leave modifiers alone", () => {
  for (const [rtl, grows] of [[false, "ArrowRight"], [true, "ArrowLeft"]]) {
    const { WF } = load({ rtl });
    const width = WF.signal(200);
    const handle = WF.el("div", {});
    WF.splitter(handle, () => width(), (v) => width.set(v), {});
    handle.dispatchEvent(key(grows));
    assert.equal(width(), 216, `${grows} widens the region before it ${rtl ? "(right to left)" : ""}`);
    handle.dispatchEvent(key(grows, { altKey: true }));
    assert.equal(width(), 216, "Alt with it is the browser's");
  }
});

test("the live region is in the page before the first message, and says each message as a line of its own", () => {
  const { WF, document } = load();
  const region = document.body.querySelector('[role="status"]');
  assert.ok(region, "made when the runtime loads, not on the first message");
  WF.announce("Saved");
  WF.announce("Saved");
  assert.equal(region.textContent, "Saved");
  assert.equal(region.childNodes.length, 1, "the earlier line is cleared as the next is added");
});

test("a store's async action is pending until every call of it has ended", async () => {
  const { WF } = load();
  const resolvers = [];
  const Api = WF.store("Api", () => ({
    actions: { sync: async () => { await new Promise((r) => resolvers.push(r)); } },
  }));
  const first = Api.sync();
  const second = Api.sync();
  assert.equal(Api.sync.pending(), true);
  resolvers[0]();
  await first;
  assert.equal(Api.sync.pending(), true, "the second call still runs");
  resolvers[1]();
  await second;
  assert.equal(Api.sync.pending(), false);
});
