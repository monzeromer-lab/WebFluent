// The third tier of built-ins in a real Chrome, against
// `tests/fixtures/tier3`: a colour, two lists to move things between, text
// that turns into a field, a question before a button acts, the matches of
// a search marked, a block for the browser alone, a speed dial, and a way
// back to the top.
//
//   cd tests/browser && npm ci && node tier3.mjs

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
const fixture = path.join(repo, "tests/fixtures/tier3");
const port = 4801;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-tier3-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const painted = fs.readFileSync(path.join(dir, "build", "index.html"), "utf8");
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
await page.setViewport({ width: 1000, height: 800 });
await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
const focused = () => page.evaluate(() => {
  const a = document.activeElement;
  return a ? a.getAttribute("aria-label") || a.textContent.trim() : "";
});

try {
  console.log("\nColorPicker");
  const swatches = () => page.$$eval("#accent .wf-color-picker__swatch", (bs) => bs.map((b) => `${b.dataset.color}:${b.getAttribute("aria-pressed")}`).join(" "));
  check("a group of swatches, the bound one pressed", (await page.$eval("#accent", (g) => g.getAttribute("role") === "group" && g.getAttribute("aria-label") === "Accent")) &&
    (await swatches()) === "#0f766e:true #b91c1c:false #1d4ed8:false", await swatches());
  await page.click('#accent [data-color="#b91c1c"]');
  await pause(50);
  check("a swatch pressed is the colour", (await text(page, "colour")) === "colour #b91c1c" && (await swatches()).includes("#b91c1c:true"));
  await page.click("#accent .wf-color-picker__hex", { clickCount: 3 });
  await page.keyboard.type("#123");
  await page.keyboard.press("Tab");
  await pause(50);
  check("a hex typed is the colour, written in full", (await text(page, "colour")) === "colour #112233" && (await page.$eval("#accent input[type=color]", (i) => i.value)) === "#112233", await text(page, "colour"));

  console.log("\nTransferList");
  const sides = () => page.evaluate(() => ["from", "to"].map((s) => [...document.querySelectorAll(`#letters .wf-transfer-list__${s} option`)].map((o) => o.textContent).join(",")).join("|"));
  check("what is chosen on one side, the rest on the other", (await sides()) === "Alpha,Gamma|Beta", await sides());
  await page.select("#letters .wf-transfer-list__from", (await page.$eval("#letters .wf-transfer-list__from option", (o) => o.value)));
  await page.click('#letters [aria-label="Move the selected across"]');
  await pause(50);
  check("the selected move across, in the options' order", (await text(page, "picked")) === "picked a,b" && (await sides()) === "Gamma|Alpha,Beta", `${await text(page, "picked")} ${await sides()}`);
  await page.click('#letters [aria-label="Move them all back"]');
  await pause(50);
  check("…and all of them back", (await text(page, "picked")) === "picked" && (await sides()) === "Alpha,Beta,Gamma|");

  console.log("\nEditable");
  check("text with what a press does", (await page.$eval("#name .wf-editable__text", (b) => b.getAttribute("aria-label"))) === "Project name: Untitled, edit");
  await page.click("#name .wf-editable__text");
  await pause(50);
  check("pressed, a field in its place with the text", await page.evaluate(() => document.activeElement.classList.contains("wf-editable__field") && document.activeElement.value === "Untitled"));
  await page.keyboard.type("Atlas");
  await page.keyboard.press("Enter");
  await pause(50);
  check("Enter keeps it, the focus back on the text", (await text(page, "named")) === "name Atlas" && (await focused()) === "Project name: Atlas, edit", await focused());
  await page.click("#name .wf-editable__text");
  await page.keyboard.type("Zeta");
  await page.keyboard.press("Escape");
  await pause(50);
  check("Escape puts it back", (await text(page, "named")) === "name Atlas");

  console.log("\nPopconfirm");
  const card = () => page.$eval("#ask .wf-popconfirm__card", (c) => c.hasAttribute("data-open"));
  await page.click("#delete");
  await pause(80);
  check("the button asks rather than acts, the focus on Cancel", (await card()) && (await text(page, "heard")) === "heard" && (await focused()) === "Cancel", await focused());
  check("…an alert dialog called by its question", await page.$eval("#ask .wf-popconfirm__card", (c) => c.getAttribute("role") === "alertdialog" && document.getElementById(c.getAttribute("aria-labelledby")).textContent === "Delete this file?"));
  await page.keyboard.press("Escape");
  await pause(80);
  check("Escape is no, the focus back on the button", !(await card()) && (await text(page, "heard")) === "heard kept" && (await focused()) === "Delete file", await focused());
  await page.click("#delete");
  await pause(80);
  await page.click("#ask .wf-btn--danger");
  await pause(80);
  check("yes runs on confirm", (await text(page, "heard")) === "heard deleted" && !(await card()));

  console.log("\nHighlight, ClientOnly");
  const marks = () => page.$$eval("#marked mark", (ms) => ms.map((m) => m.textContent).join(","));
  check("every match marked, its case ignored", (await marks()) === "ea,ea,ea,ea,ea", await marks());
  await page.click(".wf-field input", { clickCount: 3 });
  await page.keyboard.type("FAR");
  await pause(50);
  check("…and marked again as the query changes", (await marks()) === "far" && (await text(page, "marked")) === "Search near and far: each area is clear.", await marks());
  check("a client-only block is drawn in the browser, and not in the paint", (await page.$("#only")) !== null && !painted.includes("Only in the browser"));

  console.log("\nFab");
  const dial = () => page.$eval("#dial", (d) => ({
    expanded: d.querySelector(".wf-fab__trigger").getAttribute("aria-expanded"),
    shown: [...d.querySelectorAll(".wf-fab__action")].filter((a) => !a.hidden).length,
  }));
  check("a speed dial, shut", JSON.stringify(await dial()) === JSON.stringify({ expanded: "false", shown: 0 }));
  await page.click("#dial .wf-fab__trigger");
  await pause(50);
  check("its button opens the actions, the focus on the nearest", JSON.stringify(await dial()) === JSON.stringify({ expanded: "true", shown: 2 }) && (await focused()) === "Folder", await focused());
  await page.keyboard.press("ArrowUp");
  check("the arrow keys go between them", (await focused()) === "Note");
  await page.keyboard.press("Enter");
  await pause(50);
  check("one chosen runs, the dial shuts, the focus back on its button", (await text(page, "heard")) === "heard note" && (await dial()).expanded === "false" && (await focused()) === "Create", await focused());

  console.log("\nBackToTop");
  const top = () => page.$eval("#top", (b) => !b.hidden);
  check("hidden at the top of the page", !(await top()));
  await page.evaluate(() => window.scrollTo(0, 1200));
  await pause(150);
  check("shown once the page has scrolled past after:", await top());
  await page.click("#top");
  await pause(1200);
  check("a press goes back to the top, the focus on the page's main", (await page.evaluate(() => window.scrollY)) === 0 && (await page.evaluate(() => document.activeElement.id)) === "wf-main");

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
