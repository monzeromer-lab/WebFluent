// The language's own types at run time, in a real Chrome, against
// `tests/fixtures/types`: `Settings.decode` takes a value of the type and
// names where one that is not goes wrong, as the reader types; a `Json`
// value is matched by what it is; a dictionary is read and changed by key;
// an action's declared result reaches the page; and a function type passed
// as a prop is called.
//
//   cd tests/browser && npm ci && node types.mjs
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
const fixture = path.join(repo, "tests/fixtures/types");

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

function server(root, port) {
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

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-types-site-"));
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

    check("a value of the type decodes", await shows(page, "decoded", "theme dark, 3 columns"),
      await text(page, "decoded"));
    // The reader makes the columns a word: the place, and what is wrong.
    // The text arrives whole, as a paste does: half of it is no JSON at all.
    const paste = (value) => page.evaluate((value) => {
      const input = document.querySelector("input");
      input.value = value;
      input.dispatchEvent(new Event("input", { bubbles: true }));
    }, value);
    await paste('{"theme": "dark", "columns": "three", "tags": []}');
    check("one that is not names where, and what",
      await shows(page, "decoded", '$.columns is "three", not a Number'), await text(page, "decoded"));
    await paste('{"theme": "dark", "columns": 2, "tags": [1]}');
    check("…however deep", await shows(page, "decoded", "$.tags[0] is 1, not a String"),
      await text(page, "decoded"));

    check("a declared result, and a dictionary's values and keys",
      await shows(page, "scores", "8 in ada,bo"), await text(page, "scores"));
    check("a key is looked up, a dictionary keyed by an enum too",
      await shows(page, "lookup", "true 10"), await text(page, "lookup"));
    await page.evaluate(() => [...document.querySelectorAll("button")].find((b) => b.textContent === "Add cy").click());
    check("with() adds a key", await shows(page, "scores", "15 in ada,bo,cy"), await text(page, "scores"));
    await page.evaluate(() => [...document.querySelectorAll("button")].find((b) => b.textContent === "Drop ada").click());
    check("without() takes one away", await shows(page, "scores", "12 in bo,cy"), await text(page, "scores"));
    check("…and has() follows it", await shows(page, "lookup", "false 10"), await text(page, "lookup"));

    check("Json.null is matched as nothing", await shows(page, "kind", "nothing"), await text(page, "kind"));
    await page.evaluate(() => [...document.querySelectorAll("button")].find((b) => b.textContent === "A list").click());
    check("a JSON list is matched as one", await shows(page, "kind", "a list of 3"), await text(page, "kind"));
    await page.evaluate(() => [...document.querySelectorAll("button")].find((b) => b.textContent === "A word").click());
    check("a JSON string is matched as one", await shows(page, "kind", "the text hello"), await text(page, "kind"));

    check("a function passed as a prop is called", await shows(page, "label", "region south"),
      await text(page, "label"));
    check("nothing threw", thrown.length === 0, thrown.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("A single-page build", false, 4793);
await scenario("A pre-rendered build", true, 4794);

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
