// Long lists in a real Chrome, against `tests/fixtures/windowed`, built
// twice — pre-rendered and as a single page. Ten thousand rows of one
// height draw a window of them, and the scrollbar is the whole list's; a
// scroll draws the rows it reaches and lets go of the rest; a row kept in
// the window is the same node; a click on a drawn row reaches its handler;
// the list shrinking redraws the window; rows of two heights are measured,
// and `reveal:` scrolls the current match into view, to the last one; the
// paint carries a screenful, not the whole list.
//
//   cd tests/browser && npm ci && node windowed.mjs
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
const fixture = path.join(repo, "tests/fixtures/windowed");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 2000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);
const frames = (page) => page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-windowed-site-"));
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
  await page.setViewport({ width: 900, height: 1000 });
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  page.on("console", (m) => { if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text()); });
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  const drawn = (sel) => page.evaluate((s) => document.querySelectorAll(`#${s} > p, #${s} > h2`).length, sel);
  try {
    if (ssg) {
      const painted = fs.readFileSync(path.join(dir, "build", "index.html"), "utf8");
      const rows = (painted.match(/id="row \d+"/g) || []).length;
      check("the paint carries a screenful, not the whole list", rows > 0 && rows <= 60 && painted.length < 60000, `${rows} rows, ${painted.length} bytes`);
    }
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    await frames(page);

    // ── ten thousand rows of one height ──
    let n = await drawn("rows");
    check("ten thousand rows draw a window of them", n > 0 && n <= 30, String(n));
    const height = await page.$eval("#rows", (el) => el.scrollHeight);
    check("and the scrollbar is the whole list's", height === 10000 * 32, String(height));
    const first = await page.evaluate(() => { window.__row5 = document.getElementById("row 5"); return !!window.__row5; });
    await page.$eval("#rows", (el) => { el.scrollTop = 32 * 3; });
    await frames(page);
    check("a row that stays in the window is the same node", await page.evaluate(() => document.getElementById("row 5") === window.__row5) && first);
    await page.$eval("#rows", (el) => { el.scrollTop = 32 * 5000; });
    await frames(page);
    check("a scroll draws the rows it reaches", await until(page, () => !!document.getElementById("row 5003")), String(await drawn("rows")));
    check("and lets go of the rest", await page.evaluate(() => !document.getElementById("row 5") && !window.__row5.isConnected) && (await drawn("rows")) <= 30, String(await drawn("rows")));
    const at = await page.$eval("#row\\ 5003", (el) => { const r = el.getBoundingClientRect(); const box = document.getElementById("rows").getBoundingClientRect(); return Math.round(r.top - box.top); });
    check("where it belongs in the list", at === 3 * 32, String(at));
    await page.click("#row\\ 5003");
    check("a click on a drawn row reaches its handler", await until(page, () => document.getElementById("picked").textContent === "row 5003"));
    await page.click("#fewer");
    await frames(page);
    const after = await page.evaluate(() => {
      const box = document.getElementById("rows");
      const last = document.getElementById("row 49");
      const r = last && last.getBoundingClientRect();
      const b = box.getBoundingClientRect();
      return { n: box.querySelectorAll("p").length, h: box.scrollHeight, last: !!last && r.bottom <= b.bottom + 1 && r.top >= b.top, gone: !document.getElementById("row 5003") };
    });
    check("the list shrinking under the reader shows its end", after.h === 50 * 32 && after.n > 0 && after.n <= 30 && after.last && after.gone, JSON.stringify(after));

    // ── two heights, measured, and reveal ──
    const heading = await page.$eval("#h0", (el) => el.getBoundingClientRect().height);
    const line = await page.$eval("#h1", (el) => el.getBoundingClientRect().height);
    check("rows of two heights draw as they are", Math.round(heading) === 40 && Math.round(line) === 22, `${heading} ${line}`);
    await page.click("#next");
    check("reveal scrolls the current match into view", await until(page, () => {
      const row = document.getElementById("h37");
      const box = document.getElementById("hits").getBoundingClientRect();
      if (!row) return false;
      const r = row.getBoundingClientRect();
      return r.top >= box.top - 1 && r.bottom <= box.bottom + 1;
    }));
    await page.click("#last");
    check("to the last one", await until(page, () => {
      const row = document.getElementById("h1999");
      const box = document.getElementById("hits").getBoundingClientRect();
      if (!row) return false;
      const r = row.getBoundingClientRect();
      return r.top >= box.top - 1 && r.bottom <= box.bottom + 1;
    }), String(await page.$eval("#hits", (el) => el.scrollTop)));
    const total = await page.$eval("#hits", (el) => el.scrollHeight);
    check("and the measured list is as long as its rows", Math.abs(total - (200 * 40 + 1800 * 22)) < 200 * 40 * 0.05 + 40, String(total));

    check("nothing threw, and the policy refused nothing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4385);
await scenario("A single page", false, 4386);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
