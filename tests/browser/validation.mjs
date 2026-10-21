// Validation in a real Chrome, against `tests/fixtures/validation`,
// pre-rendered and as a single page: nothing is said before the reader has
// done anything; with `data-wf-validation="shown"` on the page's root, as
// an editor previews a form *Invalid*, every field says what its rules say
// at once, and taking it off says nothing again; a field speaks after the
// reader leaves it, in the author's words or the rule's; a refined type's
// range refuses its open end; a submit that fails shows every message and
// moves focus to the first; each label names its control; and every named
// pattern `wf rules --json` offers matches in Chrome what it matches in
// the compiler.
//
//   cd tests/browser && npm ci && node validation.mjs
//
// WF (default target/release/wf) and CHROME (default /usr/bin/google-chrome)
// say where they are.

import puppeteer from "puppeteer-core";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { host } from "./serve.mjs";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const WF = process.env.WF || path.join(repo, "target/release/wf");
const CHROME = process.env.CHROME || "/usr/bin/google-chrome";
const fixture = path.join(repo, "tests/fixtures/validation");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

const FIELDS = ["email", "password", "confirm", "nights"];
// What each field says, as the reader would see it.
const said = (page) =>
  page.evaluate((ids) => Object.fromEntries(ids.map((id) => {
    const control = document.getElementById(id);
    const message = control.closest(".wf-field").querySelector(".wf-field__error");
    return [id, { text: message ? message.textContent.trim() : "", invalid: control.getAttribute("aria-invalid") === "true" }];
  })), FIELDS);
const silent = (s) => Object.values(s).every((f) => f.text === "" && !f.invalid);

async function type(page, id, text) {
  await page.click(`#${id}`, { clickCount: 3 });
  await page.keyboard.press("Backspace");
  if (text) await page.type(`#${id}`, text);
}

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-validation-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("nothing is said before the reader has done anything", silent(await said(page)), JSON.stringify(await said(page)));
    const labels = await page.evaluate((ids) => ids.every((id) => document.getElementById(id).labels.length === 1), FIELDS);
    check("each label names its control", labels);
    const names = await page.evaluate((ids) => ids.map((id) => document.getElementById(id).name), FIELDS);
    check("each control is sent by its name", JSON.stringify(names) === JSON.stringify(FIELDS), names.join(","));

    // The Invalid preview.
    await page.evaluate(() => document.documentElement.setAttribute("data-wf-validation", "shown"));
    await new Promise((r) => setTimeout(r, 50));
    let s = await said(page);
    check("with the attribute, every field says what its rules say at once",
      s.email.text === "This is required" && s.password.text === "This is required" && s.email.invalid && s.password.invalid,
      JSON.stringify(s));
    check("…and a field that is fine says nothing", s.nights.text === "" && !s.nights.invalid, JSON.stringify(s.nights));
    await page.evaluate(() => document.documentElement.removeAttribute("data-wf-validation"));
    await new Promise((r) => setTimeout(r, 50));
    check("taken off, nothing is said again", silent(await said(page)), JSON.stringify(await said(page)));

    // A field speaks after the reader leaves it.
    await type(page, "password", "abc");
    s = await said(page);
    check("while the reader types, the field waits", s.password.text === "", JSON.stringify(s.password));
    await page.focus("#confirm");
    s = await said(page);
    check("left, it says the author's words", s.password.text === "Use at least 8 characters", JSON.stringify(s.password));
    await type(page, "password", "abcdefgh");
    await page.focus("#email");
    check("…and the next rule's when that one holds", (await said(page)).password.text === "Include a number");
    await type(page, "password", "abcdefg1");
    await page.focus("#email");
    check("…and nothing once all hold", (await said(page)).password.text === "");
    await type(page, "email", "not-an-address");
    await page.focus("#confirm");
    check("a rule with no message says its own", (await said(page)).email.text === "Enter an email address");

    // The type's range: `Number(1..10)` does not take 10.
    await type(page, "nights", "10");
    await page.focus("#email");
    check("a refined type's open end is refused", (await said(page)).nights.text === "Too large", JSON.stringify((await said(page)).nights));
    await type(page, "nights", "9");
    await page.focus("#email");
    check("…and the value below it is fine", (await said(page)).nights.text === "");

    // A submit that fails says everything and moves focus.
    await type(page, "confirm", "other");
    await page.click("#join");
    await new Promise((r) => setTimeout(r, 50));
    s = await said(page);
    const focused = await page.evaluate(() => document.activeElement && document.activeElement.id);
    check("a failed submit shows every message", s.email.text === "Enter an email address" && s.confirm.text === "The two passwords differ", JSON.stringify(s));
    check("…and moves focus to the first field with one", focused === "email", focused);

    // The named patterns, in Chrome's engine.
    const rules = JSON.parse(execFileSync(WF, ["rules", "--json", "-d", dir], { encoding: "utf8" }));
    const samples = ["abc1", "é", "ب", "Ab", "a!", "a b", "123", "عربي", "", "日本"];
    const compiler = rules.catalogue.patterns;
    const chrome = await page.evaluate((patterns, samples) =>
      patterns.map((p) => samples.map((t) => new RegExp(p.source, p.flags).test(t))), compiler, samples);
    const expected = {
      "a number": [true, false, false, false, false, false, true, false, false, false],
      "a letter": [true, true, true, true, true, true, false, true, false, true],
      "a capital letter": [false, false, false, true, false, false, false, false, false, false],
      "a small letter": [true, true, false, true, true, true, false, false, false, false],
      "a symbol": [false, false, false, false, true, false, false, false, false, false],
      "no spaces": [true, true, true, true, true, false, true, true, true, true],
      "only numbers": [false, false, false, false, false, false, true, false, true, false],
      "only letters": [false, true, true, true, false, false, false, true, true, true],
    };
    const mismatched = rules.catalogue.patterns
      .map((p, i) => [p.name, chrome[i]])
      .filter(([n, got]) => JSON.stringify(expected[n]) !== JSON.stringify(got));
    check(`each of ${compiler.length} named patterns compiles in Chrome and matches what it says`, chrome.length === compiler.length && mismatched.length === 0, JSON.stringify(mismatched));
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4399);
await scenario("A single page", false, 4400);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
