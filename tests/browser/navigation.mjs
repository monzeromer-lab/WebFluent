// The navigation built-ins in a real Chrome, against
// `tests/fixtures/navigation`: panels a site's navigation discloses, an
// app's menu bar, a menu a right-click opens where it happened, a phone's
// tab bar, and the pages either side.
//
//   cd tests/browser && npm ci && node navigation.mjs

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
const fixture = path.join(repo, "tests/fixtures/navigation");
const port = 4798;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-navigation-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
await page.setViewport({ width: 1000, height: 900 });
await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });

const focused = () => page.evaluate(() => {
  const a = document.activeElement;
  if (!a) return "";
  // A menu item's words, without the shortcut drawn at its end.
  const words = a.querySelector && a.querySelector(":scope > .wf-text, :scope > span");
  return a.getAttribute("aria-label") || (words || a).textContent.trim();
});
const attr = (sel, name) => page.$eval(sel, (n, k) => n.getAttribute(k), name);
const trigger = (words) => page.evaluateHandle((w) => [...document.querySelectorAll(".wf-nav-menu__trigger, .wf-menubar__trigger")].find((t) => t.textContent.trim() === w), words);
const panelShown = (words) => page.evaluate((w) => {
  const t = [...document.querySelectorAll(".wf-nav-menu__trigger")].find((x) => x.textContent.trim() === w);
  const p = document.getElementById(t.getAttribute("aria-controls"));
  return t.getAttribute("aria-expanded") === "true" && !p.hidden && p.offsetHeight > 0;
}, words);
const openMenu = () => page.evaluate(() => {
  const l = [...document.querySelectorAll(".wf-menu__items")].find((x) => x.hasAttribute("data-open"));
  return l ? l.getAttribute("aria-label") : null;
});

try {
  console.log("\nNavigationMenu");
  const products = await trigger("Products");
  check("a trigger says what it controls, shut", await products.evaluate((t) => t.getAttribute("aria-expanded") === "false" && document.getElementById(t.getAttribute("aria-controls")).hidden));
  await products.click();
  check("a press opens its panel", await panelShown("Products"));
  await (await trigger("Learn")).click();
  check("…and another closes the first", (await panelShown("Learn")) && !(await panelShown("Products")));
  await page.keyboard.press("Escape");
  check("Escape closes it, the focus on its trigger", !(await panelShown("Learn")) && (await focused()) === "Learn", await focused());
  await page.keyboard.press("ArrowRight");
  check("the arrow keys go along the row, links too", (await focused()) === "Pricing", await focused());
  await page.keyboard.press("Home");
  check("Home goes to the first", (await focused()) === "Products", await focused());
  await page.keyboard.press("ArrowDown");
  check("Down opens the panel at its first link", (await panelShown("Products")) && (await focused()) === "Editor", await focused());
  await page.mouse.click(900, 600);
  check("a press outside closes it", !(await panelShown("Products")));

  console.log("\nMenubar");
  check("a menu bar, one Tab stop", (await attr(".wf-menubar", "role")) === "menubar" &&
    (await page.$$eval(".wf-menubar__trigger", (ts) => ts.map((t) => t.tabIndex).join(","))) === "0,-1,-1");
  await (await trigger("File")).focus();
  await page.keyboard.press("ArrowRight");
  check("the arrow keys go along it, the stop with them", (await focused()) === "Edit" &&
    (await page.$$eval(".wf-menubar__trigger", (ts) => ts.map((t) => t.tabIndex).join(","))) === "-1,0,-1");
  await page.keyboard.press("ArrowDown");
  await pause(50);
  check("Down opens its menu at the first item", (await openMenu()) === "Edit" && (await focused()) === "Undo", `${await openMenu()} ${await focused()}`);
  await page.keyboard.press("ArrowRight");
  await pause(50);
  check("Right in an open menu opens the next one", (await openMenu()) === "View" && (await focused()) === "Zoom in", `${await openMenu()} ${await focused()}`);
  await page.keyboard.press("ArrowLeft");
  await pause(50);
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("ArrowRight");
  await pause(200);
  check("…but on a submenu's entry it opens the submenu", (await focused()) === "Find in files", await focused());
  await page.keyboard.press("Enter");
  await pause(50);
  check("choosing an item runs it and closes the menu", (await text(page, "heard")) === "heard find" && (await openMenu()) === null, await text(page, "heard"));
  await (await trigger("File")).click();
  await pause(50);
  await (await trigger("View")).hover();
  await pause(50);
  check("with one open, the pointer opens another", (await openMenu()) === "View");
  await page.keyboard.press("Escape");
  await pause(50);
  check("Escape closes it", (await openMenu()) === null);

  await page.evaluate(() => document.documentElement.setAttribute("dir", "rtl"));
  await (await trigger("File")).focus();
  await page.keyboard.press("ArrowLeft");
  check("right to left, the arrow toward the line's end goes on", (await focused()) === "Edit", await focused());
  await page.evaluate(() => document.documentElement.setAttribute("dir", "ltr"));

  console.log("\nContextMenu");
  const at = await page.$eval("#target", (c) => { const r = c.getBoundingClientRect(); return { x: Math.round(r.x + 20), y: Math.round(r.y + 10) }; });
  await page.mouse.click(at.x, at.y, { button: "right" });
  await pause(50);
  const placed = await page.evaluate(() => {
    const l = document.querySelector('.wf-context-menu .wf-menu__items');
    const r = l.getBoundingClientRect();
    return { x: Math.round(r.x), y: Math.round(r.y), open: l.hasAttribute("data-open") };
  });
  check("a right-click opens it where it happened", placed.open && Math.abs(placed.x - at.x) <= 2 && Math.abs(placed.y - at.y) <= 2, JSON.stringify({ placed, at }));
  check("…the focus on its first item", (await focused()) === "Rename", await focused());
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await pause(50);
  check("an item chosen runs and the menu closes", (await text(page, "heard")) === "heard delete" && (await openMenu()) === null, await text(page, "heard"));
  await page.focus("#file");
  await page.keyboard.down("Shift");
  await page.keyboard.press("F10");
  await page.keyboard.up("Shift");
  await pause(50);
  check("Shift+F10 opens it at what has the focus", (await openMenu()) === "File actions" && (await focused()) === "Rename", `${await openMenu()} ${await focused()}`);
  await page.keyboard.press("Escape");
  await pause(50);
  check("Escape closes it, the focus back where it was", (await openMenu()) === null && (await focused()) === "notes.txt", await focused());

  console.log("\nPrevNext");
  const links = await page.$$eval(".wf-prev-next a", (as) => as.map((a) => `${a.rel}:${[...a.children].map((c) => c.textContent.trim()).join(" ")}`).join("|"));
  check("a link each way, saying which and where", links === "prev:Previous Forms|next:Next Stores", links);
  check("the next one at the end of the line", await page.evaluate(() => {
    const nav = document.querySelector(".wf-prev-next").getBoundingClientRect();
    const next = document.querySelector('.wf-prev-next [data-dir="next"]').getBoundingClientRect();
    return Math.abs(nav.right - next.right) < 2;
  }));

  console.log("\nBottomNavigation");
  const bar = await page.evaluate(() => {
    const nav = document.querySelector(".wf-bottom-nav");
    const r = nav.getBoundingClientRect();
    const sizes = [...nav.querySelectorAll("a")].map((a) => Math.min(a.offsetWidth, a.offsetHeight));
    return { bottom: Math.round(r.bottom), name: nav.getAttribute("aria-label"), small: Math.min(...sizes), current: nav.querySelector('[aria-current="page"]')?.textContent.trim() };
  });
  check("along the bottom of the screen, named", bar.bottom === 900 && bar.name === "Sections", JSON.stringify(bar));
  check("every entry at least 44px to press, the current one marked", bar.small >= 44 && bar.current === "Home", JSON.stringify(bar));
  await page.click('.wf-bottom-nav a[href="/guide"]');
  await pause(150);
  check("an entry goes where it says, and is then the current one", (await page.$eval('.wf-bottom-nav [aria-current="page"]', (a) => a.textContent.trim())) === "Me");

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
