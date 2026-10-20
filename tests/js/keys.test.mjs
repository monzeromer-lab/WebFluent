//! Key combinations: `mod`, chords, and how they are written for the
//! reader on each system.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

/// The runtime as it loads on `platform` — `MacIntel`, `Linux x86_64`,
/// `Win32`.
function on(platform) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const navigator = { platform, userAgent: "" };
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", "navigator", `${fullRuntime()}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment, navigator), document };
}

const key = (k, mods = {}) => ({ type: "keydown", key: k, code: "", ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, ...mods });

test("mod is ⌘ on a Mac and Ctrl elsewhere", () => {
  const mac = on("MacIntel").WF;
  const linux = on("Linux x86_64").WF;
  assert.equal(mac.keyIs(key("s", { metaKey: true }), "mod+s"), true);
  assert.equal(mac.keyIs(key("s", { ctrlKey: true }), "mod+s"), false);
  assert.equal(linux.keyIs(key("s", { ctrlKey: true }), "mod+s"), true);
  assert.equal(linux.keyIs(key("s", { metaKey: true }), "mod+s"), false);
  // Every modifier must match: mod+s is not mod+shift+s.
  assert.equal(linux.keyIs(key("S", { ctrlKey: true, shiftKey: true }), "mod+s"), false);
  assert.equal(linux.keyIs(key("S", { ctrlKey: true, shiftKey: true }), "mod+shift+s"), true);
});

test("a letter is also its place on the keyboard: ⌥⌘P sends π", () => {
  const { WF } = on("MacIntel");
  assert.equal(WF.keyIs(key("π", { code: "KeyP", metaKey: true, altKey: true }), "mod+alt+p"), true);
  assert.equal(WF.keyIs(key("π", { code: "KeyO", metaKey: true, altKey: true }), "mod+alt+p"), false);
});

test("a layout that gives a Latin letter is read by it; one that does not, by the key's place", () => {
  const { WF } = on("Linux x86_64");
  // AZERTY: the key labelled A sits where QWERTY's Q is.
  const azertyQ = key("q", { code: "KeyA", ctrlKey: true });
  assert.equal(WF.keyIs(azertyQ, "mod+q"), true, "Ctrl+Q is mod+q");
  assert.equal(WF.keyIs(azertyQ, "mod+a"), false, "and not mod+a too");
  // An Arabic layout sends Arabic letters: the place decides.
  assert.equal(WF.keyIs(key("ح", { code: "KeyP", ctrlKey: true, shiftKey: true }), "mod+shift+p"), true);
  assert.equal(WF.keyIs(key("ح", { code: "KeyP", ctrlKey: true }), "mod+b"), false);
});

test("a combination is written as the system writes it", () => {
  const mac = on("MacIntel").WF;
  const win = on("Win32").WF;
  assert.equal(mac.keyLabel("mod+shift+p"), "⇧⌘P");
  assert.equal(mac.keyLabel("ctrl+shift+g"), "⌃⇧G");
  assert.equal(mac.keyLabel("mod+alt+p"), "⌥⌘P");
  assert.equal(mac.keyLabel("mod+k mod+s"), "⌘K ⌘S");
  assert.equal(mac.keyLabel("shift+F12"), "⇧F12");
  assert.equal(win.keyLabel("mod+shift+p"), "Ctrl+Shift+P");
  assert.equal(win.keyLabel("mod+k mod+s"), "Ctrl+K Ctrl+S");
  assert.equal(win.keyLabel("F2"), "F2");
  assert.equal(win.keyLabel("mod+."), "Ctrl+.");
  assert.equal(win.keyLabel("escape"), "Esc");
  // As `aria-keyshortcuts` spells it.
  assert.equal(mac.keyAria("mod+shift+p"), "Shift+Meta+P");
  assert.equal(win.keyAria("mod+shift+p"), "Control+Shift+P");
  assert.equal(win.keyAria("esc"), "Escape");
});

test("a chord fires on its second step, and that step is not its own shortcut", () => {
  const { WF, document } = on("Linux x86_64");
  const heard = [];
  WF.onKey(document, "mod+k mod+s", () => heard.push("help"));
  WF.onKey(document, "mod+s", () => heard.push("save"));
  document.dispatchEvent(key("s", { ctrlKey: true }));
  assert.deepEqual(heard, ["save"], "alone, mod+s is save");
  document.dispatchEvent(key("k", { ctrlKey: true }));
  document.dispatchEvent(key("s", { ctrlKey: true }));
  assert.deepEqual(heard, ["save", "help"], "after mod+k it is the chord's");
  // A modifier going down between the steps is not a step.
  document.dispatchEvent(key("k", { ctrlKey: true }));
  document.dispatchEvent(key("Control", { ctrlKey: true }));
  document.dispatchEvent(key("s", { ctrlKey: true }));
  assert.deepEqual(heard, ["save", "help", "help"]);
});
