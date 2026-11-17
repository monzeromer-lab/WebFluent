//! What a contract read from an OpenAPI spec asks of the runtime: an error
//! enum's `on 4XX` case answering every status of its class no other case
//! names, and a parameter whose name the server knows otherwise — `in
//! query "page-size"`, `in cookie "session_id"` — sent under that name.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load(fetchImpl) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    "clearTimeout", "fetch", "navigator", "AbortController", "XMLHttpRequest",
    `${fullRuntime()}\nreturn WF;`,
  );
  const WF = fn(
    window, document, Node, Element, DocumentFragment,
    (f) => globalThis.setTimeout(f, 0), globalThis.clearTimeout,
    fetchImpl, { onLine: true }, globalThis.AbortController, class {},
  );
  return { WF, document };
}

function response(body, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    headers: { forEach: () => {} },
    text: async () => (body == null ? "" : JSON.stringify(body)),
  };
}

const people = (WF) => {
  const Backend = WF.api({ name: "Backend", base: "/api", endpoints: {} });
  return WF.group(Backend, "people", {
    endpoints: {
      create: {
        method: "POST", path: "people",
        result: { enum: true, cases: { 409: "conflict", 4: "clientError", 5: "serverError" } },
      },
      list: {
        method: "GET", path: "people",
        result: {},
        locations: { pageSize: ["query", "page-size"], sid: ["cookie", "session_id"] },
      },
    },
  });
};

test("a status a case names is that case, and any other of its class the class's", async () => {
  for (const [status, want] of [[409, "conflict"], [404, "clientError"], [429, "clientError"], [503, "serverError"]]) {
    const { WF } = load(async () => response({ title: "no" }, status));
    const got = await people(WF).create({ body: {} });
    assert.deepEqual(got, ["err", [want, { title: "no" }]], `${status}`);
  }
});

test("a parameter the server names otherwise is sent under the server's name", async () => {
  const sent = [];
  const { WF, document } = load(async (url) => {
    sent.push(String(url));
    return response([]);
  });
  await people(WF).list({ pageSize: 20, q: "ada", sid: "s-1" });
  assert.equal(sent.length, 1);
  assert.match(sent[0], /^\/api\/people\?/);
  assert.match(sent[0], /[?&]q=ada(&|$)/);
  assert.match(sent[0], /[?&]page-size=20(&|$)/);
  assert.doesNotMatch(sent[0], /pageSize|sid/, "the program's names stay the program's");
  assert.match(document.cookie || "", /session_id=s-1/);
});
