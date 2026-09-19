//! What the runtime actually does when it takes over an SSG paint.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { makeDom } from "./dom.mjs";

/// Load runtime.js against a fake DOM and hand back its public surface.
function loadRuntime() {
  const { window, document, Node } = makeDom();
  const src = readFileSync(new URL("../../src/runtime/runtime.js", import.meta.url), "utf8");
  // Timers run inline: the runtime defers by a tick, and a test wants the
  // settled state.
  const setTimeout = (fn) => { fn(); return 0; };
  const fn = new Function("window", "document", "Node", "setTimeout", `${src}\nreturn WF;`);
  return { WF: fn(window, document, Node, setTimeout), document, window };
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
    const b = WF.h("button", {}, ["Add"]);
    b.addEventListener("click", () => clicks++);
    const root = WF.h("div", {}, [b]);
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
  WF.hydrate(() => WF.h("div", {}, [() => String(count())]), container);
  count.set(5);
  assert.match(container.textContent, /5/, "the visible DOM must follow the signal");
});

test("mount replaces whatever was there", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("div");
  container.appendChild(document.createElement("span"));
  WF.mount(() => WF.h("p", {}, ["fresh"]), container);
  assert.equal(container.textContent, "fresh");
});

test("activeLink marks the link to the current route, and follows navigation", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("div");
  const home = WF.h("a", { href: "/" }, ["Home"]);
  const docs = WF.h("a", { href: "/docs" }, ["Docs"]);
  const guide = WF.h("a", { href: "/docs/guide" }, ["Guide"]);
  // Links are built before the router exists, as an app shell's navbar is.
  WF.activeLink(home, "/", false);
  WF.activeLink(docs, "/docs", true);
  WF.activeLink(guide, "/docs/guide", false);

  assert.equal(home.getAttribute("aria-current"), "page");
  assert.ok(home.classList.contains("active"));
  assert.equal(docs.getAttribute("aria-current"), null);

  WF.createRouter([{ path: "*", render: () => WF.h("p", {}, ["page"]) }], container);
  WF.navigate("/docs/guide");

  assert.equal(home.getAttribute("aria-current"), null, "home is no longer current");
  assert.ok(!home.classList.contains("active"));
  assert.equal(guide.getAttribute("aria-current"), "page", "exact match");
  assert.equal(docs.getAttribute("aria-current"), "page", "prefix match on the section root");
});

test("an aria-* attribute keeps a false value, and follows a reactive one", () => {
  const { WF } = loadRuntime();
  const pressed = WF.signal(false);
  const chip = WF.h("button", { "aria-pressed": () => pressed(), "data-tone": "info", hidden: false });
  assert.equal(chip.getAttribute("aria-pressed"), "false", "false is a real ARIA state");
  assert.equal(chip.getAttribute("data-tone"), "info");
  assert.equal(chip.getAttribute("hidden"), null, "a false plain attribute is absent");
  pressed.set(true);
  assert.equal(chip.getAttribute("aria-pressed"), "true");
});

test("the router sets the document title to the page it shows", () => {
  const { WF, document } = loadRuntime();
  const container = document.createElement("div");
  WF.createRouter(
    [
      { path: "/", title: "Home", render: () => WF.h("p", {}, ["home"]) },
      { path: "/docs", title: "Routing rules", render: () => WF.h("p", {}, ["docs"]) },
    ],
    container,
  );
  assert.equal(document.title, "Home");
  WF.navigate("/docs");
  assert.equal(document.title, "Routing rules");
});

test("a store's derived value may call one of its actions", () => {
  const { WF } = loadRuntime();
  const S = WF.createStore({
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
  const select = WF.h(
    "select",
    { value: () => field() },
    WF.h("option", { value: "region" }, "region"),
    WF.h("option", { value: "status" }, "status"),
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
  WF.h(
    "button",
    { className: "wf-icon-btn", "data-icon": "close", "aria-label": "Close" },
    WF.h("span", { className: "wf-icon", "data-icon": "close" }),
  );
  assert.equal(draws, 1, "one glyph for the button and its icon span");
  WF.h("span", { className: "wf-icon", "data-icon": "close" });
  assert.equal(draws, 2, "a lone icon still draws");
});

test("a route change moves focus to the new page's heading, resets the scroll and announces the title", () => {
  const { WF, document, window } = loadRuntime();
  const container = document.createElement("main");
  document.body.appendChild(container);
  WF.createRouter(
    [
      { path: "/", title: "Home", render: () => WF.h("div", {}, [WF.h("h1", {}, ["Home"])]) },
      { path: "/docs", title: "Docs", render: () => WF.h("div", {}, [WF.h("h1", {}, ["Docs"])]) },
      { path: "/bare", title: "Bare", render: () => WF.h("p", {}, ["no heading"]) },
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

test("a carousel is a labelled region whose slides are groups, with named controls and pausable rotation", () => {
  const { WF, document, window } = loadRuntime();
  const intervals = [];
  const timers = { setInterval, clearInterval };
  globalThis.setInterval = (fn, ms) => { intervals.push({ fn, ms }); return intervals.length; };
  globalThis.clearInterval = (id) => { intervals[id - 1].cleared = true; };
  try {
  const track = WF.h("div", { className: "wf-carousel__track" }, [
    WF.h("div", { className: "wf-carousel__slide" }, ["one"]),
    WF.h("div", { className: "wf-carousel__slide" }, ["two"]),
    WF.h("div", { className: "wf-carousel__slide", "aria-label": "Team photo" }, ["three"]),
  ]);
  const root = WF.h("div", { className: "wf-carousel" }, [track]);
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
  const root = WF.h("div", { className: "wf-carousel" }, [
    WF.h("div", { className: "wf-carousel__track" }, [
      WF.h("div", { className: "wf-carousel__slide" }, ["a"]),
      WF.h("div", { className: "wf-carousel__slide" }, ["b"]),
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
  const button = WF.h("button", {}, ["Save"]);
  const tip = WF.h("span", { className: "wf-tooltip__text", role: "tooltip", id: "wf-tip-1" }, ["Saves the draft"]);
  const root = WF.h("div", { className: "wf-tooltip" }, [button, tip]);
  WF.tooltip(root, tip);
  assert.equal(button.getAttribute("aria-describedby"), "wf-tip-1", "the focusable child is described by the tip");
  assert.equal(root.getAttribute("tabindex"), null, "the wrapper stays out of the tab order");
  root.dispatchEvent({ type: "keydown", key: "Escape", stopPropagation() {} });
  assert.equal(root.getAttribute("data-dismissed"), "", "Escape hides the tip");
  root.dispatchEvent({ type: "mouseleave" });
  assert.equal(root.getAttribute("data-dismissed"), null, "and leaving resets it");

  // With nothing focusable inside, the wrapper itself becomes the trigger.
  const tip2 = WF.h("span", { className: "wf-tooltip__text", role: "tooltip", id: "wf-tip-2" }, ["Hint"]);
  const plain = WF.h("div", { className: "wf-tooltip" }, [WF.h("span", {}, ["term"]), tip2]);
  WF.tooltip(plain, tip2);
  assert.equal(plain.getAttribute("tabindex"), "0");
  assert.equal(plain.getAttribute("aria-describedby"), "wf-tip-2");
});

test("a menu's items are menuitems the arrow keys move between, and choosing one closes it", () => {
  const { WF, document } = loadRuntime();
  const open = WF.signal(false);
  const trigger = WF.h("button", { "aria-expanded": () => (open() ? "true" : "false") }, ["Actions"]);
  const edit = WF.h("li", { className: "wf-menu__item" }, ["Edit"]);
  const remove = WF.h("li", { className: "wf-menu__item" }, ["Delete"]);
  const list = WF.h("ul", { role: "menu" }, [edit, WF.h("li", { className: "wf-menu__divider" }), remove]);
  const root = WF.h("div", { className: "wf-menu" }, [trigger, list]);
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
  const input = WF.h("input", { className: "wf-input", type: "text" });
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
  const el = WF.h("div", { className: "wf-card wf-s0123abcd" });
  WF.classes(el, () => tone());
  assert.equal(el.className, "wf-card wf-s0123abcd calm wide");

  tone.set("alert");
  assert.equal(el.className, "wf-card wf-s0123abcd alert", "what it no longer names comes off");
  tone.set("");
  assert.equal(el.className, "wf-card wf-s0123abcd");
});
