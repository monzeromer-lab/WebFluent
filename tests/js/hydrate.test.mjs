//! What the runtime actually does when it takes over an SSG paint.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { makeDom } from "./dom.mjs";

/// Load runtime.js against a fake DOM and hand back its public surface.
function loadRuntime() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const src = readFileSync(new URL("../../src/runtime/runtime.js", import.meta.url), "utf8");
  // Timers run inline: the runtime defers by a tick, and a test wants the
  // settled state.
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", "setTimeout", `${src}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment, setTimeout), document, window };
}

test("hydrate leaves a live page: the click handler on the server paint works", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("div");
  // The server already painted this button — as it does for any page with static
  // content. Before, hydrate() kept THIS node and bound the handler to the one it
  // built and threw away, so clicking did nothing.
  const painted = document.createElement("button");
  painted.textContent = "Add";
  container.appendChild(painted);

  let clicks = 0;
  WF.hydrate(() => {
    const b = WF.el("button", {}, ["Add"]);
    b.addEventListener("click", () => clicks++);
    const root = WF.el("div", {}, [b]);
    return root;
  }, container);

  const button = container.querySelectorAll("button")[0];
  assert.ok(button, "a button must be present after hydration");
  button.click();
  assert.equal(clicks, 1, "clicking the visible button must run the handler");
});

test("hydrate shows reactive text, and updates it when the signal changes", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("div");
  const stale = document.createElement("p");
  stale.textContent = "0";
  container.appendChild(stale);

  const count = WF.signal(0);
  WF.hydrate(() => WF.el("div", {}, [() => String(count())]), container);
  count.set(5);
  assert.match(container.textContent, /5/, "the visible DOM must follow the signal");
});

test("mount replaces whatever was there", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("div");
  container.appendChild(document.createElement("span"));
  WF.mount(() => WF.el("p", {}, ["fresh"]), container);
  assert.equal(container.textContent, "fresh");
});

test("activeLink marks the link to the current route, and follows navigation", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("div");
  const home = WF.el("a", { href: "/" }, ["Home"]);
  const docs = WF.el("a", { href: "/docs" }, ["Docs"]);
  const guide = WF.el("a", { href: "/docs/guide" }, ["Guide"]);
  // Links are built before the router exists, as an app shell's navbar is.
  WF.activeLink(home, "/", false);
  WF.activeLink(docs, "/docs", true);
  WF.activeLink(guide, "/docs/guide", false);

  assert.equal(home.getAttribute("aria-current"), "page");
  assert.ok(home.classList.contains("active"));
  assert.equal(docs.getAttribute("aria-current"), null);

  WF.router([{ path: "*", render: () => WF.el("p", {}, ["page"]) }], container);
  WF.navigate("/docs/guide");

  assert.equal(home.getAttribute("aria-current"), null, "home is no longer current");
  assert.ok(!home.classList.contains("active"));
  assert.equal(guide.getAttribute("aria-current"), "page", "exact match");
  assert.equal(docs.getAttribute("aria-current"), "page", "prefix match on the section root");
});

test("an aria-* attribute keeps a false value, and follows a reactive one", () => {
  const { WF } = loadRuntime();
  const pressed = WF.signal(false);
  const chip = WF.el("button", { "aria-pressed": () => pressed(), "data-tone": "info", hidden: false });
  assert.equal(chip.getAttribute("aria-pressed"), "false", "false is a real ARIA state");
  assert.equal(chip.getAttribute("data-tone"), "info");
  assert.equal(chip.getAttribute("hidden"), null, "a false plain attribute is absent");
  pressed.set(true);
  assert.equal(chip.getAttribute("aria-pressed"), "true");
});

test("the router sets the document title to the page it shows", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("div");
  WF.router(
    [
      { path: "/", title: "Home", render: () => WF.el("p", {}, ["home"]) },
      { path: "/docs", title: "Routing rules", render: () => WF.el("p", {}, ["docs"]) },
    ],
    container,
  );
  assert.equal(document.title, "Home");
  WF.navigate("/docs");
  assert.equal(document.title, "Routing rules");
});

test("a store's derived value may call one of its actions", () => {
  const { WF } = loadRuntime();
  const S = WF.store({
    state: { total: 50 },
    derived: { half: (store) => store.pctOf(25) },
    actions: { pctOf: (store, part) => Math.round((part / store.total) * 100) },
  });
  assert.equal(S.half, 50);
  S.total = 100;
  assert.equal(S.half, 25);
});

test("a select's value is applied once its options exist, and follows a signal", () => {
  const { WF, document } = loadRuntime();
  // A real <select> ignores a value it has no option for; the fake DOM does
  // not, so give this one a browser's setter.
  const create = document.createElement.bind(document);
  document.createElement = (tag) => {
    const el = create(tag);
    if (tag === "select") {
      let current = "";
      Object.defineProperty(el, "value", {
        get: () => current,
        set: (v) => {
          const options = el.childNodes.map((o) => o.value);
          if (options.includes(v)) current = v;
        },
      });
    }
    return el;
  };
  const field = WF.signal("status");
  const select = WF.el(
    "select",
    { value: () => field() },
    WF.el("option", { value: "region" }, "region"),
    WF.el("option", { value: "status" }, "status"),
  );
  assert.equal(select.value, "status", "set after the options were appended");
  field.set("region");
  assert.equal(select.value, "region");
});

test("an icon button draws its glyph once", () => {
  const { WF, document } = loadRuntime();
  // The fake DOM ignores markup assigned to innerHTML; count the assignments.
  let draws = 0;
  const create = document.createElement.bind(document);
  document.createElement = (tag) => {
    const el = create(tag);
    Object.defineProperty(el, "innerHTML", {
      get: () => "",
      set: (v) => { if (String(v).includes("<svg")) draws += 1; },
    });
    return el;
  };
  WF.el(
    "button",
    { className: "wf-icon-btn", "data-icon": "close", "aria-label": "Close" },
    WF.el("span", { className: "wf-icon", "data-icon": "close" }),
  );
  assert.equal(draws, 1, "one glyph for the button and its icon span");
  WF.el("span", { className: "wf-icon", "data-icon": "close" });
  assert.equal(draws, 2, "a lone icon still draws");
});

test("a route change moves focus to the new page's heading, resets the scroll and announces the title", () => {
  const { WF, document, window } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  WF.router(
    [
      { path: "/", title: "Home", render: () => WF.el("div", {}, [WF.el("h1", {}, ["Home"])]) },
      { path: "/docs", title: "Docs", render: () => WF.el("div", {}, [WF.el("h1", {}, ["Docs"])]) },
      { path: "/bare", title: "Bare", render: () => WF.el("p", {}, ["no heading"]) },
    ],
    container,
  );
  // The first render is the page load: the browser places focus, not us.
  assert.equal(document.activeElement, null);
  assert.equal(window.scrolls.length, 0);

  WF.navigate("/docs");
  const heading = container.querySelector("h1");
  assert.equal(document.activeElement, heading, "focus lands on the new page's h1");
  assert.equal(heading.getAttribute("tabindex"), "-1", "which is made focusable");
  assert.deepEqual(window.scrolls.at(-1), [0, 0], "and the viewport returns to the top");
  const announcer = document.body.querySelector('[role="status"]');
  assert.ok(announcer, "a live region exists");
  assert.equal(announcer.textContent, "Docs", "and reads the new title");

  WF.navigate("/bare");
  assert.equal(document.activeElement, container, "with no heading, focus lands on the landmark");
});

test("a page with a sheet of its own is drawn once the sheet has loaded, and only fetched once", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  WF.router(
    [
      { path: "/", title: "Home", render: () => WF.el("h1", {}, ["Home"]) },
      { path: "/pricing", title: "Pricing", css: "Pricing", render: () => WF.el("h1", {}, ["Pricing"]) },
    ],
    container,
  );
  // The sheet takes time to arrive: hold the load until the test releases it.
  const append = document.head.appendChild.bind(document.head);
  document.head.appendChild = (n) => { n.parentNode = document.head; document.head.childNodes.push(n); return n; };
  WF.navigate("/pricing");
  document.head.appendChild = append;
  const link = document.head.querySelector('link[data-wf-page-css="Pricing"]');
  assert.ok(link, "the page's sheet is linked");
  assert.equal(link.href, "/pages/Pricing.css");
  assert.equal(container.querySelector("h1").textContent, "Home", "the old page stays until the sheet applies");

  link.onload();
  assert.equal(container.querySelector("h1").textContent, "Pricing", "then the new page is drawn");

  WF.navigate("/");
  WF.navigate("/pricing");
  assert.equal(document.head.querySelectorAll("link[data-wf-page-css]").length, 1, "a loaded sheet is not fetched again");
  assert.equal(container.querySelector("h1").textContent, "Pricing");
});

test("a carousel is a labelled region whose slides are groups, with named controls and pausable rotation", () => {
  const { WF, document, window } = loadRuntime();
  const intervals = [];
  const timers = { setInterval, clearInterval };
  globalThis.setInterval = (fn, ms) => { intervals.push({ fn, ms }); return intervals.length; };
  globalThis.clearInterval = (id) => { intervals[id - 1].cleared = true; };
  try {
  const track = WF.el("div", { className: "wf-carousel__track" }, [
    WF.el("div", { className: "wf-carousel__slide" }, ["one"]),
    WF.el("div", { className: "wf-carousel__slide" }, ["two"]),
    WF.el("div", { className: "wf-carousel__slide", "aria-label": "Team photo" }, ["three"]),
  ]);
  const root = WF.el("div", { className: "wf-carousel" }, [track]);
  document.body.appendChild(root);
  const api = WF.carousel(root, { autoplay: true, interval: 4000, label: "Product tour" });

  assert.equal(root.getAttribute("role"), "region");
  assert.equal(root.getAttribute("aria-roledescription"), "carousel");
  assert.equal(root.getAttribute("aria-label"), "Product tour");
  const slides = track.querySelectorAll(".wf-carousel__slide");
  assert.equal(slides[0].getAttribute("aria-roledescription"), "slide");
  assert.equal(slides[1].getAttribute("aria-label"), "2 of 3");
  assert.equal(slides[2].getAttribute("aria-label"), "Team photo", "a slide's own label is kept");
  assert.equal(slides[0].getAttribute("aria-hidden"), "false");
  assert.equal(slides[1].getAttribute("aria-hidden"), "true");
  assert.equal(slides[1].getAttribute("inert"), "", "an off-screen slide is out of the tab order");

  const buttons = root.querySelectorAll("button");
  const labels = buttons.map((b) => b.getAttribute("aria-label"));
  assert.ok(labels.includes("Previous slide") && labels.includes("Next slide"), labels);
  assert.ok(labels.includes("Go to slide 2"), labels);
  const pause = buttons.find((b) => b.getAttribute("aria-label") === "Stop automatic slide rotation");
  assert.ok(pause, "autoplay has a pause control");
  assert.equal(intervals.length, 1, "rotation is running");
  assert.equal(track.getAttribute("aria-live"), "off", "and the track is silent while it rotates");

  pause.dispatchEvent({ type: "click" });
  assert.ok(intervals[0].cleared, "pausing stops the rotation");
  assert.equal(track.getAttribute("aria-live"), "polite", "and the track announces manual changes");
  assert.equal(pause.getAttribute("aria-pressed"), "true");

  buttons.find((b) => b.getAttribute("aria-label") === "Next slide").dispatchEvent({ type: "click" });
  assert.equal(api.index(), 1);
  assert.equal(slides[1].getAttribute("aria-hidden"), "false");
  const dot2 = buttons.find((b) => b.getAttribute("aria-label") === "Go to slide 2");
  assert.equal(dot2.getAttribute("aria-current"), "true");
  } finally {
    Object.assign(globalThis, timers);
  }
});

test("a carousel does not rotate for a reader who asked for reduced motion", () => {
  const { WF, document, window } = loadRuntime();
  window.matchMedia = () => ({ matches: true, addEventListener() {} });
  const intervals = [];
  const realSetInterval = setInterval;
  globalThis.setInterval = (fn, ms) => { intervals.push(fn); return 1; };
  try {
  const root = WF.el("div", { className: "wf-carousel" }, [
    WF.el("div", { className: "wf-carousel__track" }, [
      WF.el("div", { className: "wf-carousel__slide" }, ["a"]),
      WF.el("div", { className: "wf-carousel__slide" }, ["b"]),
    ]),
  ]);
  document.body.appendChild(root);
  WF.carousel(root, { autoplay: true, interval: 1000 });
  assert.equal(intervals.length, 0, "no timer is started");
  assert.equal(root.querySelectorAll("button").filter((b) => (b.getAttribute("aria-label") || "").includes("rotation")).length, 0, "and there is no pause button to press");
  } finally {
    globalThis.setInterval = realSetInterval;
  }
});

test("a tooltip describes its trigger, is reachable by keyboard and dismissed by Escape", () => {
  const { WF, document } = loadRuntime();
  const button = WF.el("button", {}, ["Save"]);
  const tip = WF.el("span", { className: "wf-tooltip__text", role: "tooltip", id: "wf-tip-1" }, ["Saves the draft"]);
  const root = WF.el("div", { className: "wf-tooltip" }, [button, tip]);
  WF.tooltip(root, tip);
  assert.equal(button.getAttribute("aria-describedby"), "wf-tip-1", "the focusable child is described by the tip");
  assert.equal(root.getAttribute("tabindex"), null, "the wrapper stays out of the tab order");
  root.dispatchEvent({ type: "keydown", key: "Escape", stopPropagation() {} });
  assert.equal(root.getAttribute("data-dismissed"), "", "Escape hides the tip");
  root.dispatchEvent({ type: "mouseleave" });
  assert.equal(root.getAttribute("data-dismissed"), null, "and leaving resets it");

  // With nothing focusable inside, the wrapper itself becomes the trigger.
  const tip2 = WF.el("span", { className: "wf-tooltip__text", role: "tooltip", id: "wf-tip-2" }, ["Hint"]);
  const plain = WF.el("div", { className: "wf-tooltip" }, [WF.el("span", {}, ["term"]), tip2]);
  WF.tooltip(plain, tip2);
  assert.equal(plain.getAttribute("tabindex"), "0");
  assert.equal(plain.getAttribute("aria-describedby"), "wf-tip-2");
});

test("a menu's items are menuitems the arrow keys move between, and choosing one closes it", () => {
  const { WF, document } = loadRuntime();
  const open = WF.signal(false);
  const trigger = WF.el("button", { "aria-expanded": () => (open() ? "true" : "false") }, ["Actions"]);
  const edit = WF.el("li", { className: "wf-menu__item" }, ["Edit"]);
  const remove = WF.el("li", { className: "wf-menu__item" }, ["Delete"]);
  const list = WF.el("ul", { role: "menu" }, [edit, WF.el("li", { className: "wf-menu__divider" }), remove]);
  const root = WF.el("div", { className: "wf-menu" }, [trigger, list]);
  document.body.appendChild(root);
  WF.menu(root, trigger, list, open);

  assert.equal(edit.getAttribute("role"), "menuitem");
  assert.equal(edit.getAttribute("tabindex"), "-1", "items are reached with the arrows, not Tab");
  assert.equal(list.children[1].getAttribute("role"), "separator");

  trigger.dispatchEvent({ type: "keydown", key: "ArrowDown", preventDefault() {} });
  assert.equal(open(), true, "ArrowDown on the button opens the menu");
  assert.equal(document.activeElement, edit, "and focuses the first item");

  list.dispatchEvent({ type: "keydown", key: "ArrowDown", preventDefault() {} });
  assert.equal(document.activeElement, remove);
  list.dispatchEvent({ type: "keydown", key: "ArrowDown", preventDefault() {} });
  assert.equal(document.activeElement, edit, "the arrows wrap");
  list.dispatchEvent({ type: "keydown", key: "End", preventDefault() {} });
  assert.equal(document.activeElement, remove);

  let chosen = 0;
  remove.addEventListener("click", () => chosen++);
  list.dispatchEvent({ type: "keydown", key: "Enter", preventDefault() {} });
  assert.equal(chosen, 1, "Enter activates the focused item");
  list.dispatchEvent({ type: "click", target: remove });
  assert.equal(open(), false, "choosing an item closes the menu");
  assert.equal(document.activeElement, trigger, "and focus returns to the button");
});

test("a field labels its control, describes it, and announces its error", () => {
  const { WF, document } = loadRuntime();
  const error = WF.signal("");
  const input = WF.el("input", { className: "wf-input", type: "text" });
  const field = WF.field(input, { label: "Name", hint: "As on your passport", error: () => error() });
  document.body.appendChild(field);

  const label = field.querySelector("label");
  assert.equal(label.textContent, "Name");
  assert.equal(label.getAttribute("for"), input.id, "the label points at the control");
  const hint = field.querySelector(".wf-field__hint");
  const message = field.querySelector(".wf-field__error");
  assert.equal(input.getAttribute("aria-describedby"), `${hint.id} ${message.id}`);
  assert.equal(message.getAttribute("role"), "alert");
  assert.equal(message.getAttribute("hidden"), "", "no error, no message");
  assert.equal(input.getAttribute("aria-invalid"), "false");

  error.set("Enter your name");
  assert.equal(message.textContent, "Enter your name");
  assert.equal(message.getAttribute("hidden"), null);
  assert.equal(input.getAttribute("aria-invalid"), "true");
});

test("classes an expression names follow it, and leave the element's other classes alone", () => {
  const { WF } = loadRuntime();
  const tone = WF.signal("calm wide");
  const el = WF.el("div", { className: "wf-card wf-s0123abcd" });
  WF.classes(el, () => tone());
  assert.equal(el.className, "wf-card wf-s0123abcd calm wide");

  tone.set("alert");
  assert.equal(el.className, "wf-card wf-s0123abcd alert", "what it no longer names comes off");
  tone.set("");
  assert.equal(el.className, "wf-card wf-s0123abcd");
});

test("a match shows the arm its key names and swaps it when the key changes", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const state = WF.signal("loading");
  const data = WF.signal(null);
  let rendered = 0;
  WF.match(parent, () => state(), () => data(), {
    loading: () => WF.el("p", {}, ["…"]),
    ready: (rows) => { rendered++; return WF.el("p", {}, [`${rows.length} rows`]); },
    else: () => WF.el("p", {}, ["?"]),
  });
  assert.equal(parent.querySelector("p").textContent, "…");

  data.set([1, 2, 3]);
  state.set("ready");
  assert.equal(parent.querySelector("p").textContent, "3 rows", "the arm receives the argument");
  assert.equal(parent.querySelectorAll("p").length, 1, "the old arm is gone");

  data.set([1]);
  assert.equal(rendered, 1, "only the key decides when to redraw");

  state.set("unknown");
  assert.equal(parent.querySelector("p").textContent, "?", "an unknown key falls to else");
});

test("what a branch creates leaves with it: timers stop, effects fall silent, cleanups run", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const open = WF.signal(true);
  const count = WF.signal(0);
  let ticks = 0;
  let effectRuns = 0;
  let cleanups = 0;
  const realSetInterval = globalThis.setInterval;
  const realClearInterval = globalThis.clearInterval;
  const intervals = new Set();
  globalThis.setInterval = (fn, ms) => { const id = { fn, ms }; intervals.add(id); return id; };
  globalThis.clearInterval = (id) => intervals.delete(id);
  try {
    WF.when(parent, () => open(), () => {
      WF.every(1000, () => { ticks++; });
      WF.effect(() => { count(); effectRuns++; return () => { cleanups++; }; });
      return WF.el("p", {}, ["open"]);
    }, null);
    assert.equal(intervals.size, 1, "the timer runs while the branch shows");
    assert.equal(effectRuns, 1);
    count.set(1);
    assert.equal(effectRuns, 2, "the effect follows its signal");
    assert.equal(cleanups, 1, "the cleanup ran before the effect ran again");
    open.set(false);
    assert.equal(intervals.size, 0, "the timer stopped with the branch");
    assert.equal(cleanups, 2, "the cleanup ran when the branch left");
    count.set(2);
    assert.equal(effectRuns, 2, "a disposed effect no longer runs");
  } finally {
    globalThis.setInterval = realSetInterval;
    globalThis.clearInterval = realClearInterval;
  }
});

test("a key handler answers to one spelling, modifiers included, and a ref reads as its element", () => {
  const { WF, document } = loadRuntime();
  const press = (key, mods = {}) => ({ type: "keydown", key, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, ...mods });
  assert.equal(WF.keyIs(press("k", { ctrlKey: true }), "ctrl+k"), true);
  assert.equal(WF.keyIs(press("k"), "ctrl+k"), false, "the modifier is required");
  assert.equal(WF.keyIs(press("K", { ctrlKey: true, shiftKey: true }), "ctrl+k"), false, "an extra modifier does not match");
  assert.equal(WF.keyIs(press("Escape"), "Escape"), true);
  assert.equal(WF.keyIs(press("Escape"), "esc"), true, "the short names");
  assert.equal(WF.keyIs(press("ArrowDown"), "down"), true);

  const box = WF.ref();
  assert.equal(box.current, null);
  const input = WF.el("input", { ref: box, value: "x" });
  assert.equal(box.current, input, "the handle takes the element once drawn");
  assert.equal(box.value, "x", "a property reads through");
  box.value = "y";
  assert.equal(input.value, "y", "and writes through");
  assert.equal(typeof box.focus, "function", "a method is the element's, bound");
});

test("a persisted signal reads storage first and writes every change; the browser's values follow it", () => {
  const { WF, window } = loadRuntime();
  const store = new Map([["wf:P.draft", JSON.stringify("kept")]]);
  window.localStorage = { getItem: (k) => (store.has(k) ? store.get(k) : null), setItem: (k, v) => store.set(k, v) };
  const draft = WF.persist("P.draft", "");
  assert.equal(draft(), "kept", "what was stored stands in for the initial value");
  draft.set("new");
  assert.equal(store.get("wf:P.draft"), JSON.stringify("new"));
  const fresh = WF.persist("P.other", 3);
  assert.equal(fresh(), 3, "nothing stored: the initial value");
  window.localStorage = { getItem: () => { throw new Error("blocked"); }, setItem: () => { throw new Error("blocked"); } };
  const blocked = WF.persist("P.blocked", true);
  assert.equal(blocked(), true, "blocked storage falls back to the initial value");
  blocked.set(false);
  assert.equal(blocked(), false, "and the signal still works");

  const listeners = {};
  window.addEventListener = (type, fn) => { (listeners[type] = listeners[type] || []).push(fn); };
  window.innerWidth = 800;
  window.innerHeight = 600;
  let seen;
  WF.effect(() => { seen = WF.viewport(); });
  assert.equal(seen.md, true);
  assert.equal(seen.lg, false);
  window.innerWidth = 1100;
  for (const fn of listeners.resize) fn();
  assert.equal(seen.lg, true, "the viewport follows a resize");

  window.location.search = "?tab=two";
  window.location.hash = "#top";
  let q; let h;
  WF.effect(() => { q = WF.query(); h = WF.hash(); });
  assert.equal(q.tab, "two");
  assert.equal(h, "top");
  window.location.search = "?tab=three";
  window.location.hash = "";
  window.history.pushState(null, "", "/x?tab=three");
  assert.equal(q.tab, "three", "a navigation refreshes the query");
  assert.equal(h, "", "and the hash");
});

test("the theme the reader chose is written to the document, kept, and read back", () => {
  const { WF, document, window } = loadRuntime();
  const store = new Map([["wf:theme", "dark"]]);
  window.localStorage = { getItem: (k) => (store.has(k) ? store.get(k) : null), setItem: (k, v) => store.set(k, v), removeItem: (k) => store.delete(k) };
  let seen;
  WF.effect(() => { seen = WF.theme(); });
  assert.equal(seen, "dark", "what was kept");
  assert.equal(document.documentElement.getAttribute("data-theme"), "dark");
  WF.setTheme("light");
  assert.equal(seen, "light");
  assert.equal(document.documentElement.getAttribute("data-theme"), "light");
  assert.equal(store.get("wf:theme"), "light");
  WF.setTheme("system");
  assert.equal(seen, "system");
  assert.equal(document.documentElement.getAttribute("data-theme"), null, "system leaves it to the media query");
  assert.equal(store.has("wf:theme"), false);
  WF.setTheme("purple");
  assert.equal(seen, "system", "an unknown choice is the system's");
});

test("a form handle reports validity and values as the reader types, and a store's async action reports pending", async () => {
  const { WF, document } = loadRuntime();
  const handle = WF.form();
  const email = WF.el("input", { name: "email", value: "" });
  const agree = WF.el("input", { name: "agree", type: "checkbox" });
  const formEl = WF.el("form", { ref: handle }, email, agree);
  formEl.elements = [email, agree];
  formEl.checkValidity = () => email.value.includes("@");
  formEl.dispatchEvent({ type: "input" });
  let seen;
  WF.effect(() => { seen = { valid: handle.valid, values: handle.values }; });
  assert.equal(seen.valid, false);
  assert.deepEqual(seen.values, { email: "", agree: false });
  email.value = "a@b.c";
  agree.checked = true;
  formEl.dispatchEvent({ type: "input" });
  assert.equal(seen.valid, true, "validity follows the input");
  assert.deepEqual(seen.values, { email: "a@b.c", agree: true });

  let release;
  const store = WF.store({
    state: { n: 0 },
    actions: { sync: async (s) => { await new Promise((r) => { release = r; }); s.n = s.n + 1; }, bump: (s) => { s.n = s.n + 1; } },
  });
  assert.equal(typeof store.bump.pending, "undefined", "a plain action has no pending");
  let pending;
  WF.effect(() => { pending = store.sync.pending(); });
  assert.equal(pending, false);
  const call = store.sync();
  assert.equal(pending, true, "pending while the call runs");
  release();
  await call;
  assert.equal(pending, false);
  assert.equal(store.n, 1);
});

test("a page's head tags are written for as long as the page shows, and follow state", () => {
  const { WF, document } = loadRuntime();
  const stale = document.createElement("meta");
  stale.setAttribute("data-wf-head", "");
  document.head.appendChild(stale);
  const img = WF.signal("/a.png");
  const [, dispose] = WF.scoped(() => {
    WF.head([["meta", { property: "og:image", content: () => img() }], ["script", { src: "/x.js", defer: true }]]);
  });
  const tags = document.head.querySelectorAll("[data-wf-head]");
  assert.equal(tags.length, 2, "the static paint's tags are replaced");
  assert.equal(tags[0].getAttribute("content"), "/a.png");
  assert.equal(tags[1].getAttribute("defer"), "");
  img.set("/b.png");
  assert.equal(tags[0].getAttribute("content"), "/b.png", "an attribute follows its signal");
  dispose();
  assert.equal(document.head.querySelectorAll("[data-wf-head]").length, 0, "the page's tags leave with it");
});

test("the runtime's markdown matches the compiler's, and follows its text", () => {
  const { WF, document } = loadRuntime();
  const md = "# Title\n\nA *word* and **more**, `x < y` and [a link](https://x.y) plus ![alt](/i.png).\nSecond line.\n\n- one\n- two\n\n1. first\n2. second\n\n> quoted *text*\n\n---\n\n```js\nlet a = 1 < 2;\n```\n<script>alert(1)</script>";
  assert.equal(
    WF.markdown(md),
    "<h1>Title</h1>\n<p>A <em>word</em> and <strong>more</strong>, <code>x &lt; y</code> and <a href=\"https://x.y\">a link</a> plus <img src=\"/i.png\" alt=\"alt\">.<br>\nSecond line.</p>\n<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n<ol>\n<li>first</li>\n<li>second</li>\n</ol>\n<blockquote>\n<p>quoted <em>text</em></p>\n</blockquote>\n<hr>\n<pre><code class=\"language-js\">let a = 1 &lt; 2;\n</code></pre>\n<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>\n",
  );
  const note = WF.signal("plain");
  const el = WF.el("div", { className: "wf-markdown", markdown: () => note() });
  assert.equal(el.innerHTML, "<p>plain</p>\n");
  note.set("**bold**");
  assert.equal(el.innerHTML, "<p><strong>bold</strong></p>\n", "the render follows the text");
  // A site-relative link or image is addressed from the base path.
  WF.setBasePath("/site/");
  assert.equal(
    WF.markdown("[docs](/docs/x#a) ![i](/i.png) [out](https://a.b/) [rel](other.html)"),
    "<p><a href=\"/site/docs/x#a\">docs</a> <img src=\"/site/i.png\" alt=\"i\"> <a href=\"https://a.b/\">out</a> <a href=\"other.html\">rel</a></p>\n",
  );
  WF.setBasePath("");
});

test("the runtime's highlighter matches the compiler's, and paints a Code element", () => {
  const { WF, document } = loadRuntime();
  assert.equal(
    WF.highlight("page Home(path: \"/\") { // hi\n    state n = 0\n    Button(\"Go\", tone: .primary).lg { style { padding: $md; color: #FF0 } }\n}", "wf"),
    "<span class=\"wf-tok-kw\">page</span> <span class=\"wf-tok-name\">Home</span>(<span class=\"wf-tok-prop\">path</span>: <span class=\"wf-tok-str\">&quot;/&quot;</span>) { <span class=\"wf-tok-cmt\">// hi</span>\n    <span class=\"wf-tok-kw\">state</span> n = <span class=\"wf-tok-num\">0</span>\n    <span class=\"wf-tok-name\">Button</span>(<span class=\"wf-tok-str\">&quot;Go&quot;</span>, <span class=\"wf-tok-prop\">tone</span>: .primary).lg { <span class=\"wf-tok-kw\">style</span> { <span class=\"wf-tok-prop\">padding</span>: <span class=\"wf-tok-tok\">$md</span>; <span class=\"wf-tok-prop\">color</span>: <span class=\"wf-tok-num\">#FF0</span> } }\n}",
  );
  assert.equal(
    WF.highlight("{ \"a\": [1, true], \"b\": \"x\" }", "json"),
    "{ <span class=\"wf-tok-prop\">&quot;a&quot;</span>: [<span class=\"wf-tok-num\">1</span>, <span class=\"wf-tok-kw\">true</span>], <span class=\"wf-tok-prop\">&quot;b&quot;</span>: <span class=\"wf-tok-str\">&quot;x&quot;</span> }",
  );
  assert.equal(
    WF.highlight("$ wf build\n# then\nwf serve", "bash"),
    "<span class=\"wf-tok-prompt\">$ </span>wf build\n<span class=\"wf-tok-cmt\"># then</span>\nwf serve",
  );
  assert.equal(WF.highlight("a < b", "python"), "a &lt; b");
  assert.equal(WF.highlight("t.title", "wf"), "t.title");
  const lang = WF.signal("wf");
  const el = WF.el("code", { className: "wf-code", highlight: { code: "state n = 1", lang: () => lang() } });
  assert.equal(el.innerHTML, "<span class=\"wf-tok-kw\">state</span> n = <span class=\"wf-tok-num\">1</span>");
  lang.set("text");
  assert.equal(el.innerHTML, "state n = 1", "follows the language");
  assert.equal(WF.markdown("```wf\nstate n = 1\n```"), "<pre><code class=\"language-wf\"><span class=\"wf-tok-kw\">state</span> n = <span class=\"wf-tok-num\">1</span>\n</code></pre>\n");
});

test("a scoped slot is drawn again when a value it is handed changes, and not when its fill reads change", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const selected = WF.signal("a");
  const count = WF.signal(1);
  let fills = 0;
  WF.slot(parent, () => ({ item: selected() }), (_s) => {
    fills++;
    const item = _s.item;
    return WF.el("p", {}, [WF.text(() => `${item}:${count()}`)]);
  });
  assert.equal(parent.querySelector("p").textContent, "a:1");
  count.set(2);
  assert.equal(parent.querySelector("p").textContent, "a:2", "the fill's own text follows its signal");
  assert.equal(fills, 1, "a signal the fill reads does not redraw the slot");
  selected.set("b");
  assert.equal(parent.querySelector("p").textContent, "b:2", "a handed value redraws the fill");
  assert.equal(parent.querySelectorAll("p").length, 1);
  assert.equal(fills, 2);
});

test("format and ago speak the document's language, and the i18n locale when there is one", () => {
  const { WF, document } = loadRuntime();
  document.documentElement.lang = "en";
  assert.equal(WF.format(1234.5, "currency"), "$1,234.50");
  assert.equal(WF.format(1234.5, "currency", "EUR"), "€1,234.50");
  assert.equal(WF.format(1234.5), "1,234.5");
  assert.equal(WF.format(1234.5, "integer"), "1,235");
  assert.equal(WF.format(1234.5, "decimal"), "1,234.50");
  assert.equal(WF.format(0.256, "percent", 1), "25.6%");
  assert.equal(WF.format(1234, "compact"), "1.2K");
  assert.equal(WF.format("2024-03-05T14:07:09", "yyyy-MM-dd HH:mm:ss"), "2024-03-05 14:07:09");
  assert.equal(WF.format("2024-03-05", "EEEE, MMMM d, yyyy"), "Tuesday, March 5, 2024");
  assert.equal(WF.format("2024-03-05", "date", "long"), "March 5, 2024");
  assert.equal(WF.format(null, "currency"), "", "nothing formats to nothing");
  assert.equal(WF.ago("2024-03-05T14:00:00", "2024-03-05T14:03:00"), "3 minutes ago");
  assert.equal(WF.ago("2024-03-06T14:05:00", "2024-03-05T14:03:00"), "tomorrow");
  assert.equal(WF.ago("2024-03-05T14:03:10", "2024-03-05T14:03:00"), "now");

  const i18n = WF.locales("de", { de: {}, en: {} });
  assert.equal(WF.format(1234.5, "currency", "EUR"), "1.234,50\u00a0€", "the i18n locale");
  i18n.setLocale("en");
  assert.equal(WF.format(1234.5, "currency", "EUR"), "€1,234.50");
});

test("an enum case with a payload is its name and the payload; a match over one hands the arm the whole value", () => {
  const { WF, document } = loadRuntime();
  assert.equal(WF.caseOf("idle"), "idle");
  assert.equal(WF.caseOf(["failed", "boom"]), "failed");
  assert.equal(WF.payload(["failed", "boom"], "failed"), "boom", "one part is the value itself");
  assert.deepEqual(WF.payload(["done", 2, "two"], "done"), [2, "two"], "more parts are a list");
  assert.equal(WF.payload(["failed", "boom"], "done"), null, "another case has no payload here");
  assert.equal(WF.payload("idle", "idle"), null, "a bare case carries nothing");

  const parent = document.createElement("div");
  const s = WF.signal("idle");
  WF.match(parent, () => WF.caseOf(s()), () => s(), {
    idle: () => WF.el("p", {}, ["idle"]),
    failed: (_v) => { const r = _v[1]; return WF.el("p", {}, [`failed: ${r}`]); },
    done: (_v) => { const n = _v[1]; const l = _v[2]; return WF.el("p", {}, [`${n} ${l}`]); },
  });
  assert.equal(parent.querySelector("p").textContent, "idle");
  s.set(["failed", "boom"]);
  assert.equal(parent.querySelector("p").textContent, "failed: boom");
  s.set(["done", 2, "two"]);
  assert.equal(parent.querySelector("p").textContent, "2 two");
  assert.equal(parent.querySelectorAll("p").length, 1);
});

test("a resource loads, exposes its state, ignores a stale answer, and reloads", async () => {
  const { WF } = loadRuntime();
  const pending = [];
  const realFetch = globalThis.fetch;
  globalThis.fetch = (url) => new Promise((resolve, reject) => pending.push({ url, resolve, reject }));
  try {
    const r = WF.resource("/api/rows", null);
    assert.equal(r.state(), "loading");
    assert.equal(pending[0].url, "/api/rows");
    pending[0].resolve({ ok: true, json: async () => [1, 2] });
    await new Promise((res) => setTimeout(res, 0));
    await new Promise((res) => setTimeout(res, 0));
    assert.equal(r.state(), "ready");
    assert.deepEqual(r.data(), [1, 2]);

    r.reload();
    assert.equal(r.state(), "loading");
    r.reload();
    // The first reload's answer arrives after the second was asked: ignored.
    pending[1].resolve({ ok: true, json: async () => ["stale"] });
    pending[2].resolve({ ok: false, status: 500 });
    await new Promise((res) => setTimeout(res, 0));
    await new Promise((res) => setTimeout(res, 0));
    assert.equal(r.state(), "error");
    assert.equal(r.error().message, "HTTP 500");
    assert.deepEqual(r.data(), [1, 2], "a stale answer never landed");
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("request answers with the parsed body, sends a map body as JSON, and throws on a failed response", async () => {
  const { WF } = loadRuntime();
  const calls = [];
  const realFetch = globalThis.fetch;
  globalThis.fetch = async (url, opts) => {
    calls.push({ url, opts });
    if (url === "/fail") return { ok: false, status: 404 };
    return { ok: true, json: async () => ({ rows: [1, 2] }) };
  };
  try {
    const r = await WF.request("/api/rows", { method: "POST", body: { q: "x" } });
    assert.deepEqual(r, { rows: [1, 2] });
    assert.equal(calls[0].opts.method, "POST");
    assert.equal(calls[0].opts.body, JSON.stringify({ q: "x" }));
    assert.equal(calls[0].opts.headers["Content-Type"], "application/json");
    await assert.rejects(WF.request("/fail"), { message: "HTTP 404" });
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("emit calls the handler the caller passed, and is silent without one", () => {
  const { WF } = loadRuntime();
  const seen = [];
  WF.emit({ on: { pick: (x) => seen.push(x) } }, "pick", 42);
  WF.emit({ on: {} }, "pick", 1);
  WF.emit({}, "pick", 2);
  WF.emit(null, "pick", 3);
  assert.deepEqual(seen, [42]);
});

test("a route with a layout renders the page inside it, with the page's params", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  const shell = (page, params) => WF.el("section", { className: "shell" }, [WF.el("h1", {}, ["Shell"]), page(params)]);
  WF.router(
    [
      { path: "/d/:id", layout: shell, render: (params) => WF.el("p", {}, [`deploy ${params.id}`]) },
      { path: "/", render: () => WF.el("p", {}, ["home"]) },
    ],
    container,
  );
  WF.navigate("/d/42");
  const section = container.querySelector("section");
  assert.ok(section, "the layout wraps the page");
  assert.equal(section.querySelector("p").textContent, "deploy 42");
  WF.navigate("/");
  assert.equal(container.querySelector("section"), null, "a page without a layout has none");
  assert.equal(container.querySelector("p").textContent, "home");
});

test("a route change plays through the View Transitions API where the browser has it", async () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  let started = 0;
  let finish;
  document.startViewTransition = (update) => {
    started++;
    update();
    return { finished: new Promise((r) => { finish = r; }) };
  };
  WF.router(
    [
      { path: "/", render: () => WF.el("p", {}, ["home"]) },
      { path: "/about", render: () => WF.el("p", {}, ["about"]) },
    ],
    container,
    { transition: "slide", duration: "250ms" },
  );
  assert.equal(started, 0, "the first paint is immediate");
  WF.navigate("/about");
  assert.equal(started, 1, "a route change runs as a view transition");
  assert.equal(container.querySelector("p").textContent, "about", "the new page is painted inside it");
  assert.equal(document.documentElement.getAttribute("data-wf-transition"), "slide", "the sheet is told which transition plays");
  finish();
  await Promise.resolve();
  await Promise.resolve();
  assert.equal(document.documentElement.getAttribute("data-wf-transition"), null, "and it is cleared when the transition ends");
});

test("t() picks a plural form by count under the locale's rules", () => {
  const { WF } = loadRuntime();
  const i18n = WF.locales("en", {
    en: { "items.one": "{count} item", "items.other": "{count} items", plain: "hi {name}" },
    ar: { "items.zero": "لا عناصر", "items.one": "عنصر واحد", "items.two": "عنصران", "items.few": "{count} عناصر", "items.many": "{count} عنصرًا", "items.other": "{count} عنصر" },
  });
  assert.equal(i18n.t("items", { count: 1 }), "1 item");
  assert.equal(i18n.t("items", { count: 0 }), "0 items");
  assert.equal(i18n.t("items", { count: 5 }), "5 items");
  assert.equal(i18n.t("plain", { name: "Sam" }), "hi Sam", "a message without forms is itself");
  i18n.setLocale("ar");
  assert.equal(i18n.t("items", { count: 0 }), "لا عناصر", "zero, under Arabic rules");
  assert.equal(i18n.t("items", { count: 2 }), "عنصران", "two");
  assert.equal(i18n.t("items", { count: 5 }), "5 عناصر", "few");
  assert.equal(i18n.t("items", { count: 11 }), "11 عنصرًا", "many");
});

test("a guarded route sends the reader to its redirect until the guard holds", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  const loggedIn = WF.signal(false);
  WF.router(
    [
      { path: "/", render: () => WF.el("p", {}, ["home"]) },
      { path: "/login", render: () => WF.el("p", {}, ["login"]) },
      { path: "/account", guard: () => loggedIn(), redirect: "/login", render: () => WF.el("p", {}, ["account"]) },
    ],
    container,
  );
  WF.navigate("/account");
  assert.equal(container.querySelector("p").textContent, "login", "the guard failed, so the redirect shows");
  loggedIn.set(true);
  WF.navigate("/account");
  assert.equal(container.querySelector("p").textContent, "account", "the guard holds, so the page shows");
});

test("a navigation with a query string routes by its path and keeps the query readable", () => {
  const { WF, document, window } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  WF.router(
    [
      { path: "/", render: () => WF.el("p", {}, ["home"]) },
      { path: "/about", render: () => WF.el("p", {}, ["about"]) },
    ],
    container,
  );
  WF.navigate("/about?tab=x");
  assert.equal(container.querySelector("p").textContent, "about");
  assert.equal(window.location.search, "?tab=x");
  assert.equal(WF.query().tab, "x");
  WF.navigate("/?filter=done#top");
  assert.equal(container.querySelector("p").textContent, "home");
  assert.equal(WF.query().filter, "done");
  const link = document.createElement("a");
  WF.activeLink(link, "/?filter=done", false);
  assert.equal(link.getAttribute("aria-current"), "page", "a link to the route with a query is the current one");
});

test("a dialog opens with its read, and writes back only where it can", () => {
  const { WF, document } = loadRuntime();
  const mk = () => {
    const el = document.createElement("dialog");
    let listeners = {};
    el.showModal = () => { el.open = true; };
    el.close = () => { el.open = false; (listeners.close || []).forEach((fn) => fn()); };
    el.addEventListener = (name, fn) => { (listeners[name] = listeners[name] || []).push(fn); };
    return el;
  };
  const open = WF.signal(false);
  const a = mk();
  WF.dialog(a, () => open(), (v) => open.set(v));
  assert.equal(a.open, false);
  open.set(true);
  assert.equal(a.open, true);
  a.close();
  assert.equal(open(), false, "the browser's close writes the state back");
  // A bare signal still works as the read and the write.
  const b = mk();
  WF.dialog(b, open);
  open.set(true);
  assert.equal(b.open, true);
  b.close();
  assert.equal(open(), false);
  // A condition has no write: closing leaves the state alone.
  const id = WF.signal("x");
  const c = mk();
  WF.dialog(c, () => id() != null, null);
  assert.equal(c.open, true);
  c.close();
  assert.equal(id(), "x");
});

test("a keyed list inserts, removes and moves items without rebuilding the others", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("ul");
  const items = WF.signal([{ id: 1, t: "a" }, { id: 2, t: "b" }, { id: 3, t: "c" }]);
  let built = 0;
  WF.each(parent, () => items(), (item) => { built++; return WF.el("li", {}, [item.t]); }, { key: (item) => item.id });
  const texts = () => parent.children.map((li) => li.textContent);
  const nodes = () => Object.fromEntries(parent.children.map((li) => [li.textContent, li]));
  assert.deepEqual(texts(), ["a", "b", "c"]);
  assert.equal(built, 3);
  const before = nodes();

  // Insert in the middle: the rest keep their nodes.
  items.set([items()[0], { id: 4, t: "d" }, items()[1], items()[2]]);
  assert.deepEqual(texts(), ["a", "d", "b", "c"]);
  assert.equal(built, 4, "only the new item is built");
  assert.equal(nodes().a, before.a);
  assert.equal(nodes().c, before.c);

  // Move: same objects, new order.
  const [a, d, b, c] = items();
  items.set([c, a, b, d]);
  assert.deepEqual(texts(), ["c", "a", "b", "d"]);
  assert.equal(built, 4, "a move builds nothing");
  assert.equal(nodes().c, before.c, "the moved node is the same node");

  // Remove.
  items.set([c, b]);
  assert.deepEqual(texts(), ["c", "b"]);
  assert.equal(built, 4);

  // A changed value under the same key is rebuilt in place.
  items.set([c, { id: 2, t: "B" }]);
  assert.deepEqual(texts(), ["c", "B"]);
  assert.equal(built, 5);
  assert.equal(nodes().c, before.c);
});

test("a keyed list rebuilds an item whose index it reads when its position changes, and tells duplicate keys apart", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("ul");
  const items = WF.signal(["x", "y"]);
  let built = 0;
  WF.each(parent, () => items(), (item, i) => { built++; return WF.el("li", {}, [`${i}:${item}`]); }, { key: (item) => item, index: true });
  const texts = () => parent.children.map((li) => li.textContent);
  assert.deepEqual(texts(), ["0:x", "1:y"]);
  items.set(["y", "x"]);
  assert.deepEqual(texts(), ["0:y", "1:x"], "the index shown follows the position");
  assert.equal(built, 4);

  const warnings = [];
  const warn = console.warn;
  console.warn = (m) => warnings.push(m);
  try {
    items.set(["x", "x", "y"]);
  } finally {
    console.warn = warn;
  }
  assert.deepEqual(texts(), ["0:x", "1:x", "2:y"], "duplicates render, told apart by position");
  assert.equal(warnings.length, 1, "one warning names the shared key");
  assert.match(warnings[0], /"x"/);
});

test("a keyed list plays the exit animation on what leaves and the enter animation only on what arrives", async () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("ul");
  const items = WF.signal([{ id: 1 }, { id: 2 }]);
  WF.each(parent, () => items(), (item) => WF.el("li", {}, [String(item.id)]), { enter: "fadeIn", exit: "fadeOut", key: (item) => item.id });
  const classes = () => parent.children.map((li) => `${li.textContent}:${li.className}`);
  // The fallback timer runs inline in this harness: the enter class has come and gone.
  assert.deepEqual(classes(), ["1:", "2:"]);
  items.set([{ id: 2 }, { id: 3 }]);
  // Item 1 is on its way out; item 2 keeps its node and plays nothing.
  assert.deepEqual(parent.children.map((li) => li.textContent), ["2", "3", "1"]);
  await new Promise((r) => setImmediate(r));
  assert.deepEqual(parent.children.map((li) => li.textContent), ["2", "3"], "the leaver is removed once its animation ends");
});

test("an element marked with an exit animation plays it before its branch is removed", async () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const open = WF.signal(true);
  let inner;
  WF.when(parent, () => open(), () => {
    const card = WF.el("div", { className: "wf-card" });
    inner = WF.el("p", { "data-wf-exit": "fadeOut", "data-wf-delay": "50ms", "data-wf-easing": "ease-out" }, ["x"]);
    card.appendChild(inner);
    return card;
  }, null, null);
  assert.equal(parent.querySelectorAll("p").length, 1);
  assert.equal(inner.style.animationDelay, "50ms", "the delay is set from the marker");
  assert.equal(inner.style.animationTimingFunction, "ease-out", "the easing is set from the marker");
  open.set(false);
  // The exit class is on while the animation plays; the timer runs inline
  // here, so it has already been taken off — but the branch is only removed
  // once the promise settles.
  assert.equal(parent.querySelectorAll("div").length, 1, "still there while leaving");
  await new Promise((r) => setImmediate(r));
  assert.equal(parent.querySelectorAll("div").length, 0, "gone once the exit has played");
});

test("a component's motion markers land on its root, and a reader who asked for less motion waits for nothing", async () => {
  const { WF, document, window } = loadRuntime();
  const frag = document.createDocumentFragment();
  frag.appendChild(WF.el("div", { className: "chip" }));
  WF.mark(frag, { "data-wf-exit": "fadeOut", "data-wf-animate": "fadeIn", "data-wf-duration": "150ms", "data-wf-easing": "linear" });
  const root = frag.childNodes[0];
  assert.equal(root.getAttribute("data-wf-exit"), "fadeOut");
  assert.ok(root.classList.contains("wf-animate-fadeIn"));
  assert.equal(root.style.animationDuration, "150ms");
  assert.equal(root.style.animationTimingFunction, "linear");

  window.matchMedia = () => ({ matches: true });
  const parent = document.createElement("div");
  const open = WF.signal(true);
  WF.when(parent, () => open(), () => WF.el("div", { "data-wf-exit": "fadeOut" }), null, { enter: "fadeIn", exit: "fadeOut" });
  const el = parent.querySelectorAll("div")[0];
  assert.ok(!el.classList.contains("wf-animate-fadeIn"), "no enter animation under reduced motion");
  open.set(false);
  assert.equal(parent.querySelectorAll("div").length, 0, "removed at once under reduced motion");
});

test("a keyed list slides a moved item from where it was", () => {
  const { WF, document, window } = loadRuntime();
  const frames = [];
  window.requestAnimationFrame = (fn) => frames.push(fn);
  globalThis.requestAnimationFrame = window.requestAnimationFrame;
  try {
    const parent = document.createElement("ul");
    const items = WF.signal([{ id: 1 }, { id: 2 }]);
    // Each item sits 20px below the one before it, as a layout would put it.
    WF.each(parent, () => items(), (item) => {
      const li = WF.el("li", {}, [String(item.id)]);
      li.getBoundingClientRect = () => ({ left: 0, top: parent.children.indexOf(li) * 20 });
      return li;
    }, { key: (item) => item.id });
    const [a, b] = parent.children;
    items.set([items()[1], items()[0]]);
    // Measured before the move, each node is held at its old offset until
    // the next frame plays the slide to where it now sits.
    assert.equal(a.style.transform, "translate(0px, -20px)", `${a.style.transform}`);
    assert.equal(b.style.transform, "translate(0px, 20px)");
    assert.equal(frames.length, 2);
    for (const f of frames) f();
    // The timer runs inline here, so the transition has already been taken
    // off with the transform: the node rests where it now sits.
    assert.equal(a.style.transform, "", "the slide plays to the new place");
    assert.equal(a.style.transition, "");
  } finally {
    delete globalThis.requestAnimationFrame;
  }
});

test("a router with a transition plays the old page out and the new one in, then settles focus", async () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  const seen = [];
  const page = (text) => () => {
    const p = WF.el("p", {}, [text]);
    const h = WF.el("h1", {}, [text]);
    h.focus = () => seen.push(`focus ${text}`);
    return WF.el("section", {}, [h, p]);
  };
  WF.router(
    [
      { path: "/", title: "Home", render: page("home") },
      { path: "/about", title: "About", render: page("about") },
    ],
    container,
    { transition: "slide", duration: "150ms" },
  );
  // The first paint is immediate.
  assert.equal(container.querySelector("p").textContent, "home");
  WF.navigate("/about");
  // The old page is on its way out: it is still there, sliding.
  assert.equal(container.querySelector("p").textContent, "home");
  await new Promise((r) => setImmediate(r));
  await new Promise((r) => setImmediate(r));
  assert.equal(container.querySelector("p").textContent, "about");
  await new Promise((r) => setImmediate(r));
  assert.deepEqual(seen, ["focus about"], "focus settles once the new page has arrived");
});
