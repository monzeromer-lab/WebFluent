// What a project script returns, held to its `@returns`, in a real Chrome,
// against `tests/fixtures/returns`: a function that keeps its word gives
// its value; one that does not throws a `DecodeError` naming the function
// and the place; a promise that settles to the wrong thing is refused when
// it settles — under `wf serve` and in a production build alike.
//
//   cd tests/browser && npm ci && node returns.mjs
//
// WF (default target/release/wf) and CHROME (default /usr/bin/google-chrome)
// say where they are.

import puppeteer from "puppeteer-core";
import { execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const WF = process.env.WF || path.join(repo, "target/release/wf");
const CHROME = process.env.CHROME || "/usr/bin/google-chrome";
const fixture = path.join(repo, "tests/fixtures/returns");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 5000) =>
  page.waitForFunction(fn, { timeout, polling: 30 }, arg).then(() => true, () => false);
const text = (page, id) => page.evaluate((id) => document.getElementById(id)?.textContent ?? null, id);
const startsWith = (page, id, want) =>
  until(page, ([id, want]) => (document.getElementById(id)?.textContent ?? "").startsWith(want), [id, want]);

const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json" };

function staticHost(root, port) {
  const srv = http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
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
    start: () => new Promise((r) => srv.listen(port, "127.0.0.1", r)),
    stop: () => new Promise((r) => { srv.closeAllConnections?.(); srv.close(() => r()); }),
  };
}

/// `wf serve` on the copy, until it answers.
async function devServer(dir, port) {
  const child = spawn(WF, ["serve", "-d", dir], { stdio: "ignore" });
  for (let i = 0; i < 200; i++) {
    const up = await new Promise((r) => {
      http.get(`http://127.0.0.1:${port}/`, (res) => { res.resume(); r(res.statusCode === 200); }).on("error", () => r(false));
    });
    if (up) return { stop: async () => { child.kill(); } };
    await new Promise((r) => setTimeout(r, 100));
  }
  child.kill();
  throw new Error("wf serve did not answer");
}

async function scenario(label, dev, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-returns-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.dev.port = port;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  let site;
  if (dev) {
    site = await devServer(dir, port);
  } else {
    execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
    site = staticHost(path.join(dir, "build"), port);
    await site.start();
  }
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const thrown = [];
  page.on("pageerror", (e) => thrown.push(e.message));
  const press = (label) => page.evaluate((label) =>
    [...document.querySelectorAll("button")].find((b) => b.textContent === label).click(), label);
  try {
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "load" });
    await until(page, () => document.querySelectorAll("button").length === 3);
    await press("Good");
    check("a function that keeps its word gives its value", await startsWith(page, "said", "good: 1"),
      await text(page, "said"));
    await press("Bad");
    check("one that does not throws a DecodeError naming it and the place",
      await startsWith(page, "said", "DecodeError: `badRows` returned what its @returns does not say: $[0].title is missing"),
      await text(page, "said"));
    await press("Later");
    check("a promise that settles to the wrong thing is refused when it settles",
      await startsWith(page, "said", "DecodeError: `laterCount` returned what its @returns does not say: $"),
      await text(page, "said"));
    check("nothing escaped the catch", thrown.length === 0, thrown.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("A production build", false, 4931);
await scenario("Under wf serve", true, 4932);

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
