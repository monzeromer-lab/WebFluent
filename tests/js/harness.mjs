//! Load a bundle the compiler actually produced, and run it.
//!
//! `hydrate.test.mjs` exercises `runtime.js` on hand-written input. This goes one
//! step further: it takes the `app.js` a real `wf build` wrote for a real project
//! and executes it against the fake DOM, so the assertions are about the page a
//! user would get — the codegen and the runtime together, not either alone.

import { readFileSync } from "node:fs";
import { makeDom } from "./dom.mjs";

/// Where `tests/e2e_sites.rs` builds the fixture projects.
export function siteBundle(name) {
  const url = new URL(`../../target/e2e/${name}/build/app.js`, import.meta.url);
  return readFileSync(url, "utf8");
}

export function sitePage(name, rel = "index.html") {
  const url = new URL(`../../target/e2e/${name}/build/${rel}`, import.meta.url);
  return readFileSync(url, "utf8");
}

/// Execute a built bundle against a fresh DOM and hand back what it painted.
///
/// `path` seeds `window.location.pathname` before the bundle runs, so a router
/// build can be asked for any of its routes.
export function mountSite(name, { path = "/" } = {}) {
  const { window, document, Node, DocumentFragment } = makeDom();

  const app = document.createElement("div");
  app.id = "app";
  document.body.appendChild(app);

  window.location.pathname = path;

  // Timers run inline: the runtime uses setTimeout only to defer work by a tick,
  // and a test wants the settled page, not a pending one.
  const timers = [];
  window.setTimeout = (fn) => { timers.push(fn); return timers.length; };
  window.clearTimeout = () => {};

  const src = siteBundle(name);
  const run = new Function(
    "window", "document", "Node", "DocumentFragment", "setTimeout",
    "clearTimeout", "URLSearchParams", "console",
    `${src}\nreturn typeof WF !== "undefined" ? WF : null;`,
  );

  const WF = run(
    window, document, Node, DocumentFragment,
    window.setTimeout, window.clearTimeout, URLSearchParams, console,
  );

  // Drain whatever the mount deferred.
  while (timers.length) timers.shift()();

  return { WF, window, document, app, drain: () => { while (timers.length) timers.shift()(); } };
}

// ─── Queries ────────────────────────────────────────────────────────────

/// Every element under `root`, in document order.
export function allElements(root) {
  return root.all();
}

/// Elements carrying `cls`.
export function byClass(root, cls) {
  return root.querySelectorAll(`.${cls}`);
}

/// Elements with the given tag name.
export function byTag(root, tag) {
  return root.querySelectorAll(tag);
}

/// The first element whose text content contains `text`.
export function findByText(root, text) {
  return root.all().find((el) => el.textContent.includes(text)) || null;
}

/// The first `<button>` whose label contains `text`.
export function button(root, text) {
  return root.querySelectorAll("button").find((b) => b.textContent.includes(text)) || null;
}

/// Fire a click, then let anything the handler deferred run.
export function click(el, ctx) {
  if (!el) throw new Error("click() on a missing element");
  el.dispatchEvent({ type: "click", target: el, preventDefault() {} });
  ctx?.drain?.();
}

/// Type into a bound input: set the value and fire the event the binding listens for.
export function type(el, value, ctx, event = "input") {
  if (!el) throw new Error("type() into a missing element");
  el.value = value;
  el.dispatchEvent({ type: event, target: el, preventDefault() {} });
  ctx?.drain?.();
}
