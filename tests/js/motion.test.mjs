//! The motion engine, which runs on the Web Animations API.
//!
//! The old engine added a CSS class and took it off again around
//! `animationend`, with a `setTimeout` guessing the duration. What that
//! could not do is what these assert: an animation that is interrupted,
//! a spring, a box that opens to the height of its content, a number that
//! counts, an element that waits to be scrolled to — and, over all of it,
//! a reader who asked for less motion getting none.
//!
//! Run: node --test tests/js/motion.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { runtimeFor } from "./runtime.mjs";

/// The runtime, with the globals the motion engine reaches for.
function run({ reducedMotion = false, tokens = {} } = {}) {
  const dom = makeDom();
  const { window, document } = dom;
  for (const [name, value] of Object.entries(tokens)) window.tokens.set(name, value);
  window.matchMedia = (query) => ({
    matches: reducedMotion && query.includes("prefers-reduced-motion"),
    addEventListener() {},
  });
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "getComputedStyle", "IntersectionObserver", "requestAnimationFrame",
    `${runtimeFor("show")}\nreturn WF;`,
  );
  const WF = fn(
    window, document, dom.Node, dom.Element, dom.DocumentFragment,
    (f) => { f(); return 0; },
    window.getComputedStyle, dom.IntersectionObserver,
    // A frame is a real turn of the clock: a counting number reads the
    // time, and would never arrive if no time passed.
    (f) => globalThis.setTimeout(f, 1),
  );
  return { WF, ...dom };
}

/// Let every pending microtask run — an animation finishes on one.
const settle = () => new Promise((r) => setTimeout(r, 0));

test("an animation that is interrupted gives way to the one that replaced it", async () => {
  const { WF, document } = run();
  const el = document.createElement("div");
  el._holdAnimations = true;

  WF.animateIn(el, "fadeIn", "300ms");
  const [first] = el.getAnimations();
  assert.equal(first.id, "wf-fadeIn", "the engine names what it started");
  assert.equal(first.playState, "running");

  // The card came back before it had finished leaving.
  WF.animateIn(el, "slideUp", "300ms");
  assert.equal(first.playState, "idle", "the animation it replaced was cancelled");
  const running = el.getAnimations();
  assert.equal(running.length, 1, "one animation on the element, not two fighting");
  assert.equal(running[0].id, "wf-slideUp");
});

test("an animation the page did not start is left alone", () => {
  const { WF, document } = run();
  const el = document.createElement("div");
  el._holdAnimations = true;
  const mine = el.animate([{ opacity: 0 }], { duration: 100 });

  WF.animateIn(el, "fadeIn");
  assert.equal(mine.playState, "running", "only `wf-` animations are the engine's to cancel");
  assert.equal(el.getAnimations().length, 2);
});

test("a spring is sampled into an easing the platform can run", () => {
  const { WF, document } = run();
  const el = document.createElement("div");
  el._holdAnimations = true;
  WF.animateIn(el, "fadeIn", "300ms", null, "spring");
  const { easing } = el.getAnimations()[0].effect.options;
  assert.match(easing, /^linear\(/, "a spring overshoots, so it is not a cubic Bézier");
  const points = easing.slice("linear(".length, -1).split(", ").map(Number);
  assert.equal(points[0], 0, "it starts where it starts");
  assert.equal(points.at(-1), 1, "and settles exactly where it is going");
  assert.ok(Math.max(...points) > 1, "having gone past it first");
});

test("a named easing is the design token, resolved", () => {
  const { WF, document } = run({ tokens: { "--ease-spring": "linear(0, 0.5, 1)" } });
  const el = document.createElement("div");
  el._holdAnimations = true;
  // `easing: .spring` compiles to `var(--ease-spring)`, which the Web
  // Animations API does not take.
  WF.animateIn(el, "fadeIn", "300ms", null, "var(--ease-spring)");
  assert.equal(el.getAnimations()[0].effect.options.easing, "linear(0, 0.5, 1)");
});

test("a duration the author did not give comes from the theme", () => {
  const { WF, document } = run({ tokens: { "--animation-duration-normal": "180ms" } });
  const el = document.createElement("div");
  el._holdAnimations = true;
  WF.animateIn(el, "fadeIn");
  assert.equal(el.getAnimations()[0].effect.options.duration, 180);
  WF.animateIn(document.createElement("p"), "fadeIn", "0.4s");
  assert.equal(el.getAnimations()[0].effect.options.duration, 180);
});

test("expanding animates to the height the content actually has", async () => {
  const { WF, document } = run();
  const el = document.createElement("div");
  el._holdAnimations = true;
  el._height = 0;
  el.scrollHeight = 140;

  WF.expand(el, true, "200ms");
  const [open] = el.getAnimations();
  assert.deepEqual(
    open.effect.frames,
    [{ height: "0px" }, { height: "140px" }],
    "CSS cannot animate to `auto`; this measures what auto would be",
  );
  open.finish();
  await settle();
  assert.equal(el.style.height, "", "and hands the height back, so the box grows with it");

  el._height = 140;
  WF.expand(el, false, "200ms");
  const closing = el.getAnimations().at(-1);
  assert.deepEqual(closing.effect.frames, [{ height: "140px" }, { height: "0px" }]);
  closing.finish();
  await settle();
  assert.equal(el.style.height, "0px");
});

test("`.expand` as an animation name is the measured one, and its exit closes", () => {
  const { WF, document } = run();
  const el = document.createElement("div");
  el._holdAnimations = true;
  el.scrollHeight = 60;
  WF.animateIn(el, "expand", "100ms");
  assert.deepEqual(el.getAnimations()[0].effect.frames, [{ height: "0px" }, { height: "60px" }]);

  const out = document.createElement("div");
  out._holdAnimations = true;
  out._height = 60;
  WF.animateOut(out, "expand", "100ms");
  assert.deepEqual(out.getAnimations()[0].effect.frames, [{ height: "60px" }, { height: "0px" }]);
});

test("a number counts to where it has got to, formatted the whole way", async () => {
  const { WF, document } = run();
  const node = document.createElement("span");
  const total = WF.signal(10);
  WF.counted(node, total, "30ms", (n) => `$${Math.round(n)}`);
  // The first value is written, not counted to: a page does not open by
  // counting up from nothing.
  assert.equal(node.textContent, "$10");

  total.set(50);
  await new Promise((r) => globalThis.setTimeout(r, 80));
  assert.equal(node.textContent, "$50", "and arrives at the new one");
});

test("an element that waits to be scrolled to does not animate until it is", () => {
  const { WF, document, seen } = run();
  const el = document.createElement("section");
  el._holdAnimations = true;

  WF.onEnterView(el, "fadeIn", "300ms");
  assert.equal(el.style.opacity, "0", "until it is seen it is not there");
  assert.equal(el.getAnimations().length, 0, "and nothing has played");

  seen(el);
  assert.equal(el.style.opacity, "", "it is handed back to the stylesheet");
  assert.equal(el.getAnimations()[0].id, "wf-fadeIn");

  seen(el);
  assert.equal(el.getAnimations().length, 1, "and it does not play again");
});

test("a shared name is what the browser carries across a route change", () => {
  const { WF, document } = run();
  const el = document.createElement("img");
  WF.shared(el, "cover-12");
  assert.equal(el.style.viewTransitionName, "cover-12");
  // A view transition name is an identifier, not a string.
  WF.shared(el, "cover 12/x");
  assert.equal(el.style.viewTransitionName, "cover-12-x");
});

test("a handle plays once, and replays from the start", async () => {
  const { WF, document } = run();
  const el = document.createElement("div");
  el._holdAnimations = true;

  const handle = WF.animate(el, "shake", "200ms");
  assert.equal(el.getAnimations().length, 1, "asking about it does not play it twice");
  handle.cancel();
  assert.equal(el.getAnimations().length, 0);

  handle.play();
  assert.equal(el.getAnimations()[0].id, "wf-shake");
  WF.replay(el, "shake", "200ms");
  assert.equal(el.getAnimations().length, 1, "a replay restarts rather than stacking");
});

test("a reader who asked for less motion gets none of it", async () => {
  const { WF, document } = run({ reducedMotion: true });
  const el = document.createElement("div");
  el._holdAnimations = true;
  el.scrollHeight = 200;

  await WF.animateIn(el, "fadeIn", "300ms");
  assert.equal(el.getAnimations().length, 0, "nothing was started");
  await WF.expand(el, true, "300ms");
  assert.equal(el.getAnimations().length, 0);
  assert.equal(el.style.height, "", "the box is simply open");

  const node = document.createElement("span");
  await WF.countTo(node, 0, 99, "600ms", null);
  assert.equal(node.textContent, "99", "the number is simply there");
});

test("a page that never paints still gets the end state", async () => {
  const { WF, document } = run();
  const el = document.createElement("div");
  // An animation that is never given a frame: a background tab, a
  // prerender, a headless browser. Nothing may wait on it forever.
  el._holdAnimations = true;
  el.scrollHeight = 120;

  // The box is open the moment it is asked to be, animation or not.
  WF.expand(el, true, "20ms");
  assert.equal(el.style.height, "", "open is the stylesheet's height, at once");
  WF.expand(el, false, "20ms");
  assert.equal(el.style.height, "0px");

  // And whatever is waiting on a play — a route change, a branch being
  // removed — is released on the clock.
  const started = Date.now();
  await WF.animateIn(el, "fadeIn", "20ms");
  assert.ok(Date.now() - started < 2000, "it did not wait for a frame that never came");
  assert.equal(el.getAnimations().length, 0, "and the animation was taken off the element");
});

test("a number arrives even where no frame ever comes", async () => {
  const dom = makeDom();
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "getComputedStyle", "IntersectionObserver", "requestAnimationFrame",
    `${runtimeFor("show")}\nreturn WF;`,
  );
  const WF = fn(
    dom.window, dom.document, dom.Node, dom.Element, dom.DocumentFragment,
    (f, ms) => globalThis.setTimeout(f, ms),
    dom.window.getComputedStyle, dom.IntersectionObserver,
    // A page that paints nothing: the frame is never handed back.
    () => 0,
  );
  const node = dom.document.createElement("span");
  const total = WF.signal(0);
  WF.counted(node, total, "20ms", null);
  total.set(42);
  await new Promise((r) => globalThis.setTimeout(r, 120));
  assert.equal(node.textContent, "42");
});

test("`show` with `.expand` opens the box itself", async () => {
  const { WF, document } = run();
  const parent = document.createElement("div");
  const open = WF.signal(false);
  WF.show(parent, open, () => [document.createElement("p")], { enter: "expand" });

  const [wrapper] = parent.children;
  wrapper._holdAnimations = true;
  assert.equal(wrapper.style.display, "block", "`display: contents` has no height to animate");
  assert.equal(wrapper.style.height, "0px");

  wrapper.scrollHeight = 90;
  open.set(true);
  assert.deepEqual(wrapper.getAnimations()[0].effect.frames, [{ height: "0px" }, { height: "90px" }]);
});

test("an animation the page never painted settles at its end, not its start", async () => {
  const { WF, document } = run();
  const el = document.createElement("div");
  // A background tab, a prerender, a headless browser: the animation is
  // created and never gets a frame.
  el._holdAnimations = true;

  const done = WF.animateIn(el, "fadeIn", "20ms");
  const [animation] = el.getAnimations();
  assert.equal(animation.playState, "running", "started, and waiting for a frame");

  await done;
  assert.equal(animation.committed, true, "the end state was written to the element");
  assert.equal(animation.playState, "idle", "and the animation was taken off it");
});
