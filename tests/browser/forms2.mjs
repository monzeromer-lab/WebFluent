// The second group of form controls in a real Chrome, against
// `tests/fixtures/forms2`: boxes with one for all, a one-time code, words as
// chips, stars, a slider with two thumbs, a drop zone, a range of days, a
// textarea that grows, a password that can be shown, and a time.
//
//   cd tests/browser && npm ci && node forms2.mjs

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
const fixture = path.join(repo, "tests/fixtures/forms2");
const port = 4797;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-forms2-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
await page.setViewport({ width: 1000, height: 2000 });
await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
const box = (label) => page.evaluateHandle((l) => [...document.querySelectorAll(".wf-checkbox-group label")].find((x) => x.textContent.trim() === l).querySelector("input"), label);

try {
  console.log("\nCheckboxGroup");
  const all = await box("Every topping");
  check("the box for all is mixed while some are ticked", await all.evaluate((i) => i.indeterminate && !i.checked));
  await (await box("Onions")).click();
  await pause(50);
  check("ticking a box adds its value", (await text(page, "toppings")) === "toppings olives,onions", await text(page, "toppings"));
  await all.click();
  await pause(50);
  check("the box for all ticks every one", (await text(page, "toppings")) === "toppings olives,onions,peppers" && (await all.evaluate((i) => i.checked && !i.indeterminate)), await text(page, "toppings"));
  await all.click();
  await pause(50);
  check("…and again clears them", (await text(page, "toppings")) === "toppings", await text(page, "toppings"));
  check("a legend names the group", (await page.$eval(".wf-checkbox-group legend", (l) => l.textContent.trim())) === "Toppings");

  console.log("\nInputOTP");
  const otp = await page.$$(".wf-otp__box");
  check("a box a digit, the first one offered the code", otp.length === 4 && (await otp[0].evaluate((i) => i.autocomplete)) === "one-time-code");
  await otp[0].click();
  await page.keyboard.type("12");
  check("typing moves to the next box", (await page.evaluate(() => document.activeElement.getAttribute("aria-label"))) === "Digit 3 of 4");
  await page.keyboard.press("Backspace");
  check("Backspace in an empty box goes back and clears", (await text(page, "code")) === "code 1", await text(page, "code"));
  await otp[0].click();
  await page.evaluate(() => {
    const dt = new DataTransfer();
    dt.setData("text", "9876");
    document.querySelector(".wf-otp__box").dispatchEvent(new ClipboardEvent("paste", { clipboardData: dt, bubbles: true }));
  });
  await pause(50);
  check("a paste fills every box", (await text(page, "code")) === "code 9876", await text(page, "code"));

  console.log("\nTagInput");
  await page.click(".wf-tag-input__field");
  await page.keyboard.type("go,");
  await page.keyboard.type("zig");
  await page.keyboard.press("Enter");
  await pause(50);
  check("a comma or Enter makes a chip", (await text(page, "tags")) === "tags rust,go,zig", await text(page, "tags"));
  await page.keyboard.press("Backspace");
  await pause(50);
  check("Backspace in the empty field takes the last back", (await text(page, "tags")) === "tags rust,go", await text(page, "tags"));
  await page.click('.wf-tag-input [aria-label="Remove rust"]');
  await pause(50);
  check("a chip's own button takes it out", (await text(page, "tags")) === "tags go", await text(page, "tags"));

  console.log("\nRating");
  const group = await page.$('.wf-rating[role="radiogroup"]');
  check("a radio group, the rating checked", (await group.evaluate((g) => g.getAttribute("aria-label"))) === "Your rating" && (await page.$eval('.wf-rating [aria-checked="true"]', (r) => r.getAttribute("aria-label"))) === "3 of 5");
  await page.focus('.wf-rating [aria-checked="true"]');
  await page.keyboard.press("ArrowRight");
  await pause(50);
  check("the arrow keys rate", (await text(page, "stars")) === "stars 4", await text(page, "stars"));
  check("a read-only one is an image saying the rating", (await page.$eval("#shown", (r) => [r.getAttribute("role"), r.getAttribute("aria-label")].join("|"))) === "img|Average: 4 out of 5");

  console.log("\nSlider.range");
  const thumbs = await page.$$(".wf-slider__thumb");
  check("two thumbs, each named for its end", thumbs.length === 2 && (await thumbs[0].evaluate((t) => t.getAttribute("aria-label"))) === "Price, lowest");
  await thumbs[0].focus();
  await page.keyboard.press("ArrowRight");
  await thumbs[1].focus();
  await page.keyboard.press("ArrowLeft");
  await pause(50);
  check("each moves its end", (await text(page, "price")) === "price 21-79", await text(page, "price"));

  console.log("\nFileUpload.dropzone");
  const [chooser] = await Promise.all([page.waitForFileChooser(), page.click(".wf-dropzone__pick")]);
  const file = path.join(dir, "note.txt");
  fs.writeFileSync(file, "hello");
  await chooser.accept([file]);
  await pause(100);
  check("a file chosen is listed, with a button to take it out", (await text(page, "files")) === "files 1" && !!(await page.$('.wf-dropzone__list [aria-label="Remove note.txt"]')), await text(page, "files"));
  await page.click('.wf-dropzone__list [aria-label="Remove note.txt"]');
  await pause(50);
  check("…which does", (await text(page, "files")) === "files 0");

  console.log("\nDateRangePicker");
  await page.click(".wf-date-range__button");
  await pause(100);
  const months = await page.$$('.wf-date-range__panel [role="grid"]');
  check("its button opens two months", months.length === 2 && (await page.$eval(".wf-date-range__button", (b) => b.getAttribute("aria-expanded"))) === "true");
  await page.click(".wf-date-range__preset");
  await pause(100);
  const said = await page.$eval(".wf-date-range__button", (b) => b.textContent);
  check("a preset takes its range, closes, and the button says it", /^Stay: .+ – .+$/.test(said) && (await page.$eval(".wf-date-range__button", (b) => b.getAttribute("aria-expanded"))) === "false", said);

  console.log("\nTextarea.autosize, Input.reveal, a time");
  const before = await page.$eval('textarea[data-autosize="true"]', (t) => t.offsetHeight);
  await page.click('textarea[data-autosize="true"]');
  await page.keyboard.type("one\ntwo\nthree\nfour\nfive\nsix");
  const after = await page.$eval('textarea[data-autosize="true"]', (t) => t.offsetHeight);
  check("the textarea grows with its text", after > before, `${before} → ${after}`);
  const reveal = await page.$(".wf-reveal");
  await page.click('input[type="password"]');
  await page.keyboard.type("hunter2");
  await reveal.click();
  check("the password is shown, the button pressed and saying Hide", (await page.evaluate(() => {
    const b = document.querySelector(".wf-reveal");
    return b.previousElementSibling.type === "text" && b.getAttribute("aria-pressed") === "true" && b.textContent === "Hide";
  })));
  check("a time is a native time field", (await page.$$('input[type="time"]')).length === 1);

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
