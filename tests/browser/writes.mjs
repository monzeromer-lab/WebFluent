// Writes and what they make stale, in a real Chrome, against
// `tests/fixtures/writes`: a create whose contract says `invalidates
// [list]` refreshes the list on the page; a rename that `invalidates
// [list, byId(id)]` refreshes the item and the list; and a change shown
// with `update` before the server agrees is undone when it answers 500.
//
//   cd tests/browser && npm ci && node writes.mjs
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
const fixture = path.join(repo, "tests/fixtures/writes");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}
const until = (page, fn, arg, timeout = 6000) =>
  page.waitForFunction(fn, { timeout, polling: 20 }, arg).then(() => true, () => false);
const text = (page, id) => page.evaluate((id) => document.getElementById(id)?.textContent ?? null, id);
const shows = (page, id, want) =>
  until(page, ([id, want]) => document.getElementById(id)?.textContent === want, [id, want]);

const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json" };

/// The site, and a todo API kept in memory that counts its reads.
function server(root, port) {
  const todos = [{ id: "1", title: "Milk" }];
  const reads = { list: 0, item: 0 };
  const srv = http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    const json = (status, body, wait = 0) => setTimeout(() => {
      res.writeHead(status, { "Content-Type": "application/json", "Cache-Control": "no-store" });
      res.end(JSON.stringify(body));
    }, wait);
    const body = () => new Promise((done) => {
      let raw = "";
      req.on("data", (c) => { raw += c; });
      req.on("end", () => done(raw ? JSON.parse(raw) : null));
    });
    if (url.pathname === "/api/todos" && req.method === "GET") {
      reads.list += 1;
      return json(200, todos);
    }
    if (url.pathname === "/api/todos" && req.method === "POST") {
      return body().then((b) => {
        const made = { id: String(todos.length + 1), title: b.title };
        todos.push(made);
        json(201, made);
      });
    }
    if (url.pathname === "/api/todos/refuse") {
      return body().then(() => json(500, { title: "no" }, 300));
    }
    const one = url.pathname.match(/^\/api\/todos\/(\w+)$/);
    if (one && req.method === "GET") {
      reads.item += 1;
      const found = todos.find((t) => t.id === one[1]);
      return found ? json(200, found) : json(404, {});
    }
    if (one && req.method === "PATCH") {
      return body().then((b) => {
        const found = todos.find((t) => t.id === one[1]);
        found.title = b.title;
        json(200, found);
      });
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
    reads,
    start: () => new Promise((r) => srv.listen(port, "127.0.0.1", r)),
    stop: () => new Promise((r) => { srv.closeAllConnections?.(); srv.close(() => r()); }),
  };
}

async function scenario(label, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-writes-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
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
    check("the list and the item load", await shows(page, "list", "Milk") && await shows(page, "detail", "Milk"),
      `${await text(page, "list")} / ${await text(page, "detail")}`);

    await press("Create");
    check("a create the contract says invalidates the list refreshes it on the page",
      await shows(page, "list", "Milk,Bread"), await text(page, "list"));
    check("and the item, which it does not name, is not asked again", site.reads.item === 1, `${site.reads.item} reads`);

    await press("Rename");
    check("a rename refreshes the item", await shows(page, "detail", "Oat milk"), await text(page, "detail"));
    check("and the list", await shows(page, "list", "Oat milk,Bread"), await text(page, "list"));
    check("a positional argument is the first parameter: `rename(\"1\", …)` renamed todo 1",
      (await text(page, "said")) === "renamed Oat milk", await text(page, "said"));

    await press("Hopeful");
    check("`update` shows the change before the server answers",
      await shows(page, "list", "Oat milk,Bread,Ghost"), await text(page, "list"));
    check("and the undo takes it back when the server answers 500",
      await shows(page, "list", "Oat milk,Bread"), await text(page, "list"));
    check("the action heard why", (await text(page, "said")).startsWith("undone: HTTP 500"), await text(page, "said"));
    check("nothing threw", thrown.length === 0, thrown.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("A production build", 4951);

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
