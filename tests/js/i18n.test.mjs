//! The i18n module: `?lang=` opens the page in that locale. The build's
//! `hreflang` alternates point each language at `?lang=<code>`, and nothing
//! read it, so every alternate showed the default language.

import { test } from "node:test";
import assert from "node:assert/strict";
import { makeDom } from "./dom.mjs";
import { runtimeFor } from "./runtime.mjs";

function load(search) {
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  document.documentElement = document.documentElement || document.createElement("html");
  const location = { search, pathname: "/" };
  return new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "location",
    `${runtimeFor("i18n")}\nreturn WF;`,
  )(window, document, Node, Element, DocumentFragment, location);
}

const tables = { en: { hi: "Hello" }, ar: { hi: "أهلاً" } };

test("?lang= picks a locale the site has", () => {
  const WF = load("?lang=ar");
  const i18n = WF.locales("en", tables);
  assert.equal(i18n.locale(), "ar");
  assert.equal(i18n.dir(), "rtl");
  assert.equal(i18n.t("hi"), "أهلاً");
});

test("a locale the site does not have is ignored", () => {
  const WF = load("?lang=xx");
  assert.equal(WF.locales("en", tables).locale(), "en");
});

test("without it the default locale stands", () => {
  const WF = load("");
  assert.equal(WF.locales("en", tables).locale(), "en");
});
