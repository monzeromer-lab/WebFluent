// A storage the project gives itself, in a real Chrome, against
// `tests/fixtures/storage`, built twice — pre-rendered and as a single page.
// A project script under `src/` sets `window.wfStorage.app` before the
// first `persist` reads it, under the policy the build ships; a value kept
// there (and answered a moment later, as an IPC call is) is read back; a
// store's value is kept per project and follows the project; writes go to
// the project's storage, not the browser's.
//
//   cd tests/browser && npm ci && node storage.mjs
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
const fixture = path.join(repo, "tests/fixtures/storage");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 2000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-storage-site-"));
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
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("a page's value is read back from the project's storage", await until(page, () => document.getElementById("mode")?.textContent === "design"), await text("mode"));
    check("and a store's, under the open project's key", await until(page, () => document.getElementById("side")?.textContent === "320"), await text("side"));
    await page.click("#wider");
    check("a write goes to the project's storage", await until(page, () => window.wfWrites.includes("wf:Layout.side:halyard=330")), await page.evaluate(() => window.wfWrites.join(" ")));
    check("not the browser's", await page.evaluate(() => Object.keys(localStorage).length === 0), await page.evaluate(() => Object.keys(localStorage).join(" ")));
    await page.click("#docs");
    check("another project has its own value", await until(page, () => document.getElementById("side")?.textContent === "240"), await text("side"));
    await page.click("#wider");
    await page.click("#halyard");
    check("and coming back finds the first project's", await until(page, () => document.getElementById("side")?.textContent === "330"), await text("side"));
    check("each was written under its own key", await page.evaluate(() => window.wfWrites.includes("wf:Layout.side:docs=250")), await page.evaluate(() => window.wfWrites.join(" ")));
    check("nothing threw, and the policy refused nothing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4387);
await scenario("A single page", false, 4388);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
