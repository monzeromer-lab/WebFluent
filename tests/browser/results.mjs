// A service's failures as values, in a real Chrome, against
// `tests/fixtures/results`: a `422` handled three ways — every case matched
// where the call is made, passed up with `try` as the action's own error,
// and read from the action's `.result` — a `409` reaching its own case, a
// success as `.ok`, and a `503` passed up into an error enum's `http` case.
//
//   cd tests/browser && npm ci && node results.mjs
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
const fixture = path.join(repo, "tests/fixtures/results");

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
  const json = (res, status, value, type = "application/json") =>
    res.writeHead(status, { "Content-Type": type }).end(JSON.stringify(value));
  const srv = http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    if (url.pathname.startsWith("/api/")) {
      let body = "";
      req.on("data", (c) => { body += c; });
      req.on("end", () => {
        if (url.pathname === "/api/users" && req.method === "POST") {
          const { name } = JSON.parse(body || "{}");
          if (name === "taken") {
            return json(res, 422, { title: "Invalid", status: 422, errors: [{ field: "name", message: "is taken" }] },
              "application/problem+json");
          }
          if (name === "dup") return json(res, 409, { title: "Already there" }, "application/problem+json");
          return json(res, 201, { id: "1", name });
        }
        if (url.pathname === "/api/broken") return json(res, 503, { title: "Down" }, "application/problem+json");
        return json(res, 404, {});
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
    start: () => new Promise((r) => srv.listen(port, "127.0.0.1", r)),
    stop: () => new Promise((r) => { srv.closeAllConnections?.(); srv.close(() => r()); }),
  };
}

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-results-site-"));
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
  const press = (label) => page.evaluate((label) =>
    [...document.querySelectorAll("button")].find((b) => b.textContent === label).click(), label);
  try {
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "load" });

    await press("Submit taken");
    check("a 422 reaches its own case, with its problem", await shows(page, "said", "invalid: Invalid"),
      await text(page, "said"));
    await press("Submit dup");
    check("a 409 reaches its own case", await shows(page, "said", "already there"), await text(page, "said"));
    await press("Submit ada");
    check("a success is .ok, with the answer", await shows(page, "said", "created ada"), await text(page, "said"));

    await press("Invite taken");
    check("try passes a 422 up as the action's own, read from .result",
      await shows(page, "invite", "refused: 1 problem"), await text(page, "invite"));
    await press("Invite bo");
    check("…and a success returns the value as .ok", await shows(page, "invite", "invited bo"),
      await text(page, "invite"));

    await press("Probe");
    check("a 503 from a call declared `-> User!` passes up into the enum's http case",
      await shows(page, "probe", "503 passed up"), await text(page, "probe"));
    check("nothing threw", thrown.length === 0, thrown.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("A single-page build", false, 4795);
await scenario("A pre-rendered build", true, 4796);

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
