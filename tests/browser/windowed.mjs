// Long lists in a real Chrome, against `tests/fixtures/windowed`, built
// twice — pre-rendered and as a single page. Ten thousand rows of one
// height draw a window of them, and the scrollbar is the whole list's; a
// scroll draws the rows it reaches and lets go of the rest; a row kept in
// the window is the same node; a click on a drawn row reaches its handler;
// the list shrinking redraws the window; rows of two heights are measured,
// and `reveal:` scrolls the current match into view, to the last one; a
// list built before it is in the page paints its first frame with the rows
// in view and the overscan, not a window's height of them; the paint
// carries a screenful, not the whole list.
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
  // What reaches the window's `error` and not the console: a
  // ResizeObserver that changed what it observes, among it.
  await page.evaluateOnNewDocument(() => {
    window.__errors = [];
    window.addEventListener("error", (e) => window.__errors.push(e.message));
  });
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

    // ── options: where each is, focus, a row that changes shape or grows ──
    const pos = await page.$eval("#o3", (el) => [el.getAttribute("aria-posinset"), el.getAttribute("aria-setsize")]);
    check("an option says where it is in the whole list", JSON.stringify(pos) === JSON.stringify(["4", "500"]), JSON.stringify(pos));
    const before = await page.$eval("#o4", (el) => el.getBoundingClientRect().top);
    await page.click("#flip");
    await frames(page);
    const flipped = await page.evaluate(() => {
      const two = document.getElementById("o2");
      const box = document.getElementById("opts");
      return { tag: two && two.tagName, inBox: !!two && box.contains(two), stray: [...document.querySelectorAll("#opts > p#o2")].length, top4: document.getElementById("o4").getBoundingClientRect().top };
    });
    check("a row whose `if` flips is drawn as it is now, in its place", flipped.tag === "H2" && flipped.inBox && flipped.stray === 0, JSON.stringify(flipped));
    check("and the rows after it move down by what it grew", Math.round(flipped.top4 - before) > 20, `${before} → ${flipped.top4}`);
    await page.$eval("#opts", (el) => { el.scrollTop = 24 * 300; });
    await frames(page);
    await page.$eval("#opts", (el) => { el.scrollTop = 0; });
    await frames(page);
    const back = await page.evaluate(() => [...document.querySelectorAll("#o2")].map((n) => n.tagName));
    check("scrolled away and back, it is one row, still a heading", JSON.stringify(back) === JSON.stringify(["H2"]), JSON.stringify(back));
    const short4 = await page.$eval("#o4", (el) => el.getBoundingClientRect().top);
    await page.click("#grow");
    await frames(page);
    await frames(page);
    const grown = await page.evaluate(() => ({
      h2: document.getElementById("o2").getBoundingClientRect().height,
      h3: document.getElementById("o3").getBoundingClientRect().height,
      h: document.getElementById("o5").getBoundingClientRect().height,
    }));
    // Scrolled past it, a row far down stands where the real heights put
    // it: a stale height for row three would leave it short by the growth.
    await page.$eval("#opts", (el) => { el.scrollTop = 24 * 100; });
    await frames(page);
    await frames(page);
    const [index, far] = await page.evaluate(() => {
      const box = document.getElementById("opts");
      const rows = [...box.querySelectorAll("[data-wf-index]")];
      const row = rows[Math.floor(rows.length / 2)];
      return [Number(row.getAttribute("data-wf-index")), row.offsetTop - box.firstElementChild.offsetTop];
    });
    const expected = (index - 2) * grown.h + grown.h2 + grown.h3;
    check("a row that grows after it is drawn is measured again", grown.h3 > 40 && Math.abs(far - expected) <= 3, JSON.stringify({ far, expected, grown, short4 }));
    await page.$eval("#opts", (el) => { el.scrollTop = 0; });
    await frames(page);
    await page.focus("#o1");
    await page.$eval("#opts", (el) => { el.scrollTop = 24 * 300; });
    await frames(page);
    const focus = await page.evaluate(() => ({ gone: !document.getElementById("o1"), on: document.activeElement && document.activeElement.id }));
    check("a focused row scrolled away hands focus to the list", focus.gone && focus.on === "opts", JSON.stringify(focus));

    // ── a list built before it is in the page ──
    // Shown from inside a frame: the next frame's first callback, queued
    // before anything the list could queue, sees what this frame painted.
    const late = await page.evaluate(() => new Promise((resolve) => {
      requestAnimationFrame(() => {
        document.getElementById("show-late").click();
        const box = document.getElementById("late");
        const built = { there: !!box, rows: box ? box.querySelectorAll(":scope > p").length : -1 };
        requestAnimationFrame(() => {
          const rows = [...box.querySelectorAll(":scope > p")];
          resolve({
            built,
            height: box.clientHeight,
            ids: rows.map((r) => r.id),
            top: rows.length ? Math.round(rows[0].getBoundingClientRect().top - box.getBoundingClientRect().top) : null,
          });
        });
      });
    }));
    // 320 / 32 = 10 in view, one partial, six overscan either side.
    const most = 320 / 32 + 1 + 2 * 6;
    const inOrder = late.ids.every((id, i) => id === `late ${i}`);
    check("built before it is in the page, it draws no screenful it cannot see",
      late.built.there && late.built.rows <= 6, JSON.stringify(late.built));
    check("its first painted frame holds the rows in view and no more than the overscan past them",
      late.height === 320 && late.ids.length >= 10 && late.ids.length <= most, `${late.ids.length} rows, at most ${most}`);
    check("…the right rows, in order, from the top", inOrder && late.top === 0, JSON.stringify({ first: late.ids[0], last: late.ids.at(-1), top: late.top }));
    await page.$eval("#late", (el) => { el.scrollTop = 32 * 150; });
    await frames(page);
    check("and it scrolls as any list does", await until(page, () => !!document.getElementById("late 150") && !document.getElementById("late 0")));

    const errors = await page.evaluate(() => window.__errors || []);
    check("no layout loop was reported", errors.length === 0, errors.join(" | "));

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
