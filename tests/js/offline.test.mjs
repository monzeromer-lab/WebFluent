//! The page's side of `offline` — the runtime's `offline` module — against a
//! fake `navigator.serviceWorker`: where it registers, that the dev server
//! takes the worker away, and the reload-once rule of the update flow. What
//! only a real browser can show (storing, serving offline, writes sent) is
//! `tests/browser/offline.mjs`.

import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { runtimeFor } from "./runtime.mjs";

/// A service worker container that records what is asked of it.
function fakeWorkers({ controlled, waiting = null } = {}) {
  const container = new EventTarget();
  const registration = new EventTarget();
  registration.waiting = waiting;
  registration.installing = null;
  registration.update = async () => {};
  const calls = { register: [], unregistered: 0 };
  container.controller = controlled ? {} : null;
  container.register = async (url, options) => {
    calls.register.push({ url, options });
    return registration;
  };
  container.getRegistrations = async () => [{ unregister: () => { calls.unregistered += 1; } }];
  return { container, registration, calls };
}

function load({ workers, devServer = false, base = "" }) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  document.querySelector = (selector) =>
    devServer && selector.includes("/__wf/dev.js") ? {} : null;
  const reloads = { count: 0 };
  const navigator = { serviceWorker: workers.container };
  const location = { reload: () => { reloads.count += 1; } };
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "navigator", "location",
    `${runtimeFor("offline")}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment, navigator, location);
  if (base) WF.setBasePath(base);
  return { WF, reloads };
}

const tick = () => new Promise((r) => setTimeout(r, 0));

test("it registers sw.js under the base path, past the HTTP cache", async () => {
  const workers = fakeWorkers({ controlled: false });
  const { WF } = load({ workers, base: "/site" });
  WF.offline({ sync: false });
  await tick();
  assert.deepEqual(workers.calls.register, [
    { url: "/site/sw.js", options: { scope: "/site/", updateViaCache: "none" } },
  ]);
});

test("under wf serve it takes any worker away and registers none", async () => {
  const workers = fakeWorkers({ controlled: true });
  const { WF } = load({ workers, devServer: true });
  WF.offline({ sync: false });
  await tick();
  assert.equal(workers.calls.register.length, 0);
  assert.equal(workers.calls.unregistered, 1);
});

test("the first install takes over a working page without reloading it", async () => {
  const workers = fakeWorkers({ controlled: false });
  const { WF, reloads } = load({ workers });
  WF.offline({});
  await tick();
  workers.container.dispatchEvent(new Event("controllerchange"));
  assert.equal(reloads.count, 0);
});

test("an update is offered, and taking it reloads once — not twice", async () => {
  const waiting = { messages: [], postMessage(m) { this.messages.push(m); } };
  const workers = fakeWorkers({ controlled: true, waiting });
  const { WF, reloads } = load({ workers });
  assert.equal(WF.update().available, false);
  WF.offline({});
  await tick();
  assert.equal(WF.update().available, true, "a waiting version is offered");
  WF.update().apply();
  assert.deepEqual(waiting.messages, [{ type: "wf:activate" }], "the waiting worker is told to take over");
  workers.container.dispatchEvent(new Event("controllerchange"));
  workers.container.dispatchEvent(new Event("controllerchange"));
  assert.equal(reloads.count, 1);
});

test("a version found later is offered once it has installed", async () => {
  const workers = fakeWorkers({ controlled: true });
  const { WF } = load({ workers });
  WF.offline({});
  await tick();
  assert.equal(WF.update().available, false);
  const worker = new EventTarget();
  worker.state = "installing";
  workers.registration.installing = worker;
  workers.registration.dispatchEvent(new Event("updatefound"));
  worker.state = "installed";
  workers.registration.waiting = worker;
  worker.dispatchEvent(new Event("statechange"));
  assert.equal(WF.update().available, true);
});

test("without a service worker the page reads the values and nothing breaks", () => {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const WF = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "navigator", "location",
    `${runtimeFor("offline")}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment, {}, {});
  WF.offline({ sync: true });
  assert.equal(WF.update().available, false);
  WF.update().apply();
});
