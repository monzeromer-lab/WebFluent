//! Layers: where a popover, a menu or a tooltip goes against its anchor,
//! the one stack Escape and a press outside close the top of, and what a
//! layer is given beyond its own props, as any element is.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function loadRuntime() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  window.innerWidth = 1000;
  window.innerHeight = 600;
  const src = fullRuntime();
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", "setTimeout", `${src}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment, setTimeout), document, window };
}

const viewport = { width: 1000, height: 600 };
const rect = (left, top, width, height) => ({ left, top, right: left + width, bottom: top + height });

test("a layer sits on its side of the anchor, aligned as asked", () => {
  const { WF } = loadRuntime();
  const anchor = rect(100, 100, 80, 30);
  const size = { width: 200, height: 120 };
  assert.deepEqual(WF.placeAt(anchor, size, viewport, "bottom-start", 6), { x: 100, y: 136, placement: "bottom-start" });
  assert.deepEqual(WF.placeAt(anchor, size, viewport, "bottom-end", 6), { x: 8, y: 136, placement: "bottom-end" }, "slid back on screen");
  assert.deepEqual(WF.placeAt(anchor, size, viewport, "right-start", 6), { x: 186, y: 100, placement: "right-start" });
  assert.deepEqual(WF.placeAt(rect(400, 100, 80, 30), size, viewport, "bottom", 6), { x: 340, y: 136, placement: "bottom-center" });
  assert.deepEqual(WF.placeAt(anchor, size, viewport, "rightStart", 6), { x: 186, y: 100, placement: "right-start" }, "as a `.case` writes it");
});

test("with no room on its side it flips, and stays inside the viewport", () => {
  const { WF } = loadRuntime();
  const size = { width: 200, height: 120 };
  // Near the bottom: below has 600-570-6-8 = 16 px, above has room.
  const low = rect(100, 540, 80, 30);
  assert.deepEqual(WF.placeAt(low, size, viewport, "bottom-start", 6), { x: 100, y: 414, placement: "top-start" });
  // Near the right edge: a right-side layer flips to the left.
  const edge = rect(900, 100, 60, 30);
  assert.equal(WF.placeAt(edge, size, viewport, "right-start", 6).placement, "left-start");
  // At a point — a context menu where the pointer asked for it.
  const point = { left: 990, right: 990, top: 20, bottom: 20 };
  assert.deepEqual(WF.placeAt(point, size, viewport, "bottom-start", 0), { x: 792, y: 20, placement: "bottom-start" });
});

test("Escape closes the newest layer only, and the state that opened it follows", () => {
  const { WF, document } = loadRuntime();
  const button = document.createElement("button");
  document.body.appendChild(button);
  button.focus();
  const outer = document.createElement("div");
  const inner = document.createElement("div");
  const item = document.createElement("li");
  item.setAttribute("role", "menuitem");
  outer.appendChild(item);
  document.body.appendChild(outer);
  document.body.appendChild(inner);
  const first = WF.signal(true);
  const second = WF.signal(true);
  const closed = [];
  WF.layer(outer, first, undefined, { anchor: button, onClose: (why) => closed.push(["outer", why]) });
  assert.equal(outer.hasAttribute("data-open"), true);
  assert.equal(document.activeElement, item, "a menu's first item takes the focus");
  WF.layer(inner, second, undefined, { anchor: button, focus: false, onClose: (why) => closed.push(["inner", why]) });

  document.dispatchEvent({ type: "keydown", key: "Escape", preventDefault() {}, stopPropagation() {} });
  assert.equal(second(), false, "the newest closed");
  assert.equal(first(), true, "the one under it did not");
  assert.deepEqual(closed, [["inner", "escape"]]);
  assert.equal(inner.hasAttribute("data-open"), false);

  document.dispatchEvent({ type: "keydown", key: "Escape", preventDefault() {}, stopPropagation() {} });
  assert.equal(first(), false);
  assert.equal(document.activeElement, button, "focus is back where it was");
});

test("a press outside closes the top layer; a press on its anchor is the anchor's", () => {
  const { WF, document } = loadRuntime();
  const anchor = document.createElement("button");
  const pop = document.createElement("div");
  const elsewhere = document.createElement("p");
  document.body.appendChild(anchor);
  document.body.appendChild(pop);
  document.body.appendChild(elsewhere);
  const open = WF.signal(true);
  WF.layer(pop, open, undefined, { anchor, focus: false });
  document.dispatchEvent({ type: "pointerdown", target: anchor });
  assert.equal(open(), true, "the anchor's own click toggles it");
  document.dispatchEvent({ type: "pointerdown", target: pop });
  assert.equal(open(), true, "inside is inside");
  document.dispatchEvent({ type: "pointerdown", target: elsewhere });
  assert.equal(open(), false);
});

// A key as the browser sends it: the document hears it first, on its way
// down (where the layers listen), then the element with focus, unless the
// document stopped it there.
function press(document, key) {
  const event = {
    type: "keydown",
    key,
    prevented: false,
    stopped: false,
    preventDefault() { this.prevented = true; },
    stopPropagation() { this.stopped = true; },
  };
  document.dispatchEvent(event);
  const at = document.activeElement;
  if (!event.stopped && at) {
    event.target = at;
    at.dispatchEvent(event);
  }
  return event;
}

// A tooltip around a button, shown as a hover shows it.
function hovered(WF, document, words) {
  const button = WF.el("button", {}, [words]);
  const tip = WF.el("span", { className: "wf-tooltip__text", role: "tooltip", id: `tip-${words}` }, [words]);
  const root = WF.el("div", { className: "wf-tooltip" }, [button, tip]);
  document.body.appendChild(root);
  WF.tooltip(root, tip, { delay: 0 });
  root.dispatchEvent({ type: "pointerenter" });
  return { button, tip, root };
}

test("a tooltip on top hides on Escape, and the layer under it that can be dismissed closes", () => {
  const { WF, document } = loadRuntime();
  const input = document.createElement("input");
  const typed = [];
  input.addEventListener("keydown", (e) => typed.push(e.key));
  const anchor = document.createElement("button");
  const pop = document.createElement("div");
  document.body.appendChild(input);
  document.body.appendChild(anchor);
  document.body.appendChild(pop);
  input.focus();
  const open = WF.signal(true);
  const closed = [];
  WF.layer(pop, open, undefined, { anchor, focus: false, onClose: (why) => closed.push(why) });
  const { tip } = hovered(WF, document, "Save");
  assert.equal(tip.hasAttribute("data-open"), true, "the tooltip is on top");

  const key = press(document, "Escape");
  assert.equal(tip.hasAttribute("data-open"), false, "the tooltip hid");
  assert.equal(open(), false, "the popover under it closed");
  assert.deepEqual(closed, ["escape"]);
  assert.equal(key.prevented && key.stopped, true, "the popover used the key up");
  assert.deepEqual(typed, [], "so the field did not hear it");
});

test("with only a tooltip open and focus elsewhere, Escape hides it and goes on to the field", () => {
  const { WF, document } = loadRuntime();
  const input = document.createElement("input");
  const typed = [];
  input.addEventListener("keydown", (e) => typed.push(e.key));
  document.body.appendChild(input);
  input.focus();
  const { tip, root } = hovered(WF, document, "Save");
  assert.equal(tip.hasAttribute("data-open"), true);

  const key = press(document, "Escape");
  assert.equal(tip.hasAttribute("data-open"), false, "the tooltip hid");
  assert.equal(key.prevented, false, "the key was not prevented");
  assert.equal(key.stopped, false, "nor stopped");
  assert.deepEqual(typed, ["Escape"], "the field heard it: an editor's Escape is its own");

  // With nothing open, Escape is the page's as it always was.
  const again = press(document, "Escape");
  assert.equal(again.prevented || again.stopped, false);
  assert.deepEqual(typed, ["Escape", "Escape"]);

  // Hidden by Escape, it stays hidden while the pointer stays, and the
  // next hover shows it again.
  root.dispatchEvent({ type: "pointerleave" });
  root.dispatchEvent({ type: "pointerenter" });
  assert.equal(tip.hasAttribute("data-open"), true, "the next hover shows it");
});

test("a tooltip Escape hid shows again when keyboard focus reaches its control", () => {
  const { WF, document } = loadRuntime();
  const input = document.createElement("input");
  document.body.appendChild(input);
  input.focus();
  const { button, tip, root } = hovered(WF, document, "Save");
  press(document, "Escape");
  assert.equal(tip.hasAttribute("data-open"), false, "hidden, the pointer still on it");
  // Tab reaches its button: the state the stack wrote back lets it show.
  button.focus();
  root.dispatchEvent({ type: "focusin", target: button });
  assert.equal(tip.hasAttribute("data-open"), true, "focus shows it again");
});

test("with focus on a tooltip's control, Escape is the tooltip's: it hides and the key is used up", () => {
  const { WF, document } = loadRuntime();
  const anchor = document.createElement("button");
  const pop = document.createElement("div");
  document.body.appendChild(anchor);
  document.body.appendChild(pop);
  const open = WF.signal(true);
  WF.layer(pop, open, undefined, { anchor, focus: false });
  const { button, tip } = hovered(WF, document, "Save");
  const heard = [];
  button.addEventListener("keydown", (e) => heard.push(e.key));
  button.focus();

  const key = press(document, "Escape");
  assert.equal(tip.hasAttribute("data-open"), false, "the tooltip hid");
  assert.equal(key.prevented && key.stopped, true, "and used the key up");
  assert.deepEqual(heard, [], "its control did not hear it");
  assert.equal(open(), true, "the popover under it is still open");

  const next = press(document, "Escape");
  assert.equal(open(), false, "the next Escape closes the popover");
  assert.equal(next.prevented && next.stopped, true);
});

test("focus inside a layer that cannot be dismissed keeps its Escape", () => {
  const { WF, document } = loadRuntime();
  const anchor = document.createElement("button");
  const panel = document.createElement("div");
  const inside = document.createElement("button");
  panel.appendChild(inside);
  document.body.appendChild(anchor);
  document.body.appendChild(panel);
  const shown = WF.signal(true);
  WF.layer(panel, shown, undefined, { anchor, focus: false, dismiss: false });
  inside.focus();
  const key = press(document, "Escape");
  assert.equal(shown(), false, "it hid, and its state followed");
  assert.equal(key.prevented && key.stopped, true);
});

// ─── What a layer takes as any element does ─────────────────────────

/// The `wf` this checkout built most recently, release or debug.
function newestWf() {
  const built = ["release", "debug"]
    .map((p) => fileURLToPath(new URL(`../../target/${p}/wf`, import.meta.url)))
    .filter((p) => existsSync(p));
  built.sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs);
  return built[0] || null;
}

/// Build a one-page project of `body` with the compiler and mount what it
/// wrote against the fake DOM: the codegen and the runtime together.
function mountPage(body) {
  const wf = newestWf();
  if (!wf) return null;
  const dir = mkdtempSync(join(tmpdir(), "wf-layer-args-"));
  try {
    mkdirSync(join(dir, "src/pages"), { recursive: true });
    writeFileSync(join(dir, "webfluent.app.json"), JSON.stringify({ name: "Args", build: { output: "./build", ssg: false, minify: false } }));
    writeFileSync(join(dir, "src/pages/Home.wf"), `page Home(path: "/", title: "Args", description: "Layers and what they are given.") {\n    Heading("Args").h1\n${body}\n}\n`);
    execFileSync(wf, ["build", "-d", dir], { stdio: "pipe" });
    let src = readFileSync(join(dir, "build/app.js"), "utf8");
    const pages = join(dir, "build/pages");
    if (existsSync(pages)) {
      for (const file of readdirSync(pages).sort()) {
        if (file.endsWith(".js")) src += "\n" + readFileSync(join(pages, file), "utf8");
      }
    }
    const { window, document, Node, Element, DocumentFragment } = makeDom();
    window.innerWidth = 1000;
    window.innerHeight = 600;
    const app = document.createElement("div");
    app.id = "app";
    document.body.appendChild(app);
    const timers = [];
    window.setTimeout = (fn) => { timers.push(fn); return timers.length; };
    window.clearTimeout = () => {};
    const run = new Function(
      "window", "document", "Node", "Element", "DocumentFragment", "setTimeout", "clearTimeout", "URLSearchParams", "console",
      `${src}\nreturn typeof WF !== "undefined" ? WF : null;`,
    );
    const WF = run(window, document, Node, Element, DocumentFragment, window.setTimeout, window.clearTimeout, URLSearchParams, console);
    const drain = () => { while (timers.length) timers.shift()(); };
    drain();
    return { WF, document, drain };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

test("a popover, a menu, a tooltip and a modal are given role, tabindex, aria-* and data-*, and hear their handlers", () => {
  const page = mountPage(`    state open = true
    state keys = 0
    state closes = 0
    state why = ""
    Button("Anchor", ref: anchor)
    Popover(visible: open, anchor: anchor, label: "Choices", role: "listbox", tabindex: "-1", data-popup: "row", aria-describedby: "hint", id: "pop") {
        on keydown(e) { keys = keys + 1 }
        on close(reason) {
            closes = closes + 1
            why = reason
        }
        Text("one", role: "option")
    }
    Menu(trigger: "View", role: "group", data-kind: "view", id: "view") {
        on keydown(e) { keys = keys + 1 }
        Menu.Item { Text("x") }
    }
    Tooltip("Save", data-tip: "save", id: "tip") {
        on mouseenter { keys = keys + 1 }
        Button("Save")
    }
    Modal(visible: false, title: "M", role: "alertdialog", data-kind: "modal") {
        on keydown(e) { keys = keys + 1 }
        Text("m")
    }
    Text("{keys} {closes} {why}", id: "said")
    Text("hint", id: "hint")`);
  if (!page) return; // no compiler built: the Rust tests cover the build
  const { document, drain } = page;
  const said = () => document.querySelector("#said").textContent;
  const pop = document.querySelector("#pop");
  assert.ok(pop, "the popover is drawn");
  assert.equal(pop.getAttribute("role"), "listbox", "its author's role, in place of dialog");
  assert.equal(pop.getAttribute("tabindex"), "-1");
  assert.equal(pop.getAttribute("data-popup"), "row");
  assert.equal(pop.getAttribute("aria-describedby"), "hint");
  assert.equal(pop.getAttribute("aria-label"), "Choices");
  assert.equal(pop.hasAttribute("data-open"), true, "open, as its state says");

  pop.dispatchEvent({ type: "keydown", key: "a", target: pop });
  drain();
  assert.equal(said(), "1 0 ", "its `on keydown` is its own element's");

  document.dispatchEvent({ type: "keydown", key: "Escape", preventDefault() {}, stopPropagation() {} });
  drain();
  assert.equal(pop.hasAttribute("data-open"), false, "Escape closed it");
  assert.equal(said(), "1 1 escape", "and `on close` heard why, once");

  const menu = document.querySelector("#view");
  assert.equal(menu.getAttribute("role"), "group");
  assert.equal(menu.getAttribute("data-kind"), "view");
  menu.dispatchEvent({ type: "keydown", key: "a", target: menu });
  drain();
  assert.equal(said(), "2 1 escape", "a menu's `on keydown` is its root's");

  const tip = document.querySelector("#tip");
  assert.equal(tip.getAttribute("data-tip"), "save");
  tip.dispatchEvent({ type: "mouseenter", target: tip });
  drain();
  assert.equal(said(), "3 1 escape", "a tooltip's handler is its wrapper's");

  const modal = document.querySelector("dialog");
  assert.equal(modal.getAttribute("role"), "alertdialog");
  assert.equal(modal.getAttribute("data-kind"), "modal");
  modal.dispatchEvent({ type: "keydown", key: "a", target: modal });
  drain();
  assert.equal(said(), "4 1 escape", "a modal's handler is its dialog's");
});

test("`html:` on a layer, or on any element, is an attribute of that name and never markup", () => {
  // The runtime's own key for markup is a hook; an author's spelling of it
  // reaches `WF.el` as `attr:html`, which is written as it is.
  const { WF } = loadRuntime();
  const markup = "<img src=x onerror=alert(1)>";
  const plain = WF.el("div", { "attr:html": markup, "attr:markdown": "# Hi" }, ["kept"]);
  assert.equal(plain.getAttribute("html"), markup);
  assert.equal(plain.getAttribute("markdown"), "# Hi");
  assert.equal(plain.innerHTML, "kept", "its children are its own, no markup put in");
  const said = WF.signal("one");
  const live = WF.el("div", { "attr:html": () => said() });
  said.set("two");
  assert.equal(live.getAttribute("html"), "two", "a value that reads state follows it");
  said.set(null);
  assert.equal(live.hasAttribute("html"), false, "and null is no attribute");

  const page = mountPage(`    state open = true
    state markup = "<img src=x onerror=alert(1)>"
    Button("Anchor", ref: anchor)
    Stack(html: markup, id: "stack") { Text("stack") }
    Popover(visible: open, anchor: anchor, html: markup, markdown: markup, id: "pop") { Text("pop") }
    Tooltip("Save", html: markup, id: "tip") { Button("Save") }
    Modal(visible: false, title: "M", html: markup, id: "modal") { Text("m") }
    Menu(trigger: "View", html: markup, id: "menu") { Menu.Item(html: markup, id: "item") { Text("x") } }
    Element("x-card", html: markup, id: "custom")`);
  if (!page) return; // no compiler built: the Rust tests cover the build
  const { document } = page;
  for (const id of ["stack", "pop", "tip", "modal", "menu", "item", "custom"]) {
    const el = document.querySelector(`#${id}`);
    assert.ok(el, `#${id} is drawn`);
    assert.equal(el.getAttribute("html"), "<img src=x onerror=alert(1)>", `#${id} has it as an attribute`);
    assert.equal(el._html, undefined, `#${id} has no markup put in`);
  }
  assert.equal(document.querySelector("#pop").getAttribute("markdown"), "<img src=x onerror=alert(1)>");
  assert.match(document.querySelector("#stack").textContent, /stack/, "its children are drawn");
});
