// `offline` in a real Chrome, against `tests/fixtures/offline`, built twice —
// pre-rendered and as a single page — each driven through what a reader
// meets: the worker stores the site; a stored route loads with the server
// gone and one that is not shows the fallback; a write made then is counted,
// its call says it was kept, and it is sent once when the server returns,
// asking again for what it invalidates — one the server refuses is listed in
// `network.refused` and never sent again; a new build is offered and taken
// with one reload, and the old version's store goes.
//
//   cd tests/browser && npm ci && node offline.mjs
//
// WF (default target/release/wf) and CHROME (default /usr/bin/google-chrome)
// say where they are. Service workers need a real browser: none of this can
// be run against the fake DOM the other suites use.

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
const fixture = path.join(repo, "tests/fixtures/offline");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const until = (page, fn, arg, timeout = 15000) =>
  page.waitForFunction(fn, { timeout, polling: 200 }, arg).then(() => true, () => false);

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-offline-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const build = path.join(dir, "build");
  const worker = fs.readFileSync(path.join(build, "sw.js"), "utf8");
  const stores = JSON.parse(/const C = (\{.*?\});\n/.exec(worker)[1]).precache.slice().sort();

  const site = host(build, port);
  await site.start();
  const origin = `http://localhost:${port}`;
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "wf-offline-chrome-"));
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, userDataDir: profile, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  let loads = 0;
  page.on("load", () => { loads += 1; });
  try {
    await page.goto(`${origin}/`, { waitUntil: "load" });
    check("the worker controls the page after the first visit",
      await until(page, () => navigator.serviceWorker.controller !== null));
    const stored = await page.evaluate(async () => {
      const store = (await caches.keys()).find((k) => /^wf-[0-9a-f]{16}$/.test(k));
      return store ? (await (await caches.open(store)).keys()).map((r) => new URL(r.url).pathname).sort() : [];
    });
    check("it stored everything the worker names", JSON.stringify(stored) === JSON.stringify(stores), stored.join(" "));

    await site.stop();
    await page.goto(`${origin}/about/`, { waitUntil: "load" });
    check("a stored route loads with the server gone",
      await until(page, () => document.querySelector("h1")?.textContent === "About Us"),
      await page.evaluate(() => document.querySelector("h1")?.textContent));
    await page.goto(`${origin}/contact/`, { waitUntil: "load" });
    check("a route not stored shows the fallback",
      await until(page, () => document.querySelector("h1")?.textContent === "You are offline"),
      await page.evaluate(() => `${location.pathname} ${document.querySelector("h1")?.textContent}`));

    await page.goto(`${origin}/`, { waitUntil: "load" });
    await page.waitForSelector("#save");
    await page.click("#save");
    check("a write made offline is kept, and network.queued counts it",
      await until(page, () => document.getElementById("queued")?.textContent === "queued:1", undefined, 5000));
    check("its call says it was kept, not answered: .offline(queued: true)",
      await until(page, () => document.getElementById("saved")?.textContent === "saved:kept", undefined, 5000),
      await page.evaluate(() => document.getElementById("saved")?.textContent));
    await page.click("#refuse");
    check("a second write is kept behind it",
      await until(page, () => document.getElementById("queued")?.textContent === "queued:2", undefined, 5000)
        && await until(page, () => document.getElementById("refuse-said")?.textContent === "refuse:kept", undefined, 5000));

    const notesBefore = site.fetched.filter((u) => u === "/api/notes").length;
    await site.start();
    await page.evaluate(() => window.dispatchEvent(new Event("online")));
    check("they are sent when the server returns",
      await until(page, () => document.getElementById("queued")?.textContent === "queued:0"));
    await sleep(1500);
    check("the server received each once, in order, as written",
      site.received.length === 2 && site.received[0].includes("written while offline") && site.received[1].includes("will refuse"),
      JSON.stringify(site.received));
    check("what the sent write invalidates is asked for again",
      site.fetched.filter((u) => u === "/api/notes").length === notesBefore + 1,
      JSON.stringify(site.fetched));
    check("the write the server refused is listed in network.refused",
      await until(page, () => document.getElementById("refused")?.textContent === "refused:409 /api/refuse"),
      await page.evaluate(() => document.getElementById("refused")?.textContent));
    await page.click("#dismiss");
    check("and network.clearRefused() forgets it",
      await until(page, () => document.getElementById("refused")?.textContent === "refused:"),
      await page.evaluate(() => document.getElementById("refused")?.textContent));
    site.received.length = 0;
    await page.evaluate(() => window.dispatchEvent(new Event("online")));
    await sleep(1000);
    check("a refused write is never sent again", site.received.length === 0, JSON.stringify(site.received));

    const home = path.join(dir, "src/App.wf");
    fs.writeFileSync(home, fs.readFileSync(home, "utf8").replace('Heading("Offline test")', 'Heading("Offline test, v2")'));
    execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
    await page.goto(`${origin}/`, { waitUntil: "load" });
    await page.evaluate(async () => (await navigator.serviceWorker.getRegistration()).update());
    check("a new build is offered: update.available", await until(page, () => !!document.getElementById("update")));
    const before = loads;
    await page.click("#update");
    check("update.apply() takes it, and the page shows it",
      await until(page, () => document.querySelector("h1")?.textContent === "Offline test, v2"));
    await sleep(2500);
    check("the page reloaded once, not twice", loads - before === 1, `${loads - before} loads`);
    const versions = await page.evaluate(async () => (await caches.keys()).filter((k) => /^wf-[0-9a-f]{16}$/.test(k)));
    check("the old version's store is gone", versions.length === 1, versions.join(", "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(profile, { recursive: true, force: true });
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4371);
await scenario("A single page", false, 4372);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
