//! What a `validate` block does: the rules, when their message shows, what
//! the form says about itself, and what happens on a submit that fails.
//!
//! Run: node --test tests/js/
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const setTimeout = (fn) => globalThis.setTimeout(fn, 0);
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout",
    `${fullRuntime()}\nreturn WF;`,
  );
  return { WF: fn(window, document, Node, Element, DocumentFragment, setTimeout), document };
}

const settle = () => new Promise((done) => globalThis.setTimeout(done, 0));

test("each rule says what it says, and an empty value is one fault, not many", () => {
  const { WF } = load();
  const email = WF.signal("");
  const check = WF.validate(() => email(), [{ name: "required" }, { name: "email" }], { show: "live" });
  assert.equal(check.message(), "This is required", "only `required` speaks for an empty value");
  email.set("not-an-address");
  assert.equal(check.message(), "Enter an email address");
  email.set("ada@example.com");
  assert.equal(check.message(), "");
});

test("the rules that take a bound, and the one that names another value", () => {
  const { WF } = load();
  const password = WF.signal("short");
  const check = WF.validate(
    () => password(),
    [{ name: "minLength", args: [8], message: () => "Use at least 8" }, { name: "pattern", args: [/[0-9]/] }],
    { show: "live" },
  );
  assert.equal(check.message(), "Use at least 8", "the author's message, not the rule's");
  password.set("longenough");
  assert.equal(check.message(), "That is not the right shape", "the rule's own, when none is written");
  password.set("longenough1");
  assert.equal(check.message(), "");

  // `matches` follows what it names.
  const confirm = WF.signal("");
  const same = WF.validate(() => confirm(), [{ name: "matches", args: [() => password()] }], { show: "live" });
  assert.equal(same.message(), "The two do not match");
  confirm.set("longenough1");
  assert.equal(same.message(), "");
  password.set("changed1");
  assert.equal(same.message(), "The two do not match", "and changes when it changes");
});

test("a message waits until the reader has left the field, unless it is asked to be live", () => {
  const { WF } = load();
  const email = WF.signal("");
  const onBlur = WF.validate(() => email(), [{ name: "required" }], {});
  assert.equal(onBlur.message(), "This is required", "the fault is known");
  assert.equal(onBlur.shown(), "", "and not yet shown");
  onBlur.touched.set(true);
  assert.equal(onBlur.shown(), "This is required");

  const live = WF.validate(() => email(), [{ name: "required" }], { show: "live" });
  assert.equal(live.shown(), "This is required", "a live field says so at once");
});

test("a form is what its fields say, together", () => {
  const { WF } = load();
  const form = WF.form();
  const email = WF.signal("");
  const name = WF.signal("Ada");
  WF.validate(() => email(), [{ name: "required" }], { name: "email", form });
  WF.validate(() => name(), [{ name: "required" }], { name: "name", form });

  assert.equal(form.valid, false);
  assert.deepEqual(form.errors, { email: "This is required" });
  email.set("ada@example.com");
  assert.equal(form.valid, true);
  assert.deepEqual(form.errors, {});
});

test("a submit that fails shows every message and does not go on", () => {
  const { WF, document } = load();
  const form = WF.form();
  const email = WF.signal("");
  const check = WF.validate(() => email(), [{ name: "required" }], { name: "email", form });
  const element = document.createElement("form");
  form.current = element;

  assert.equal(check.shown(), "", "nothing shown before the submit");
  assert.equal(form.__submitting(), false, "and the submit does not go on");
  assert.equal(check.shown(), "This is required", "but every message is shown");

  email.set("ada@example.com");
  assert.equal(form.__submitting(), true);
});

test("what the server said is shown on the fields it named", () => {
  const { WF, document } = load();
  const form = WF.form();
  const email = WF.signal("ada@example.com");
  const check = WF.validate(() => email(), [{ name: "required" }], { name: "email", form });
  form.current = document.createElement("form");

  form.apply({ email: ["That address is taken"] });
  assert.equal(check.shown(), "That address is taken");
});

test("an async rule waits for the rest to pass, and says so while it asks", async () => {
  const { WF } = load();
  const asked = [];
  const email = WF.signal("");
  const check = WF.validate(
    () => email(),
    [
      { name: "required" },
      { name: "email" },
      { name: "async", message: () => "That address is taken", check: async (v) => { asked.push(v); return v !== "taken@example.com"; } },
    ],
    { show: "live" },
  );
  await settle();
  assert.deepEqual(asked, [], "an address that is not one is not asked about");

  email.set("taken@example.com");
  await settle();
  assert.deepEqual(asked, ["taken@example.com"]);
  assert.equal(check.message(), "That address is taken");

  email.set("free@example.com");
  await settle();
  assert.equal(check.message(), "");
});

test("a form with no rules still answers to the browser's own checks", () => {
  const { WF, document } = load();
  const form = WF.form();
  const element = document.createElement("form");
  element.checkValidity = () => false;
  form.current = element;
  assert.equal(form.valid, false, "the browser said no, and nothing else was asked");
});
