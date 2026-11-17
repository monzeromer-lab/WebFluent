//! What a hands-on pass over 7.0 found in the runtime, kept from coming
//! back: a server's message for a control no rule guards, an error thrown
//! under a busy button, and a parameter's default sent when a call leaves it
//! out. The compiler's halves are `tests/hands_on.rs`.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load(extra = {}) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const setTimeout = extra.setTimeout || ((f) => globalThis.setTimeout(f, 0));
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "fetch", "navigator", "AbortController", "XMLHttpRequest",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(
    window, document, Node, Element, DocumentFragment, setTimeout, globalThis.clearTimeout,
    extra.fetch || (async () => { throw new Error("no network"); }),
    { onLine: true }, globalThis.AbortController, class {},
  );
  return { WF, document };
}

const settle = () => new Promise((done) => globalThis.setTimeout(done, 0));

test("a server's message reaches a named control no rule guards, and goes when it is changed", () => {
  const { WF, document } = load();
  const form = WF.form();
  const element = document.createElement("form");
  const email = document.createElement("input");
  email.setAttribute("name", "email");
  element.appendChild(email);
  document.body.appendChild(element);
  form.current = element;

  form.apply({ email: ["Use your work address"] });
  const id = email.id + "-server";
  const note = document.getElementById(id);
  assert.ok(note, "the message is drawn beside the control");
  assert.equal(note.textContent, "Use your work address");
  assert.equal(note.getAttribute("role"), "alert");
  assert.equal(email.getAttribute("aria-invalid"), "true");
  assert.ok((email.getAttribute("aria-describedby") || "").includes(id));
  assert.equal(document.activeElement, email, "the focus goes to it");

  email.dispatchEvent(plainEvent("input"));
  assert.equal(document.getElementById(id), null, "an edit takes it away");
  assert.equal(email.getAttribute("aria-invalid"), "false");
  assert.equal(email.getAttribute("aria-describedby"), null);

  form.apply({ email: "Again" });
  assert.ok(document.getElementById(id));
  form.reset();
  assert.equal(document.getElementById(id), null, "a reset takes it away");
});

// The fake DOM's events are plain objects with a type.
function plainEvent(type) {
  return { type, target: null, preventDefault() {}, stopPropagation() {} };
}

test("an error thrown under a busy button still reaches the page's error reporting", async () => {
  const thrown = [];
  const { WF, document } = load({
    setTimeout: (f) => globalThis.setTimeout(() => { try { f(); } catch (e) { thrown.push(e); } }, 0),
  });
  const button = document.createElement("button");
  const handler = WF.busy(button, () => Promise.reject(new Error("the save failed")));
  handler({ preventDefault() {} });
  await settle();
  await settle();
  assert.equal(button.getAttribute("aria-busy"), null, "the button is free again");
  assert.deepEqual(thrown.map((e) => e.message), ["the save failed"]);
});

test("a call that leaves a parameter out sends its default", async () => {
  const asked = [];
  const { WF } = load({
    fetch: async (url) => {
      asked.push(String(url));
      return { ok: true, status: 200, headers: { forEach: () => {} }, text: async () => "[]" };
    },
  });
  const Backend = WF.api({ name: "Backend", base: "/api", endpoints: {} });
  const people = WF.group(Backend, "people", {
    endpoints: {
      list: {
        method: "GET", path: "people", result: {},
        locations: { pageSize: ["query", "page-size"] },
        defaults: { pageSize: 2, tone: "calm" },
      },
    },
  });
  await people.list({});
  await people.list({ pageSize: 5, tone: undefined });
  assert.match(asked[0], /page-size=2/);
  assert.match(asked[0], /tone=calm/);
  assert.match(asked[1], /page-size=5/, "what a call gives wins");
  assert.match(asked[1], /tone=calm/, "and an undefined argument is left out");
});
