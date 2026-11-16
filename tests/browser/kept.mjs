// What storage hands back, held to the declared type, in a real Chrome,
// against `tests/fixtures/kept`: a value that fits is read back, and one an
// older build left in another shape — a record missing a field, text where
// a number was — is not taken: the default is used and the page loads.
//
//   cd tests/browser && npm ci && node kept.mjs
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
const fixture = path.join(repo, "tests/fixtures/kept");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 2000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-kept-site-"));
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
  // What an older build left: a record without its `columns`, text where a
  // number is kept, and a scheme that is still the right type.
  await page.evaluateOnNewDocument(() => {
    if (sessionStorage.getItem("seeded")) return;
    sessionStorage.setItem("seeded", "1");
    localStorage.setItem("wf:Settings.prefs", JSON.stringify({ cols: 4 }));
    localStorage.setItem("wf:Home.count", JSON.stringify("seven"));
    localStorage.setItem("wf:Settings.scheme", JSON.stringify("dark"));
  });
  const text = (id) => page.evaluate((i) => document.getElementById(i)?.textContent ?? null, id);
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("a kept value of its type is read back", await until(page, () => document.getElementById("scheme")?.textContent === "dark"), await text("scheme"));
    check("a record in an older shape falls back to the default", await until(page, () => document.getElementById("columns")?.textContent === "3"), await text("columns"));
    check("text kept where a number is falls back to the default", await until(page, () => document.getElementById("count")?.textContent === "0"), await text("count"));
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("pre-rendered", true, 47391);
await scenario("single page", false, 47392);
const passed = results.filter(Boolean).length;
console.log(`\n${passed} of ${results.length} passed`);
process.exit(passed === results.length ? 0 : 1);
