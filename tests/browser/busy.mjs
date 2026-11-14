// Interactions that hold up, in a real Chrome, against `tests/fixtures/busy`:
//
// - a double click on a button whose action is still running runs it once,
//   and the button is busy (and says so) until the action ends;
// - a `concurrent` action runs every call; one called with other arguments
//   (another row's toggle) runs beside the first;
// - a form submitted twice while its handler's work runs is sent once;
// - `Button(loading:)` is busy while the state holds, and a press does
//   nothing;
// - `on input(debounce:)` runs once the typing stops, `on scroll(throttle:)`
//   at most once a while;
// - `on visible` runs when the element is scrolled into view;
// - what is playing its exit takes no press;
// - `wf verify` names a control drawn under 24×24.
//
//   cd tests/browser && npm ci && node busy.mjs

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
const fixture = path.join(repo, "tests/fixtures/busy");
const port = 4793;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());
const attr = (page, id, a) => page.$eval(`#${id}`, (n, a) => n.getAttribute(a), a);

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-busy-site-"));
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

try {
  console.log("\nA double click");
  await page.click("#save");
  await page.click("#save");
  await page.click("#save");
  const during = { busy: await attr(page, "save", "aria-busy"), disabled: await attr(page, "save", "aria-disabled"), focus: await page.evaluate(() => document.activeElement.id) };
  check("while its action runs the button is busy, and keeps the focus", during.busy === "true" && during.disabled === "true" && during.focus === "save", JSON.stringify(during));
  await pause(500);
  check("…the action ran once", (await text(page, "saved")) === "saved 1", await text(page, "saved"));
  check("…and the button is itself again", (await attr(page, "save", "aria-busy")) === null);
  await page.click("#save");
  await pause(500);
  check("pressed again once it is done, it runs again", (await text(page, "saved")) === "saved 2", await text(page, "saved"));

  console.log("\nconcurrent, and other arguments");
  await page.click("#poll");
  await page.click("#poll2");
  await pause(350);
  check("a concurrent action runs every call", (await text(page, "polled")) === "polled 2", await text(page, "polled"));
  await page.click("#ta");
  await page.click("#tb");
  await pause(350);
  check("a call with other arguments runs beside the first", (await text(page, "toggled")) === "toggled a,b", await text(page, "toggled"));

  console.log("\nA form sent twice");
  await page.click("#pay");
  await page.click("#pay");
  check("the form is busy while its work runs", (await attr(page, "form", "aria-busy")) === "true");
  await pause(450);
  check("…and was sent once", (await text(page, "paid")) === "paid 1", await text(page, "paid"));

  console.log("\nButton(loading:)");
  await page.click("#start");
  await pause(50);
  check("while the state holds it is busy", (await attr(page, "hold", "aria-busy")) === "true");
  await page.click("#hold");
  await pause(50);
  check("…and a press does nothing", (await text(page, "paid")) === "paid 1", await text(page, "paid"));
  await pause(350);
  const spin = await page.$eval("#hold", (b) => ({ width: b.getBoundingClientRect().width, label: getComputedStyle(b).webkitTextFillColor, spinner: getComputedStyle(b, "::after").opacity }));
  check("past a moment its label gives way to a spinner", spin.spinner === "1" && spin.label === "rgba(0, 0, 0, 0)", JSON.stringify(spin));
  await page.click("#start");
  await pause(50);
  await page.click("#hold");
  await pause(50);
  check("once it is done, a press works", (await text(page, "paid")) === "paid 101", await text(page, "paid"));

  console.log("\ndebounce and throttle");
  await page.click("#search");
  await page.keyboard.type("lamp", { delay: 40 });
  await pause(100);
  check("while the reader types, nothing is asked", (await text(page, "asked")) === "asked  0", await text(page, "asked"));
  await pause(350);
  check("once the typing stops, it runs once, with what was typed", (await text(page, "asked")) === "asked lamp 1", await text(page, "asked"));
  for (let i = 1; i <= 20; i++) {
    await page.$eval("#scroller", (s, i) => { s.scrollTop = i * 40; }, i);
    await pause(20);
  }
  await pause(300);
  const scrolls = Number((await text(page, "scrolls")).split(" ")[1]);
  check("twenty scrolls in 400ms run it a few times, the last included", scrolls >= 2 && scrolls <= 5, String(scrolls));

  console.log("\nLeaving");
  await page.click("#hide");
  await pause(50);
  const leaving = await page.$eval("#leaving", (n) => n.inert);
  const inside = await page.$eval("#inside", (b) => { const r = b.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; });
  await page.mouse.click(inside.x, inside.y);
  check("a card playing its exit is inert, and takes no press", leaving && (await text(page, "pressed")) === "pressed 0", String(leaving));
  await pause(700);
  check("…then it is gone", (await page.$("#leaving")) === null);

  console.log("\non visible");
  check("off screen, it has not run", (await text(page, "loads")) === "loads 0", await text(page, "loads"));
  await page.$eval("#end", (n) => n.scrollIntoView({ behavior: "instant", block: "center" }));
  await pause(300);
  check("scrolled into view, it runs", (await text(page, "loads")) === "loads 1", await text(page, "loads"));

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
}

console.log("\nwf verify");
try {
  const out = execFileSync(WF, ["verify", dir, "--json"], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  const report = JSON.parse(out);
  const small = report.routes.flatMap((r) => r.small);
  check("it names the button drawn 16×16, and only that", small.length === 1 && small[0] === 'button "x" 16×16', JSON.stringify(small));
} catch (e) {
  check("wf verify runs", false, `${e.stdout || ""}${e.stderr || ""}`);
}
fs.rmSync(dir, { recursive: true, force: true });

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
