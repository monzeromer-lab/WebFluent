//! Load a bundle the compiler actually produced, and run it.
//!
//! `hydrate.test.mjs` exercises `runtime.js` on hand-written input. This goes one
//! step further: it takes the `app.js` a real `wf build` wrote for a real project
//! and executes it against the fake DOM, so the assertions are about the page a
//! user would get — the codegen and the runtime together, not either alone.

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { makeDom } from "./dom.mjs";

/// Where `tests/e2e_sites.rs` builds the fixture projects.
///
/// A build writes app.js and one chunk per page under pages/; a browser runs
/// them in that order in one global scope, so they are joined the same way.
export function siteBundle(name) {
  const build = new URL(`../../target/e2e/${name}/build/`, import.meta.url);
  let src = readFileSync(new URL("app.js", build), "utf8");
  const pages = new URL("pages/", build);
  if (existsSync(pages)) {
    for (const file of readdirSync(pages).sort()) {
      if (file.endsWith(".js")) src += "\n" + readFileSync(new URL(file, pages), "utf8");
    }
  }
  return src;
}

/// Every rule a site ships: the shared sheet and each page's own, joined as a
/// browser that has visited every page would hold them.
export function siteStyles(name) {
  const build = new URL(`../../target/e2e/${name}/build/`, import.meta.url);
  let css = readFileSync(new URL("styles.css", build), "utf8");
  const pages = new URL("pages/", build);
  if (existsSync(pages)) {
    for (const file of readdirSync(pages).sort()) {
      if (file.endsWith(".css")) css += "\n" + readFileSync(new URL(file, pages), "utf8");
    }
  }
  return css;
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
  const { window, document, Node, Element, DocumentFragment } = makeDom();

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
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "URLSearchParams", "console",
    `${src}\nreturn typeof WF !== "undefined" ? WF : null;`,
  );

  const WF = run(
    window, document, Node, Element, DocumentFragment,
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
export function mountPage(body) {
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
