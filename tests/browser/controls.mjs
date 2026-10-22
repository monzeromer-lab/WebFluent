// Controls in a real Chrome, against `tests/fixtures/controls`, pre-rendered
// and as a single page: a switch, a box and a choice are drawn with their
// inputs before the script runs, ticked as their state starts; a switch
// and a box given their value follow it and tell the page of a click; and
// a bound box with a handler of its own does both.
//
//   cd tests/browser && npm ci && node controls.mjs

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
const fixture = path.join(repo, "tests/fixtures/controls");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

const state = (page) => page.evaluate(() => ({
  said: document.getElementById("said")?.textContent.trim(),
  inputs: Object.fromEntries(["bound", "given", "box", "sm", "md"].map((id) => {
    // An id names the input, which its label holds.
    const input = document.getElementById(id);
    if (input && input.tagName !== "INPUT") return [id, { notAnInput: input.tagName }];
    return [id, input ? { checked: input.checked, aria: input.getAttribute("aria-checked"), role: input.getAttribute("role") } : null];
  })),
}));

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-controls-site-"));
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
  try {
    if (ssg) {
      // Before any script: the page as written.
      await page.setJavaScriptEnabled(false);
      await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
      const painted = await state(page);
      check("pre-rendered, every control has its input, ticked as its state starts",
        painted.inputs.bound?.checked === true && painted.inputs.given?.checked === true && painted.inputs.box?.checked === false
          && painted.inputs.sm?.checked === false && painted.inputs.md?.checked === true,
        JSON.stringify(painted.inputs));
      check("…a switch is a switch to a screen reader", painted.inputs.bound?.role === "switch" && painted.inputs.given?.aria === "true", JSON.stringify(painted.inputs));
      await page.setJavaScriptEnabled(true);
    }
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    let s = await state(page);
    check("live, every control is drawn as its state is", s.said === "true true false md" && s.inputs.given?.checked === true && s.inputs.md?.checked === true, JSON.stringify(s));
    await page.click("label:has(#given)");
    await new Promise((r) => setTimeout(r, 30));
    s = await state(page);
    check("a switch given its value tells the page of a click, and follows it", s.said === "true false false md" && s.inputs.given?.checked === false && s.inputs.given?.aria === "false", JSON.stringify(s));
    await page.click("label:has(#box)");
    s = await state(page);
    check("so does a box", s.said === "true false true md" && s.inputs.box?.checked === true, JSON.stringify(s));
    await page.click("label:has(#bound)");
    await page.click("label:has(#sm)");
    s = await state(page);
    await page.click("label:has(#kept)");
    const heard = await page.$eval("#heard", (el) => el.textContent.trim());
    check("a bound box with its own handler writes its state and runs the handler", heard === "true 1", heard);
    check("a bound switch and a bound choice write their state", s.said === "false false true sm" && s.inputs.sm?.checked === true && s.inputs.md?.checked === false, JSON.stringify(s));
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4405);
await scenario("A single page", false, 4406);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
