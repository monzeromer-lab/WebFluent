// A page that asks a service for things, in a real Chrome, against
// `tests/fixtures/fetch`: a server that records every request the page
// makes. Two readers of one endpoint make one request; typing into a field
// a request's body reads asks again; `invalidate()` redraws what is shown;
// a `POST` that timed out is sent once, where a `GET` is tried again; and
// with the network gone, a `POST` fails rather than being kept, where a
// `PUT` is kept and sent once the network is back.
//
//   cd tests/browser && npm ci && node fetch.mjs
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
const fixture = path.join(repo, "tests/fixtures/fetch");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const until = (page, fn, arg, timeout = 5000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);
const text = (page, id) => page.evaluate((id) => document.getElementById(id)?.textContent ?? null, id);

const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json" };

/// The build, and a service that records what it is asked.
function server(root, port) {
  const asked = [];
  let users = 0;
  // While down, a request for the service gets no answer: the connection
  // is dropped, as a server that is gone drops it.
  let down = false;
  const json = (res, value) => res.writeHead(200, { "Content-Type": "application/json" }).end(JSON.stringify(value));
  const srv = http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    if (url.pathname.startsWith("/api/")) {
      if (down) return req.socket.destroy();
      let body = "";
      req.on("data", (c) => { body += c; });
      req.on("end", () => {
        asked.push({ method: req.method, path: url.pathname, body });
        const route = `${req.method} ${url.pathname}`;
        if (route === "GET /api/users") {
          users += 1;
          return json(res, Array.from({ length: users }, (_, i) => ({ id: String(i), name: `u${i}` })));
        }
        if (route === "POST /api/search") return json(res, { query: JSON.parse(body || "{}").query });
        // Slower than the service's timeout: the request was taken, and
        // its answer never arrived in time.
        if (url.pathname === "/api/slow-read" || url.pathname === "/api/slow-write") {
          return setTimeout(() => { if (!res.writableEnded) json(res, {}); }, 1000);
        }
        return json(res, { ok: true });
      });
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
  return {
    asked,
    count: (method, p) => asked.filter((a) => a.method === method && a.path === p).length,
    setDown: (v) => { down = v; },
    start: () => new Promise((r) => srv.listen(port, "127.0.0.1", r)),
    stop: () => new Promise((r) => { srv.closeAllConnections?.(); srv.close(() => r()); }),
  };
}

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-fetch-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });

  const site = server(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const thrown = [];
  page.on("pageerror", (e) => thrown.push(e.message));
  try {
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "load" });

    // Two resources over one endpoint, made as the page draws.
    const both = await until(page, () =>
      document.getElementById("first")?.textContent === "first:1"
      && document.getElementById("second")?.textContent === "second:1");
    check("two readers of one endpoint are both answered", both,
      `${await text(page, "first")} ${await text(page, "second")}`);
    check("…by one request", site.count("GET", "/api/users") === 1, JSON.stringify(site.asked));

    // A body that reads state is read again when it changes.
    await page.type("input", "ab");
    check("typing into a field the body reads asks again",
      await until(page, () => document.getElementById("found")?.textContent === "found:ab"),
      await text(page, "found"));
    check("…with the body as it now is",
      site.asked.some((a) => a.path === "/api/search" && JSON.parse(a.body).query === "ab"),
      JSON.stringify(site.asked.filter((a) => a.path === "/api/search")));

    // An endpoint's invalidate() reaches what is shown.
    await page.click("#refresh");
    check("invalidate() redraws every resource over the endpoint",
      await until(page, () =>
        document.getElementById("first")?.textContent === "first:2"
        && document.getElementById("second")?.textContent === "second:2"),
      `${await text(page, "first")} ${await text(page, "second")}`);
    check("…asking once for both", site.count("GET", "/api/users") === 2, String(site.count("GET", "/api/users")));

    // A write that timed out may have been taken: it is not sent again.
    await page.click("#slow-write");
    check("a POST that timed out fails as a timeout",
      await until(page, () => document.getElementById("said")?.textContent === "said:write:timeout", undefined, 8000),
      await text(page, "said"));
    await sleep(2500);
    check("…and was sent once", site.count("POST", "/api/slow-write") === 1, String(site.count("POST", "/api/slow-write")));

    // A read is safe to ask again.
    await page.click("#slow-read");
    check("a GET that timed out fails as a timeout, after its retries",
      await until(page, () => document.getElementById("said")?.textContent === "said:read:timeout", undefined, 10000),
      await text(page, "said"));
    check("…having been asked three times", site.count("GET", "/api/slow-read") === 3, String(site.count("GET", "/api/slow-read")));

    // With the network gone, a POST is not kept for later; a PUT is. The
    // service is down too: the service worker's Background Sync is not
    // held by the page's offline mode, and would send the PUT at once.
    site.setDown(true);
    await page.setOfflineMode(true);
    await page.click("#order");
    check("a POST made offline fails as offline",
      await until(page, () => document.getElementById("said")?.textContent === "said:order:offline"),
      await text(page, "said"));
    check("…and is not kept", (await text(page, "queued")) === "queued:0", await text(page, "queued"));
    await page.click("#note");
    check("a PUT made offline is kept",
      await until(page, () => document.getElementById("queued")?.textContent === "queued:1"),
      await text(page, "queued"));
    site.setDown(false);
    await page.setOfflineMode(false);
    await page.evaluate(() => window.dispatchEvent(new Event("online")));
    check("…and sent when the network is back",
      await until(page, () => document.getElementById("queued")?.textContent === "queued:0", undefined, 8000),
      await text(page, "queued"));
    await sleep(500);
    check("the server took the PUT once and the POST never",
      site.count("PUT", "/api/notes/n1") === 1 && site.count("POST", "/api/orders") === 0,
      JSON.stringify(site.asked.filter((a) => a.method !== "GET")));

    check("nothing threw", thrown.length === 0, thrown.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("A single-page build", false, 4791);
await scenario("A pre-rendered build", true, 4792);

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
