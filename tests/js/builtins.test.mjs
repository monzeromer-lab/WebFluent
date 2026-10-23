//! Every built-in takes what any element takes, at run time: what the
//! runtime gives a built-in's root itself — a splitter's `role` and
//! `tabindex`, a carousel's `role` — gives way to what its author wrote, a
//! picture's image takes the attributes and handlers written on it, and a
//! long list is the listbox an editor's references list needs.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { mountPage } from "./harness.mjs";
import { fullRuntime, runtimeFor } from "./runtime.mjs";

function load(src = fullRuntime()) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  window.innerWidth = 1000;
  window.innerHeight = 600;
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", "setTimeout", `${src}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment, setTimeout), document };
}

test("a splitter is a separator in the tab order unless its author wrote what it is", () => {
  const { WF } = load();
  const width = WF.signal(200);
  const plain = WF.el("div", {});
  WF.splitter(plain, () => width(), (v) => width.set(v), {});
  assert.equal(plain.getAttribute("role"), "separator");
  assert.equal(plain.getAttribute("tabindex"), "0");

  const own = WF.el("div", { role: "slider", tabindex: "-1" });
  WF.splitter(own, () => width(), (v) => width.set(v), {});
  assert.equal(own.getAttribute("role"), "slider", "its author's role");
  assert.equal(own.getAttribute("tabindex"), "-1", "its author's tabindex");
  assert.equal(own.getAttribute("aria-valuenow"), "200", "the rest is still the splitter's");
});

test("a carousel is a region called a carousel unless its author wrote what it is", () => {
  const { WF } = load();
  const build = (attrs) => {
    const root = WF.el("div", attrs);
    const track = WF.el("div", { className: "wf-carousel__track" });
    track.appendChild(WF.el("div", { className: "wf-carousel__slide" }));
    root.appendChild(track);
    WF.carousel(root, { label: "Shots" });
    return root;
  };
  const plain = build({});
  assert.equal(plain.getAttribute("role"), "region");
  assert.equal(plain.getAttribute("aria-roledescription"), "carousel");
  const own = build({ role: "group", "aria-roledescription": "slides" });
  assert.equal(own.getAttribute("role"), "group");
  assert.equal(own.getAttribute("aria-roledescription"), "slides");
  assert.equal(own.getAttribute("aria-label"), "Shots", "its label still names it");
});

test("a picture's image takes the attributes and handlers written on it", () => {
  const { WF } = load();
  const handle = WF.ref();
  const loads = [];
  const asset = { src: "/hero.png", width: 600, height: 400, sources: [{ type: "image/webp", srcset: "/hero.600.webp 600w" }] };
  const picture = WF.picture(asset, {
    alt: "The harbour",
    className: "wf-image",
    attrs: { id: "hero", "data-k": "v", "aria-describedby": "caption", ref: handle, "on:load": () => loads.push("load") },
  });
  assert.equal(picture.tagName.toLowerCase(), "picture");
  const img = picture.querySelector("img");
  assert.equal(img.getAttribute("id"), "hero");
  assert.equal(img.getAttribute("data-k"), "v");
  assert.equal(img.getAttribute("aria-describedby"), "caption");
  assert.equal(img.getAttribute("alt"), "The harbour");
  assert.equal(img.getAttribute("width"), "600", "the build's own width stays");
  assert.equal(handle.current, img, "the handle is the image");
  img.dispatchEvent({ type: "load" });
  assert.deepEqual(loads, ["load"], "its `on load` is the image's");
});

test("with no layers in the build, a modal opens and closes on its own", () => {
  // `overlay` without `layer`: the stack it joins is not there, and the
  // dialog is driven as it always was.
  const { WF, document } = load(runtimeFor("overlay"));
  const dialog = document.createElement("dialog");
  document.body.appendChild(dialog);
  const open = WF.signal(true);
  WF.dialog(dialog, open);
  assert.equal(dialog.open, true);
  dialog.close();
  assert.equal(open(), false, "the browser's close writes the state back");
});

test("a long list is a listbox: its role, its active descendant and its keys are the box's", () => {
  const page = mountPage(`    state rows = [{ id: "a" }, { id: "b" }, { id: "c" }]
    state at = 0
    VirtualList(rowHeight: 24, label: "References", role: "listbox", tabindex: "0", aria-activedescendant: "ref-{at}", data-list: "refs", id: "refs") {
        on keydown(e) {
            if e.key == "ArrowDown" { at = at + 1 }
        }
        for r in rows by r.id { Text(r.id, id: "ref-{r.id}", role: "option") }
    }`);
  if (!page) return; // no compiler built: the Rust tests cover the build
  const { document, drain } = page;
  const box = document.querySelector("#refs");
  assert.ok(box, "the list is drawn");
  assert.equal(box.getAttribute("role"), "listbox");
  assert.equal(box.getAttribute("tabindex"), "0");
  assert.equal(box.getAttribute("data-list"), "refs");
  assert.equal(box.getAttribute("aria-label"), "References");
  assert.equal(box.getAttribute("aria-activedescendant"), "ref-0");
  box.dispatchEvent({ type: "keydown", key: "ArrowDown", target: box });
  drain();
  assert.equal(box.getAttribute("aria-activedescendant"), "ref-1", "its `on keydown` ran, and the attribute follows the state");
});

test("a splitter and a control in a box are given what any element is, through the compiler", () => {
  const page = mountPage(`    state w = 240
    state size = 3
    state keys = 0
    state wraps = false
    Splitter(bind: w, role: "slider", tabindex: "-1", data-k: "v", id: "split") {
        on keydown(e) { keys = keys + 1 }
    }
    Slider(bind: size, label: "Size", id: "size", role: "spinbutton", disabled: true, data-k: "v", lang: "ar")
    Switch(bind: wraps, label: "On", role: "menuitemcheckbox", id: "on")
    Text("{keys}", id: "keys")`);
  if (!page) return; // no compiler built: the Rust tests cover the build
  const { document, drain } = page;
  const split = document.querySelector("#split");
  assert.equal(split.getAttribute("role"), "slider", "its author's role, kept by the splitter");
  assert.equal(split.getAttribute("tabindex"), "-1");
  assert.equal(split.getAttribute("data-k"), "v");
  split.dispatchEvent({ type: "keydown", key: "a", target: split });
  drain();
  assert.equal(document.querySelector("#keys").textContent, "1", "its `on keydown` is its own");
  const range = document.querySelector("#size");
  assert.equal(range.tagName.toLowerCase(), "input", "a slider's id is its input's");
  assert.equal(range.getAttribute("role"), "spinbutton");
  assert.equal(range.disabled, true, "a disabled slider is disabled");
  assert.equal(range.parentNode.getAttribute("lang"), "ar", "its language is its box's");
  const on = document.querySelector("#on");
  assert.equal(on.getAttribute("role"), "menuitemcheckbox", "a switch's role, in place of switch");
});
