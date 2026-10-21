// Motion in a real Chrome, against `tests/fixtures/motion`, pre-rendered and
// as a single page: an element plays its animation as it stands, one
// repeats, the project's own plays, and a branch enters and leaves through
// the Web Animations API — and with `data-wf-motion="reduce"` on the
// page's root, as an editor previews reduced motion, every one of them is
// stilled exactly as for a reader whose system asks for less: the
// stylesheet's animations end at once and stop repeating, a branch arrives
// and leaves with nothing played; taking the attribute off plays them
// again.
//
//   cd tests/browser && npm ci && node motion.mjs
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
const fixture = path.join(repo, "tests/fixtures/motion");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

const styles = (page) =>
  page.evaluate(() =>
    Object.fromEntries(["still", "pulse", "drop"].map((id) => {
      const s = getComputedStyle(document.getElementById(id));
      return [id, { name: s.animationName, duration: s.animationDuration, count: s.animationIterationCount }];
    })));

// What the runtime starts on the branch: Web Animations, not the
// stylesheet's (a CSSAnimation is the stylesheet's).
const played = (page) =>
  page.evaluate(() => document.getAnimations().filter((a) => !(a instanceof CSSAnimation)).length);

async function toggle(page) {
  await page.click("#toggle");
  await new Promise((r) => setTimeout(r, 40));
}

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-motion-site-"));
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
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    let s = await styles(page);
    check("an element plays its animation as it stands", s.still.name === "wf-fadeIn" && s.still.duration === "0.3s", JSON.stringify(s.still));
    check("…one repeats", s.pulse.name === "wf-pulse" && s.pulse.count === "infinite", JSON.stringify(s.pulse));
    check("…and the project's own plays", s.drop.name === "Drop", JSON.stringify(s.drop));
    await toggle(page);
    check("a branch enters through the Web Animations API", (await page.$("#branch")) && (await played(page)) > 0);
    await new Promise((r) => setTimeout(r, 400));
    await toggle(page);
    check("…and plays its exit before it leaves", (await page.$("#branch")) !== null && (await played(page)) > 0);
    await new Promise((r) => setTimeout(r, 500));
    check("…then it is gone", (await page.$("#branch")) === null);

    // The editor's preview asks for less motion.
    await page.evaluate(() => document.documentElement.setAttribute("data-wf-motion", "reduce"));
    s = await styles(page);
    const still = (x) => parseFloat(x.duration) < 0.001 && x.count === "1";
    check("with the attribute, the stylesheet's animations end at once", still(s.still) && still(s.drop), JSON.stringify(s));
    check("…and the one that repeats stops repeating", still(s.pulse), JSON.stringify(s.pulse));
    await toggle(page);
    check("…a branch arrives with nothing played", (await page.$("#branch")) !== null && (await played(page)) === 0);
    await toggle(page);
    check("…and leaves at once", (await page.$("#branch")) === null && (await played(page)) === 0);

    // As a reader whose system asks for it sees it: the same.
    await page.evaluate(() => document.documentElement.removeAttribute("data-wf-motion"));
    await page.emulateMediaFeatures([{ name: "prefers-reduced-motion", value: "reduce" }]);
    const reader = await styles(page);
    check("the reader's setting stills the same animations the same way", JSON.stringify(reader) === JSON.stringify(s), `${JSON.stringify(reader)} vs ${JSON.stringify(s)}`);
    await toggle(page);
    const readerPlayed = await played(page);
    await toggle(page);
    check("…and plays nothing on the branch either", readerPlayed === 0 && (await page.$("#branch")) === null);

    // Off again: everything plays.
    await page.emulateMediaFeatures([{ name: "prefers-reduced-motion", value: "no-preference" }]);
    s = await styles(page);
    check("without either, the animations are back", s.still.duration === "0.3s" && s.pulse.count === "infinite", JSON.stringify(s));
    await toggle(page);
    check("…and the branch plays again", (await played(page)) > 0);
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4397);
await scenario("A single page", false, 4398);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
