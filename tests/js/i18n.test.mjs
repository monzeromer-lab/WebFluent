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

test("a count picks the locale's form, and exactly none is `zero` in any language", () => {
  const WF = load("");
  const i18n = WF.locales("en", {
    en: { "files.zero": "No files", "files.one": "{count} file", "files.other": "{count} files" },
    ar: { "files.one": "ملف", "files.two": "ملفان", "files.few": "{count} ملفات", "files.many": "{count} ملفًا", "files.other": "{count} ملف" },
  });
  assert.equal(i18n.t("files", { count: 0 }), "No files", "English's rules have no zero; the file's does");
  assert.equal(i18n.t("files", { count: 1 }), "1 file");
  assert.equal(i18n.t("files", { count: 2 }), "2 files");
  i18n.setLocale("ar");
  assert.equal(i18n.t("files", { count: 2 }), "ملفان");
  assert.equal(i18n.t("files", { count: 5 }), "5 ملفات");
  assert.equal(i18n.t("files", { count: 11 }), "11 ملفًا");
  assert.equal(i18n.t("files", { count: 0 }), "0 ملف", "no zero in the file: Arabic's rules pick zero, which falls back to other");
});

test("a locale with a region is written in its language's direction", () => {
  const WF = load("");
  const i18n = WF.locales("ar-EG", { "ar-EG": { hi: "أهلاً" }, "en-GB": { hi: "Hello" } });
  assert.equal(i18n.dir(), "rtl");
  i18n.setLocale("en-GB");
  assert.equal(i18n.dir(), "ltr");
  i18n.setLocale("ckb");
  assert.equal(i18n.dir(), "rtl");
});
