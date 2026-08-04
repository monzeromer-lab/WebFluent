//! What a built site actually does once it runs.
//!
//! Run: node --test tests/js/rendered_site.test.mjs
//! Requires `cargo test --test e2e_sites` to have built the fixtures first;
//! `e2e_rendered.rs` does both in one step.
//!
//! Everything above this file checks the shape of the output. This checks that
//! the output *works*: that the page paints, that a click changes what is on
//! screen, that a store reaches every page that used it, that a route resolves.
//! No amount of asserting on strings proves any of that.

import { test } from "node:test";
import assert from "node:assert/strict";
import { mountSite, byClass, byTag, button, click, type } from "./harness.mjs";

// ─── The page paints at all ─────────────────────────────────────────────

test("every built site paints a non-empty page", () => {
  for (const site of ["gallery", "marketing", "dashboard", "bespoke"]) {
    const { app } = mountSite(site);
    assert.ok(
      app.children.length > 0,
      `${site} mounted but painted nothing into #app`,
    );
    assert.ok(
      app.textContent.trim().length > 50,
      `${site} painted ${app.textContent.trim().length} characters of text — effectively blank`,
    );
  }
});

// ─── Built-in components, live ──────────────────────────────────────────

test("the gallery paints every family of built-in component", () => {
  const { app } = mountSite("gallery");

  // One representative class per family. If a component stops rendering, its
  // class disappears from the live DOM even though the source still names it.
  const expected = [
    "wf-container", "wf-row", "wf-col", "wf-grid", "wf-stack", "wf-spacer", "wf-divider",
    "wf-navbar", "wf-breadcrumb", "wf-link", "wf-menu", "wf-tabs", "wf-tab-page",
    "wf-card", "wf-table", "wf-list", "wf-badge", "wf-avatar", "wf-tooltip", "wf-tag",
    "wf-input", "wf-select", "wf-checkbox", "wf-radio", "wf-switch", "wf-slider",
    "wf-datepicker", "wf-file-upload", "wf-form",
    "wf-alert", "wf-modal", "wf-dialog", "wf-spinner", "wf-progress", "wf-skeleton",
    "wf-btn", "wf-icon-btn", "wf-btn-group", "wf-dropdown",
    "wf-image", "wf-video", "wf-icon", "wf-carousel",
    "wf-text", "wf-heading", "wf-code", "wf-blockquote",
  ];

  const missing = expected.filter((cls) => byClass(app, cls).length === 0);
  assert.deepEqual(missing, [], `these components never reached the live DOM: ${missing}`);
});

test("built-ins render the HTML element they promise, not a div soup", () => {
  const { app } = mountSite("gallery");
  const cases = [
    ["wf-btn", "BUTTON"],
    ["wf-input", "INPUT"],
    ["wf-select", "SELECT"],
    ["wf-table", "TABLE"],
    ["wf-list", "UL"],
    ["wf-link", "A"],
    ["wf-navbar", "NAV"],
    ["wf-form", "FORM"],
    ["wf-image", "IMG"],
    ["wf-video", "VIDEO"],
    ["wf-code", "CODE"],
    ["wf-blockquote", "BLOCKQUOTE"],
    ["wf-badge", "SPAN"],
    ["wf-progress", "PROGRESS"],
    ["wf-divider", "HR"],
  ];
  for (const [cls, tag] of cases) {
    const el = byClass(app, cls)[0];
    assert.ok(el, `no .${cls} in the live DOM`);
    assert.equal(el.tagName, tag, `.${cls} rendered as <${el.tagName.toLowerCase()}>`);
  }
});

test("a heading modifier produces a real heading element of that level", () => {
  const { app } = mountSite("gallery");
  // The gallery declares one h1 and headings down to h6.
  assert.equal(byTag(app, "h1").length, 1, "the page should have exactly one h1");
  for (const level of ["h2", "h3", "h4", "h5", "h6"]) {
    assert.ok(
      byTag(app, level).length > 0,
      `no <${level}> in the live DOM — the level modifier did not reach the tag`,
    );
  }
});

test("input type modifiers reach the live input element", () => {
  const { app } = mountSite("gallery");
  const types = byTag(app, "input").map((i) => i.getAttribute("type"));
  for (const t of ["text", "email", "password", "datetime-local", "range", "date", "file"]) {
    assert.ok(types.includes(t), `no input of type ${t} — got ${JSON.stringify(types)}`);
  }
});

test("no element ships an empty class attribute", () => {
  for (const site of ["gallery", "marketing", "dashboard"]) {
    const { app } = mountSite(site);
    const empty = app.all().filter((el) => el.getAttribute("class") === "");
    assert.equal(empty.length, 0, `${site} painted ${empty.length} elements with class=""`);
  }
});

test("void elements are painted without children", () => {
  const VOID = ["HR", "IMG", "INPUT", "BR"];
  for (const site of ["gallery", "marketing"]) {
    const { app } = mountSite(site);
    for (const el of app.all()) {
      if (VOID.includes(el.tagName)) {
        assert.equal(
          el.childNodes.length, 0,
          `${site}: <${el.tagName.toLowerCase()}> was given ${el.childNodes.length} child node(s)`,
        );
      }
    }
  }
});

// ─── Reactivity ─────────────────────────────────────────────────────────

test("clicking a button updates the state it assigns and everything derived from it", () => {
  const ctx = mountSite("dashboard", { path: "/" });
  const { app } = ctx;

  assert.match(app.textContent, /count:0/, "the counter did not start at zero");
  assert.match(app.textContent, /doubled:0/, "the derived value did not start at zero");

  const increment = button(app, "Increment");
  assert.ok(increment, "no Increment button on the overview page");

  click(increment, ctx);
  assert.match(app.textContent, /count:1/, "state did not update after a click");
  assert.match(app.textContent, /doubled:2/, "the derived value did not follow the state");

  click(increment, ctx);
  click(increment, ctx);
  assert.match(app.textContent, /count:3/);
  assert.match(app.textContent, /doubled:6/);

  click(button(app, "Reset"), ctx);
  assert.match(app.textContent, /count:0/, "Reset did not put the counter back");
  assert.match(app.textContent, /doubled:0/);
});

test("a conditional block swaps its branches when the condition changes", () => {
  const ctx = mountSite("dashboard", { path: "/" });
  const { app } = ctx;

  assert.match(
    app.textContent, /Click increment a few times/,
    "the else branch was not painted initially",
  );
  assert.doesNotMatch(app.textContent, /clicked that rather a lot/);

  const increment = button(app, "Increment");
  click(increment, ctx);
  click(increment, ctx);
  click(increment, ctx);

  assert.match(
    app.textContent, /clicked that rather a lot/,
    "the condition became true but the then branch never appeared",
  );
  assert.doesNotMatch(
    app.textContent, /Click increment a few times/,
    "the else branch was left on screen alongside the then branch",
  );
});

// ─── Stores ─────────────────────────────────────────────────────────────

test("a store's state reaches the page that used it", () => {
  const { app } = mountSite("dashboard", { path: "/" });
  // Two of the three seeded incidents are open, one is resolved.
  assert.match(app.textContent, /Open/, "the overview never rendered the Open card");
  assert.match(app.textContent, /Closed/);
  const numbers = app.textContent.match(/\d+/g) || [];
  assert.ok(numbers.includes("2"), `derived open count missing from ${JSON.stringify(numbers)}`);
  assert.ok(numbers.includes("1"), `derived closed count missing from ${JSON.stringify(numbers)}`);
});

test("a list renders one row per item in the store", () => {
  const { app } = mountSite("dashboard", { path: "/incidents" });
  const rows = byTag(app, "tr");
  // One header row plus one per seeded incident.
  assert.equal(rows.length, 4, `expected a header and three incidents, got ${rows.length} rows`);
  assert.match(app.textContent, /Payment webhook timing out/);
  assert.match(app.textContent, /Stale cache on the pricing page/);
  assert.match(app.textContent, /Duplicate signup emails/);
});

test("typing into a bound input and submitting reaches the store action", () => {
  const ctx = mountSite("dashboard", { path: "/incidents" });
  const { app } = ctx;

  const before = byTag(app, "tr").length;
  const input = byClass(app, "wf-input")[0];
  assert.ok(input, "no bound input on the incidents page");

  type(input, "Search index is stale", ctx);
  click(button(app, "Report"), ctx);

  const after = byTag(app, "tr").length;
  assert.equal(after, before + 1, "reporting an incident did not add a row");
  assert.match(
    app.textContent, /Search index is stale/,
    "the typed value never reached the store",
  );
});

// ─── Routing ────────────────────────────────────────────────────────────

test("the router paints the page that matches the current path", () => {
  const cases = [
    ["/", /Invoicing that gets out of the way/],
    ["/pricing", /Two plans/],
    ["/does-not-exist", /invoiced away/],
  ];
  for (const [path, expected] of cases) {
    const { app } = mountSite("marketing", { path });
    assert.match(
      app.textContent, expected,
      `route ${path} did not paint its page`,
    );
  }
});

test("the app shell paints around the router on every route", () => {
  for (const path of ["/", "/pricing", "/nowhere"]) {
    const { app } = mountSite("marketing", { path });
    assert.ok(byClass(app, "wf-navbar").length > 0, `no navbar on ${path}`);
    assert.match(app.textContent, /Ledger/, `the brand is missing on ${path}`);
    assert.match(app.textContent, /Every invoice, eventually paid/, `no footer on ${path}`);
  }
});

test("a router nested inside a sidebar layout still resolves, and the sidebar survives", () => {
  const ctx = mountSite("dashboard", { path: "/incidents" });
  const { app } = ctx;

  assert.ok(byClass(app, "wf-sidebar").length > 0, "the sidebar vanished");
  assert.match(app.textContent, /Ops Console/, "the sidebar header vanished");
  assert.match(app.textContent, /Incidents/, "the nested router did not paint its route");

  // And the sibling route still resolves through the same shell.
  const overview = mountSite("dashboard", { path: "/" });
  assert.ok(byClass(overview.app, "wf-sidebar").length > 0);
  assert.match(overview.app.textContent, /What is on fire/);
});

// ─── Author-supplied design, live ───────────────────────────────────────

test("hand-authored style blocks are applied to the painted elements", () => {
  const { app } = mountSite("bespoke");

  const styled = app.all().filter((el) => el.style._props.size > 0);
  assert.ok(styled.length >= 5, `only ${styled.length} elements received author styling`);

  const declarations = styled.flatMap((el) => [...el.style._props.entries()].map(([k, v]) => `${k}:${v}`));
  const joined = declarations.join(" ");

  for (const decl of ["68rem", "0.3em", "4rem"]) {
    assert.ok(joined.includes(decl), `the author's ${decl} never reached a live element`);
  }
});

test("a component handled by a special emitter still receives its style block", () => {
  // The Modal in the bespoke fixture carries a style block, and Modal is built
  // by a dedicated emitter that used to return before styling was applied.
  const { app } = mountSite("bespoke");
  const modal = byClass(app, "wf-modal")[0];
  assert.ok(modal, "no modal in the live DOM");
  assert.ok(
    modal.style._props.size > 0,
    "the modal's style block was dropped — the special emitter skipped it",
  );
});

test("a nested user component renders with its props bound", () => {
  const { app } = mountSite("marketing", { path: "/" });
  // FeatureCard is called three times with different props.
  assert.match(app.textContent, /Nine seconds/);
  assert.match(app.textContent, /It chases for you/);
  assert.match(app.textContent, /Books that reconcile/);
  assert.ok(
    byClass(app, "wf-card").length >= 3,
    "the component's own markup did not render three times",
  );
});

// ─── Accessibility, as rendered ─────────────────────────────────────────

test("a modal is a real dialog with an accessible name", () => {
  const { app } = mountSite("bespoke");
  const dialog = byClass(app, "wf-modal")[0];
  assert.ok(dialog, "no modal in the live DOM");
  assert.equal(
    dialog.tagName, "DIALOG",
    "a div modal has no focus trap, no inert background and no Escape handling",
  );
  assert.ok(
    dialog.getAttribute("aria-labelledby"),
    "the dialog has a title but no aria-labelledby pointing at it",
  );
  const labelId = dialog.getAttribute("aria-labelledby");
  assert.ok(
    app.querySelectorAll(`#${labelId}`).length === 1,
    `aria-labelledby points at #${labelId}, which does not exist`,
  );
});

test("a modal opens and closes through the dialog API, and Escape writes back to state", () => {
  const ctx = mountSite("bespoke");
  const { app } = ctx;
  const dialog = byClass(app, "wf-modal")[0];

  assert.equal(dialog.open, false, "the modal should start closed");

  click(button(app, "Enquire"), ctx);
  assert.equal(dialog.open, true, "clicking the trigger did not open the dialog");
  assert.equal(
    dialog.getAttribute("aria-modal"), "true",
    "showModal() was not used, so the background is not inert",
  );

  // The browser closes a dialog on Escape without asking us. If the signal does
  // not follow, the state says open while the screen says closed and the next
  // click appears to do nothing.
  dialog.close();
  ctx.drain();
  click(button(app, "Enquire"), ctx);
  assert.equal(dialog.open, true, "state drifted after the browser closed the dialog");
});

test("tabs carry the roles and relationships that make them tabs", () => {
  const { app } = mountSite("gallery");

  const tablist = app.querySelectorAll("div").find((d) => d.getAttribute("role") === "tablist");
  assert.ok(tablist, "no role=tablist in the live DOM");

  const tabs = app.all().filter((e) => e.getAttribute("role") === "tab");
  const panels = app.all().filter((e) => e.getAttribute("role") === "tabpanel");
  assert.ok(tabs.length >= 2, `expected tabs, found ${tabs.length}`);
  assert.equal(panels.length, tabs.length, "every tab needs a panel");

  for (const tab of tabs) {
    const controls = tab.getAttribute("aria-controls");
    assert.ok(controls, "a tab with no aria-controls is not linked to its panel");
    const panel = app.querySelectorAll(`#${controls}`)[0];
    assert.ok(panel, `aria-controls points at #${controls}, which does not exist`);
    assert.equal(
      panel.getAttribute("aria-labelledby"), tab.getAttribute("id"),
      "the panel must name its tab back",
    );
    assert.ok(["true", "false"].includes(tab.getAttribute("aria-selected")));
  }

  // Roving tabindex: exactly one tab is in the tab order.
  const inOrder = tabs.filter((t) => String(t.getAttribute("tabindex")) === "0");
  assert.equal(inOrder.length, 1, "exactly one tab should be tabbable at a time");
});

test("a dropdown reports whether it is open", () => {
  const ctx = mountSite("gallery");
  const { app } = ctx;
  const trigger = app
    .querySelectorAll("button")
    .find((b) => b.getAttribute("aria-haspopup") === "true");
  assert.ok(trigger, "no popup trigger with aria-haspopup");
  assert.equal(trigger.getAttribute("aria-expanded"), "false", "starts closed");

  click(trigger, ctx);
  assert.equal(
    trigger.getAttribute("aria-expanded"), "true",
    "the menu opened but never said so",
  );
});

test("alerts are live regions, chosen by severity", () => {
  const { app } = mountSite("gallery");
  const alerts = byClass(app, "wf-alert");
  assert.ok(alerts.length >= 2, "expected several alerts in the gallery");

  const roles = alerts.map((a) => a.getAttribute("role"));
  assert.ok(roles.every((r) => r === "alert" || r === "status"), `got ${roles}`);
  assert.ok(roles.includes("alert"), "a danger alert must interrupt");
  assert.ok(roles.includes("status"), "a success alert must not interrupt");
});

test("images reserve space and stay off the critical path", () => {
  const { app } = mountSite("gallery");
  for (const img of byTag(app, "img")) {
    assert.equal(img.getAttribute("loading"), "lazy", "images should default to lazy");
    assert.equal(img.getAttribute("decoding"), "async");
  }
});
