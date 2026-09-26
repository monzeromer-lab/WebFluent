//! Every page the guide shows, run: mounted against the fake DOM, every
//! button clicked, every field typed into — and nothing may throw.
//!
//! `tests/guide_runs.rs` builds the examples into `target/e2e/guide/` and
//! lists them in `examples.json`; this is the half that runs them.

import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { makeDom } from "./dom.mjs";

const root = new URL("../../target/e2e/guide/", import.meta.url);
const examples = existsSync(new URL("examples.json", root))
  ? JSON.parse(readFileSync(new URL("examples.json", root), "utf8"))
  : [];

function bundle(name) {
  const build = new URL(`${name}/build/`, root);
  let src = readFileSync(new URL("app.js", build), "utf8");
  const pages = new URL("pages/", build);
  if (existsSync(pages)) {
    for (const file of readdirSync(pages).sort()) {
      if (file.endsWith(".js")) src += "\n" + readFileSync(new URL(file, pages), "utf8");
    }
  }
  return src;
}

/// The routes an example declares, with a value for each parameter.
function routes(name) {
  const src = readFileSync(new URL(`${name}/src/App.wf`, root), "utf8");
  const found = [...src.matchAll(/^page \w+\(path: "([^"]*)"/gm)].map((m) => m[1]);
  return found.map((p) => (p === "*" ? "/nowhere" : p.replace(/:\w+/g, "x")));
}

function mount(name, path, errors) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const app = document.createElement("div");
  app.id = "app";
  document.body.appendChild(app);
  window.location.pathname = path;
  const timers = [];
  const setTimeout = (fn) => { timers.push(fn); return timers.length; };
  const noop = () => 0;
  const store = new Map();
  const storage = {
    getItem: (k) => (store.has(k) ? store.get(k) : null),
    setItem: (k, v) => store.set(k, String(v)),
    removeItem: (k) => store.delete(k),
  };
  const location = { pathname: path, search: "", hash: "", href: `http://localhost${path}`, origin: "http://localhost", reload: noop };
  const navigator = { language: "en", onLine: true, clipboard: { writeText: async () => {} }, serviceWorker: undefined };
  // The network answers with an empty list: an example's resource settles,
  // and nothing is fetched from anywhere.
  // A collection (`/api/rows`) answers with an empty list, anything else
  // (`/api/note`) with an empty object.
  const fetch = async (url) => {
    const last = String(url).split("?")[0].split("/").filter(Boolean).pop() || "";
    const body = last.endsWith("s") ? "[]" : "{}";
    return {
      ok: true, status: 200, headers: new Map([["content-type", "application/json"]]),
      text: async () => body, json: async () => JSON.parse(body), blob: async () => ({}),
    };
  };
  class Socket { constructor() { this.readyState = 0; } send() {} close() {} }
  const report = (e) => errors.push(e && e.stack ? e.stack.split("\n").slice(0, 3).join("\n") : String(e));
  const quiet = { ...console, error: (...a) => report(a.map(String).join(" ")), warn: noop, log: noop };
  const run = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout", "clearTimeout",
    "setInterval", "clearInterval", "requestAnimationFrame", "URLSearchParams", "console",
    "localStorage", "sessionStorage", "location", "navigator", "fetch", "WebSocket", "EventSource",
    "matchMedia", "alert", "confirm",
    `${bundle(name)}\nreturn typeof WF !== "undefined" ? WF : null;`,
  );
  window.localStorage = storage;
  window.sessionStorage = storage;
  window.fetch = fetch;
  window.navigator = navigator;
  window.matchMedia = () => ({ matches: false, addEventListener: noop, removeEventListener: noop });
  const WF = run(
    window, document, Node, Element, DocumentFragment, setTimeout, noop, noop, noop, noop,
    URLSearchParams, quiet, storage, storage, location, navigator, fetch, Socket, Socket,
    window.matchMedia, noop, () => true,
  );
  const drain = () => { let n = 0; while (timers.length && n++ < 200) { try { timers.shift()(); } catch (e) { report(e); } } };
  drain();
  return { WF, app, drain };
}

const settle = async (ctx) => {
  for (let i = 0; i < 5; i++) {
    ctx.drain();
    await new Promise((r) => setImmediate(r));
  }
};

/// An error that is the page's fault: not a request the fake network
/// refused, nor a browser API a fake DOM does not have.
const ours = (e) => !/NetworkError|is not available|not implemented|showModal|animate is not a function|getAnimations|IntersectionObserver|scrollTo|focus is not a function|AbortController/.test(e);

for (const name of examples) {
  test(`md-docs ${name} runs`, async () => {
    const errors = [];
    const onRejection = (e) => errors.push(`unhandled: ${e && e.stack ? e.stack.split("\n").slice(0, 3).join("\n") : e}`);
    process.on("unhandledRejection", onRejection);
    try {
      for (const path of routes(name)) {
        let ctx;
        try {
          ctx = mount(name, path, errors);
        } catch (e) {
          errors.push(`mounting ${path}: ${e.stack || e}`);
          continue;
        }
        await settle(ctx);
        // Every field, then every button — found afresh each time, since a
        // click may add or remove them.
        for (const input of ctx.app.querySelectorAll("input")) {
          // A file input's files come from the reader; there are none here.
          if (input.getAttribute("type") === "file") continue;
          try {
            input.value = input.getAttribute("type") === "number" ? "3" : "text";
            input.dispatchEvent({ type: "input", target: input, preventDefault() {} });
            input.dispatchEvent({ type: "change", target: input, preventDefault() {} });
          } catch (e) { errors.push(`typing: ${e.stack || e}`); }
        }
        await settle(ctx);
        const seen = new Set();
        for (let round = 0; round < 3; round++) {
          for (const button of ctx.app.querySelectorAll("button")) {
            const key = button.textContent + round;
            if (seen.has(key)) continue;
            seen.add(key);
            try {
              button.dispatchEvent({ type: "click", target: button, currentTarget: button, preventDefault() {}, stopPropagation() {} });
            } catch (e) { errors.push(`clicking "${button.textContent}": ${e.stack || e}`); }
            await settle(ctx);
          }
        }
      }
    } finally {
      process.off("unhandledRejection", onRejection);
    }
    const real = errors.filter(ours);
    assert.deepEqual(real, [], `md-docs ${name}:\n${real.join("\n\n")}`);
  });
}
