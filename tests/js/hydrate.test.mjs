//! What the runtime actually does when it takes over an SSG paint.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

/// Load the whole runtime against a fake DOM and hand back its public surface.
function loadRuntime() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const src = fullRuntime();
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

test("an attribute a component's call gives its root is written as on an element", () => {
  const { WF, document } = loadRuntime();
  const given = { "data-x": true, "data-y": false, "data-n": 3, "aria-hidden": false, hidden: true, title: null };
  const element = WF.el("p", given);
  const root = document.createElement("p");
  WF.rootAttrs(root, given);
  const written = (el) => Object.keys(given).map((k) => `${k}=${el.getAttribute(k)}`).join(" ");
  assert.equal(written(root), written(element));
  assert.equal(root.getAttribute("data-x"), "true", "`true` is the word, as `[data-x=\"true\"]` selects");
  assert.equal(root.getAttribute("data-y"), null, "`false` is no attribute");
  assert.equal(root.getAttribute("aria-hidden"), "false", "but an ARIA state keeps it");
  // And as it changes.
  const on = WF.signal(true);
  const live = document.createElement("p");
  WF.rootAttrs(live, { "data-on": () => on() });
  assert.equal(live.getAttribute("data-on"), "true");
  on.set(false);
  assert.equal(live.getAttribute("data-on"), null);
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
  const S = WF.store("S", () => ({
    state: { total: 50 },
    derived: { half: (store) => store.pctOf(25) },
    actions: { pctOf: (store, part) => Math.round((part / store.total) * 100) },
  }));
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

test("a select's value takes once options appended after it arrive", async () => {
  const { WF, document } = loadRuntime();
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
  // What codegen emits: the element first, its options as later statements.
  const select = WF.el("select", { value: "b" });
  select.appendChild(WF.el("option", { value: "a" }, "A"));
  select.appendChild(WF.el("option", { value: "b" }, "B"));
  assert.equal(select.value, "", "nothing to select yet");
  await Promise.resolve();
  assert.equal(select.value, "b", "taken once the options are there");
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

test("a settings page's code and monitor icons draw as glyphs, not as their names", () => {
  const { WF } = loadRuntime();
  for (const name of ["code", "monitor"]) {
    const icon = WF.el("span", { className: "wf-icon", "data-icon": name });
    assert.match(icon.innerHTML, /^<svg [^>]*viewBox="0 0 24 24"[^>]*>.*stroke="currentColor".*<\/svg>$/, name);
  }
  const unknown = WF.el("span", { className: "wf-icon", "data-icon": "not-an-icon" });
  assert.equal(unknown.innerHTML, "not-an-icon", "an unknown name is still its word");
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

test("a tooltip describes its trigger, shows above everything, and is dismissed by Escape", () => {
  const { WF } = loadRuntime();
  const button = WF.el("button", {}, ["Save"]);
  const tip = WF.el("span", { className: "wf-tooltip__text", role: "tooltip", id: "wf-tip-1" }, ["Saves the draft"]);
  const root = WF.el("div", { className: "wf-tooltip" }, [button, tip]);
  WF.tooltip(root, tip, { delay: 0 });
  assert.equal(button.getAttribute("aria-describedby"), "wf-tip-1", "the focusable child is described by the tip");
  assert.equal(root.getAttribute("tabindex"), null, "the wrapper stays out of the tab order");
  assert.ok(tip.classList.contains("wf-layer"), "the tip is a layer");
  assert.equal(tip.getAttribute("data-open"), null, "hidden at first");
  root.dispatchEvent({ type: "pointerenter" });
  assert.equal(tip.getAttribute("data-open"), "", "a hover shows it");
  root.dispatchEvent({ type: "keydown", key: "Escape", stopPropagation() {} });
  assert.equal(tip.getAttribute("data-open"), null, "Escape hides the tip");
  root.dispatchEvent({ type: "pointerenter" });
  root.dispatchEvent({ type: "pointerleave" });
  assert.equal(tip.getAttribute("data-open"), null, "and leaving hides it");

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

test("a class map turns each key on with its condition, and a list joins strings and maps", () => {
  const { WF } = loadRuntime();
  const done = WF.signal(true);
  const open = WF.signal(false);
  const el = WF.el("div", { className: "wf-card" });
  WF.classes(el, () => ["feature", { "is-done": done(), "is-open glow": open() }, open() ? "x" : null, false]);
  assert.equal(el.className, "wf-card feature is-done");

  // A class a script put on is not WebFluent's to take off.
  el.classList.add("from-script");
  open.set(true);
  assert.equal(el.className, "wf-card feature is-done from-script is-open glow x");
  done.set(false);
  open.set(false);
  assert.equal(el.className, "wf-card feature from-script");
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
  const store = WF.store("Sync", () => ({
    state: { n: 0 },
    actions: { sync: async (s) => { await new Promise((r) => { release = r; }); s.n = s.n + 1; }, bump: (s) => { s.n = s.n + 1; } },
  }));
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
  const md = "# Title\n\nA *word* and **more**, `x < y` and [a link](https://x.y) plus ![alt](/i.png).\\\nSecond line,\nthe same paragraph.\n\n- one\n- two\n\n1. first\n2. second\n\n> quoted *text*\n\n---\n\n```js\nlet a = 1 < 2;\n```\n<script>alert(1)</script>";
  assert.equal(
    WF.markdown(md),
    "<h1>Title</h1>\n<p>A <em>word</em> and <strong>more</strong>, <code>x &lt; y</code> and <a href=\"https://x.y\">a link</a> plus <img src=\"/i.png\" alt=\"alt\">.<br>\nSecond line,\nthe same paragraph.</p>\n<ul>\n<li>one</li>\n<li>two</li>\n</ul>\n<ol>\n<li>first</li>\n<li>second</li>\n</ol>\n<blockquote>\n<p>quoted <em>text</em></p>\n</blockquote>\n<hr>\n<pre><code class=\"language-js\">let a = 1 &lt; 2;\n</code></pre>\n<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>\n",
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
  // The raw and block forms are one string each, as in `codegen::highlight`.
  assert.equal(
    WF.highlight("const S = #\"a \"b\" c\"#", "wf"),
    "<span class=\"wf-tok-kw\">const</span> <span class=\"wf-tok-name\">S</span> = <span class=\"wf-tok-str\">#&quot;a &quot;b&quot; c&quot;#</span>",
  );
  assert.equal(
    WF.highlight("x = \"\"\"\n  a \"b\"\n  \"\"\"", "wf"),
    "x = <span class=\"wf-tok-str\">&quot;&quot;&quot;\n  a &quot;b&quot;\n  &quot;&quot;&quot;</span>",
  );
  assert.equal(
    WF.highlight("color: #FF0", "css"),
    "<span class=\"wf-tok-prop\">color</span>: <span class=\"wf-tok-num\">#FF0</span>",
  );
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

/// A response as the browser gives one: a body read as text, and headers.
function ok(body, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    headers: { forEach: () => {} },
    text: async () => (body == null ? "" : JSON.stringify(body)),
  };
}

test("a resource loads, exposes its state, ignores a stale answer, and reloads", async () => {
  const { WF } = loadRuntime();
  const pending = [];
  const realFetch = globalThis.fetch;
  globalThis.fetch = (url) => new Promise((resolve, reject) => pending.push({ url, resolve, reject }));
  try {
    const r = WF.resource("/api/rows", null);
    assert.equal(r.state(), "loading");
    assert.equal(pending[0].url, "/api/rows");
    pending[0].resolve(ok([1, 2]));
    await new Promise((res) => setTimeout(res, 0));
    await new Promise((res) => setTimeout(res, 0));
    assert.equal(r.state(), "ready");
    assert.deepEqual(r.data(), [1, 2]);

    // A reload of what is shown is a refresh: it stays "ready", with the
    // data, while it runs.
    r.reload();
    assert.equal(r.state(), "ready");
    assert.equal(r.refreshing(), true);
    assert.equal(r.pending(), true);
    r.reload();
    // The first reload's answer arrives after the second was asked: ignored.
    pending[1].resolve(ok(["stale"]));
    pending[2].resolve(ok(null, 500));
    await new Promise((res) => setTimeout(res, 0));
    await new Promise((res) => setTimeout(res, 0));
    // A refresh that fails keeps what arrived, and says why.
    assert.equal(r.state(), "ready");
    assert.equal(r.refreshing(), false);
    assert.equal(r.pending(), false);
    assert.equal(r.error().message, "HTTP 500");
    assert.equal(r.error().kind, "status", "and it says what kind of failure it was");
    assert.equal(r.error().status, 500);
    assert.deepEqual(r.data(), [1, 2], "a stale answer never landed");

    // The next answer clears the error.
    r.reload();
    pending[3].resolve(ok([1, 2, 3]));
    await new Promise((res) => setTimeout(res, 0));
    await new Promise((res) => setTimeout(res, 0));
    assert.equal(r.error(), null);
    assert.deepEqual(r.data(), [1, 2, 3]);
  } finally {
    globalThis.fetch = realFetch;
  }
});

/// A server the test answers by hand: `asked` is each request in order,
/// and `answer(i, body)` resolves the i-th.
function heldServer() {
  const asked = [];
  const fetch = (url) => new Promise((resolve) => asked.push({ url, resolve }));
  const answer = async (i, body, status = 200) => {
    // A fresh parse, as a browser's would be.
    asked[i].resolve(ok(body === undefined ? body : JSON.parse(JSON.stringify(body)), status));
    await new Promise((res) => setTimeout(res, 0));
    await new Promise((res) => setTimeout(res, 0));
  };
  return { asked, fetch, answer };
}

test("a first load that fails is the error state, and a reload of it is a first load again", async () => {
  const { WF } = loadRuntime();
  const server = heldServer();
  const realFetch = globalThis.fetch;
  globalThis.fetch = server.fetch;
  try {
    const r = WF.resource("/api/rows", null);
    assert.equal(r.pending(), true);
    assert.equal(r.refreshing(), false, "nothing is shown, so nothing is being refreshed");
    await server.answer(0, null, 500);
    assert.equal(r.state(), "error");
    r.reload();
    assert.equal(r.state(), "loading", "nothing has arrived: it shows loading");
    assert.equal(r.error(), null);
    await server.answer(1, [1]);
    assert.equal(r.state(), "ready");
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("an address that reads state asks again in loading when it changes, and refreshes in ready when it does not", async () => {
  const { WF } = loadRuntime();
  const server = heldServer();
  const realFetch = globalThis.fetch;
  globalThis.fetch = server.fetch;
  try {
    const stage = WF.signal("test");
    const seen = [];
    const r = WF.resource(() => `/api/servers?env=${stage()}`, null);
    WF.effect(() => { seen.push(r.state()); });
    await server.answer(0, ["t1"]);
    r.reload();
    await server.answer(1, ["t1"]);
    assert.deepEqual(seen, ["loading", "ready"], "the refresh never left ready");
    stage.set("prod");
    assert.equal(r.state(), "loading", "another request: what is shown would be wrong");
    assert.equal(server.asked[2].url, "/api/servers?env=prod");
    await server.answer(2, ["p1"]);
    assert.deepEqual(r.data(), ["p1"]);
    assert.deepEqual(seen, ["loading", "ready", "loading", "ready"]);
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("an endpoint's argument that changes is another request; the page `paginate` moves is not", async () => {
  const { WF } = loadRuntime();
  const calls = [];
  const endpoint = (args) => new Promise((resolve) => calls.push({ args, resolve }));
  const kind = WF.signal("deploy");
  const page = WF.signal(1);
  const r = WF.resource(endpoint, {
    call: true,
    args: () => ({ kind: kind(), page: page() }),
    paginate: { by: "page", set: (v) => page.set(v) },
  });
  const tick = () => new Promise((res) => setTimeout(res, 0));
  calls[0].resolve(["a", "b"]);
  await tick();
  assert.equal(r.state(), "ready");
  r.loadMore();
  assert.equal(calls[1].args.page, 2);
  assert.equal(r.state(), "ready", "the next page is asked for with the first still shown");
  assert.equal(r.refreshing(), true);
  r.loadMore();
  assert.equal(calls.length, 2, "a page in flight is not asked for twice");
  calls[1].resolve(["c"]);
  await tick();
  assert.deepEqual(r.items(), ["a", "b", "c"]);
  kind.set("audit");
  assert.equal(r.state(), "loading", "another kind is another request");
});

test("a refresh hands back what did not change as the object it was", async () => {
  const { WF } = loadRuntime();
  const server = heldServer();
  const realFetch = globalThis.fetch;
  globalThis.fetch = server.fetch;
  try {
    const r = WF.resource("/api/board", null);
    const board = {
      count: 2,
      meta: { region: "eu", tags: ["a", "b"] },
      rows: [{ id: 1, name: "a" }, { id: 2, name: "b" }],
      plain: [{ k: "x" }, { k: "y" }],
    };
    await server.answer(0, board);
    const first = r.data();
    let changes = 0;
    WF.effect(() => { r.data(); changes++; });

    r.reload();
    await server.answer(1, board);
    assert.equal(r.data(), first, "an answer that changed nothing is what was there");
    assert.equal(changes, 1, "and nothing that reads it runs again");

    r.reload();
    await server.answer(2, {
      ...board,
      count: 3,
      rows: [{ id: 3, name: "c" }, { id: 2, name: "b" }, { id: 1, name: "A" }],
      plain: [{ k: "y" }, { k: "z" }],
    });
    const next = r.data();
    assert.notEqual(next, first);
    assert.equal(next.count, 3);
    assert.equal(next.meta, first.meta, "an unchanged part is the same object");
    assert.equal(next.rows[1], first.rows[1], "a row is matched by its id, wherever it moved");
    assert.notEqual(next.rows[2], first.rows[0], "a row that changed is a new object");
    assert.equal(next.rows[2].name, "A");
    assert.equal(next.plain[0], first.plain[1], "an item with no id is matched by its value");
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("a live keyed list keeps an item's nodes under its key and tells it its new value and place", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("ul");
  const items = WF.signal([{ id: 1, t: "a" }, { id: 2, t: "b" }]);
  let built = 0;
  // for item, i in items by item.id { Text("{i}: {item.t}") }, as compiled.
  WF.each(parent, () => items(), ($$item, $$index, $item, $i) => {
    built++;
    return WF.el("li", {}, [() => `${$i()}: ${$item().t}`]);
  }, { key: (item) => item.id, index: true, live: true });
  const lis = () => parent.children;
  const [a, b] = lis();
  assert.deepEqual(lis().map((n) => n.textContent), ["0: a", "1: b"]);

  // New objects, as a refreshed response's are; one changed, one moved in front.
  items.set([{ id: 3, t: "c" }, { id: 2, t: "B" }, { id: 1, t: "a" }]);
  assert.deepEqual(lis().map((n) => n.textContent), ["0: c", "1: B", "2: a"]);
  assert.equal(built, 3, "only the new item is built");
  assert.equal(lis()[1], b, "a changed value under the same key keeps its node");
  assert.equal(lis()[2], a, "a moved item keeps its node and is told its index");

  // The same list again draws nothing.
  const list = items();
  items.set([...list]);
  assert.equal(built, 3);
});

test("a match arm reads its value live, and is drawn again only when the arm changes", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const state = WF.signal("loading");
  const data = WF.signal(null);
  const error = WF.signal(null);
  let built = 0;
  let readOnLeaving = [];
  WF.match(parent, () => state(), () => (state() === "error" ? error() : data()), {
    loading: () => WF.el("p", {}, ["…"]),
    // ready(d) { Text("{d.n} rows") }, as compiled.
    ready: ($$value, $d) => {
      built++;
      WF.effect(() => { readOnLeaving.push($d()); });
      return WF.el("p", {}, [() => `${$d().n} rows`]);
    },
    error: ($$value, $e) => WF.el("p", { className: "e" }, [() => $e().message]),
  });
  data.set({ n: 1 });
  state.set("ready");
  const p = parent.querySelector("p");
  assert.equal(p.textContent, "1 rows");
  data.set({ n: 2 });
  assert.equal(parent.querySelector("p"), p, "the same arm, the same node");
  assert.equal(p.textContent, "2 rows");
  assert.equal(built, 1);

  // Leaving for the error arm: what the ready arm reads is still data.
  readOnLeaving = [];
  error.set({ message: "down" });
  state.set("error");
  assert.ok(readOnLeaving.every((v) => v && v.n === 2), "an arm on its way out never sees the next arm's value");
  assert.equal(parent.querySelector("p.e").textContent, "down");
  error.set({ message: "still down" });
  assert.equal(parent.querySelector("p.e").textContent, "still down");
});

test("an else arm stands for every key it answers, and is not drawn again between them", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const key = WF.signal("loading");
  let built = 0;
  WF.match(parent, () => key(), null, {
    ready: () => WF.el("p", {}, ["ready"]),
    else: () => { built++; return WF.el("p", {}, ["waiting"]); },
  });
  key.set("error");
  assert.equal(built, 1);
  key.set("ready");
  key.set("loading");
  assert.equal(built, 2);
});

test("an `if let` reads its value live, and keeps the last one while it leaves", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const picked = WF.signal({ name: "Ada" });
  let built = 0;
  const reads = [];
  // if let p = picked { Text(p.name) }, as compiled.
  WF.when(parent, () => picked() != null, () => {
    built++;
    const $p = WF.held(() => picked());
    WF.effect(() => { reads.push($p()); });
    return WF.el("p", {}, [() => $p().name]);
  }, null, null);
  const p = parent.querySelector("p");
  picked.set({ name: "Bo" });
  assert.equal(parent.querySelector("p"), p);
  assert.equal(p.textContent, "Bo");
  picked.set(null);
  assert.equal(parent.querySelector("p"), null);
  assert.ok(reads.every((v) => v != null), "never null inside the branch");
  assert.equal(built, 1);
});

test("request answers with the parsed body, sends a map body as JSON, and throws on a failed response", async () => {
  const { WF } = loadRuntime();
  const calls = [];
  const realFetch = globalThis.fetch;
  globalThis.fetch = async (url, opts) => {
    calls.push({ url, opts });
    if (url === "/fail") return ok(null, 404);
    return ok({ rows: [1, 2] });
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

test("the language's own types do the same arithmetic in the browser as at build time", () => {
  const { WF } = loadRuntime();
  // The answers are `src/codegen/static_eval.rs`'s, which paints them into
  // the static HTML: a page must not change when it hydrates.
  assert.equal(WF.plus("2026-03-14", { days: 5 }), "2026-03-19");
  assert.equal(WF.plus("2026-01-31", { months: 1 }), "2026-02-28", "a month lands on a real day");
  assert.equal(WF.minus("2026-03-14", { years: 1 }), "2025-03-14");
  assert.equal(WF.weekday("2026-03-14"), 6, "Saturday, counted from Monday");
  assert.equal(WF.year("2026-03-14"), 2026);
  assert.equal(WF.month("2026-03-14"), 3);
  assert.equal(WF.day("2026-03-14"), 14);
  assert.equal(WF.isBefore("2026-03-14", "2026-06-01"), true);
  assert.equal(WF.isAfter("2026-03-14", "2026-06-01"), false);
  assert.equal(WF.until("2026-03-14", "2026-03-15"), 86400000);
  assert.equal(WF.startOfWeek("2026-03-14"), "2026-03-09", "the Monday of its week");
  assert.equal(WF.startOfMonth("2026-03-14"), "2026-03-01");
  assert.equal(WF.plus("09:30", { minutes: 45 }), "10:15", "a time stays a time");
  assert.equal(WF.dateOf("2026-03-14T09:30:00Z"), "2026-03-14");
  assert.equal(WF.timeOf("2026-03-14T09:30:00Z"), "09:30");

  assert.equal(WF.days(259200000), 3);
  assert.equal(WF.hours(5400000), 1.5);

  const price = { amount: 1299, currency: "EUR" };
  assert.deepEqual(WF.times(price, 3), { amount: 3897, currency: "EUR" });
  assert.deepEqual(WF.plus(price, { amount: 1, currency: "EUR" }), { amount: 1300, currency: "EUR" });
  assert.deepEqual(WF.convert(price, 1.1, "USD"), { amount: 1429, currency: "USD" });
  assert.throws(() => WF.plus(price, { amount: 1, currency: "USD" }), /convert/);

  assert.equal(WF.host("https://example.com/a/b?page=2"), "example.com");
  assert.equal(WF.path("https://example.com/a/b?page=2"), "/a/b");
  assert.deepEqual(WF.urlQuery("https://example.com/a?page=2"), { page: "2" });
  assert.equal(WF.urlWith("https://example.com/a?page=2", { query: { page: 3 } }), "https://example.com/a?page=3");
  assert.equal(WF.domain("ada@example.com"), "example.com");

  assert.equal(WF.mix("#0F766E", "#FFFFFF", 0.2), "#3F918B");
  assert.equal(WF.contrast("#0F766E", "#FFFFFF"), 5.47);
  assert.equal(WF.alpha("#0F766E", 0.5), "rgba(15, 118, 110, 0.5)");

  // A value with a method of its own answers for itself: nothing the
  // language adds takes a name away from a record.
  assert.equal(WF.plus({ plus: () => "mine" }, {}), "mine");
});

test("what an if branch, a match arm or a list item made goes when what owns it goes", () => {
  const { WF, document } = loadRuntime();
  const gone = [];
  const parent = document.createElement("div");
  const on = WF.signal(true);
  const [, disposePage] = WF.scoped(() => {
    WF.when(parent, () => on(), () => { WF.onCleanup(() => gone.push("branch")); return WF.el("p", {}, ["x"]); });
    WF.match(parent, () => "ready", () => null, { ready: () => { WF.onCleanup(() => gone.push("arm")); return WF.el("p", {}, ["y"]); } });
    WF.each(parent, () => [1, 2], (n) => { WF.onCleanup(() => gone.push(`item ${n}`)); return WF.el("p", {}, [String(n)]); }, { key: (n) => n });
    WF.each(parent, () => [3], (n) => { WF.onCleanup(() => gone.push(`item ${n}`)); return WF.el("p", {}, [String(n)]); });
  });
  assert.deepEqual(gone, []);
  // A route change disposes the page: everything nested in it goes too.
  disposePage();
  assert.deepEqual(gone.sort(), ["arm", "branch", "item 1", "item 2", "item 3"]);
  // And a branch already gone is not disposed twice.
  on.set(false);
  assert.equal(gone.length, 5);
});

// ─── A branch inside a branch ────────────────────────────────────────────
//
// An `if`, a `match` or a `for` at the top of a branch body is drawn into
// the branch's fragment, so its marker — and what it draws beside the
// marker later — lands in the same parent as the branch's own nodes. The
// branch used to remember the nodes it drew once, when it drew them, and
// to take only those away; what the inner one drew since stayed on the
// page after the branch had gone.

/// A branch body, as the compiler writes one: a fragment the body draws into.
function body(document, draw) {
  const f = document.createDocumentFragment();
  draw(f);
  return f;
}

const texts = (parent, sel) => parent.querySelectorAll(sel).map((n) => n.textContent);

test("an if inside an if-let that drew again goes when the outer branch does", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const picked = WF.signal(null);
  const act = WF.signal(null);
  // if let m = picked { if act == null { Stack(class: "sheet") { … } } }
  WF.when(parent, () => picked() != null, () => body(document, (f) => {
    const m = picked();
    WF.when(f, () => act() == null, () => body(document, (g) => {
      g.appendChild(WF.el("div", { className: "sheet" }, [`member ${m.id}`]));
    }), null, null);
  }), null, null);

  picked.set({ id: 1 });
  assert.deepEqual(texts(parent, ".sheet"), ["member 1"]);
  act.set("remove");
  assert.deepEqual(texts(parent, ".sheet"), [], "the inner branch closed");
  act.set(null);
  assert.deepEqual(texts(parent, ".sheet"), ["member 1"], "and opened again, drawing a new sheet");
  picked.set(null);
  assert.deepEqual(texts(parent, ".sheet"), [], "the outer branch takes the inner one's new sheet with it");
  picked.set({ id: 2 });
  assert.deepEqual(texts(parent, ".sheet"), ["member 2"], "one sheet, not the old one beside the new");
});

test("a match whose list drew again goes when the tab it sits in closes", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const tab = WF.signal("a");
  const state = WF.signal("ready");
  const rows = WF.signal(["one", "two"]);
  // if tab == "a" { match res { ready(xs) { for x in xs { … } } } } else { … }
  WF.when(parent, () => tab() === "a", () => body(document, (f) => {
    WF.match(f, () => state(), () => rows(), {
      loading: () => WF.el("p", { className: "spinner" }, ["…"]),
      ready: (xs) => body(document, (g) => {
        WF.each(g, () => rows(), (x) => WF.el("li", { className: "a" }, [x]), { key: (x) => x });
      }),
    });
  }), () => body(document, (f) => {
    f.appendChild(WF.el("li", { className: "b" }, ["tab b"]));
  }), null);

  assert.deepEqual(texts(parent, "li"), ["one", "two"]);
  // The resource fetches again when the window takes the focus: the arm is
  // drawn again, and so is the list it holds.
  state.set("loading");
  rows.set(["one", "two", "three"]);
  state.set("ready");
  assert.deepEqual(texts(parent, "li"), ["one", "two", "three"]);
  rows.set(["three", "four"]);
  assert.deepEqual(texts(parent, "li"), ["three", "four"]);
  tab.set("b");
  assert.deepEqual(texts(parent, "li"), ["tab b"], "tab a's list goes with tab a");
  tab.set("a");
  assert.deepEqual(texts(parent, "li"), ["three", "four"], "and comes back once");
});

test("an else-if chain shows one arm, whichever drew last", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const a = WF.signal(false);
  const b = WF.signal(true);
  // if a { A } else if b { B } else { C } — the else is another `when`.
  WF.when(parent, () => a(), () => body(document, (f) => f.appendChild(WF.el("p", {}, ["A"]))),
    () => body(document, (f) => {
      WF.when(f, () => b(), () => body(document, (g) => g.appendChild(WF.el("p", {}, ["B"]))),
        () => body(document, (g) => g.appendChild(WF.el("p", {}, ["C"]))), null);
    }), null);
  assert.deepEqual(texts(parent, "p"), ["B"]);
  b.set(false);
  assert.deepEqual(texts(parent, "p"), ["C"]);
  a.set(true);
  assert.deepEqual(texts(parent, "p"), ["A"], "the else arm's latest drawing went with it");
  a.set(false);
  assert.deepEqual(texts(parent, "p"), ["C"]);
});

test("a keyed item whose own if drew again is moved and taken away whole", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("ul");
  const items = WF.signal([{ id: 1, open: WF.signal(false) }, { id: 2, open: WF.signal(false) }]);
  WF.each(parent, () => items(), (item) => body(document, (f) => {
    f.appendChild(WF.el("li", { className: "row" }, [`row ${item.id}`]));
    WF.when(f, () => item.open(), () => body(document, (g) => {
      g.appendChild(WF.el("li", { className: "detail" }, [`detail ${item.id}`]));
    }), null, null);
  }), { key: (item) => item.id });
  const [one, two] = items();
  one.open.set(true);
  assert.deepEqual(texts(parent, "li"), ["row 1", "detail 1", "row 2"]);
  items.set([two, one]);
  assert.deepEqual(texts(parent, "li"), ["row 2", "row 1", "detail 1"], "the detail moves with its row");
  items.set([two]);
  assert.deepEqual(texts(parent, "li"), ["row 2"], "and goes with it");
});

test("a list inside a list item that drew again goes with the item, and an unkeyed rebuild leaves nothing", () => {
  const { WF, document } = loadRuntime();
  for (const key of [(g) => g, undefined]) {
    const parent = document.createElement("div");
    const inner = WF.signal(["x"]);
    const groups = WF.signal(["g1"]);
    WF.each(parent, () => groups(), (g) => body(document, (f) => {
      f.appendChild(WF.el("h3", {}, [g]));
      WF.each(f, () => inner(), (x) => WF.el("p", {}, [`${g}/${x}`]), { key: (x) => x });
    }), key ? { key } : undefined);
    inner.set(["x", "y"]);
    assert.deepEqual(texts(parent, "p"), ["g1/x", "g1/y"]);
    groups.set(["g2"]);
    assert.deepEqual(texts(parent, "p"), ["g2/x", "g2/y"], `the old group's items went (${key ? "keyed" : "unkeyed"})`);
    groups.set([]);
    assert.deepEqual(texts(parent, "p"), []);
    assert.deepEqual(texts(parent, "h3"), []);
  }
});

test("a scoped slot whose fill holds an if that drew again is drawn again cleanly", () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const selected = WF.signal("a");
  const open = WF.signal(false);
  WF.slot(parent, () => ({ item: selected() }), (_s) => body(document, (f) => {
    WF.when(f, () => open(), () => body(document, (g) => g.appendChild(WF.el("p", {}, [_s.item]))), null, null);
  }));
  open.set(true);
  assert.deepEqual(texts(parent, "p"), ["a"]);
  selected.set("b");
  assert.deepEqual(texts(parent, "p"), ["b"], "the fill's inner branch went with the old fill");
});

test("a branch with an exit takes what its inner branch drew since, after the exit and at once on a return", async () => {
  const { WF, document } = loadRuntime();
  const parent = document.createElement("div");
  const outer = WF.signal(true);
  const inner = WF.signal(true);
  WF.when(parent, () => outer(), () => body(document, (f) => {
    f.appendChild(WF.el("h2", {}, ["panel"]));
    WF.when(f, () => inner(), () => body(document, (g) => g.appendChild(WF.el("p", {}, ["on"]))),
      () => body(document, (g) => g.appendChild(WF.el("p", {}, ["off"]))), null);
  }), null, { exit: "fadeOut" });
  inner.set(false);
  assert.deepEqual(texts(parent, "p"), ["off"]);
  outer.set(false);
  assert.deepEqual(texts(parent, "p"), ["off"], "still there while the exit plays");
  await new Promise((r) => setImmediate(r));
  assert.deepEqual(texts(parent, "p"), [], "gone once it has played");
  assert.deepEqual(texts(parent, "h2"), []);

  // Back before the exit is over: the leaving nodes go at once.
  outer.set(true);
  inner.set(false);
  inner.set(true);
  outer.set(false);
  outer.set(true);
  assert.deepEqual(texts(parent, "p"), ["on"]);
  assert.deepEqual(texts(parent, "h2"), ["panel"]);
  await new Promise((r) => setImmediate(r));
  assert.deepEqual(texts(parent, "p"), ["on"], "the cancelled exit removes nothing that is showing");
});

test("a painted page taken over keeps a branch inside a branch whole", () => {
  const { WF, document } = loadRuntime();
  WF.setSsgMode(true);
  const container = document.createElement("main");
  paint(document, container, [
    ["section", { class: "panel" }, [["#text", "Panel"]]],
    ["p", { class: "note" }, [["#text", "open"]]],
    ["footer", {}, [["#text", "end"]]],
  ]);
  const section = container.children[0];
  const outer = WF.signal(true);
  const inner = WF.signal(true);
  WF.hydrate(() => body(document, (frag) => {
    WF.when(frag, () => outer(), () => body(document, (f) => {
      f.appendChild(WF.el("section", { className: "panel" }, ["Panel"]));
      WF.when(f, () => inner(), () => body(document, (g) => g.appendChild(WF.el("p", { className: "note" }, ["open"]))),
        () => body(document, (g) => g.appendChild(WF.el("p", { className: "note" }, ["closed"]))), null);
    }), null, null);
    frag.appendChild(WF.el("footer", {}, ["end"]));
  }), container);
  assert.equal(container.children[0], section, "the painted section is kept");
  inner.set(false);
  assert.deepEqual(container.children.map((n) => n.textContent), ["Panel", "closed", "end"]);
  outer.set(false);
  assert.deepEqual(container.children.map((n) => n.textContent), ["end"], "the painted section and the redrawn note go together");
  outer.set(true);
  assert.deepEqual(container.children.map((n) => n.textContent), ["Panel", "closed", "end"], "and come back before the footer");
});

test("a lifetime on a component's call belongs to its root element", () => {
  const { WF, document } = loadRuntime();
  const frag = document.createDocumentFragment();
  const root = WF.el("span", { className: "chip" });
  frag.appendChild(root);
  const seen = [];
  const [, dispose] = WF.scoped(() => {
    WF.attach(frag, (n) => { seen.push(n.className); return "handle"; }, null, (h) => seen.push(`cleanup ${h}`));
  });
  assert.deepEqual(seen, ["chip"]);
  dispose();
  assert.deepEqual(seen, ["chip", "cleanup handle"]);
});

test("wf:render is said on the document once a route is drawn, with the route", async () => {
  const { WF, document } = loadRuntime();
  const heard = [];
  document.addEventListener("wf:render", (e) => heard.push(e.detail.route));
  const container = document.createElement("div");
  WF.router([
    { path: "/", render: () => WF.el("p", {}, ["home"]) },
    { path: "/about", render: () => WF.el("p", {}, ["about"]) },
  ], container);
  await Promise.resolve();
  WF.navigate("/about");
  await Promise.resolve();
  assert.deepEqual(heard, ["/", "/about"]);
});

test("a persisted value with a key is one stored value per instance, across a reload", () => {
  const store = new Map();
  const storage = { getItem: (k) => (store.has(k) ? store.get(k) : null), setItem: (k, v) => store.set(k, v) };
  // What `persist open = false { key: title }` compiles to, in two panels.
  const first = loadRuntime();
  first.window.localStorage = storage;
  const north = first.WF.persist("Panel.open:" + String("North"), false);
  const south = first.WF.persist("Panel.open:" + String("South"), false);
  north.set(true);
  assert.equal(store.get("wf:Panel.open:North"), "true");
  assert.equal(store.has("wf:Panel.open:South"), false, "the other panel wrote nothing");
  // The next visit: each panel reads its own.
  const again = loadRuntime();
  again.window.localStorage = storage;
  assert.equal(again.WF.persist("Panel.open:North", false)(), true);
  assert.equal(again.WF.persist("Panel.open:South", false)(), false);
  assert.equal(south(), false);
});

// ─── Taking over a pre-rendered page in place ─────────────

/// What a static build hands the browser: markup, painted before any script.
function paint(document, parent, spec) {
  for (const [tag, attrs, kids] of spec) {
    if (tag === "#text") { parent.appendChild(document.createTextNode(attrs)); continue; }
    const el = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs || {})) el.setAttribute(k, v);
    paint(document, el, kids || []);
    parent.appendChild(el);
  }
  return parent;
}

test("a painted page is kept: what nothing holds stays, what a handler holds stands in its place", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  paint(document, container, [["div", { class: "page" }, [
    ["p", { class: "lead" }, [["#text", "Building systems."]]],
    ["button", { class: "wf-btn" }, [["#text", "Add"]]],
  ]]]);
  const page = container.children[0];
  const lead = page.children[0];
  const paintedButton = page.children[1];

  let clicks = 0;
  WF.hydrate(() => {
    const b = WF.el("button", { className: "wf-btn" }, ["Add"]);
    b.addEventListener("click", () => clicks++);
    return WF.el("div", { className: "page" }, [WF.el("p", { className: "lead" }, ["Building systems."]), b]);
  }, container);

  assert.equal(container.children[0], page, "the painted page is the page");
  assert.equal(page.children[0], lead, "its paragraph was never moved or replaced");
  assert.notEqual(page.children[1], paintedButton, "the button a handler holds is the drawn one");
  page.children[1].click();
  assert.equal(clicks, 1, "and it answers");
});

test("text a signal keeps current changes inside the painted element", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  paint(document, container, [["p", { class: "count" }, [["#text", "Count: 0"]]]]);
  const p = container.children[0];
  const count = WF.signal(0);
  WF.hydrate(() => WF.el("p", { className: "count" }, ["Count: ", () => String(count())]), container);
  assert.equal(container.children[0], p, "the paragraph is the painted one");
  assert.equal(p.textContent, "Count: 0");
  count.set(3);
  assert.equal(p.textContent, "Count: 3", "and what it says follows the signal");
});

test("an if and a list draw into the painted parent after the takeover", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  paint(document, container, [["ul", { class: "list" }, [
    ["li", {}, [["#text", "a"]]],
    ["li", {}, [["#text", "b"]]],
  ]], ["p", { class: "note" }, [["#text", "open"]]]]);
  const ul = container.children[0];
  const items = WF.signal(["a", "b"]);
  const open = WF.signal(true);
  WF.hydrate(() => {
    const frag = document.createDocumentFragment();
    const list = WF.el("ul", { className: "list" });
    WF.each(list, () => items(), (it) => WF.el("li", {}, [it]), { key: (it) => it });
    frag.appendChild(list);
    WF.when(frag, () => open(), () => WF.el("p", { className: "note" }, ["open"]));
    return frag;
  }, container);
  assert.equal(container.children[0], ul, "the painted list is kept");
  assert.equal(ul.children.length, 2, "with as many items as the drawing");
  items.set(["a", "b", "c"]);
  assert.deepEqual(ul.children.map((li) => li.textContent), ["a", "b", "c"], "an item added lands in it");
  open.set(false);
  assert.equal(container.querySelector("p"), null, "a branch closed leaves the painted parent");
  open.set(true);
  assert.equal(container.querySelector("p").textContent, "open", "and one opened comes back to it");
});

test("the router draws the first page into the painted main, and the app around it is kept", () => {
  const { WF, document } = loadRuntime();
  const host = document.createElement("div");
  host.id = "app";
  document.body.appendChild(host);
  paint(document, host, [
    ["header", { class: "site" }, [["a", { href: "/", class: "brand" }, [["#text", "Home"]]]]],
    ["main", { id: "wf-main" }, [["h1", { class: "wf-heading" }, [["#text", "Welcome"]]]]],
  ]);
  const [header, main] = host.children;
  const heading = main.children[0];
  WF.setSsgMode(true);
  const app = WF.hydrating(host);
  assert.notEqual(app, host, "a painted page is drawn beside itself");
  app.appendChild(WF.el("header", { className: "site" }, [WF.el("a", { href: "/", className: "brand" }, ["Home"])]));
  const routerEl = document.createElement("main");
  routerEl.id = "wf-main";
  app.appendChild(routerEl);
  WF.router([{ path: "/", title: "Home", render: () => WF.el("h1", { className: "wf-heading" }, ["Welcome"]) }], routerEl);
  assert.deepEqual(host.children, [header, main], "the painted header and main are the page's");
  assert.equal(main.children[0], heading, "the painted heading was kept");
});

test("a painted page that differs from the drawing ends up as the drawing", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  paint(document, container, [
    ["div", { class: "a" }, []],
    ["aside", { class: "stale" }, []],
    ["p", { class: "b old-order" }, [["#text", "old"]]],
  ]);
  WF.hydrate(() => {
    const frag = document.createDocumentFragment();
    frag.appendChild(WF.el("div", { className: "a" }));
    frag.appendChild(WF.el("p", { className: "old-order b" }, ["new"]));
    frag.appendChild(WF.el("span", { className: "c" }, ["added"]));
    return frag;
  }, container);
  const tags = container.children.map((n) => n.tagName.toLowerCase());
  assert.deepEqual(tags, ["div", "p", "span"], "the extra element went, the missing one came");
  assert.equal(container.children[1].textContent, "new", "text the drawing has wins");
  assert.equal(container.children[2].textContent, "added");
});

test("an element the paint lacks, of the same tag as the next one, does not take that one's place", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  // A drawer draws a scrim and a toggle the static paint has no use for;
  // the scrim is a div, as is the article after it.
  paint(document, container, [["div", { class: "row" }, [
    ["aside", { class: "wf-sidebar" }, []],
    ["div", { class: "wf-stack docs-main" }, [["h1", {}, [["#text", "State"]]]]],
    ["div", { class: "wf-stack rail" }, []],
  ]]]);
  const row = container.children[0];
  const [aside, article, rail] = row.children;
  const heading = article.children[0];
  WF.hydrate(() => WF.el("div", { className: "row" }, [
    WF.el("aside", { className: "wf-sidebar" }),
    WF.el("div", { className: "wf-sidebar__scrim" }),
    WF.el("button", { className: "wf-sidebar__toggle" }),
    WF.el("div", { className: "wf-stack docs-main" }, [WF.el("h1", {}, ["State"])]),
    WF.el("div", { className: "wf-stack rail" }),
  ]), container);
  assert.deepEqual(row.children.map((n) => n.className), ["wf-sidebar", "wf-sidebar__scrim", "wf-sidebar__toggle", "wf-stack docs-main", "wf-stack rail"]);
  assert.equal(row.children[0], aside);
  assert.equal(row.children[3], article, "the article is the painted one");
  assert.equal(article.children[0], heading, "and so is its heading");
  assert.equal(row.children[4], rail);
});

test("what an if or a keyed list drew is the painted node, and is taken away and moved as one", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("main");
  paint(document, container, [
    ["h1", { class: "title" }, [["#text", "Button"]]],
    ["ul", {}, [["li", {}, [["#text", "a"]]], ["li", {}, [["#text", "b"]]], ["li", {}, [["#text", "c"]]]]],
  ]);
  const title = container.children[0];
  const ul = container.children[1];
  const [a, b, c] = ul.children;
  const entry = WF.signal({ name: "Button" });
  const items = WF.signal(["a", "b", "c"]);
  WF.hydrate(() => {
    const frag = document.createDocumentFragment();
    WF.when(frag, () => entry(), () => WF.el("h1", { className: "title" }, [entry().name]));
    const list = WF.el("ul", {});
    WF.each(list, () => items(), (it) => WF.el("li", {}, [it]), { key: (it) => it });
    frag.appendChild(list);
    return frag;
  }, container);
  assert.equal(container.querySelector("h1"), title, "the heading an `if let` drew is the painted one");
  assert.deepEqual(ul.children, [a, b, c], "so are the list's items");
  items.set(["c", "a"]);
  assert.deepEqual(ul.children, [c, a], "a keyed change moves and takes away the painted items");
  entry.set(null);
  assert.equal(container.querySelector("h1"), null, "a branch that closes takes the painted heading with it");
});

test("a script loaded after the page is asked for once the page has loaded, not before", () => {
  const { WF, document, window } = loadRuntime();
  WF.loadAfter([
    { src: "https://tags.example/t.js", integrity: "sha384-abc" },
    { src: "/widget.js" },
  ]);
  const scripts = () => document.head.childNodes.filter((n) => n.tagName === "SCRIPT");
  assert.equal(scripts().length, 0, "nothing is fetched while the page loads");
  window._fire("load", {});
  const [tag, widget] = scripts();
  assert.equal(tag.src, "https://tags.example/t.js");
  assert.equal(tag.integrity, "sha384-abc");
  assert.equal(tag.crossOrigin, "anonymous");
  assert.equal(tag.async, true);
  assert.equal(widget.src, "/widget.js", "a site-relative one is the site's");
});
