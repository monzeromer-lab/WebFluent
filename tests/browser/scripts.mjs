// The project's own JavaScript in a real Chrome, against
// `tests/fixtures/scripts`, built twice — pre-rendered and as a single page.
// A script under `src/` is linked and its function called from `.wf`; a
// `mount:` hands an element to it, `update:` follows state, `cleanup:` runs
// when an `if` hides the element and when the route changes; a class a
// script added survives `class:` repainting; a script reports through an
// action and a `class:` map follows; a custom element placed with `Element`
// upgrades, hears its attribute change and fires its own event; `wf:render`
// is said once per drawing; and the policy the build ships refuses nothing.
//
//   cd tests/browser && npm ci && node scripts.mjs
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
const fixture = path.join(repo, "tests/fixtures/scripts");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 5000) =>
  page.waitForFunction(fn, { timeout, polling: 50 }, arg).then(() => true, () => false);

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-scripts-site-"));
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
  // A request that failed is named by its URL; the browser's own guess at a
  // favicon is not the site's to answer.
  page.on("console", (m) => { if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text()); });
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  const taken = () => page.evaluate(() => ({ events: events.splice(0), renders: renders.splice(0) }));
  const click = (text) => page.evaluate((t) => [...document.querySelectorAll("button, a")].find((b) => b.textContent === t).click(), text);
  const classes = (id) => page.evaluate((i) => document.getElementById(i)?.className ?? null, id);
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("a script's function, called from .wf, paints",
      await until(page, () => document.getElementById("money")?.textContent === "$12.00"),
      await page.evaluate(() => document.getElementById("money")?.textContent));

    let seen = await taken();
    check("mount hands the element over, and update runs with the handle",
      JSON.stringify(seen.events) === JSON.stringify(["mount 10", "update 10"]), seen.events.join(", "));
    check("wf:render is said once for the first drawing", JSON.stringify(seen.renders) === JSON.stringify(["/"]), seen.renders.join(", "));
    check("the element the script was handed is the one on the page",
      await page.evaluate(() => document.getElementById("tilt").dataset.tilt === "10"));

    await click("More");
    await until(page, () => events.length > 0);
    seen = await taken();
    check("update follows the state it reads", JSON.stringify(seen.events) === JSON.stringify(["update 15"]), seen.events.join(", "));
    const tilt = await classes("tilt");
    check("a class map follows state, and a script's class survives it",
      /\bis-on\b/.test(tilt) && /\bis-max\b/.test(tilt) && /\bfrom-script\b/.test(tilt), tilt);

    check("a script reports through an action, and the class map follows",
      await until(page, () => document.getElementById("seen")?.classList.contains("is-visible")),
      await classes("seen"));

    check("a custom element placed with Element upgrades and fires its own event",
      await until(page, () => document.getElementById("ready")?.textContent === "badge ready"),
      await page.evaluate(() => `${document.getElementById("badge")?.textContent} / ${document.getElementById("ready")?.textContent}`));
    await click("Relabel");
    check("its attribute follows state",
      await until(page, () => document.getElementById("badge")?.textContent === "badge: two"),
      await page.evaluate(() => document.getElementById("badge")?.textContent));

    await click("Toggle");
    await until(page, () => events.length > 0);
    seen = await taken();
    check("cleanup runs when an if hides the element", JSON.stringify(seen.events) === JSON.stringify(["cleanup"]), seen.events.join(", "));
    await click("Toggle");
    await until(page, () => events.length > 1);
    seen = await taken();
    check("and mount again when it comes back", JSON.stringify(seen.events) === JSON.stringify(["mount 15", "update 15"]), seen.events.join(", "));

    if (!ssg) {
      // A pre-rendered site loads a new document per route, which is a new
      // page for its scripts too; a single page keeps them, so its route
      // change is the one to hold to the lifetime.
      await click("Away");
      await until(page, () => document.querySelector("h1")?.textContent === "Away");
      await until(page, () => renders.length > 0);
      seen = await taken();
      check("a route change runs cleanup", seen.events.includes("cleanup"), seen.events.join(", "));
      check("and wf:render says the new route", JSON.stringify(seen.renders) === JSON.stringify(["/away"]), seen.renders.join(", "));
    }

    check("nothing threw, and the policy refused nothing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4381);
await scenario("A single page", false, 4382);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
