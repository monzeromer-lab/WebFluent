// Keys in a real Chrome, against `tests/fixtures/keys`, built twice —
// pre-rendered and as a single page. A shortcut on the app is heard on
// every page and in a field; `mod` is Ctrl here (Linux); a chord fires on
// its second step and the second step alone does not fire its own
// shortcut; a step too late is no chord; `on keydown` with `keyIs` matches
// a table of commands; `keyLabel` and a menu item's shortcut are written as
// this system writes them; and `trapFocus:` keeps Tab inside a panel that
// is not a dialog, then gives focus back.
//
//   cd tests/browser && npm ci && node keys.mjs
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
const fixture = path.join(repo, "tests/fixtures/keys");

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
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-keys-site-"));
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
  page.on("console", (m) => { if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text()); });
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  const text = (id) => page.evaluate((i) => document.getElementById(i)?.textContent ?? null, id);
  const focused = () => page.evaluate(() => document.activeElement?.id || document.activeElement?.tagName || null);
  const chord = async (...keys) => {
    for (const k of keys) await page.keyboard.down(k);
    for (const k of keys.reverse()) await page.keyboard.up(k);
  };
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });

    // ── mod, on the app ──
    await chord("Control", "s");
    check("mod+s on the app is Ctrl+S here", await until(page, () => document.getElementById("saves")?.textContent === "1"), await text("saves"));
    await page.focus("#after");
    await chord("Meta", "s");
    await wait(50);
    check("and not Meta+S", (await text("saves")) === "1", await text("saves"));

    // ── a chord ──
    await chord("Control", "k");
    await chord("Control", "s");
    check("a chord fires on its second step", await until(page, () => document.getElementById("said")?.textContent === "shortcut help"), await text("said"));
    check("and its second step does not fire its own shortcut", (await text("saves")) === "1", await text("saves"));

    // ── a command table ──
    await chord("Control", "Shift", "P");
    check("on keydown with keyIs matches a table of commands", await until(page, () => document.getElementById("said")?.textContent === "palette"), await text("said"));
    await chord("Control", "b");
    check("each row of it", await until(page, () => document.getElementById("said")?.textContent === "sidebar"), await text("said"));

    // Said is "sidebar" now: a chord that does not finish leaves it.
    await chord("Control", "k");
    await wait(1700);
    await chord("Control", "s");
    await wait(50);
    check("a second step too late is no chord: it is the shortcut", (await text("saves")) === "2" && (await text("said")) === "sidebar", `${await text("saves")} ${await text("said")}`);

    // ── labels ──
    if (ssg) {
      const painted = fs.readFileSync(path.join(dir, "build", "index.html"), "utf8");
      check("the paint writes a label as most systems do", painted.includes(">Ctrl+Shift+P<"), painted.match(/id="label"[^<]*</)?.[0]);
    }
    check("keyLabel writes it as this system does", (await text("label")) === "Ctrl+Shift+P", await text("label"));
    const item = await page.$eval("#item", (el) => [el.querySelector("kbd")?.textContent, el.getAttribute("aria-keyshortcuts")]);
    check("a menu item's shortcut too, and its aria-keyshortcuts", item[0] === "Ctrl+Shift+P" && item[1] === "Control+Shift+P", JSON.stringify(item));

    // ── a focus trap ──
    await page.click("#open");
    check("a trap moves focus in", await until(page, () => document.activeElement?.id === "query"), await focused());
    await page.keyboard.press("Tab");
    await page.keyboard.press("Tab");
    await page.keyboard.press("Tab");
    check("Tab goes through it", (await focused()) === "close", await focused());
    await page.keyboard.press("Tab");
    check("and round from the last to the first", (await focused()) === "query", await focused());
    await page.keyboard.down("Shift");
    await page.keyboard.press("Tab");
    await page.keyboard.up("Shift");
    check("Shift+Tab from the first goes to the last", (await focused()) === "close", await focused());
    await page.focus("#options");
    await page.keyboard.press("Enter");
    check("a modal opened over a held trap keeps its focus", await until(page, () => document.activeElement?.id === "word"), await focused());
    await page.keyboard.press("Tab");
    check("and the trap does not pull Tab out of it", (await focused()) !== "query" && (await page.evaluate(() => !document.getElementById("panel").contains(document.activeElement))), await focused());
    await page.keyboard.press("Escape");
    check("closing the modal gives focus back inside the trap", await until(page, () => document.activeElement?.id === "options"), await focused());
    await page.click("#after");
    check("focus that lands outside is brought back", await until(page, () => document.getElementById("panel")?.contains(document.activeElement)), await focused());
    await page.focus("#close");
    await page.keyboard.press("Enter");
    check("when it stops holding, focus goes back to what opened it", await until(page, () => document.activeElement?.id === "open"), await focused());
    await page.keyboard.press("Tab");
    check("and Tab is free again", (await focused()) === "after", await focused());

    check("nothing threw, and the policy refused nothing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4393);
await scenario("A single page", false, 4394);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
