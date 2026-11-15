// A read in each of its cases, in a real Chrome, against
// `tests/fixtures/reads`: `.loading` while the first answer is on its way,
// `.loaded(value, refreshing, stale)` through a refresh and a refresh that
// fails, `.loading` again when an argument changes, `.failed(e)` for a first
// load that fails, `placeholder: .previous` keeping the last value, and a
// read that asks again when the network comes back. Every request is held
// by the server until the test answers it, so each case can be seen.
//
//   cd tests/browser && npm ci && node reads.mjs
//
// WF (default target/release/wf) and CHROME (default /usr/bin/google-chrome)
// say where they are.

import puppeteer from "puppeteer-core";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const WF = process.env.WF || path.join(repo, "target/release/wf");
const CHROME = process.env.CHROME || "/usr/bin/google-chrome";
const fixture = path.join(repo, "tests/fixtures/reads");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 5000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);
const text = (page, id) => page.evaluate((id) => document.getElementById(id)?.textContent ?? null, id);
const shows = (page, id, want) =>
  until(page, ([id, want]) => document.getElementById(id)?.textContent === want, [id, want]);

const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json" };

/// The site, and an API that holds every request until `answer` is called.
function server(root, port) {
  const held = [];
  const asked = [];
  const srv = http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    if (url.pathname.startsWith("/api/")) {
      asked.push(url.pathname + url.search);
      held.push({ path: url.pathname + url.search, res });
      return;
    }
    let file = path.join(root, decodeURIComponent(url.pathname));
    if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");
    if (!fs.existsSync(file)) {
      if (path.extname(file) !== "") return res.writeHead(404).end("not found");
      file = path.join(root, "index.html");
    }
    res.writeHead(200, { "Content-Type": TYPES[path.extname(file)] || "application/octet-stream" });
    fs.createReadStream(file).pipe(res);
  });
  const waiting = async (prefix) => {
    for (let i = 0; i < 200; i++) {
      const at = held.findIndex((h) => h.path.startsWith(prefix));
      if (at >= 0) return held.splice(at, 1)[0];
      await new Promise((r) => setTimeout(r, 25));
    }
    throw new Error(`nothing asked for ${prefix}; asked: ${asked.join(", ")}`);
  };
  return {
    asked,
    /// Answer the oldest request held for `prefix`.
    async answer(prefix, status, body) {
      const h = await waiting(prefix);
      h.res.writeHead(status, { "Content-Type": "application/json", "Cache-Control": "no-store" });
      h.res.end(JSON.stringify(body));
    },
    start: () => new Promise((r) => srv.listen(port, "127.0.0.1", r)),
    stop: () => new Promise((r) => {
      for (const h of held) h.res.destroy();
      srv.closeAllConnections?.();
      srv.close(() => r());
    }),
  };
}

const items = (n) => Array.from({ length: n }, (_, i) => ({ id: String(i), name: `item ${i}` }));

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-reads-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });

  if (ssg) {
    const painted = fs.readFileSync(path.join(dir, "build", "index.html"), "utf8");
    check("the painted page shows the `.loading` arm", /id="items"[^>]*>loading</.test(painted));
  }

  const site = server(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const thrown = [];
  page.on("pageerror", (e) => thrown.push(e.message));
  const press = (label) => page.evaluate((label) =>
    [...document.querySelectorAll("button")].find((b) => b.textContent === label).click(), label);
  try {
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "load" });

    // A first load: nothing to show yet.
    check("a first load is `.loading`", await shows(page, "items", "loading"), await text(page, "items"));
    check("and its members say so", (await text(page, "is-loaded")) === "no" && (await text(page, "value")) === "-1");
    check("a match expression reads the case", await shows(page, "expr", "L"), await text(page, "expr"));

    await site.answer("/api/items?q=a", 200, items(2));
    check("the answer is `.loaded(value)`", await shows(page, "items", "loaded: 2"), await text(page, "items"));
    check("not refreshing, nothing stale",
      (await text(page, "refreshing")) === "settled" && (await text(page, "stale")) === "fresh");
    check("`isLoaded` and `value` follow it",
      await shows(page, "is-loaded", "yes") && await shows(page, "value", "2"));
    check("the match expression too", await shows(page, "expr", "D2"), await text(page, "expr"));

    // A refresh: the value stays, refreshing.
    await press("Reload");
    check("a refresh stays `.loaded`, refreshing", await shows(page, "refreshing", "refreshing"),
      await text(page, "refreshing"));
    check("with the value it had", (await text(page, "items")) === "loaded: 2", await text(page, "items"));

    // A refresh that fails: still loaded, the failure stale.
    await site.answer("/api/items?q=a", 500, { message: "down" });
    check("a refresh that fails keeps the value, its failure `stale`",
      await shows(page, "stale", "stale: HTTP 500"), await text(page, "stale"));
    check("and is loaded, not refreshing",
      (await text(page, "items")) === "loaded: 2" && (await text(page, "refreshing")) === "settled");

    await press("Reload");
    await site.answer("/api/items?q=a", 200, items(3));
    check("the next answer clears what was stale",
      await shows(page, "items", "loaded: 3") && await shows(page, "stale", "fresh"), await text(page, "stale"));

    await site.answer("/api/kept?q=a", 200, items(1));
    check("a read with `placeholder: .previous` loads as any other", await shows(page, "kept", "1 false"),
      await text(page, "kept"));

    // An argument changes: another request, a first load again.
    await press("Next");
    check("an argument that changes is a first load again, `.loading`", await shows(page, "items", "loading"),
      await text(page, "items"));
    check("`placeholder: .previous` keeps the last value shown, refreshing", await shows(page, "kept", "1 true"),
      await text(page, "kept"));

    // A first load that fails.
    await site.answer("/api/items?q=b", 500, { message: "down" });
    check("a first load that fails is `.failed(e)`", await shows(page, "items", "failed: HTTP 500"),
      await text(page, "items"));
    check("`if let .failed(e)` takes it", await shows(page, "if-let", "HTTP 500"), await text(page, "if-let"));
    check("and the match expression", await shows(page, "expr", "F"), await text(page, "expr"));
    check("`isLoaded` is false, `value` null",
      await shows(page, "is-loaded", "no") && await shows(page, "value", "-1"));

    await site.answer("/api/kept?q=b", 200, items(4));
    check("the kept read shows the new value once it arrives", await shows(page, "kept", "4 false"),
      await text(page, "kept"));

    // A read that says nothing asks again when the network comes back.
    await site.answer("/api/ping", 200, items(1));
    await shows(page, "ping", "pinged");
    const before = site.asked.filter((a) => a.startsWith("/api/ping")).length;
    await page.setOfflineMode(true);
    await page.setOfflineMode(false);
    await page.evaluate(() => window.dispatchEvent(new Event("online")));
    await site.answer("/api/ping", 200, items(1));
    const after = site.asked.filter((a) => a.startsWith("/api/ping")).length;
    check("a read asks again when the network comes back (`.onReconnect`, unset)", after > before,
      `${before} → ${after}`);
    check("a read with `refresh: .never` does not",
      site.asked.filter((a) => a.startsWith("/api/items")).length === 4, site.asked.join(", "));

    check("nothing threw", thrown.length === 0, thrown.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("A single-page build", false, 4921);
await scenario("A pre-rendered build", true, 4922);

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
