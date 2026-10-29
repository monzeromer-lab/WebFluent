// Gestures in a real Chrome, against `tests/fixtures/gestures`, built twice
// — pre-rendered and as a single page. A splitter follows the pointer
// between its bounds, resets on a double-click and moves with the keys; a
// tab dragged across a strip lands before or after another; a tree row
// lands before, after or inside a folder, and the drag says where it would
// land as it moves; Escape cancels a drag; a drag's last click is not a
// click; a long press is heard and its click is not; a finger swipes.
//
//   cd tests/browser && npm ci && node gestures.mjs
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
const fixture = path.join(repo, "tests/fixtures/gestures");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 2000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-gestures-site-"));
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
  await page.setViewport({ width: 900, height: 900 });
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  page.on("console", (m) => { if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text()); });
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  const text = (id) => page.evaluate((i) => document.getElementById(i)?.textContent ?? null, id);
  const box = (sel) => page.$eval(sel, (el) => { const b = el.getBoundingClientRect(); return { x: b.left, y: b.top, w: b.width, h: b.height, cx: b.left + b.width / 2, cy: b.top + b.height / 2 }; });
  // A mouse drag from one point to another, in steps, as a hand moves.
  const drag = async (from, to, { before } = {}) => {
    await page.mouse.move(from.x, from.y);
    await page.mouse.down();
    await page.mouse.move(from.x + 6, from.y + 2, { steps: 2 });
    await page.mouse.move(to.x, to.y, { steps: 8 });
    if (before) await before();
    await page.mouse.up();
  };
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });

    // ── the splitter ──
    let split = await box("#split");
    const role = await page.$eval("#split", (el) => [el.getAttribute("role"), el.getAttribute("aria-orientation"), el.getAttribute("aria-valuenow"), el.getAttribute("aria-label")]);
    check("a splitter is a separator that says its size", JSON.stringify(role) === JSON.stringify(["separator", "vertical", "200", "Sidebar width"]), JSON.stringify(role));
    await page.mouse.move(split.cx, split.cy);
    await page.mouse.down();
    await page.mouse.move(split.cx + 60, split.cy, { steps: 5 });
    const dragging = await page.$eval("#split", (el) => el.hasAttribute("data-dragging"));
    await page.mouse.up();
    let side = await box("#side");
    check("dragged, it sizes what it is bound to", (await text("width")) === "260" && Math.round(side.w) === 260, `${await text("width")} ${side.w}`);
    check("and says it is being dragged while it is", dragging);
    split = await box("#split");
    await page.mouse.move(split.cx, split.cy);
    await page.mouse.down();
    await page.mouse.move(split.cx + 500, split.cy, { steps: 5 });
    await page.mouse.up();
    check("it stops at max", (await text("width")) === "400", await text("width"));
    split = await box("#split");
    await page.mouse.click(split.cx, split.cy, { clickCount: 2 });
    check("a double-click resets it", await until(page, () => document.getElementById("width").textContent === "200"), await text("width"));
    await page.focus("#split");
    await page.keyboard.press("ArrowLeft");
    check("the arrow keys move it", (await text("width")) === "184", await text("width"));
    await page.keyboard.press("Home");
    check("Home goes to min", (await text("width")) === "120", await text("width"));
    await page.keyboard.press("Enter");
    check("Enter resets it", (await text("width")) === "200", await text("width"));

    // ── a pane whose size is its share until it is dragged ──
    let pane = await box("#pane");
    const share = Math.round(pane.w);
    let ps = await box("#pane-split");
    await page.mouse.move(ps.cx, ps.cy);
    await page.mouse.down();
    await page.mouse.move(ps.cx - 30, ps.cy, { steps: 4 });
    await page.mouse.up();
    pane = await box("#pane");
    check("a drag starts from the size as drawn, not from the bound 0", (await text("pane-size")) === String(share + 30) && Math.round(pane.w) === share + 30, `${share} → ${await text("pane-size")}, drawn ${pane.w}`);
    await page.focus("#pane-split");
    await page.keyboard.press("ArrowRight");
    check("…and so does a key", (await text("pane-size")) === String(share + 30 - 16), await text("pane-size"));
    ps = await box("#pane-split");
    await page.mouse.click(ps.cx, ps.cy, { clickCount: 2 });
    pane = await box("#pane");
    check("a reset is taken as written, below min: back to the share", (await text("pane-size")) === "0" && Math.round(pane.w) === share, `${await text("pane-size")} ${pane.w}`);

    // ── tabs across a strip ──
    let a = await box("#tab-App\\.wf");
    let c = await box("#tab-Card\\.wf");
    await drag({ x: a.cx, y: a.cy }, { x: c.x + c.w * 0.8, y: c.cy });
    check("a tab dropped on the end of another lands after it", (await text("order")) === "Home.wf,Card.wf,App.wf", await text("order"));
    a = await box("#tab-App\\.wf");
    const h = await box("#tab-Home\\.wf");
    let mark = null;
    await drag({ x: a.cx, y: a.cy }, { x: h.x + h.w * 0.2, y: h.cy }, {
      before: async () => { mark = await page.evaluate(() => [document.getElementById("tab-Home.wf").getAttribute("data-drop"), document.getElementById("tab-Home.wf").getAttribute("data-drop-axis"), !!document.querySelector(".wf-drag-ghost"), document.getElementById("tab-App.wf").hasAttribute("data-dragging")]); },
    });
    check("while it moves, the target shows where it would land, and a ghost follows", JSON.stringify(mark) === JSON.stringify(["before", "x", true, true]), JSON.stringify(mark));
    check("on its start it lands before it", (await text("order")) === "App.wf,Home.wf,Card.wf", await text("order"));
    check("and the drop's marks are gone", await page.evaluate(() => !document.querySelector("[data-drop], [data-dragging], .wf-drag-ghost")));

    // ── a tree, with folders ──
    let readme = await box("#row-README\\.md");
    let src = await box("#row-src");
    let said = null;
    await drag({ x: readme.cx, y: readme.cy }, { x: src.cx, y: src.cy }, { before: async () => { said = await text("landing"); } });
    check("the drag says where it would land as it moves", said === "goes inside src", said);
    check("on a folder's middle it lands inside", (await text("into")) === "README.md into src" && (await text("rows")) === "src,public", `${await text("into")} / ${await text("rows")}`);
    check("and the sentence goes when it lands", (await text("landing")) === "", await text("landing"));
    const pub = await box("#row-public");
    src = await box("#row-src");
    await drag({ x: pub.cx, y: pub.cy }, { x: src.cx, y: src.y + 2 });
    check("on its top edge it lands before", (await text("rows")) === "public,src", await text("rows"));

    // ── Escape cancels ──
    src = await box("#row-src");
    const top = await box("#row-public");
    await page.mouse.move(src.cx, src.cy);
    await page.mouse.down();
    await page.mouse.move(top.cx, top.y + 2, { steps: 6 });
    await page.keyboard.press("Escape");
    await page.mouse.up();
    check("Escape cancels a drag: nothing moves", (await text("rows")) === "public,src" && (await page.evaluate(() => !document.querySelector("[data-drop], .wf-drag-ghost"))), await text("rows"));

    // ── rows drawn by a component's call ──
    const fa = await box("#file-a\\.wf");
    const lib = await box("#file-lib");
    await drag({ x: fa.cx, y: fa.cy }, { x: lib.cx, y: lib.cy });
    check("a component's call is dragged by its root, and its root receives", (await text("moved")) === "a.wf inside lib", await text("moved"));
    const fb = await box("#file-b\\.wf");
    const fa2 = await box("#file-a\\.wf");
    await drag({ x: fb.cx, y: fb.cy }, { x: fa2.cx, y: fa2.y + 2 });
    check("and hears where it lands", (await text("moved")) === "b.wf before a.wf", await text("moved"));
    await page.mouse.click(fb.cx, fb.cy, { button: "right" });
    check("a DOM event on the call is still its root's", (await text("menu")) === "b.wf contextmenu", await text("menu"));
    await page.keyboard.press("Escape");

    // ── a long press ──
    const press = await box("#press");
    await page.mouse.move(press.cx, press.cy);
    await page.mouse.down();
    await wait(650);
    await page.mouse.up();
    check("a long press is heard, and the click it ends with is not", await until(page, () => document.getElementById("pressed").textContent === "held") && (await wait(100), (await text("pressed")) === "held"), await text("pressed"));
    await page.mouse.click(press.cx, press.cy);
    check("a short one is a click", await until(page, () => document.getElementById("pressed").textContent === "clicked"), await text("pressed"));

    // ── a swipe, by a finger ──
    const cdp = await page.createCDPSession();
    const sw = await box("#swipe");
    const touch = (type, x, y) => cdp.send("Input.dispatchTouchEvent", { type, touchPoints: type === "touchEnd" ? [] : [{ x, y }] });
    await touch("touchStart", sw.cx + 60, sw.cy);
    for (let i = 1; i <= 6; i++) await touch("touchMove", sw.cx + 60 - i * 20, sw.cy + 2);
    await touch("touchEnd");
    check("a finger swept left is a swipe left", await until(page, () => document.getElementById("swiped").textContent === "left"), await text("swiped"));
    await page.mouse.move(sw.cx - 60, sw.cy);
    await page.mouse.down();
    await page.mouse.move(sw.cx + 60, sw.cy, { steps: 6 });
    await page.mouse.up();
    check("a mouse is not a finger: it does not swipe", (await text("swiped")) === "left", await text("swiped"));

    check("nothing threw, and the policy refused nothing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4395);
await scenario("A single page", false, 4396);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
