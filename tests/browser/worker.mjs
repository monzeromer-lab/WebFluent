// A worker a store holds, in a real Chrome, against `tests/fixtures/worker`,
// built twice — pre-rendered and as a single page. It starts with the
// store, under the policy the build ships; a request is answered; a failure
// the worker posts is a failure the action catches; what it posts with no
// id is a message the store hears; `pending` counts what is out; a newer
// request supersedes an older one and the worker is told; a request never
// answered times out; a crash fails what was out and starts it again once.
//
//   cd tests/browser && npm ci && node worker.mjs
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
const fixture = path.join(repo, "tests/fixtures/worker");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 3000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-worker-site-"));
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
  const is = (id, want) => until(page, (a) => document.getElementById(a[0])?.textContent === a[1], [id, want]);
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("the worker starts with the store, and the policy lets it", await is("state", "open"), await text("state"));
    await page.click("#upper");
    check("a request is answered", await is("answer", "HALYARD"), await text("answer"));
    await page.click("#shout");
    check("a page's own worker answers too", await is("echoed", "PAGE"), await text("echoed"));
    await page.click("#fail");
    check("a failure the worker posts is one the action catches", await is("failed", "no such word"), await text("failed"));
    await page.click("#lint");
    check("what it posts with no id is a message the store hears", await is("pushed", "9 characters"), await text("pushed"));
    await page.click("#slow");
    check("pending counts what is out", await is("pending", "1"), await text("pending"));
    check("a newer request supersedes the older", await is("failed", "Superseded by a newer request"), await text("failed"));
    check("and the newer is answered", await is("answer", "slow 2"), await text("answer"));
    check("and the worker was told to stop the older", await until(page, () => /^\d+$/.test(document.getElementById("cancelled").textContent)), await text("cancelled"));
    check("and nothing is out after", await is("pending", "0"), await text("pending"));
    await page.click("#never");
    check("a request never answered times out", await is("failed", "The worker did not answer `never` in 1000 ms"), await text("failed"));
    await page.click("#never");
    await page.click("#crash");
    check("a crash fails what was out", await until(page, () => /fell over/.test(document.getElementById("failed").textContent)), await text("failed"));
    check("and starts it again, once", await is("restarted", "1"), await text("restarted"));
    await page.click("#upper");
    await page.evaluate(() => { document.getElementById("answer").dataset.seen = "1"; });
    check("the new one answers", await is("state", "open") && await is("answer", "HALYARD"), `${await text("state")} ${await text("answer")}`);
    await page.click("#crash");
    check("a second crash, with no restarts left, is an error", await until(page, () => /^error: /.test(document.getElementById("state").textContent)), await text("state"));
    // The crashes themselves are the worker's errors on the console: what
    // matters is that nothing in the page threw.
    const own = problems.filter((p) => !/fell over/.test(p));
    check("nothing else threw, and the policy refused nothing", own.length === 0, own.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4383);
await scenario("A single page", false, 4384);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
