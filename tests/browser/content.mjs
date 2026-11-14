// The content built-ins in a real Chrome, against `tests/fixtures/content`:
// what each draws, what a screen reader is told, and what pressing one does
// — a callout, a key, a figure and how it moved, a code block that copies, a
// chip that toggles and one that is removed, the theme and the language,
// an accordion that keeps one open, a field around a control, an input that
// clears, and the page's own headings.
//
//   cd tests/browser && npm ci && node content.mjs

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
const fixture = path.join(repo, "tests/fixtures/content");
const port = 4794;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const tree = (page) => page.accessibility.snapshot({ interestingOnly: true });
const find = (n, f) => (f(n) ? n : (n.children || []).map((c) => find(c, f)).find(Boolean));

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-content-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
await browser.defaultBrowserContext().overridePermissions(`http://localhost:${port}`, ["clipboard-read", "clipboard-write", "clipboard-sanitized-write"]);
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
await page.setViewport({ width: 1000, height: 800 });
await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });

try {
  console.log("\nCallout, Kbd, Stat");
  const notes = await page.$$eval('[role="note"]', (n) => n.map((c) => [c.querySelector(".wf-callout__title").textContent.trim(), c.querySelector(".wf-icon svg") !== null]));
  check("a callout is a note, with its icon and its title", JSON.stringify(notes) === JSON.stringify([["Before you push", true], ["Note", true]]), JSON.stringify(notes));
  check("a key is written for this system", (await page.$eval(".wf-kbd", (k) => k.textContent)) === "Ctrl+K");
  const change = await page.$$eval(".wf-stat__change", (c) => c.map((n) => n.textContent.replace(/\s+/g, " ").trim()));
  check("a figure's change says which way it moved", change[0] === "Up 12.4%" && change[1] === "Down 2.0%", JSON.stringify(change));
  check("…and the figure is formatted", (await page.$eval(".wf-stat__value", (v) => v.textContent)) === "$1,234.50");

  console.log("\nCodeBlock");
  const lines = await page.$eval(".wf-code-block__lines", (l) => l.textContent.split("\n").length);
  check("its lines are numbered", lines === 3, String(lines));
  await page.click(".wf-code-block .wf-copy-button");
  await pause(100);
  const copied = { said: await page.$eval(".wf-code-block .wf-copy-button", (b) => b.textContent), clip: await page.evaluate(() => navigator.clipboard.readText()) };
  check("Copy copies the code, and says so", copied.said === "Copied" && copied.clip.startsWith("page Home"), JSON.stringify(copied));
  await pause(200);
  check("…and a screen reader hears it", (await page.$eval('[role="status"]', (r) => r.textContent)) === "Copied");
  await pause(1600);
  check("…then it says Copy again", (await page.$eval(".wf-code-block .wf-copy-button", (b) => b.textContent)) === "Copy");

  console.log("\nChip");
  const remove = await page.$('button[aria-label="Remove Rust"]');
  check("a removable chip's button is named for it", !!remove);
  await remove.click();
  await pause(100);
  check("…and pressing it removes it", (await page.$('button[aria-label="Remove Rust"]')) === null && !!(await page.$('button[aria-label="Remove Go"]')));
  const toggle = await page.$(".wf-chip__action");
  await toggle.click();
  await pause(50);
  check("a chip that can be selected is a toggle button, pressed by a click", (await toggle.evaluate((b) => b.getAttribute("aria-pressed"))) === "true");

  console.log("\nThemeToggle and LocaleSwitch");
  const before = await page.$eval(".wf-theme-toggle", (b) => b.getAttribute("aria-label"));
  await page.click(".wf-theme-toggle");
  await pause(50);
  const after = await page.evaluate(() => ({
    theme: document.documentElement.getAttribute("data-theme"),
    label: document.querySelector(".wf-theme-toggle").getAttribute("aria-label"),
    sun: getComputedStyle(document.querySelector('.wf-theme-toggle [data-shows="dark"]')).display,
    moon: getComputedStyle(document.querySelector('.wf-theme-toggle [data-shows="light"]')).display,
  }));
  check("the theme toggle switches to dark, and is named for what it switches to next",
    before === "Switch to the dark theme" && after.theme === "dark" && after.label === "Switch to the light theme" && after.sun !== "none" && after.moon === "none", JSON.stringify(after));
  const locales = await page.$$eval(".wf-locale-switch button", (b) => b.map((n) => [n.textContent, n.getAttribute("lang"), n.getAttribute("aria-pressed")]));
  check("a button per locale, each in its own language, the current one pressed",
    JSON.stringify(locales) === JSON.stringify([["English", "en", "true"], ["العربية", "ar", "false"]]), JSON.stringify(locales));
  await page.click('.wf-locale-switch button[lang="ar"]');
  await pause(100);
  const ar = await page.evaluate(() => ({ dir: document.documentElement.dir, pressed: document.querySelector('.wf-locale-switch button[lang="ar"]').getAttribute("aria-pressed") }));
  check("pressing one switches the page's language", ar.dir === "rtl" && ar.pressed === "true", JSON.stringify(ar));
  await page.click('.wf-locale-switch button[lang="en"]');
  await pause(100);

  console.log("\nAccordion");
  await page.click(".wf-accordion__item:first-child > summary");
  await pause(50);
  const open = await page.$$eval(".wf-accordion__item", (d) => d.map((n) => n.open));
  check("opening one closes the other, one open at a time", JSON.stringify(open) === "[true,false]", JSON.stringify(open));

  console.log("\nField");
  const field = await page.evaluate(() => {
    const select = document.querySelector(".wf-field--custom select");
    const label = document.querySelector(".wf-field--custom > .wf-label");
    return { labelled: select.getAttribute("aria-labelledby") === label.id && label.getAttribute("for") === select.id, described: (select.getAttribute("aria-describedby") || "").split(" ").length, invalid: select.getAttribute("aria-invalid") };
  });
  check("its label names the control, its hint and error describe it", field.labelled && field.described === 2 && field.invalid === null, JSON.stringify(field));
  const named = find(await tree(page), (n) => n.role === "combobox");
  check("…so a screen reader hears the select called Plan", named && named.name === "Plan", JSON.stringify(named));

  console.log("\nInputGroup");
  await page.click(".wf-input-group input");
  await page.keyboard.type("lamp");
  await pause(50);
  const shown = await page.$eval(".wf-input-group__clear", (b) => !b.hidden);
  await page.click(".wf-input-group__clear");
  await pause(50);
  const cleared = await page.evaluate(() => ({ value: document.querySelector(".wf-input-group input").value, focus: document.activeElement === document.querySelector(".wf-input-group input"), hidden: document.querySelector(".wf-input-group__clear").hidden, said: document.getElementById("said").textContent }));
  check("Clear shows once there is text, empties the input and its state, and gives it the focus back",
    shown && cleared.value === "" && cleared.focus && cleared.hidden && cleared.said === "q ", JSON.stringify(cleared));

  console.log("\nOnThisPage");
  const toc = await page.$$eval(".wf-on-this-page__link", (l) => l.map((a) => [a.textContent, a.getAttribute("href")]));
  check("it lists the page's headings, each a link to it", JSON.stringify(toc.map((t) => t[0])) === JSON.stringify(["Chips", "Questions", "Last"]) && toc.every((t) => t[1].startsWith("#")), JSON.stringify(toc));
  await page.evaluate(() => document.querySelector("h2:last-of-type").scrollIntoView({ behavior: "instant" }));
  await pause(300);
  const current = await page.$$eval('.wf-on-this-page__link[aria-current="location"]', (l) => l.map((a) => a.textContent));
  check("…and marks the one being read", current.length === 1 && current[0] === "Last", JSON.stringify(current));

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  // The fixture's own tests, through `wf test`: a chip removed and an
  // accordion opened by a click, as a reader would.
  try {
    execFileSync(WF, ["test", dir], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
    check("wf test: the content built-ins answer clicks in a test", true);
  } catch (e) {
    check("wf test: the content built-ins answer clicks in a test", false, `${e.stdout || ""}${e.stderr || ""}`);
  }
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
