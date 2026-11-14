// The built-ins in a real Chrome, against `tests/fixtures/standards`, held to
// the rules a widget is held to where only a real layout can tell:
//
// - a switch is reached and toggled from the keyboard, and is a switch to a
//   screen reader;
// - a sidebar shut off a narrow page is out of the Tab order, and opening
//   it takes the focus into it;
// - a press on a modal's backdrop closes it, and a press inside does not;
// - a toast shown over a modal is drawn above it and is heard, and goes back
//   to the page when the modal closes;
// - a `show` whose exit ends after it came back stays shown;
// - an action called twice is pending until both calls end;
// - in a right-to-left box a submenu opens to the left, on ArrowLeft.
//
//   cd tests/browser && npm ci && node standards.mjs

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
const fixture = path.join(repo, "tests/fixtures/standards");
const port = 4791;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());
const heard = async (page, words) => JSON.stringify(await page.accessibility.snapshot({ interestingOnly: false })).includes(words);

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-standards-site-"));
fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
const problems = [];
page.on("pageerror", (e) => problems.push(e.message));
const open = async (width) => {
  await page.setViewport({ width, height: 800 });
  await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
};

try {
  console.log("\nSwitch");
  await open(1000);
  const reached = await page.evaluate(() => {
    const input = document.getElementById("notify");
    input.focus();
    return document.activeElement === input;
  });
  check("the keyboard reaches a switch", reached);
  await page.keyboard.press("Space");
  check("…and Space toggles it", (await text(page, "notified")) === "on", await text(page, "notified"));
  const tree = await page.accessibility.snapshot({ interestingOnly: true });
  const find = (n, f) => (f(n) ? n : (n.children || []).map((c) => find(c, f)).find(Boolean));
  const sw = find(tree, (n) => n.role === "switch");
  check("…and a screen reader hears a switch called Notifications, on", sw && sw.name === "Notifications" && sw.checked === true, JSON.stringify(sw));

  console.log("\nSidebar, on a narrow page");
  await open(400);
  const shut = await page.evaluate(() => {
    const nav = document.getElementById("nav");
    const link = nav.querySelector("a");
    link.focus();
    return { visibility: getComputedStyle(nav).visibility, focused: document.activeElement === link };
  });
  check("shut, it is hidden, and its links cannot take the focus", shut.visibility === "hidden" && !shut.focused, JSON.stringify(shut));
  await page.click(".wf-sidebar__toggle");
  await pause(50);
  const opened = await page.evaluate(() => {
    const nav = document.getElementById("nav");
    return { visibility: getComputedStyle(nav).visibility, inside: nav.contains(document.activeElement) };
  });
  check("opened from its toggle, it shows and the focus goes into it", opened.visibility === "visible" && opened.inside, JSON.stringify(opened));
  await page.keyboard.press("Escape");
  await pause(400);
  const closed = await page.evaluate(() => ({
    visibility: getComputedStyle(document.getElementById("nav")).visibility,
    back: document.activeElement.classList.contains("wf-sidebar__toggle"),
  }));
  check("Escape shuts it, hides it once it has slid out, and the focus goes back to its toggle", closed.visibility === "hidden" && closed.back, JSON.stringify(closed));

  console.log("\nModal");
  await open(1000);
  await page.click("#edit");
  await pause(50);
  await page.click("#modal h3");
  await pause(100);
  check("a press inside the modal leaves it open", (await page.$eval("#modal", (d) => d.open)) && (await text(page, "editing")) === "editing");
  await page.mouse.click(5, 5);
  await pause(100);
  check("a press on its backdrop closes it, and its state follows", !(await page.$eval("#modal", (d) => d.open)) && (await text(page, "editing")) === "closed", await text(page, "editing"));

  console.log("\nA toast over a modal");
  await page.click("#edit");
  await pause(50);
  await page.click("#save-modal");
  await pause(400);
  const toast = await page.evaluate(() => {
    const t = document.querySelector(".wf-toast");
    if (!t) return { found: false };
    const r = t.getBoundingClientRect();
    const top = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
    return { found: true, onTop: !!top && t.contains(top), inModal: !!t.closest("dialog") };
  });
  check("it is drawn above the modal", toast.found && toast.onTop, JSON.stringify(toast));
  check("…and a screen reader can reach it", await heard(page, "Profile saved"));
  await page.keyboard.press("Escape");
  await pause(100);
  const home = await page.evaluate(() => {
    const c = document.querySelector(".wf-toast-container");
    const t = document.querySelector(".wf-toast");
    const r = t && t.getBoundingClientRect();
    return { parent: c && c.parentNode === document.body, shown: !!r && r.width > 0 };
  });
  check("when the modal closes the toast goes back to the page, still shown", home.parent && home.shown, JSON.stringify(home));

  console.log("\nshow");
  await page.click("#toggle");
  await pause(150);
  await page.click("#toggle");
  await pause(900);
  const shown = await page.$eval("#shown", (n) => n.getClientRects().length > 0);
  check("an exit that ends after the content came back hides nothing", shown);

  console.log("\nAn action called twice");
  await page.click("#slow");
  await page.click("#fast");
  await pause(300);
  check("it is pending while the slower call still runs", (await text(page, "busy")) === "busy", await text(page, "busy"));
  await pause(600);
  check("…and not once both have ended", (await text(page, "busy")) === "idle" && (await text(page, "said")) === "saved 600", `${await text(page, "busy")} ${await text(page, "said")}`);

  console.log("\nA submenu in a right-to-left box");
  await page.focus("#rtl .wf-menu__trigger");
  await page.keyboard.press("ArrowDown");
  await pause(50);
  await page.keyboard.press("ArrowLeft");
  await pause(100);
  const sub = await page.evaluate(() => {
    const item = document.activeElement;
    const list = item && item.closest("ul");
    const owner = document.querySelector("#rtl [aria-haspopup='menu'][aria-expanded='true']:not(.wf-menu__trigger)");
    if (!list || !owner) return { opened: false, focused: item && item.textContent };
    return { opened: true, focused: item.textContent.trim(), left: list.getBoundingClientRect().right <= owner.getBoundingClientRect().left + 4 };
  });
  check("ArrowLeft opens it and moves into it", sub.opened && sub.focused === "الوارد", JSON.stringify(sub));
  check("…and it opens to the left of its item", sub.left === true, JSON.stringify(sub));
  await page.keyboard.press("ArrowRight");
  await pause(50);
  const back = await page.evaluate(() => document.activeElement && document.activeElement.textContent.trim());
  check("ArrowRight goes back to the item", back.startsWith("نقل إلى"), back);

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
