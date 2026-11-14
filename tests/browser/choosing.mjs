// The choosing built-ins in a real Chrome, against `tests/fixtures/choosing`:
// a toggle, a group of them (one or several), radio cards, pages, steps, a
// number with − and +, and a calendar a keyboard can work — each held to
// what it says to a screen reader and what the keys do, once more in a
// right-to-left page where the direction matters.
//
//   cd tests/browser && npm ci && node choosing.mjs

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
const fixture = path.join(repo, "tests/fixtures/choosing");
const port = 4795;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());
const focused = (page) => page.evaluate(() => document.activeElement.getAttribute("aria-label") || document.activeElement.textContent.trim());

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-choosing-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
await page.setViewport({ width: 1000, height: 1600 });
const open = async (rtl = false) => {
  await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
  if (rtl) await page.evaluate(() => document.documentElement.setAttribute("dir", "rtl"));
};

try {
  await open();
  console.log("\nToggle");
  const toggle = await page.$(".wf-toggle");
  check("it is pressed when its state holds, and not before", (await toggle.evaluate((b) => b.getAttribute("aria-pressed"))) === "false");
  await toggle.click();
  await pause(50);
  check("…a press presses it and flips its state", (await toggle.evaluate((b) => b.getAttribute("aria-pressed"))) === "true" && (await text(page, "bold")) === "bold true");

  console.log("\nToggleGroup");
  const views = await page.$$eval('[aria-label="View"] button', (b) => b.map((n) => [n.textContent.trim(), n.getAttribute("aria-pressed"), n.getAttribute("tabindex")]));
  check("a named group, the chosen one pressed and the one Tab stop",
    JSON.stringify(views) === JSON.stringify([["Grid", "true", "0"], ["List", "false", "-1"], ["Board", "false", "-1"]]), JSON.stringify(views));
  await page.focus('[aria-label="View"] button');
  await page.keyboard.press("ArrowRight");
  check("the arrow keys move between them", (await focused(page)) === "List", await focused(page));
  await page.keyboard.press("Enter");
  await pause(50);
  check("…and Enter chooses", (await text(page, "view")) === "view list", await text(page, "view"));
  const style = await page.$$('[aria-label="Text style"] button');
  await style[0].click();
  await style[1].click();
  await pause(50);
  check(".multiple holds several", (await text(page, "styles")) === "styles b,i", await text(page, "styles"));
  await style[0].click();
  await pause(50);
  check("…and a second press lets one go", (await text(page, "styles")) === "styles i", await text(page, "styles"));

  console.log("\nRadioGroup");
  const group = await page.evaluate(() => {
    const set = document.querySelector(".wf-radio-group");
    const team = [...set.querySelectorAll("label")].find((l) => l.textContent.trim() === "Team").querySelector("input");
    return { legend: set.querySelector("legend").textContent.trim(), cards: set.dataset.look, description: team.getAttribute("aria-description") };
  });
  check("a fieldset with its legend, drawn as cards, each hint read with its option",
    group.legend === "Plan" && group.cards === "cards" && group.description === "€8 a seat", JSON.stringify(group));
  await page.evaluate(() => [...document.querySelectorAll(".wf-radio-group label")].find((l) => l.textContent.trim() === "Free").querySelector("input").focus());
  await page.keyboard.press("ArrowDown");
  await pause(50);
  check("the arrow keys choose the next", (await text(page, "plan")) === "plan team", await text(page, "plan"));

  console.log("\nPagination");
  const pages = await page.$$eval(".wf-pagination > *", (n) => n.map((e) => e.textContent.trim() || e.getAttribute("aria-label")));
  check("the first, the last, those around the current one, and gaps",
    JSON.stringify(pages) === JSON.stringify(["Previous page", "1", "…", "4", "5", "6", "…", "12", "Next page"]), JSON.stringify(pages));
  check("the current page is aria-current", (await page.$eval('.wf-pagination [aria-current="page"]', (b) => b.textContent.trim())) === "5");
  await page.click('.wf-pagination [aria-label="Next page"]');
  await pause(50);
  check("Next moves on", (await text(page, "page")) === "page 6", await text(page, "page"));
  await page.click('.wf-pagination [aria-label="Page 12"]');
  await pause(50);
  check("…a page goes there, and Next stops at the last", (await text(page, "page")) === "page 12" && (await page.$eval('.wf-pagination [aria-label="Next page"]', (b) => b.disabled)));

  console.log("\nStepper");
  const steps = async () => page.$$eval(".wf-stepper__item > button", (b) => b.map((n) => [n.textContent.trim(), n.dataset.state, n.getAttribute("aria-current")]));
  check("each step's state, the current one marked", JSON.stringify(await steps()) === JSON.stringify([["Account", "current", "step"], ["Billing", "upcoming", null], ["Confirm", "upcoming", null]]), JSON.stringify(await steps()));
  check("…and only its block shown", (await page.$eval(".wf-stepper__panel:not([hidden])", (p) => p.textContent.trim())) === "The account step");
  await page.click(".wf-stepper__item:nth-child(3) > button");
  await pause(50);
  check("pressing a step goes to it, and the one passed is done",
    (await page.$eval(".wf-stepper__panel:not([hidden])", (p) => p.textContent.trim())) === "The confirm step" && (await steps())[1][1] === "done", JSON.stringify(await steps()));

  console.log("\nNumberInput");
  const spin = await page.evaluate(() => {
    const input = document.querySelector(".wf-number-input input");
    const label = document.getElementById(input.getAttribute("aria-labelledby"));
    return { type: input.type, label: label && label.textContent.trim(), min: input.min, max: input.max };
  });
  check("a number input named by its label, with its ends", spin.type === "number" && spin.label === "Quantity" && spin.min === "1" && spin.max === "5", JSON.stringify(spin));
  await page.click('[aria-label="Increase"]');
  await page.click('[aria-label="Increase"]');
  await page.click('[aria-label="Increase"]');
  await pause(50);
  check("+ steps it up and stops at its end", (await text(page, "qty")) === "qty 5" && (await page.$eval('[aria-label="Increase"]', (b) => b.disabled)), await text(page, "qty"));
  await page.focus(".wf-number-input input");
  await page.keyboard.press("PageDown");
  await pause(50);
  check("PageDown goes ten steps, held at the lowest", (await text(page, "qty")) === "qty 1", await text(page, "qty"));

  console.log("\nCalendar");
  const cal = await page.evaluate(() => {
    const box = document.querySelector('.wf-calendar[aria-label="Delivery day"]');
    return { grid: box.querySelector('[role="grid"]') !== null, stops: box.querySelectorAll('.wf-calendar__day[tabindex="0"]').length, title: box.querySelector(".wf-calendar__title").textContent };
  });
  check("a named grid of the month with one Tab stop", cal.grid && cal.stops === 1, JSON.stringify(cal));
  await page.evaluate(() => {
    // Begin on a known day: 2026-03-10, a Tuesday.
    document.querySelector('.wf-calendar[aria-label="Delivery day"] [data-day]').focus();
  });
  await page.click('.wf-calendar[aria-label="Delivery day"] [aria-label="Next month"]');
  await page.click('.wf-calendar[aria-label="Delivery day"] [aria-label="Previous month"]');
  const target = '.wf-calendar[aria-label="Delivery day"] [data-day="2026-03-10"]';
  const shown = await page.$(target);
  if (!shown) {
    // The page shows today's month: go to March 2026 by the month buttons.
    const months = await page.evaluate(() => {
      const t = new Date();
      return (2026 - t.getFullYear()) * 12 + (2 - t.getMonth());
    });
    const button = months > 0 ? "Next month" : "Previous month";
    for (let i = 0; i < Math.abs(months); i++) await page.click(`.wf-calendar[aria-label="Delivery day"] [aria-label="${button}"]`);
  }
  await page.focus(target);
  await page.keyboard.press("ArrowRight");
  check("ArrowRight goes to the next day", (await page.evaluate(() => document.activeElement.dataset.day)) === "2026-03-11");
  await page.keyboard.press("ArrowDown");
  check("ArrowDown a week on", (await page.evaluate(() => document.activeElement.dataset.day)) === "2026-03-18");
  await page.keyboard.press("PageDown");
  check("PageDown a month on", (await page.evaluate(() => document.activeElement.dataset.day)) === "2026-04-18");
  await page.keyboard.press("PageUp");
  await page.keyboard.press("Enter");
  await pause(50);
  check("Enter chooses the day, and the grid marks it", (await text(page, "day")) === "day 2026-03-18" &&
    (await page.$eval('.wf-calendar[aria-label="Delivery day"] [data-day="2026-03-18"]', (b) => b.parentNode.getAttribute("aria-selected"))) === "true", await text(page, "day"));
  await page.click('.wf-calendar[aria-label="Delivery day"] [data-day="2026-03-21"]');
  await page.click('.wf-calendar[aria-label="Delivery day"] [data-day="2026-03-02"]');
  await pause(50);
  check("a Saturday (disabled: by a test) and a day before min: cannot be chosen", (await text(page, "day")) === "day 2026-03-18", await text(page, "day"));
  const range = '#stay';
  await page.click(`${range} [aria-label="Next month"]`);
  const days = await page.$$eval(`${range} [data-day]:not([data-outside])`, (b) => b.slice(4, 9).map((n) => n.dataset.day));
  await page.click(`${range} [data-day="${days[0]}"]`);
  await page.click(`${range} [data-day="${days[3]}"]`);
  await pause(50);
  const picked = await page.$$eval(`${range} td[aria-selected="true"]`, (t) => t.length);
  check("a range takes a first and a last day, and marks those between", (await text(page, "stayed")) === `stay ${days[0]} ${days[3]}` && picked === 4, `${await text(page, "stayed")} ${picked}`);

  console.log("\nRight to left");
  await open(true);
  await page.focus('[aria-label="View"] button');
  await page.keyboard.press("ArrowLeft");
  check("ArrowLeft goes forward through a group", (await focused(page)) === "List", await focused(page));
  const first = await page.$eval('.wf-calendar[aria-label="Delivery day"] [tabindex="0"]', (b) => b.dataset.day);
  await page.focus('.wf-calendar[aria-label="Delivery day"] [tabindex="0"]');
  await page.keyboard.press("ArrowLeft");
  const next = await page.evaluate(() => document.activeElement.dataset.day);
  const d = new Date(`${first}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() + 1);
  check("…and through the days of a calendar", next === d.toISOString().slice(0, 10), `${first} → ${next}`);

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
