// A service's middleware against a real server, in a real Chrome, against
// `tests/fixtures/transport`: the steps run in the order written — each
// sees what the one before it added — and three requests that hear `401`
// at once share one refresh, then are each sent again with the new token.
//
//   cd tests/browser && npm ci && node transport.mjs
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
const fixture = path.join(repo, "tests/fixtures/transport");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 8000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);
const text = (page, id) => page.evaluate((id) => document.getElementById(id)?.textContent ?? null, id);

const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json" };

/// The site, an API that answers `401` to any token but the new one, and a
/// refresh that takes a moment — long enough for all three `401`s to land
/// while it runs.
function server(root, port) {
  const seen = { me: [], refreshes: 0 };
  const srv = http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    const json = (status, body) => {
      res.writeHead(status, { "Content-Type": "application/json", "Cache-Control": "no-store" });
      res.end(JSON.stringify(body));
    };
    if (url.pathname === "/auth/refresh") {
      seen.refreshes += 1;
      setTimeout(() => json(200, { token: "new" }), 150);
      return;
    }
    if (url.pathname === "/api/me") {
      seen.me.push({ n: url.searchParams.get("n"), headers: req.headers });
      if (req.headers.authorization !== "Bearer new") return json(401, { title: "expired" });
      return json(200, { name: `Ada ${url.searchParams.get("n")}` });
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
  return {
    seen,
    start: () => new Promise((r) => srv.listen(port, "127.0.0.1", r)),
    stop: () => new Promise((r) => { srv.closeAllConnections?.(); srv.close(() => r()); }),
  };
}

async function scenario(label, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-transport-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const site = server(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const thrown = [];
  page.on("pageerror", (e) => thrown.push(e.message));
  try {
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "load" });
    await page.evaluate(() => [...document.querySelectorAll("button")].find((b) => b.textContent === "Three").click());
    check("each request is answered once the credentials are fresh",
      await until(page, () => document.getElementById("said")?.textContent === "Ada 1,Ada 2,Ada 3"),
      await text(page, "said"));
    check("three 401s at once share one refresh", site.seen.refreshes === 1, `${site.seen.refreshes} refreshes`);
    check("the page's store saw one refresh", (await text(page, "refreshes")) === "1", await text(page, "refreshes"));
    const first = site.seen.me.slice(0, 3);
    const again = site.seen.me.slice(3);
    check("the first three went with the old token", first.length === 3 && first.every((m) => m.headers.authorization === "Bearer old"),
      JSON.stringify(first.map((m) => m.headers.authorization)));
    check("each was sent again, through bearer, with the new one",
      again.length === 3 && again.every((m) => m.headers.authorization === "Bearer new"),
      JSON.stringify(again.map((m) => m.headers.authorization)));
    check("the steps run in the order written: each sees the one before it",
      site.seen.me.every((m) => m.headers["x-first"] === "first" && m.headers["x-second"] === "after first"),
      JSON.stringify(site.seen.me.map((m) => [m.headers["x-first"], m.headers["x-second"]])));
    check("nothing threw", thrown.length === 0, thrown.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("A production build", 4941);

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
