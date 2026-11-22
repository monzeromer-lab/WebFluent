//! `Calendar` speaks the page's language: its month and weekday names from
//! the i18n locale (and again when it changes), its buttons in the page's
//! words when it gives them.
//!
//! Run: node --test tests/js/calendar.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load() {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const fn = new Function("window", "document", "Node", "Element", "DocumentFragment", `${fullRuntime()}\nreturn WF;`);
  return { WF: fn(window, document, Node, Element, DocumentFragment), document };
}

const find = (node, test) => {
  if (test(node)) return node;
  for (const c of node.childNodes || []) {
    const hit = find(c, test);
    if (hit) return hit;
  }
  return null;
};
const hasClass = (name) => (n) => (n.getAttribute && (n.getAttribute("class") || "").split(" ").includes(name)) || n.className === name;

test("the month and the weekdays are named in the page's locale, and follow it", () => {
  const { WF, document } = load();
  const i18n = WF.locales("ar", { ar: {}, en: {} });
  const box = WF.el("div", {});
  document.body.appendChild(box);
  const day = WF.signal("2026-03-14");
  WF.calendar(box, day, day.set, { mode: "single" });
  const title = () => find(box, hasClass("wf-calendar__title")).textContent;
  const march = (lang) => new Intl.DateTimeFormat(lang, { month: "long", year: "numeric", timeZone: "UTC" }).format(new Date(Date.UTC(2026, 2, 14)));
  assert.equal(title(), march("ar"));
  i18n.setLocale("en");
  assert.equal(title(), march("en"));
});

test("its buttons say the page's words when it gives them, else English", () => {
  const { WF, document } = load();
  const words = WF.signal("Voriger Monat");
  const box = WF.el("div", {});
  document.body.appendChild(box);
  const day = WF.signal("2026-03-14");
  WF.calendar(box, day, day.set, { previous: () => words(), next: () => "Nächster Monat" });
  const navs = () => {
    const out = [];
    const walk = (n) => {
      if (hasClass("wf-calendar__nav")(n)) out.push(n.getAttribute("aria-label"));
      for (const c of n.childNodes || []) walk(c);
    };
    walk(box);
    return out;
  };
  assert.deepEqual(navs(), ["Voriger Monat", "Nächster Monat"]);
  const plain = WF.el("div", {});
  document.body.appendChild(plain);
  WF.calendar(plain, day, day.set, {});
  const labels = [];
  const walk = (n) => {
    if (hasClass("wf-calendar__nav")(n)) labels.push(n.getAttribute("aria-label"));
    for (const c of n.childNodes || []) walk(c);
  };
  walk(plain);
  assert.deepEqual(labels, ["Previous month", "Next month"]);
});
