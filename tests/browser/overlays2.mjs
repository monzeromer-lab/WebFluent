// The second group of overlays in a real Chrome, against
// `tests/fixtures/overlays2`: a card a link previews on hover, a region
// covered while it is busy, pictures opened full screen, a tour of the
// page, and a consent banner that loads a script only on a yes.
//
//   cd tests/browser && npm ci && node overlays2.mjs

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
const fixture = path.join(repo, "tests/fixtures/overlays2");
const port = 4799;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-overlays2-site-"));
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
  return a ? a.getAttribute("aria-label") || a.getAttribute("alt") || a.textContent.trim() : "";
});
const cardOpen = () => page.$eval(".wf-hover-card__content", (c) => c.hasAttribute("data-open"));
const lightbox = () => page.$eval(".wf-lightbox__dialog", (d) => ({
  open: d.open,
  caption: d.querySelector(".wf-lightbox__caption").textContent,
  count: d.querySelector(".wf-lightbox__count").textContent,
  zoomed: d.hasAttribute("data-zoomed"),
}));
const tourCard = () => page.evaluate(() => {
  const c = document.querySelector(".wf-tour__card");
  const step = [...c.querySelectorAll(".wf-tour__step")].find((s) => !s.hidden);
  return {
    open: c.hasAttribute("data-open"),
    title: step && step.querySelector("h2").textContent,
    count: c.querySelector(".wf-tour__count").textContent,
    named: document.getElementById(c.getAttribute("aria-labelledby"))?.textContent,
  };
});

try {
  console.log("\nHoverCard");
  check("its card is hidden at first", !(await cardOpen()));
  await page.hover(".wf-hover-card > a");
  await pause(80);
  check("a pointer passing does not show it", !(await cardOpen()));
  await pause(250);
  check("a pointer that rests does", await cardOpen());
  await page.hover(".wf-hover-card__content p");
  await pause(450);
  check("it stays while the pointer is on it", await cardOpen());
  await page.mouse.move(900, 850);
  await pause(450);
  check("…and goes once the pointer leaves both", !(await cardOpen()));
  await page.focus(".wf-hover-card > a");
  await pause(50);
  check("focus shows it at once", await cardOpen());
  await page.keyboard.press("Escape");
  await pause(50);
  check("Escape hides it", !(await cardOpen()));

  console.log("\nLoadingOverlay");
  check("the region is not busy at first", (await page.$eval("#region", (r) => r.getAttribute("aria-busy"))) === "false");
  await page.click("#busy");
  await pause(50);
  const busy = await page.$eval("#region", (r) => {
    const cover = r.querySelector(".wf-loading-overlay__cover");
    const a = r.getBoundingClientRect();
    const b = cover.getBoundingClientRect();
    return {
      busy: r.getAttribute("aria-busy"),
      cover: getComputedStyle(cover).display,
      inert: !!document.getElementById("inside").closest("[inert]"),
      covers: a.width === b.width && a.height === b.height,
    };
  });
  check("busy, its content out of reach and the cover over the region alone", busy.busy === "true" && busy.cover === "flex" && busy.inert && busy.covers, JSON.stringify(busy));
  await page.click("#busy");
  await pause(50);
  check("…and back", (await page.$eval("#region", (r) => r.getAttribute("aria-busy"))) === "false" && !(await page.$eval("#inside", (b) => !!b.closest("[inert]"))));

  console.log("\nLightbox");
  check("each picture is a button that opens a dialog", await page.$$eval(".wf-lightbox picture img", (is) => is.length === 2 && is.every((i) => i.getAttribute("role") === "button" && i.tabIndex === 0 && i.getAttribute("aria-haspopup") === "dialog")));
  await page.click(".wf-lightbox img");
  await pause(100);
  let box = await lightbox();
  check("a press opens it at that picture, the focus on Close", box.open && box.caption === "The harbour" && box.count === "1 of 2" && (await focused()) === "Close", JSON.stringify(box) + (await focused()));
  await page.keyboard.press("ArrowRight");
  await pause(50);
  box = await lightbox();
  check("the arrow keys go to the next", box.caption === "The hills" && box.count === "2 of 2", JSON.stringify(box));
  await page.keyboard.press("ArrowRight");
  await pause(50);
  check("…and round to the first", (await lightbox()).count === "1 of 2");
  await page.evaluate(() => {
    // A finger dragged from right to left across the picture.
    const stage = document.querySelector(".wf-lightbox__stage");
    const at = (type, x) => stage.dispatchEvent(new PointerEvent(type, { bubbles: true, pointerId: 7, pointerType: "touch", clientX: x, clientY: 400, isPrimary: true }));
    at("pointerdown", 700);
    at("pointermove", 550);
    at("pointermove", 400);
    at("pointerup", 300);
  });
  await pause(50);
  check("a swipe goes to the next", (await lightbox()).count === "2 of 2", (await lightbox()).count);
  await page.keyboard.press("Home");
  await page.keyboard.press("+");
  await pause(50);
  check("+ zooms", (await lightbox()).zoomed && (await page.$eval(".wf-lightbox__zoom", (b) => b.getAttribute("aria-pressed"))) === "true");
  await page.keyboard.press("Escape");
  await pause(300);
  check("Escape closes it, the focus back on the picture", !(await lightbox()).open && (await focused()) === "The harbour", await focused());
  await page.focus(".wf-lightbox picture:nth-child(2) img");
  await page.keyboard.press("Enter");
  await pause(100);
  check("Enter on a picture opens it there", (await lightbox()).count === "2 of 2");
  await page.click(".wf-lightbox__close");
  await pause(300);

  console.log("\nTour");
  await page.click("#start");
  await pause(300);
  let card = await tourCard();
  check("it opens at its first step, named by its title", card.open && card.title === "New" && card.named === "New" && card.count === "1 of 2", JSON.stringify(card));
  check("the focus on Next", (await focused()) === "Next", await focused());
  const lit = await page.evaluate(() => {
    const s = document.querySelector(".wf-tour__spot").getBoundingClientRect();
    const t = document.getElementById("new").getBoundingClientRect();
    return Math.abs(s.left - (t.left - 6)) < 2 && Math.abs(s.top - (t.top - 6)) < 2;
  });
  check("its target is lit through the cut-out", lit);
  await page.keyboard.press("Enter");
  await pause(300);
  card = await tourCard();
  check("Next goes to the next step", card.title === "Search" && card.count === "2 of 2" && (await page.$eval(".wf-tour__next", (b) => b.textContent)) === "Done", JSON.stringify(card));
  const near = await page.evaluate(() => {
    const c = document.querySelector(".wf-tour__card").getBoundingClientRect();
    const t = document.querySelector(".wf-field input").getBoundingClientRect();
    return c.top >= t.bottom - 1 && c.top - t.bottom < 40;
  });
  check("the card stands at that step's target", near);
  await page.click(".wf-tour__next");
  await pause(200);
  check("Done ends it: open is written back, and on finish runs",
    !(await tourCard()).open && (await text(page, "touring")) === "idle" && (await text(page, "heard")) === "heard finished",
    `${await text(page, "touring")} ${await text(page, "heard")}`);
  await page.click("#start");
  await pause(300);
  await page.keyboard.press("Escape");
  await pause(200);
  check("Escape ends it too", (await text(page, "touring")) === "idle" && !(await page.$eval(".wf-tour__spot", (s) => s.matches(":popover-open"))));

  console.log("\nConsentBanner");
  const banner = () => page.$eval(".wf-consent", (b) => !b.hidden && b.offsetHeight > 0);
  check("it asks, and nothing is loaded before the answer", (await banner()) && (await page.evaluate(() => window.counted)) === undefined && (await page.$$('script[src*="count.js"]')).length === 0);
  check("Decline comes first, as easy as Accept", (await page.$$eval(".wf-consent button", (bs) => bs.map((b) => b.textContent + ":" + b.className).join("|"))) === "Decline:wf-btn|Accept:wf-btn");
  await page.click('[data-consent="decline"]');
  await pause(100);
  check("declined, it goes and loads nothing", !(await banner()) && (await page.evaluate(() => window.counted)) === undefined);
  await page.reload({ waitUntil: "networkidle0" });
  check("…and does not ask again", !(await banner()));
  await page.evaluate(() => localStorage.clear());
  await page.reload({ waitUntil: "networkidle0" });
  await page.click('[data-consent="accept"]');
  await pause(300);
  check("accepted, the script loads", (await page.evaluate(() => window.counted)) === 1);
  await page.reload({ waitUntil: "networkidle0" });
  await pause(300);
  check("…and on the next visit, with no question", (await page.evaluate(() => window.counted)) === 1 && !(await banner()));

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
