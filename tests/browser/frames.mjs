// A `Frame(srcdoc:)` in a real Chrome, against `tests/fixtures/frames`,
// pre-rendered and as a single page: the frame draws the HTML it is given,
// before the script runs when the page is pre-rendered, sandboxed as
// `sandbox:` says, and draws the new HTML when the state it reads changes.
//
//   cd tests/browser && npm ci && node frames.mjs

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
const fixture = path.join(repo, "tests/fixtures/frames");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

/// What the frame shows: the text of its document, read through the
/// frame itself, since a sandboxed frame's document is another origin's.
async function shown(page) {
  const child = page.frames().find((f) => f !== page.mainFrame());
  if (!child) return null;
  return child.evaluate(() => document.getElementById("said")?.textContent ?? null).catch(() => null);
}

async function until(read, want) {
  for (let i = 0; i < 50; i++) {
    const v = await read();
    if (v === want) return v;
    await new Promise((r) => setTimeout(r, 40));
  }
  return read();
}

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-frames-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  const said = execFileSync(WF, ["build", "-d", dir], { encoding: "utf8" });
  check("the build says nothing about `srcdoc`", !said.includes("srcdoc"), said);
  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  try {
    if (ssg) {
      await page.setJavaScriptEnabled(false);
      await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
      check("pre-rendered, the frame draws its HTML before any script", (await until(() => shown(page), "Version 1")) === "Version 1");
      await page.setJavaScriptEnabled(true);
    }
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("live, the frame draws the HTML it is given", (await until(() => shown(page), "Version 1")) === "Version 1");
    const sandbox = await page.$eval("iframe", (f) => f.getAttribute("sandbox"));
    check("…sandboxed as `sandbox:` says", sandbox === "", String(sandbox));
    const scripts = await page.frames().find((f) => f !== page.mainFrame())
      .evaluate(() => { try { return typeof window.parent.document; } catch { return "refused"; } })
      .catch(() => "refused");
    check("…so its document cannot reach the page's", scripts === "refused", scripts);
    await page.click("button");
    check("a change of the state it reads draws the new HTML", (await until(() => shown(page), "Version 2")) === "Version 2");
    await page.click("button");
    check("…every time", (await until(() => shown(page), "Version 3")) === "Version 3");
    check("one frame, not one per change", (await page.$$("iframe")).length === 1);
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4431);
await scenario("A single page", false, 4432);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
