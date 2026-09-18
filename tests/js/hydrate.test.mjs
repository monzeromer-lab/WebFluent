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
  const fn = new Function("window", "document", "Node", `${src}\nreturn WF;`);
  return { WF: fn(window, document, Node), document };
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
