//! A read over an endpoint that says how it pages — `pages .number(param:
//! page)`, `.cursor(param: after, next: …)`, `.link` — holds every page's
//! items, in order: `loadMore()` asks for the next page, `hasMore` says
//! whether there is one, and the read writes no state of the page's. A read
//! over a `streamed` endpoint is `.loaded` from its first line, and grows.

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
const paged = (endpoint, pages) => Object.assign(endpoint, { pages });

test("numbered pages gather in order, and an empty page ends them", async () => {
  const WF = load();
  const PAGES = { 1: ["a", "b"], 2: ["c"], 3: [] };
  const asked = [];
  const endpoint = paged((args) => {
    asked.push(args.page);
    return Promise.resolve(PAGES[args.page ?? 1]);
  }, { style: "number", param: "page" });
  const list = WF.resource(endpoint, { call: true, args: () => ({}) });
  await tick();
  assert.deepEqual(list.items(), ["a", "b"]);
  assert.deepEqual(list.value(), ["a", "b"], "the read's value is what it gathered");
  assert.equal(list.hasMore(), true);
  list.loadMore();
  await tick();
  assert.deepEqual(list.items(), ["a", "b", "c"]);
  list.loadMore();
  await tick();
  assert.equal(list.hasMore(), false);
  list.loadMore();
  await tick();
  assert.deepEqual(asked, [undefined, 2, 3], "the first call is as written; nothing is asked for after an empty page");
});

test("a cursor comes from the page before, and a page with none is the last", async () => {
  const WF = load();
  const FEED = {
    "": { posts: ["p1", "p2"], nextCursor: "c2" },
    c2: { posts: ["p3"], nextCursor: "c3" },
    c3: { posts: ["p4"], nextCursor: null },
  };
  const asked = [];
  const endpoint = paged((args) => {
    asked.push(args.after ?? "");
    return Promise.resolve(FEED[args.after ?? ""]);
  }, { style: "cursor", param: "after", next: (p) => p.nextCursor, items: (p) => p.posts });
  const feed = WF.resource(endpoint, { call: true, args: () => ({}) });
  await tick();
  assert.deepEqual(feed.items(), ["p1", "p2"]);
  feed.loadMore();
  await tick();
  feed.loadMore();
  await tick();
  assert.deepEqual(feed.items(), ["p1", "p2", "p3", "p4"]);
  assert.equal(feed.hasMore(), false);
  assert.deepEqual(asked, ["", "c2", "c3"]);
});

test("a `Link` header's next address is asked for as the server wrote it", async () => {
  const WF = load();
  const asked = [];
  const endpoint = paged((args) => {
    asked.push(args.$address ?? "first");
    if (!args.$address) {
      return Promise.resolve({
        value: ["a"],
        url: "https://api.example.com/search?q=x",
        headers: { link: '<https://api.example.com/search?q=x&page=1>; rel="prev", </search?q=x&page=2>; rel="next last"' },
      });
    }
    return Promise.resolve({ value: ["b"], url: args.$address, headers: {} });
  }, { style: "link" });
  const found = WF.resource(endpoint, { call: true, args: () => ({ q: "x" }) });
  await tick();
  assert.equal(found.hasMore(), true);
  found.loadMore();
  await tick();
  assert.deepEqual(found.items(), ["a", "b"]);
  assert.equal(found.hasMore(), false, "no `rel=\"next\"`, no more");
  assert.deepEqual(asked, ["first", "https://api.example.com/search?q=x&page=2"], "a relative link is read against the address it came from");
});

test("a refresh asks again for every page gathered, and shows them together", async () => {
  const WF = load();
  const PAGES = { 1: ["a"], 2: ["b"], 3: ["c"] };
  const asked = [];
  const endpoint = paged((args) => {
    asked.push(args.page ?? 1);
    return Promise.resolve(PAGES[args.page ?? 1].slice());
  }, { style: "number", param: "page" });
  const list = WF.resource(endpoint, { call: true, args: () => ({}) });
  await tick();
  list.loadMore();
  await tick();
  asked.length = 0;
  PAGES[2] = ["b", "b2"];
  list.reload();
  assert.equal(list.isRefreshing(), true);
  await tick();
  assert.deepEqual(asked, [1, 2], "the two pages it had, in order");
  assert.deepEqual(list.items(), ["a", "b", "b2"]);
  assert.equal(list.hasMore(), true);
});

test("another request starts again from its first page, with nothing gathered", async () => {
  const WF = load();
  const answers = { a: { 1: ["a1"], 2: ["a2"] }, b: { 1: ["b1"], 2: [] } };
  const endpoint = paged((args) => Promise.resolve(answers[args.q][args.page ?? 1]), { style: "number", param: "page" });
  const q = WF.signal("a");
  const list = WF.resource(endpoint, { call: true, args: () => ({ q: q() }) });
  await tick();
  list.loadMore();
  await tick();
  assert.deepEqual(list.items(), ["a1", "a2"]);
  q.set("b");
  assert.deepEqual(list.items(), [], "what the other request gathered is not this one's");
  assert.equal(list.isLoading(), true);
  await tick();
  assert.deepEqual(list.items(), ["b1"]);
});

test("a next page that fails keeps what was gathered, and says it is stale", async () => {
  const WF = load();
  let fail = false;
  const endpoint = paged((args) => (fail ? Promise.reject(Object.assign(["offline", false], { kind: "offline", message: "down" })) : Promise.resolve(["a"])),
    { style: "number", param: "page" });
  const list = WF.resource(endpoint, { call: true, args: () => ({}) });
  await tick();
  fail = true;
  list.loadMore();
  await tick();
  assert.equal(list.isLoaded(), true);
  assert.deepEqual(list.items(), ["a"]);
  assert.ok(list.error(), "the failure is the read's stale error");
});

test("a streamed read is loaded from its first line, and grows", async () => {
  const WF = load();
  let push;
  let end;
  const endpoint = () => Promise.reject(new Error("a streamed read is not awaited whole"));
  endpoint.streamed = true;
  endpoint.stream = (args, onLine) => new Promise((resolve) => { push = onLine; end = resolve; });
  const lines = WF.resource(endpoint, { call: true, args: () => ({ id: "x" }) });
  await tick();
  assert.equal(lines.isLoading(), true);
  push({ n: 1 });
  assert.equal(lines.isLoaded(), true, "loaded from the first line");
  assert.deepEqual(lines.value(), [{ n: 1 }]);
  push({ n: 2 });
  assert.deepEqual(lines.value(), [{ n: 1 }, { n: 2 }]);
  assert.equal(lines.pending(), true, "still arriving");
  end();
  await tick();
  assert.equal(lines.pending(), false);
  assert.deepEqual(lines.value(), [{ n: 1 }, { n: 2 }]);
});
