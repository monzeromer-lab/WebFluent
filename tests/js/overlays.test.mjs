//! Overlays without a browser: a toast's action, its close button, the
//! three that show at once, `toastPromise` following its promise, and the
//! assertive region `announce(…, "assertive")` speaks in. What only a real
//! browser can tell — where the focus goes, what closes a dialog, a sheet
//! dragged shut — is `tests/browser/overlays.mjs`.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const timers = new Map();
  let ids = 0;
  const setTimeout = (fn, ms) => { const id = ++ids; timers.set(id, { fn, ms }); return id; };
  const clearTimeout = (id) => { timers.delete(id); };
  // Every timer due within `ms`, and those they set.
  const run = (ms = Infinity) => {
    for (let n = 0; n < 50; n++) {
      const due = [...timers].filter(([, t]) => !(t.ms > ms));
      if (!due.length) return;
      for (const [id, t] of due) { timers.delete(id); t.fn(); }
    }
  };
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout", "clearTimeout",
    `${fullRuntime()}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment, setTimeout, clearTimeout);
  return { WF, document, run };
}

const shown = (document) => document.querySelectorAll(".wf-toast").filter((n) => !n.classList.contains("wf-toast--exit"));

test("a toast's action runs and closes it; its close button closes it", () => {
  const { WF, document } = load();
  let undone = 0;
  WF.toast("Archived", { action: "Undo", onAction: () => { undone += 1; } });
  const [t] = shown(document);
  assert.equal(t.querySelector(".wf-toast__text").textContent, "Archived");
  t.querySelector(".wf-toast__action").dispatchEvent({ type: "click" });
  assert.equal(undone, 1);
  assert.equal(shown(document).length, 0, "the action closes it");
  WF.toast("Saved");
  const [u] = shown(document);
  assert.equal(u.querySelector(".wf-toast__close").getAttribute("aria-label"), "Close");
  u.querySelector(".wf-toast__close").dispatchEvent({ type: "click" });
  assert.equal(shown(document).length, 0);
});

test("no more than three toasts show; a fourth closes the oldest", () => {
  const { WF, document } = load();
  for (const w of ["one", "two", "three", "four"]) WF.toast(w);
  const words = shown(document).map((n) => n.querySelector(".wf-toast__text").textContent);
  assert.deepEqual(words, ["two", "three", "four"]);
});

test("a toast stays while the pointer is on it, then goes", () => {
  const { WF, document, run } = load();
  WF.toast("Hold me", { duration: 1000 });
  const [t] = shown(document);
  t.dispatchEvent({ type: "mouseenter" });
  run(1000);
  assert.equal(shown(document).length, 1, "paused under the pointer");
  t.dispatchEvent({ type: "mouseleave" });
  run(1000);
  assert.equal(shown(document).length, 0);
});

test("the old call, a tone and a duration, still works", () => {
  const { WF, document } = load();
  WF.toast("Saved!", "success", 2000);
  assert.ok(shown(document)[0].classList.contains("wf-toast--success"));
});

test("toastPromise says one thing while it runs, another when it is done", async () => {
  const { WF, document } = load();
  let finish;
  const p = new Promise((r) => { finish = r; });
  const back = WF.toastPromise(p, { loading: "Saving…", done: (v) => `Saved ${v}` });
  assert.equal(back, p, "the promise, for the action to await");
  const text = () => shown(document)[0].querySelector(".wf-toast__text").textContent;
  assert.equal(text(), "Saving…");
  finish(3);
  await p;
  await null;
  assert.equal(text(), "Saved 3");
  assert.ok(shown(document)[0].classList.contains("wf-toast--success"));
});

test("announce speaks politely, or in the assertive region when asked", () => {
  const { WF, document, run } = load();
  WF.announce("Saved");
  WF.announce("Connection lost", "assertive");
  run();
  const polite = document.querySelector('[aria-live="polite"]');
  const urgent = document.querySelector('[aria-live="assertive"]');
  assert.equal(polite.textContent, "Saved");
  assert.equal(urgent.textContent, "Connection lost");
});
