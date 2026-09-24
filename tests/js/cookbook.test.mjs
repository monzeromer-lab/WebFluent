//! The cookbook's three applications, built from the guide and then run.
//!
//! `rendered_site.test.mjs` drives the fixture projects under `tests/fixtures/`.
//! This one drives the apps printed in `md-docs/20-cookbook.md` itself, so a
//! reader who pastes one into a fresh project gets something that works. The
//! sources are extracted by `tests/cookbook.rs`, which builds them into
//! `target/e2e/cookbook-*` before handing over here.
//!
//! These go through the store actions rather than only reading the first paint:
//! a store is an object of actions, and a codegen that mistook one of them for
//! a list method shipped a bundle that painted perfectly and threw on click.

import assert from "node:assert/strict";
import test from "node:test";

import { mountSite } from "./harness.mjs";

/// Run everything the page deferred, and let the promises those timers resolve
/// carry on.
///
/// `ctx.drain()` alone only empties the timer queue. An exit animation removes
/// its node in a `.then()` after the fallback timeout fires, so a test that
/// never yields to the microtask queue sees the old node still in place.
async function settle(ctx) {
  for (let i = 0; i < 20; i++) {
    ctx.drain();
    await new Promise((resolve) => setImmediate(resolve));
  }
}

function click(el, ctx) {
  assert.ok(el, "click() on a missing element");
  el.dispatchEvent({ type: "click", target: el, preventDefault() {} });
}

function type(el, value, ctx) {
  assert.ok(el, "type() into a missing element");
  el.value = value;
  el.dispatchEvent({ type: "input", target: el, preventDefault() {} });
}

const buttonSaying = (root, text) =>
  root.querySelectorAll("button").find((b) => (b.textContent ?? "").includes(text)) || null;

const labelled = (root, text) =>
  root.all().find((e) => (e.getAttribute("aria-label") || "").includes(text)) || null;

const textInput = (root) =>
  root.querySelectorAll("input").find((e) => e.getAttribute("type") !== "checkbox") || null;

const checkboxes = (root) =>
  root.querySelectorAll("input").filter((e) => e.getAttribute("type") === "checkbox");

// ─── App 1: Todos ───────────────────────────────────────────────────────

/// Add three todos through the store action behind the Add button.
async function withThreeTodos() {
  const ctx = mountSite("cookbook-todos");
  await settle(ctx);
  for (const title of ["write tests", "fix remove", "ship it"]) {
    type(textInput(ctx.app), title, ctx);
    await settle(ctx);
    click(buttonSaying(ctx.app, "Add"), ctx);
    await settle(ctx);
  }
  return ctx;
}

test("the todo app adds items through its store action", async () => {
  const ctx = await withThreeTodos();
  const painted = ctx.app.textContent;
  for (const title of ["write tests", "fix remove", "ship it"]) {
    assert.match(painted, new RegExp(title), `"${title}" never reached the page`);
  }
  assert.match(painted, /Open \(3\)/, "the derived open count did not follow the list");
});

test("toggling a todo marks it done and the derived counts follow", async () => {
  const ctx = await withThreeTodos();
  const box = checkboxes(ctx.app)[0];
  assert.ok(box, "no checkbox rendered for a todo");
  box.checked = true;
  box.dispatchEvent({ type: "change", target: box, preventDefault() {} });
  await settle(ctx);

  assert.match(ctx.app.textContent, /Open \(2\)/, "the open count did not drop");
  assert.match(ctx.app.textContent, /Done \(1\)/, "the done count did not rise");
});

test("removing a todo calls the store's own `remove` action, not a list method", async () => {
  const ctx = await withThreeTodos();

  // `Todos.remove(todo.id)` is an action the store declares. Emitting it as a
  // list's `splice` left the handler throwing `Todos.splice is not a function`
  // — the page painted, and the first click on a trash button broke it.
  click(labelled(ctx.app, "Remove write tests"), ctx);
  await settle(ctx);

  const painted = ctx.app.textContent;
  assert.doesNotMatch(painted, /write tests/, "the removed todo is still on the page");
  assert.match(painted, /fix remove/, "removing one todo dropped another");
  assert.match(painted, /ship it/, "removing one todo dropped another");
  assert.match(painted, /Open \(2\)/, "the derived open count did not follow the removal");
});

test("clearing the done todos empties only those", async () => {
  const ctx = await withThreeTodos();
  const box = checkboxes(ctx.app)[0];
  box.checked = true;
  box.dispatchEvent({ type: "change", target: box, preventDefault() {} });
  await settle(ctx);

  click(buttonSaying(ctx.app, "Clear done"), ctx);
  await settle(ctx);

  const painted = ctx.app.textContent;
  assert.doesNotMatch(painted, /write tests/, "the done todo was not cleared");
  assert.match(painted, /fix remove/, "clearing done removed an open todo");
  assert.match(painted, /Done \(0\)/, "the done count did not return to zero");
});

// ─── App 2: a blog, statically built ────────────────────────────────────

test("the blog paints the posts from its data file", async () => {
  const ctx = mountSite("cookbook-blog");
  await settle(ctx);
  const painted = ctx.app.textContent;
  assert.ok(painted.length > 0, "the blog painted nothing");
  assert.match(painted, /Hello|Second/, "no post from the data file reached the page");
});

// ─── App 3: a dashboard behind a login ──────────────────────────────────

test("the dashboard paints its login route", async () => {
  const ctx = mountSite("cookbook-dashboard", { path: "/login" });
  await settle(ctx);
  assert.ok(ctx.app.textContent.length > 0, "the login route painted nothing");
});

test("the dashboard's guarded route redirects a signed-out visitor", async () => {
  const ctx = mountSite("cookbook-dashboard", { path: "/" });
  await settle(ctx);
  // The guard is `Session.loggedIn`, which is false with no persisted user.
  assert.ok(ctx.app.textContent.length > 0, "the guarded route painted nothing at all");
});
