// A resource a store holds, in a real Chrome, against
// `tests/fixtures/store_resource`, built twice — pre-rendered and as a
// single page. It is asked for when the store is first read; a page reads
// its state, its data through `match`, and a derived value over it; an
// action moves the address it reads and it asks again; another action
// reloads it, keeping what it has; it outlives the page that first read it,
// and is not asked for again on the next; a `.route` store's is asked for
// again when its route comes back; a failure is its error.
//
//   cd tests/browser && npm ci && node store_resource.mjs
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
const fixture = path.join(repo, "tests/fixtures/store_resource");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 3000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);
/// Whether `fn()` comes true, here rather than in the page, within `timeout`.
async function eventually(fn, timeout = 3000) {
  for (const end = Date.now() + timeout; Date.now() < end; await new Promise((r) => setTimeout(r, 30))) {
    if (fn()) return true;
  }
  return fn();
}

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-store-resource-site-"));
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
  // Every request for data, in order.
  const asked = [];
  page.on("request", (r) => { if (r.url().includes("/data/")) asked.push(new URL(r.url()).pathname); });
  page.on("pageerror", (e) => problems.push(e.message));
  page.on("console", (m) => { if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text()); });
  // The page that does not exist is the failure the scenario asks for.
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$|rows-3\.json$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  const text = (id) => page.evaluate((i) => document.getElementById(i)?.textContent ?? null, id);
  const is = (id, want) => until(page, (a) => document.getElementById(a[0])?.textContent === a[1], [id, want]);
  const count = (file) => asked.filter((p) => p === file).length;
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("the store asks for it when the page reads it, and it arrives", await is("state", "ready"), await text("state"));
    check("a match over it shows what it brought", await is("rows", "AdaGrace"), await text("rows"));
    check("a derived value over its data follows it", await is("count", "2") && await is("loaded", "true"), `${await text("count")} ${await text("loaded")}`);
    check("it was asked for once", count("/data/rows-1.json") === 1, asked.join(" "));
    check("a `.route` store's resource arrives too", await is("title", "A draft"), await text("title"));

    await page.click("#next");
    check("an action moves the address it reads, and it asks again", await is("rows", "EdsgerBarbaraDonald"), await text("rows"));
    check("and the derived value follows", await is("count", "3"), await text("count"));
    check("the new address was asked for", count("/data/rows-2.json") === 1, asked.join(" "));

    const before = count("/data/rows-2.json");
    await page.click("#again");
    check("an action reloads it", await eventually(() => count("/data/rows-2.json") > before), asked.join(" "));
    check("keeping what it has while it asks", await is("state", "ready") && await is("count", "3"), `${await text("state")} ${await text("count")}`);

    const rowsBefore = asked.filter((p) => p.startsWith("/data/rows-")).length;
    const notesBefore = count("/data/note.json");
    await page.click("#other");
    check("the next page is drawn", await until(page, () => !!document.getElementById("home")), await page.url());
    if (ssg) {
      // A pre-rendered site's link loads another document: the store is
      // built again there, and asks again.
      check("another document builds the store again, and asks for it again", await is("count", "2") && await is("state", "ready"), `${await text("count")} ${await text("state")}`);
    } else {
      check("another page reads the same store", await is("count", "3") && await is("state", "ready"), `${await text("count")} ${await text("state")}`);
      check("and it is not asked for again: it outlived the page that first read it", asked.filter((p) => p.startsWith("/data/rows-")).length === rowsBefore, asked.join(" "));
    }

    await page.click("#home");
    check("home is drawn again", await until(page, () => !!document.getElementById("other")), await page.url());
    check("back home, the `.route` store's resource is asked for again", await eventually(() => count("/data/note.json") === notesBefore + 1) && await is("title", "A draft"), `${count("/data/note.json")} after ${notesBefore}`);
    if (!ssg) {
      check("and the app store's still holds what it had", await is("rows", "EdsgerBarbaraDonald") && await is("count", "3"), await text("rows"));
    } else {
      check("the store starts again on the new document", await is("rows", "AdaGrace"), await text("rows"));
      await page.click("#next");
      check("and moves on as before", await is("count", "3"), await text("count"));
    }

    await page.click("#next");
    check("an address with nothing there is the resource's error", await is("state", "error") && await is("rows", "failed"), `${await text("state")} ${await text("rows")}`);
    check("and a derived value over its state follows it", await is("loaded", "false"), await text("loaded"));
    check("and one over its data: the last page's answer is not this one's", await is("count", "0"), await text("count"));

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
