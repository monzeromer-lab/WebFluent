//! `paginate: .page` on a resource over an endpoint: `.items` gathers the
//! pages, `loadMore()` asks for the next one, `.hasMore` turns false on an
//! empty page. The compiler emitted the option and the runtime ignored it,
//! so `.items` and `.hasMore` were undefined on every page that used them.

import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { runtimeFor } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  return new Function(
    "window", "document", "Node", "Element", "DocumentFragment",
    `${runtimeFor("net")}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment);
}

const tick = () => new Promise((r) => setTimeout(r, 0));
const PAGES = { 1: ["a", "b"], 2: ["c"], 3: [] };

test("pages gather onto items, and an empty page ends them", async () => {
  const WF = load();
  const asked = [];
  const endpoint = (args) => { asked.push(args.page); return Promise.resolve(PAGES[args.page]); };
  const page = WF.signal(1);
  const list = WF.resource(endpoint, {
    call: true,
    args: () => ({ page: page() }),
    paginate: { by: "page", set: (v) => page.set(v) },
  });
  await tick();
  assert.deepEqual(list.items(), ["a", "b"]);
  assert.equal(list.hasMore(), true);
  list.loadMore();
  await tick();
  assert.deepEqual(list.items(), ["a", "b", "c"]);
  list.loadMore();
  await tick();
  assert.deepEqual(list.items(), ["a", "b", "c"]);
  assert.equal(list.hasMore(), false);
  list.loadMore();
  await tick();
  assert.deepEqual(asked, [1, 2, 3], "nothing more is asked for once a page comes back empty");
});

test("a page number that is not a state still moves on", async () => {
  const WF = load();
  const endpoint = (args) => Promise.resolve(PAGES[args.page]);
  const list = WF.resource(endpoint, { call: true, args: () => ({ page: 1 }), paginate: { by: "page" } });
  await tick();
  list.loadMore();
  await tick();
  assert.deepEqual(list.items(), ["a", "b", "c"]);
});

test("another request starts again from its first page, with nothing gathered", async () => {
  const WF = load();
  const asked = [];
  const answers = { a: { 1: ["a1", "a2"], 2: ["a3"] }, b: { 1: ["b1"], 2: [] } };
  const endpoint = (args) => { asked.push(`${args.q}${args.page}`); return Promise.resolve(answers[args.q][args.page]); };
  const q = WF.signal("a");
  const list = WF.resource(endpoint, { call: true, args: () => ({ q: q(), page: 1 }), paginate: { by: "page" } });
  await tick();
  list.loadMore();
  await tick();
  assert.deepEqual(list.items(), ["a1", "a2", "a3"]);
  q.set("b");
  assert.deepEqual(list.items(), [], "what the other request gathered is not this one's");
  assert.equal(list.hasMore(), true);
  await tick();
  assert.deepEqual(list.items(), ["b1"]);
  assert.deepEqual(asked, ["a1", "a2", "b1"], "it asked for its first page, not the page the other had reached");
});
