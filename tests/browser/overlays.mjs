// Every overlay in a real Chrome, against `tests/fixtures/overlays`, held to
// the rules an overlay is held to:
//
// - when it opens the focus goes in (to what `initialFocus:` names, else
//   its first control), and when it closes the focus comes back to what
//   had it;
// - what closes it is what `dismiss:` says: Escape, a press outside, or
//   neither;
// - it plays its exit before it leaves the top layer;
// - a popover's control says what it opens and whether it is open;
// - an alert dialog is one to a screen reader, and `await ask(…)` hands
//   back the answer;
// - a sheet comes in from its edge and closes when dragged toward it;
// - a toast has its action and its close button, and no more than three
//   show; `toast.promise` follows its promise;
// - `announce(…, .assertive)` is heard at once.
//
//   cd tests/browser && npm ci && node overlays.mjs

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
const fixture = path.join(repo, "tests/fixtures/overlays");
const port = 4792;

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const text = (page, id) => page.$eval(`#${id}`, (n) => n.textContent.trim());
const focused = (page) => page.evaluate(() => {
  const a = document.activeElement;
  return a ? a.id || a.getAttribute("aria-label") || a.textContent.trim() : "";
});
const isOpen = (page, id) => page.$eval(`#${id}`, (d) => d.open);
const tree = async (page) => page.accessibility.snapshot({ interestingOnly: true });
const find = (n, f) => (f(n) ? n : (n.children || []).map((c) => find(c, f)).find(Boolean));

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-overlays-site-"));
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
// A press on the backdrop: the top-left corner, outside every card.
const backdrop = () => page.mouse.click(5, 790);

try {
  console.log("\nModal, dismiss: .escape, initialFocus:");
  await page.click("#edit");
  await pause(50);
  check("it opens with the focus on what initialFocus names", (await focused(page)) === "name", await focused(page));
  await backdrop();
  await pause(50);
  check("a press on its backdrop does not close it", await isOpen(page, "modal"));
  await page.keyboard.press("Escape");
  const exiting = await page.$eval("#modal", (d) => ({ open: d.open, shown: getComputedStyle(d).display !== "none", playing: d.getAnimations().length > 0 }));
  check("Escape closes it, and its exit plays before it goes", !exiting.open && exiting.shown && exiting.playing, JSON.stringify(exiting));
  await pause(300);
  check("…then it is gone, its state follows, and the focus is back on Edit",
    (await page.$eval("#modal", (d) => getComputedStyle(d).display)) === "none" && (await text(page, "editing")) === "closed" && (await focused(page)) === "edit",
    `${await text(page, "editing")} ${await focused(page)}`);

  console.log("\nAlertDialog");
  await page.click("#discard");
  await pause(50);
  const alert = find(await tree(page), (n) => n.role === "alertdialog");
  check("a screen reader hears an alert dialog called by its title, described by its body",
    alert && alert.name === "Discard changes?" && (alert.description || "").includes("Your edits will be lost"), JSON.stringify(alert && { name: alert.name, description: alert.description }));
  check("the focus is in it", await page.evaluate(() => document.getElementById("alert").contains(document.activeElement)));
  await backdrop();
  await pause(50);
  check("a press beside it is not an answer", await isOpen(page, "alert"));
  await page.keyboard.press("Escape");
  await pause(250);
  check("Escape is, and the focus goes back to Discard", !(await isOpen(page, "alert")) && (await focused(page)) === "discard", await focused(page));

  console.log("\nawait ask(…)");
  await page.click("#ask");
  await pause(50);
  const asked = await page.evaluate(() => {
    const d = [...document.querySelectorAll("dialog[role=alertdialog]")].find((n) => n.open);
    return d && { title: d.querySelector("h3").textContent, focus: document.activeElement.textContent, yes: d.querySelector(".wf-btn--danger")?.textContent };
  });
  check("it asks in an alert dialog, with the focus on the answer that loses nothing",
    asked && asked.title === "Delete the project?" && asked.focus === "Cancel" && asked.yes === "Delete", JSON.stringify(asked));
  await page.keyboard.press("Tab");
  await page.keyboard.press("Enter");
  await pause(250);
  check("Delete answers yes", (await text(page, "answer")) === "answer yes", await text(page, "answer"));
  check("…and the focus is back where it was", (await focused(page)) === "ask", await focused(page));
  await page.click("#ask");
  await pause(50);
  await page.keyboard.press("Escape");
  await pause(250);
  check("Escape answers no", (await text(page, "answer")) === "answer no", await text(page, "answer"));
  await pause(400);
  check("…and the question is gone from the page", (await page.$$("dialog[role=alertdialog]:not(#alert)")).length === 0);

  console.log("\nSheet, from the bottom, snap: [0.5, 1]");
  await page.click("#filters");
  await pause(400);
  const sheet = await page.$eval("#sheet", (d) => {
    const r = d.getBoundingClientRect();
    return { bottom: Math.round(r.bottom), height: Math.round(r.height), focus: d.contains(document.activeElement), modal: d.matches(":modal") };
  });
  check("it opens against the bottom edge, half the window tall, modal, with the focus in it",
    sheet.bottom === 800 && Math.abs(sheet.height - 400) <= 2 && sheet.focus && sheet.modal, JSON.stringify(sheet));
  const handle = await page.$eval("#sheet .wf-sheet__handle", (h) => { const r = h.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; });
  await page.mouse.move(handle.x, handle.y);
  await page.mouse.down();
  await page.mouse.move(handle.x, handle.y + 60, { steps: 4 });
  await pause(300);
  await page.mouse.up();
  await pause(400);
  check("dragged a little and let go slowly, it springs back", await isOpen(page, "sheet"));
  await page.mouse.move(handle.x, handle.y);
  await page.mouse.down();
  await page.mouse.move(handle.x, handle.y + 250, { steps: 6 });
  await page.mouse.up();
  await pause(400);
  check("dragged past 30% of its height, it closes, and its state follows",
    !(await isOpen(page, "sheet")) && (await text(page, "filtering")) === "shut", await text(page, "filtering"));
  check("…and the focus is back on Filters", (await focused(page)) === "filters", await focused(page));

  console.log("\nSheet, side: .end");
  await page.click("#details");
  await pause(400);
  const end = await page.$eval("#side", (d) => { const r = d.getBoundingClientRect(); return { right: Math.round(r.right), left: Math.round(r.left), height: Math.round(r.height) }; });
  check("it stands against the end of the line, the window's height", end.right === 1000 && end.left > 500 && end.height === 800, JSON.stringify(end));
  await backdrop();
  await pause(400);
  check("a press outside closes it", !(await isOpen(page, "side")));
  await page.evaluate(() => document.documentElement.setAttribute("dir", "rtl"));
  await page.click("#details");
  await pause(400);
  const rtl = await page.$eval("#side", (d) => Math.round(d.getBoundingClientRect().left));
  check("in a right-to-left page the end is the left", rtl === 0, String(rtl));
  await page.keyboard.press("Escape");
  await pause(400);
  await page.evaluate(() => document.documentElement.setAttribute("dir", "ltr"));

  console.log("\nPopover, dismiss: .escape");
  const shut = await page.$eval("#branches", (b) => ({ expanded: b.getAttribute("aria-expanded"), popup: b.getAttribute("aria-haspopup") }));
  check("its control says it opens a dialog, and that it is shut", shut.expanded === "false" && shut.popup === "dialog", JSON.stringify(shut));
  await page.click("#branches");
  await pause(100);
  const opened = await page.$eval("#branches", (b) => ({ expanded: b.getAttribute("aria-expanded"), controls: b.getAttribute("aria-controls"), focus: document.activeElement.textContent.trim() }));
  check("open, it says so and names the popover, and the focus is on its first control",
    opened.expanded === "true" && opened.controls === "pop" && opened.focus === "main", JSON.stringify(opened));
  await page.mouse.click(900, 700);
  await pause(50);
  check("a press outside does not close it", (await page.$eval("#branches", (b) => b.getAttribute("aria-expanded"))) === "true");
  await page.focus("#pop button");
  await page.keyboard.press("Escape");
  await pause(250);
  check("Escape does, and the focus goes back to its control",
    (await page.$eval("#branches", (b) => b.getAttribute("aria-expanded"))) === "false" && (await focused(page)) === "branches", await focused(page));

  console.log("\nToast");
  await page.click("#archive");
  // In once it has slid in from the window's edge.
  await pause(450);
  const t = await page.evaluate(() => {
    const n = document.querySelector(".wf-toast");
    return n && { action: n.querySelector(".wf-toast__action")?.textContent, close: n.querySelector(".wf-toast__close")?.getAttribute("aria-label"), top: n.parentNode.matches(":popover-open") };
  });
  check("it has its action and a close button, in the top layer", t && t.action === "Undo" && t.close === "Close" && t.top, JSON.stringify(t));
  await page.click(".wf-toast__action");
  await pause(500);
  check("its action runs, and closes it", (await text(page, "answer")) === "answer undone" && (await page.$$(".wf-toast")).length === 0, await text(page, "answer"));
  for (let i = 0; i < 5; i++) await page.click("#archive");
  await pause(500);
  check("no more than three show at once", (await page.$$(".wf-toast:not(.wf-toast--exit)")).length === 3, String((await page.$$(".wf-toast:not(.wf-toast--exit)")).length));
  await page.hover(".wf-toast");
  await pause(100);
  for (const b of await page.$$(".wf-toast__close")) await b.evaluate((n) => n.click());
  await pause(500);
  await page.click("#save");
  await pause(60);
  const saving = await page.evaluate(() => document.querySelector(".wf-toast__text")?.textContent);
  await pause(300);
  const saved = await page.evaluate(() => document.querySelector(".wf-toast:not(.wf-toast--exit) .wf-toast__text")?.textContent);
  check("toast.promise says Saving… while it runs, then Saved", saving === "Saving…" && saved === "Saved", `${saving} ${saved}`);

  console.log("\nannounce");
  await page.click("#lose");
  await pause(150);
  const urgent = await page.evaluate(() => document.querySelector('[aria-live="assertive"]')?.textContent);
  check("announce(…, .assertive) speaks in the assertive region", urgent === "Connection lost", String(urgent));

  check("no errors on the page", problems.length === 0, problems.join("; "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((r) => !r).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
