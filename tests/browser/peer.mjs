// `peer` in a real Chrome: `tests/fixtures/peer` opened twice in one
// browser — `?role=a`, the initiator, and `?role=b` — signalling over a
// `broadcast` channel. They connect over WebRTC, a message goes each way,
// and when one page leaves the other sees its peer close.
//
//   cd tests/browser && npm ci && node peer.mjs
//
// WF and CHROME as in offline.mjs. WebRTC needs a real browser: nothing in
// the fake DOM the other suites use has an RTCPeerConnection.

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

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
// Polled on a timer: a background tab runs no animation frames, which is
// what `waitForFunction` polls on by default.
const until = (page, fn, timeout = 15000) =>
  page.waitForFunction(fn, { timeout, polling: 200 }).then(() => true, () => false);
const text = (page, id) => page.$eval(`#${id}`, (e) => e.textContent).catch(() => "(none)");

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-peer-site-"));
fs.cpSync(path.join(repo, "tests/fixtures/peer"), dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
const site = host(path.join(dir, "build"), 4373);
await site.start();
const origin = "http://localhost:4373";
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "wf-peer-chrome-"));
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, userDataDir: profile, args: ["--no-sandbox"] });

console.log("\nTwo pages, one peer each");
try {
  // The one that waits first, then the one that offers.
  const b = await browser.newPage();
  await b.goto(`${origin}/?role=b`, { waitUntil: "load" });
  const a = await browser.newPage();
  await a.goto(`${origin}/?role=a`, { waitUntil: "load" });

  const bothOpen = (await until(a, () => document.getElementById("state")?.textContent === "open"))
    && (await until(b, () => document.getElementById("state")?.textContent === "open"));
  check("they connect: both read `open`", bothOpen, `a: ${await text(a, "state")}, b: ${await text(b, "state")}`);

  // A page in the background does not take a click in headless Chrome.
  await a.bringToFront();
  await a.click("#send");
  check("a message from the initiator arrives",
    await until(b, () => document.getElementById("got")?.textContent === "got:hello from a"), await text(b, "got"));
  await b.bringToFront();
  await b.click("#send");
  check("and one the other way",
    await until(a, () => document.getElementById("got")?.textContent === "got:hello from b"), await text(a, "got"));

  // Leaving the page closes its peer, and the other side sees it go.
  await a.close();
  check("when one page leaves, the other sees its peer close",
    await until(b, () => document.getElementById("state")?.textContent === "closed"), await text(b, "state"));
} catch (e) {
  check(`the run (${e.message})`, false);
} finally {
  await browser.close();
  await site.stop().catch(() => {});
  fs.rmSync(profile, { recursive: true, force: true });
  fs.rmSync(dir, { recursive: true, force: true });
}
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
