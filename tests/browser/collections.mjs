// The collection built-ins in a real Chrome, against
// `tests/fixtures/collections`: a list of options (one, or several with
// Space, Shift with an arrow, Ctrl+A), a tree a keyboard can open and walk,
// a table that sorts and chooses rows, a field that offers matches and
// keeps chips, and a command palette — the keys again right to left where
// the direction matters.
//
//   cd tests/browser && npm ci && node collections.mjs

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
const fixture = path.join(repo, "tests/fixtures/collections");
const port = 4796;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());
const focusedText = (page) => page.evaluate(() => (document.activeElement.textContent || "").trim());

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-collections-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
await page.setViewport({ width: 1000, height: 2000 });
const open = async (rtl = false) => {
  await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
  if (rtl) await page.evaluate(() => document.documentElement.setAttribute("dir", "rtl"));
};

try {
  await open();
  console.log("\nListbox");
  const fruit = '[aria-label="Fruit"]';
  const opts = await page.$$eval(`${fruit} [role="option"]`, (o) => o.map((n) => [n.textContent.trim(), n.getAttribute("aria-selected"), n.getAttribute("tabindex")]));
  check("options, the chosen one selected and the one Tab stop", JSON.stringify(opts.map((o) => o[1])) === '["true","false","false","false"]' && opts[0][2] === "0", JSON.stringify(opts));
  await page.focus(`${fruit} [role="option"]`);
  await page.keyboard.press("ArrowDown");
  await pause(30);
  check("in a single list the choice follows the arrow keys", (await text(page, "fruit")) === "fruit banana", await text(page, "fruit"));
  await page.keyboard.press("ArrowDown");
  check("…and a disabled option is passed over", (await focusedText(page)) === "Date", await focusedText(page));
  await page.keyboard.press("b");
  check("a letter goes to the option it starts", (await focusedText(page)) === "Banana", await focusedText(page));
  const fruits = '[aria-label="Fruits"]';
  check("a multiple list says so", (await page.$eval(fruits, (b) => b.getAttribute("aria-multiselectable"))) === "true");
  await page.focus(`${fruits} [role="option"]`);
  await page.keyboard.press("Space");
  await page.keyboard.down("Shift");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.up("Shift");
  await pause(30);
  check("Space chooses, and Shift with an arrow takes the next in too", (await text(page, "fruits")) === "fruits apple,banana", await text(page, "fruits"));
  await page.keyboard.down("Control");
  await page.keyboard.press("a");
  await page.keyboard.up("Control");
  await pause(30);
  check("Ctrl+A takes them all", (await text(page, "fruits")) === "fruits apple,banana,date", await text(page, "fruits"));

  console.log("\nTree");
  const rows = () => page.$$eval('[role="tree"] [role="treeitem"]', (r) => r.map((n) => [n.textContent.trim(), n.getAttribute("aria-level"), n.getAttribute("aria-expanded")]));
  check("a tree of the top items, a closed folder saying so", JSON.stringify(await rows()) === JSON.stringify([["src", "1", "false"], ["README.md", "1", null]]), JSON.stringify(await rows()));
  const tree = await page.$('[role="tree"]');
  check("…named, with one Tab stop", (await tree.evaluate((t) => [t.getAttribute("aria-label"), t.querySelectorAll('[tabindex="0"]').length].join())) === "Files,1");
  await page.focus('[role="treeitem"]');
  await page.keyboard.press("ArrowRight");
  await pause(30);
  check("ArrowRight opens a folder", JSON.stringify((await rows()).map((r) => r[0])) === JSON.stringify(["src", "pages", "App.wf", "README.md"]), JSON.stringify(await rows()));
  await page.keyboard.press("ArrowRight");
  check("…and again goes into it", (await focusedText(page)) === "pages", await focusedText(page));
  await page.keyboard.press("ArrowLeft");
  check("ArrowLeft on a closed one goes to its parent", (await focusedText(page)) === "src", await focusedText(page));
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await pause(30);
  check("Enter chooses, by the item's key", (await text(page, "file")) === "file src/App.wf", await text(page, "file"));
  const chosen = await page.$$eval('[role="treeitem"][aria-selected="true"]', (r) => r.map((n) => n.textContent.trim()));
  check("…and the tree marks it", JSON.stringify(chosen) === '["App.wf"]', JSON.stringify(chosen));

  console.log("\nDataTable");
  const table = ".wf-data-table";
  const cells = () => page.$$eval(`${table} tbody tr`, (r) => r.map((n) => [...n.querySelectorAll("td")].slice(1).map((t) => t.textContent.trim())));
  check("a row a record, a cell a column, a cell block drawn", JSON.stringify(await cells()) === JSON.stringify([["web", "live", "3"], ["api", "failed", "1"], ["cron", "live", "2"]]), JSON.stringify(await cells()));
  check("…captioned by its label", (await page.$eval(`${table} caption`, (c) => c.textContent)) === "Deploys");
  await page.click(`${table} th:nth-child(4) button`);
  await pause(30);
  check("a sortable header sorts, and says which way", JSON.stringify((await cells()).map((r) => r[2])) === '["1","2","3"]' && (await page.$eval(`${table} th:nth-child(4)`, (t) => t.getAttribute("aria-sort"))) === "ascending", JSON.stringify(await cells()));
  await page.click(`${table} th:nth-child(4) button`);
  // The rows slide to their places; a press waits for them.
  await pause(400);
  check("…and again the other way", JSON.stringify((await cells()).map((r) => r[2])) === '["3","2","1"]' && (await page.$eval(`${table} th:nth-child(4)`, (t) => t.getAttribute("aria-sort"))) === "descending");
  await page.click(`${table} tbody tr:first-child .wf-data-table__pick`);
  await pause(30);
  check("a row's box chooses it", (await text(page, "picked")) === "picked 1" && (await page.$eval(`${table} tbody tr:first-child`, (r) => r.getAttribute("aria-selected"))) === "true", await text(page, "picked"));
  await page.click(`${table} thead .wf-data-table__pick`);
  await pause(30);
  check("the header's box chooses them all", (await text(page, "picked")).split(" ")[1].split(",").length === 3, await text(page, "picked"));
  await page.click("#clear");
  await pause(30);
  check("with no rows its empty block shows", (await page.$eval(`${table} tbody`, (b) => b.textContent.trim())) === "No deploys");

  console.log("\nCombobox");
  const person = '[aria-label="Person"][role="combobox"]';
  await page.click(person);
  await page.keyboard.type("a");
  await pause(50);
  const offered = await page.$$eval('[role="listbox"][aria-label="Person"] [role="option"]', (o) => o.map((n) => n.textContent.trim()));
  check("typing offers what matches", JSON.stringify(offered) === '["Ada","Alan","Grace"]', JSON.stringify(offered));
  await page.keyboard.type("l");
  await page.keyboard.press("ArrowDown");
  await pause(30);
  const combo = await page.$eval(person, (i) => ({ expanded: i.getAttribute("aria-expanded"), active: i.getAttribute("aria-activedescendant"), focus: document.activeElement === i }));
  check("the arrows go through them with the focus kept in the field", combo.expanded === "true" && !!combo.active && combo.focus, JSON.stringify(combo));
  await page.keyboard.press("Enter");
  await pause(30);
  check("Enter takes one, by its key, and shows its words", (await text(page, "person")) === "person alan" && (await page.$eval(person, (i) => i.value)) === "Alan", await text(page, "person"));
  const team = '[aria-label="Team"][role="combobox"]';
  await page.click(team);
  await page.keyboard.type("gr");
  await page.keyboard.press("Enter");
  await page.keyboard.type("ad");
  await page.keyboard.press("Enter");
  await pause(30);
  check(".multiple keeps several, each a chip", (await text(page, "team")) === "team grace,ada" && (await page.$$eval('.wf-combobox .wf-chip', (c) => c.length)) === 2, await text(page, "team"));
  await page.keyboard.press("Backspace");
  await pause(30);
  check("…and Backspace in the empty field takes the last back", (await text(page, "team")) === "team grace", await text(page, "team"));

  const city = '[aria-label="City"][role="combobox"]';
  await page.keyboard.press("Escape");
  await page.click(city);
  await page.keyboard.type("pa");
  await pause(400);
  const cities = await page.$$eval('[role="listbox"][aria-label="City"] [role="option"]', (o) => o.map((n) => n.innerText.replace(/\s+/g, " ").trim()));
  check("search: asks once the typing rests, and draws each with its option block", JSON.stringify(cities) === '["Paris France","Palermo Italy"]', JSON.stringify(cities));
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await pause(50);
  check("…and Enter takes one", (await text(page, "city")) === "city Paris", await text(page, "city"));
  await page.click(city, { clickCount: 3 });
  await page.keyboard.type("zz");
  await pause(400);
  check("with nothing found its empty block shows", (await page.$eval('[role="listbox"][aria-label="City"]', (l) => l.textContent.trim())) === "No city by that name");

  console.log("\nCommandPalette");
  await page.keyboard.press("Escape");
  await pause(200);
  await page.click("#open");
  await pause(100);
  const pal = await page.evaluate(() => ({ open: document.querySelector(".wf-command-palette dialog").open, focus: document.activeElement.getAttribute("role") }));
  check("it opens with the focus in its field", pal.open && pal.focus === "combobox", JSON.stringify(pal));
  await page.keyboard.type("set");
  await pause(30);
  const left = await page.$$eval('.wf-command-palette [role="option"]:not([hidden])', (o) => o.map((n) => n.textContent.trim()));
  const groups = await page.$$eval('.wf-command-palette [role="group"]:not([hidden])', (g) => g.map((n) => n.getAttribute("aria-label")));
  check("typing leaves what matches, and a group with nothing left goes", JSON.stringify(left) === '["Settings"]' && JSON.stringify(groups) === '["Pages"]', JSON.stringify([left, groups]));
  await page.keyboard.press("Enter");
  await pause(300);
  check("Enter runs it and closes the palette", (await text(page, "ran")) === "ran settings" && !(await page.$eval(".wf-command-palette dialog", (d) => d.open)), await text(page, "ran"));
  await page.click("#open");
  await pause(100);
  const reset = await page.$$eval('.wf-command-palette [role="option"]:not([hidden])', (o) => o.length);
  check("…and opened again it shows everything", reset === 3, String(reset));
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await pause(300);
  check("the arrow keys go through the items", (await text(page, "ran")) === "ran theme", await text(page, "ran"));

  console.log("\nRight to left");
  await open(true);
  await page.focus('[role="treeitem"]');
  await page.keyboard.press("ArrowLeft");
  await pause(30);
  check("in a right-to-left page ArrowLeft opens a folder", (await rows()).length === 4, JSON.stringify(await rows()));

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
