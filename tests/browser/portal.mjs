// `Portal` in a real Chrome, against `tests/fixtures/portal`, built twice —
// pre-rendered and as a single page. A plain script makes two empty popup
// nodes outside the page's root, as an editor makes its tooltips; the page
// draws a block into one of them. The block lands there and adds no box to
// the layout (`display: contents`); a `mount:` in it meets its node in the
// document; a button in it whose `on mousedown(e) { e.preventDefault() }`
// keeps the focus in the field the reader is typing in, and its click
// reaches the page's state; `into:` naming the other node moves the same
// box there, with what it shows; naming none keeps it aside, and the same
// box comes back; the pre-rendered page is taken over with no mismatch.
//
//   cd tests/browser && npm ci && node portal.mjs
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
const fixture = path.join(repo, "tests/fixtures/portal");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 5000) =>
  page.waitForFunction(fn, { timeout, polling: 50 }, arg).then(() => true, () => false);

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-portal-site-"));
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
  // As under `wf serve`: the runtime says so when a page it took over is
  // not the page it drew.
  await page.evaluateOnNewDocument(() => { window.__WF_DEV__ = true; });
  const problems = [];
  const warnings = [];
  page.on("pageerror", (e) => problems.push(e.message));
  page.on("console", (m) => {
    if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text());
    if (m.type() === "warn") warnings.push(m.text());
  });
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  const press = (text) => page.evaluate((t) => [...document.querySelectorAll("#app button")].find((b) => b.textContent === t).click(), text);
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("the block is drawn into the node the script made, outside the page's root",
      await until(page, () => !!document.querySelector("#popup-a > .wf-portal #pick")),
      await page.evaluate(() => document.getElementById("popup-layer")?.outerHTML));
    check("nothing of it is drawn where it is written",
      await page.evaluate(() => !document.querySelector("#app .wf-portal") && !document.querySelector("#app #pick")));
    check("the box adds nothing to the layout where it lands",
      await page.evaluate(() => getComputedStyle(document.querySelector("#popup-a > .wf-portal")).display === "contents"));
    check("a data-* argument is the box's",
      await page.evaluate(() => document.querySelector("#popup-a > .wf-portal").dataset.popup === "hint"));
    const mounted = await page.evaluate(() => JSON.stringify(window.mounted || []));
    check("a mount inside it meets its node in the document", mounted === "[true]", mounted);

    // Typing in the page's field, then pressing a button in the popup.
    await page.click("#search");
    await page.keyboard.type("ab");
    await page.evaluate(() => { window.box = document.querySelector("#popup-a > .wf-portal"); });
    await page.click("#pick");
    check("a press on it keeps the focus in the field being typed in",
      await page.evaluate(() => document.activeElement?.id === "search"),
      await page.evaluate(() => document.activeElement?.outerHTML));
    check("and its click reaches the page's state",
      await until(page, () => document.getElementById("picks")?.textContent === "picks 1"),
      await page.evaluate(() => document.getElementById("picks")?.textContent));
    check("what it shows follows the page's state",
      await until(page, () => document.getElementById("typed")?.textContent === "typed ab"),
      await page.evaluate(() => document.getElementById("typed")?.textContent));
    await page.keyboard.type("c");
    check("the field still has the keys", await page.evaluate(() => document.getElementById("search").value === "abc"));

    await press("Use B");
    check("into: naming the other node moves the same box there",
      await until(page, () => document.querySelector("#popup-b > .wf-portal") === window.box && !document.querySelector("#popup-a > *")));
    check("with what it shows", await page.evaluate(() => document.getElementById("typed")?.textContent === "typed abc"));
    check("and its mount is not run again", await page.evaluate(() => (window.mounted || []).length === 1));

    await press("Park");
    check("naming none keeps it aside",
      await until(page, () => window.box.isConnected === false && !document.querySelector(".popup > *")));
    await press("Use A");
    check("and the same box comes back",
      await until(page, () => document.querySelector("#popup-a > .wf-portal") === window.box));
    await page.click("#pick");
    check("still live", await until(page, () => document.getElementById("picks")?.textContent === "picks 2"));

    if (ssg) {
      check("the pre-rendered page has only the portal's place",
        await page.evaluate(async () => {
          const html = await (await fetch("/")).text();
          return html.includes("<!--wf-portal-->") && !html.includes('id="pick"');
        }));
      const mismatch = warnings.filter((w) => w.includes("differs"));
      check("the page taken over is the page drawn", mismatch.length === 0, mismatch.join(" | "));
    }
    check("nothing threw, and the policy refused nothing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4411);
await scenario("A single page", false, 4412);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
