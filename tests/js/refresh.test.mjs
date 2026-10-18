//! A resource asked again for what it already shows — on focus, on an
//! interval, by `reload()` — and what the page does about it.
//!
//! Run: node --test tests/js/refresh.test.mjs
//! Requires the `refresh` fixture built to target/e2e/refresh (the Rust test
//! `e2e_rendered.rs` builds it, then runs this). Each page is the compiler's
//! output for `tests/fixtures/refresh`, run against the fake DOM with a
//! server the test answers.
//!
//! Up to 5.6.2 every refresh set the resource to "loading": the `match` over
//! it swapped the whole page for its loading arm and built the ready arm
//! again — a skeleton flash, the scroll lost, the focus lost, every row and
//! every component in it new. The ready arm was handed its data once, so it
//! could not have followed a refresh in place anyway.

import { test } from "node:test";
import assert from "node:assert/strict";
import { mountSite, byClass, button, click, type } from "./harness.mjs";

function response(body, status = 200) {
  return {
    ok: status >= 200 && status < 300,
    status,
    headers: { forEach: () => {} },
    text: async () => JSON.stringify(body),
  };
}

/// A server the test writes: `answers[path]` is what that address answers
/// now — a body, or `{ status }` for a failure — and `asked` every address
/// asked for, in order.
function serve(answers) {
  const asked = [];
  globalThis.fetch = async (url) => {
    asked.push(url);
    const answer = answers[url];
    if (answer === undefined) return response({ error: "no such thing" }, 404);
    if (answer && answer.status) return response({ error: "down" }, answer.status);
    // A fresh parse, as a browser's would be: nothing the page holds is
    // the object the test wrote.
    return response(JSON.parse(JSON.stringify(answer)));
  };
  return asked;
}

/// Every answer in flight arrives, and whatever the runtime deferred runs.
async function settle(page) {
  for (let i = 0; i < 6; i++) {
    await new Promise((done) => setImmediate(done));
    page.drain();
  }
}

const text = (root, cls) => byClass(root, cls).map((n) => n.textContent);
const one = (root, cls) => {
  const found = byClass(root, cls);
  assert.equal(found.length, 1, `one .${cls}, found ${found.length}`);
  return found[0];
};

const BOARD = {
  count: 3,
  lead: { id: "a", name: "Ada", status: "open" },
  rows: [
    { id: "a", name: "Ada", status: "open" },
    { id: "b", name: "Bo", status: "open" },
    { id: "c", name: "Cy", status: "stuck" },
  ],
};

test("the first load shows the loading arm, then the data", async () => {
  serve({ "/api/board": BOARD });
  const page = mountSite("refresh", { path: "/" });
  assert.deepEqual(text(page.app, "loading"), ["Loading the board"]);
  assert.equal(byClass(page.app, "count").length, 0);
  await settle(page);
  assert.equal(byClass(page.app, "loading").length, 0);
  assert.equal(one(page.app, "count").textContent, "3 open");
  assert.deepEqual(text(page.app, "row-label"), ["Ada · open", "Bo · open", "Cy · stuck"]);
});

test("a refresh with new data keeps the nodes on the page and changes their text", async () => {
  const answers = { "/api/board": BOARD };
  serve(answers);
  const page = mountSite("refresh", { path: "/" });
  await settle(page);
  const count = one(page.app, "count");
  const lead = one(page.app, "lead");
  const labels = byClass(page.app, "row-label");
  const heading = page.app.querySelector("h1");

  answers["/api/board"] = {
    count: 4,
    lead: { id: "b", name: "Bo", status: "open" },
    rows: [
      { id: "a", name: "Ada", status: "done" },
      { id: "b", name: "Bo", status: "open" },
      { id: "c", name: "Cy", status: "open" },
    ],
  };
  page.window._fire("focus");
  // While it is asked for, what was there stays — and says it is refreshing.
  assert.equal(byClass(page.app, "loading").length, 0, "a refresh does not show the loading arm");
  assert.equal(one(page.app, "count"), count);
  assert.deepEqual(text(page.app, "refreshing"), ["Refreshing"]);

  await settle(page);
  assert.equal(byClass(page.app, "refreshing").length, 0);
  assert.equal(one(page.app, "count"), count, "the same node");
  assert.equal(count.textContent, "4 open", "with the new text");
  assert.equal(one(page.app, "lead"), lead, "an `if let` keeps its branch");
  assert.equal(lead.textContent, "Lead: Bo", "and reads the new value");
  const after = byClass(page.app, "row-label");
  assert.deepEqual(after.map((n) => n.textContent), ["Ada · done", "Bo · open", "Cy · open"]);
  after.forEach((n, i) => assert.equal(n, labels[i], `row ${i} is the same node`));
  assert.equal(page.app.querySelector("h1"), heading);
});

test("a keyed list keeps its rows' nodes and their own state, and moves and inserts", async () => {
  const answers = { "/api/board": BOARD };
  serve(answers);
  const page = mountSite("refresh", { path: "/" });
  await settle(page);
  const byName = () => Object.fromEntries(byClass(page.app, "row").map((n) => [n.textContent.split(" · ")[0], n]));
  const before = byName();
  // Bo's card is opened: that is the component's own state.
  click(button(page.app, "More about Bo"), page);
  assert.deepEqual(text(page.app, "row-details"), ["Details of Bo"]);

  answers["/api/board"] = {
    ...BOARD,
    rows: [
      { id: "c", name: "Cy", status: "stuck" },
      { id: "d", name: "Di", status: "new" },
      { id: "b", name: "Bo", status: "done" },
      { id: "a", name: "Ada", status: "open" },
    ],
  };
  click(button(page.app, "Refresh"), page);
  await settle(page);

  assert.deepEqual(text(page.app, "row-label"), ["Cy · stuck", "Di · new", "Bo · done", "Ada · open"]);
  const after = byName();
  assert.equal(after.Ada, before.Ada, "a row that did not change is the same node");
  assert.equal(after.Cy, before.Cy, "and so is one that moved");
  assert.equal(after.Bo, before.Bo, "and one whose value changed");
  assert.ok(after.Di && !Object.values(before).includes(after.Di), "a new row is drawn");
  assert.deepEqual(text(page.app, "row-details"), ["Details of Bo"], "Bo's card is still open");
});

test("an input in the ready arm keeps the focus and what was typed across a refresh", async () => {
  const answers = { "/api/board": BOARD };
  serve(answers);
  const page = mountSite("refresh", { path: "/" });
  await settle(page);
  const input = page.app.querySelectorAll("input")[0];
  type(input, "half a thought", page);
  input.focus();
  assert.equal(page.document.activeElement, input);

  answers["/api/board"] = { ...BOARD, count: 5 };
  page.window._fire("focus");
  await settle(page);

  assert.equal(one(page.app, "count").textContent, "5 open");
  assert.equal(page.app.querySelectorAll("input")[0], input, "the same input");
  assert.equal(page.document.activeElement, input, "still focused");
  assert.equal(input.value, "half a thought");
});

test("a refresh that fails keeps the data on the page and says so", async () => {
  const answers = { "/api/board": BOARD };
  serve(answers);
  const page = mountSite("refresh", { path: "/" });
  await settle(page);
  const count = one(page.app, "count");

  answers["/api/board"] = { status: 503 };
  page.window._fire("focus");
  await settle(page);
  assert.equal(byClass(page.app, "wf-alert--danger").length, 0, "not the error arm");
  assert.equal(one(page.app, "count"), count, "what arrived stays");
  assert.equal(count.textContent, "3 open");
  assert.match(page.app.textContent, /Could not refresh/, "and `error` says the refresh failed");

  // The next answer that arrives clears it.
  answers["/api/board"] = { ...BOARD, count: 2 };
  page.window._fire("focus");
  await settle(page);
  assert.equal(count.textContent, "2 open");
  assert.doesNotMatch(page.app.textContent, /Could not refresh/);
});

test("a first load that fails shows the error arm, as it did", async () => {
  serve({ "/api/board": { status: 500 } });
  const page = mountSite("refresh", { path: "/" });
  await settle(page);
  assert.match(page.app.textContent, /Could not load: HTTP 500/);
  assert.equal(byClass(page.app, "count").length, 0);
});

test("a request that changes with a state shows loading again, not the old answer", async () => {
  const answers = {
    "/api/servers?env=test": [{ id: "t1", name: "test-1" }],
    "/api/servers?env=prod": [{ id: "p1", name: "prod-1" }, { id: "p2", name: "prod-2" }],
  };
  const asked = serve(answers);
  const page = mountSite("refresh", { path: "/servers" });
  await settle(page);
  assert.deepEqual(text(page.app, "server"), ["test-1"]);

  // The same request again is a refresh: the list stays.
  page.window._fire("focus");
  assert.deepEqual(text(page.app, "server"), ["test-1"]);
  assert.equal(byClass(page.app, "loading").length, 0);
  await settle(page);

  // Another environment's servers are another request: test-1 would be
  // wrong while prod's are on their way.
  click(button(page.app, "Production"), page);
  assert.deepEqual(text(page.app, "loading"), ["Loading servers"]);
  assert.deepEqual(text(page.app, "server"), []);
  await settle(page);
  assert.deepEqual(text(page.app, "server"), ["prod-1", "prod-2"]);
  assert.deepEqual(asked, ["/api/servers?env=test", "/api/servers?env=test", "/api/servers?env=prod"]);
});

test("an enum's arm follows a new payload of the same case in place, and is drawn again for another case", () => {
  const page = mountSite("refresh", { path: "/phase" });
  assert.deepEqual(text(page.app, "idle"), ["Idle"]);
  click(button(page.app, "Time out"), page);
  const failed = one(page.app, "failed");
  assert.equal(failed.textContent, "Failed: timed out");
  click(button(page.app, "Refuse"), page);
  assert.equal(one(page.app, "failed"), failed, "the same case keeps its arm");
  assert.equal(failed.textContent, "Failed: refused", "and shows the new payload");
  click(button(page.app, "Finish"), page);
  assert.equal(byClass(page.app, "failed").length, 0);
  assert.deepEqual(text(page.app, "done"), ["3 rows"]);
});
