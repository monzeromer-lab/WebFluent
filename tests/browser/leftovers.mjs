// What the built-ins lacked at the end of the plan, in a real Chrome,
// against `tests/fixtures/leftovers`: marked lines and a tab a file in a
// code block, a stat's trend, a client-only block's placeholder, a
// gallery's columns per breakpoint, a lightbox's and a tour's own words, a
// table's filter and pages, a panel that folds away, and a group's boxes
// drawn by a `for`.
//
//   cd tests/browser && npm ci && node leftovers.mjs

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
const fixture = path.join(repo, "tests/fixtures/leftovers");
const port = 4802;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-leftovers-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const painted = fs.readFileSync(path.join(dir, "build", "index.html"), "utf8");
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
await page.setViewport({ width: 1000, height: 900 });
await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });

try {
  console.log("\nCodeBlock");
  const mark = await page.evaluate(() => {
    const block = document.getElementById("marked");
    const code = block.querySelector(".wf-code");
    const marks = [...block.querySelectorAll(".wf-code-block__mark")];
    const on = marks.findIndex((m) => m.dataset.on === "true");
    const line = parseFloat(getComputedStyle(code).lineHeight);
    const top = code.getBoundingClientRect().top + parseFloat(getComputedStyle(code).paddingTop);
    return { lines: marks.length, on, lineHeight: line, offset: Math.round(marks[on].getBoundingClientRect().top - top) };
  });
  check("a mark for each line, the second one marked, standing at that line",
    mark.lines === 4 && mark.on === 1 && Math.abs(mark.offset - mark.lineHeight) <= 1, JSON.stringify(mark));
  const tabs = await page.$$eval("#files [role=tab]", (ts) => ts.map((t) => t.textContent.trim()).join(","));
  check("files: is a tab for each file", tabs === "App.wf,notes.txt", tabs);
  await page.click("#files [role=tab]:nth-child(2)");
  await pause(50);
  check("…each its own code block", await page.evaluate(() => [...document.querySelectorAll("#files [role=tabpanel]")].find((p) => !p.hidden).textContent.includes("hello")));

  console.log("\nStat, ClientOnly");
  check("a stat's trend is a sparkline under it", await page.$eval("#stat", (s) => !!s.querySelector(".wf-sparkline polyline")));
  check("a client-only block paints its placeholder, and the page draws its own block",
    painted.includes("Waiting for the page") && !painted.includes("The page is running") && (await page.$("#running")) !== null && (await page.$("#waiting")) === null);

  console.log("\nGallery, per breakpoint");
  const across = () => page.$$eval("#gallery img", (is) => new Set(is.map((i) => Math.round(i.getBoundingClientRect().top))).size);
  check("three across on a wide screen", (await across()) === 1);
  await page.setViewport({ width: 500, height: 900 });
  await pause(100);
  check("one across on a narrow one", (await across()) === 3);
  await page.setViewport({ width: 1000, height: 900 });

  console.log("\nLightbox and Tour, in their own words");
  await page.click("#box img");
  await pause(100);
  const box = await page.evaluate(() => {
    const d = document.querySelector("#box .wf-lightbox__dialog");
    return [d.querySelector(".wf-lightbox__next").getAttribute("aria-label"), d.querySelector(".wf-lightbox__close").getAttribute("aria-label"), d.querySelector(".wf-lightbox__count").textContent].join("|");
  });
  check("a lightbox says what it is told", box === "Weiter|Schließen|1 von 2", box);
  await page.keyboard.press("Escape");
  await pause(300);
  await page.click("#start");
  await pause(300);
  const tour = await page.evaluate(() => [document.querySelector(".wf-tour__next").textContent, document.querySelector(".wf-tour__count").textContent].join("|"));
  check("a tour says what it is told", tour === "Weiter|1 von 2", tour);
  await page.click(".wf-tour__next");
  await pause(200);
  check("…its last step's button too", (await page.$eval(".wf-tour__next", (b) => b.textContent)) === "Fertig");
  await page.keyboard.press("Escape");
  await pause(200);

  console.log("\nDataTable, filtered and in pages");
  const table = () => page.evaluate(() => ({
    names: [...document.querySelectorAll("#table tbody tr")].map((r) => r.cells[0].textContent).join(","),
    where: document.querySelector("#table .wf-data-table__where").textContent,
  }));
  check("painted and drawn a page at a time", painted.includes("1–2 of 5") && JSON.stringify(await table()) === JSON.stringify({ names: "web,api", where: "1–2 of 5" }), JSON.stringify(await table()));
  await page.click("#table .wf-data-table__pages button:last-child");
  await pause(50);
  check("Next shows the next page", (await table()).names === "cron,mail");
  await page.type(".wf-field input", "FAIL");
  await pause(100);
  check("a filter keeps the rows that hold it, from the first page", JSON.stringify(await table()) === JSON.stringify({ names: "api,search", where: "1–2 of 2" }), JSON.stringify(await table()));

  console.log("\nPanelGroup, folding");
  await page.focus("#panels .wf-panel-group__handle");
  await page.keyboard.press("Enter");
  await pause(50);
  check("Enter on the handle folds the panel before it away", (await text(page, "sizes")) === "sizes 0,100" && (await page.$eval("#panels .wf-panel-group__panel", (p) => p.hidden)), await text(page, "sizes"));
  await page.keyboard.press("Enter");
  await pause(50);
  check("…and again brings it back as it was", (await text(page, "sizes")) === "sizes 30,70", await text(page, "sizes"));

  console.log("\nCheckboxGroup, drawn by a for");
  await page.evaluate(() => [...document.querySelectorAll("#toppings input[type=checkbox]")].at(-1).click());
  await pause(50);
  check("a box a for draws ticks its value", (await text(page, "picked")) === "toppings onions", await text(page, "picked"));

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
