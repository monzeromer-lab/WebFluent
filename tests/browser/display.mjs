// The second group of display and layout built-ins in a real Chrome,
// against `tests/fixtures/display`: pairs, events, a row of avatars, an
// item, a gallery, a masonry, a ring, a meter, a sparkline, a relative
// time, a box that keeps its shape, one that scrolls, one that centres, one
// that sticks, and panels with handles between them.
//
//   cd tests/browser && npm ci && node display.mjs

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
const fixture = path.join(repo, "tests/fixtures/display");
const port = 4800;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-display-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
await page.setViewport({ width: 1000, height: 800 });
await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
const rect = (sel) => page.$eval(sel, (n) => { const r = n.getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height, right: r.right, bottom: r.bottom }; });

try {
  console.log("\nDescriptionList, Timeline");
  const pair = await page.evaluate(() => {
    const dt = document.querySelector("#pairs dt").getBoundingClientRect();
    const dd = document.querySelector("#pairs dd").getBoundingClientRect();
    return dt.right <= dd.left && Math.abs(dt.top - dd.top) < 2;
  });
  check("a grid list puts each term beside its value", pair);
  check("a timeline is an ordered list, each time a <time>", await page.$eval("#events", (ol) =>
    ol.tagName === "OL" && ol.children.length === 2 && ol.querySelector("time").getAttribute("datetime") === "2026-11-14"));

  console.log("\nAvatarGroup, Item");
  const avatars = () => page.$eval("#team", (g) => ({
    shown: [...g.children].filter((c) => !c.hidden).length,
    more: g.querySelector(".wf-avatar-group__more")?.textContent,
    said: g.querySelector(".wf-avatar-group__more")?.getAttribute("aria-label"),
  }));
  let team = await avatars();
  check("past max:, the rest are counted", team.shown === 4 && team.more === "+2" && team.said === "2 more", JSON.stringify(team));
  await page.click("#add");
  await pause(100);
  team = await avatars();
  check("…and counted again when one is added", team.more === "+3" && team.shown === 4, JSON.stringify(team));
  const row = await page.evaluate(() => {
    const x = (s) => document.querySelector(`#row ${s}`).getBoundingClientRect().left;
    return x(".wf-item__media") < x(".wf-item__content") && x(".wf-item__content") < x(".wf-item__actions");
  });
  check("an item's media, then its words, then its actions", row);

  console.log("\nGallery, Masonry");
  const pics = await page.$$eval("#gallery img", (is) => is.map((i) => { const r = i.getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; }));
  check("a gallery of columns: 2, each picture its ratio", pics.length === 2 && Math.abs(pics[0].y - pics[1].y) < 1 && pics[1].x > pics[0].x && Math.abs(pics[0].h - pics[0].w * 0.75) < 2, JSON.stringify(pics));
  check("a masonry in its columns", (await page.$eval("#masonry", (m) => getComputedStyle(m).columnCount)) === "2");

  console.log("\nCircularProgress, Meter, Sparkline, RelativeTime");
  const ring = () => page.$eval("#ring", (r) => ({ now: r.getAttribute("aria-valuenow"), dash: r.querySelector(".wf-circular-progress__bar").getAttribute("stroke-dasharray") }));
  check("a ring says its value and fills to it", JSON.stringify(await ring()) === JSON.stringify({ now: "40", dash: "40 100" }), JSON.stringify(await ring()));
  await page.click("#more");
  await pause(50);
  check("…and follows it", (await ring()).now === "70");
  check("with no value it turns, and says no value", await page.$eval("#spinning", (r) => r.hasAttribute("data-indeterminate") && !r.hasAttribute("aria-valuenow") && r.getAttribute("role") === "progressbar"));
  check("a meter is a native meter with its range", await page.$eval("#disk", (m) => m.tagName === "METER" && m.value === 0.95 && m.high === 0.9 && m.getAttribute("aria-label") === "Disk"));
  check("a sparkline is a picture with a name", await page.$eval("#spark", (s) => s.getAttribute("role") === "img" && s.getAttribute("aria-label") === "Sign-ups this week" && !!s.querySelector("svg polyline")));
  check("…and .bar draws bars", (await page.$$("#bars svg rect")).length === 3);
  const when = await page.$eval("#when", (t) => ({ text: t.textContent, at: t.getAttribute("datetime"), title: t.getAttribute("title") }));
  check("a relative time says how long ago, the exact time its title", /years ago/.test(when.text) && when.at === "2020-01-01T00:00:00Z" && /2020/.test(when.title), JSON.stringify(when));

  console.log("\nAspectRatio, ScrollArea, Center");
  const box = await rect("#box");
  check("a box keeps its ratio", Math.abs(box.h - box.w * 9 / 16) < 2, JSON.stringify(box));
  const log = () => page.$eval("#log", (l) => ({ h: l.clientHeight, more: l.scrollHeight > l.clientHeight, before: l.hasAttribute("data-before"), after: l.hasAttribute("data-after"), top: l.scrollTop }));
  let area = await log();
  check("a scroll area holds to its height, the fade at its far edge", area.h <= 120 && area.more && !area.before && area.after, JSON.stringify(area));
  check("…named, and a Tab stop", await page.$eval("#log", (l) => l.tabIndex === 0 && l.getAttribute("role") === "region" && l.getAttribute("aria-label") === "Log"));
  await page.focus("#log");
  await page.keyboard.press("End");
  await pause(300);
  area = await log();
  check("a keyboard scrolls it, and the fades follow", area.top > 0 && area.before && !area.after, JSON.stringify(area));
  const centred = await page.evaluate(() => {
    const b = document.getElementById("middle").getBoundingClientRect();
    const t = document.querySelector("#middle > *").getBoundingClientRect();
    return Math.abs((b.left + b.right) / 2 - (t.left + t.right) / 2) < 2 && Math.abs((b.top + b.bottom) / 2 - (t.top + t.bottom) / 2) < 2;
  });
  check("a centre centres both ways", centred);

  console.log("\nPanelGroup");
  const handle = '#panels .wf-panel-group__handle';
  check("a separator between the panels, saying where it is", await page.$eval(handle, (h) => h.getAttribute("role") === "separator" && h.getAttribute("aria-valuenow") === "30" && h.tabIndex === 0));
  const share = () => page.evaluate(() => {
    const g = document.getElementById("panels").getBoundingClientRect().width;
    return Math.round((document.querySelector("#panels .wf-panel-group__panel").getBoundingClientRect().width / g) * 100);
  });
  check("its panels at the bound sizes", Math.abs((await share()) - 30) <= 1, String(await share()));
  await page.focus(handle);
  await page.keyboard.press("ArrowRight");
  await pause(50);
  check("an arrow key moves it, and the binding follows", (await text(page, "sizes")) === "sizes 32,68", await text(page, "sizes"));
  await page.keyboard.press("Home");
  await pause(50);
  check("Home takes it to the panel's min:", (await text(page, "sizes")) === "sizes 20,80", await text(page, "sizes"));
  const h = await rect(handle);
  await page.mouse.move(h.x + h.w / 2, h.y + h.h / 2);
  await page.mouse.down();
  await page.mouse.move(h.x + h.w / 2 + 300, h.y + h.h / 2, { steps: 5 });
  await page.mouse.up();
  await pause(50);
  const dragged = Number((await text(page, "sizes")).split(" ")[1].split(",")[0]);
  check("dragged, it follows the pointer", dragged > 40, String(dragged));

  console.log("\nSticky");
  await page.evaluate(() => window.scrollTo(0, document.getElementById("bar").offsetTop + 600));
  await pause(200);
  const stuck = await page.$eval("#bar", (b) => ({ stuck: b.hasAttribute("data-stuck"), top: Math.round(b.getBoundingClientRect().top) }));
  check("scrolled past, it sticks at its offset and says so", stuck.stuck && stuck.top === 8, JSON.stringify(stuck));
  await page.evaluate(() => window.scrollTo(0, 0));
  await pause(200);
  check("…and not before", !(await page.$eval("#bar", (b) => b.hasAttribute("data-stuck"))));

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
