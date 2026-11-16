// Reads that answer more than once, in a real Chrome, against
// `tests/fixtures/paging`: an endpoint that pages by number, by cursor and
// by its `Link` header gathers its pages on the page, in order, and stops
// where the server says; a refresh asks again for every page gathered; a
// `streamed` endpoint's read is loaded from its first line and grows as
// the rest arrive, and `for await` walks the same lines in an action.
//
//   cd tests/browser && npm ci && node paging.mjs
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
const fixture = path.join(repo, "tests/fixtures/paging");

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
const post = (id) => ({ id, title: id });

/// The site, and an API that pages three ways and streams a log, recording
/// every request it answers.
function server(root, port) {
  const asked = [];
  const srv = http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    const json = (body, headers = {}) => {
      res.writeHead(200, { "Content-Type": "application/json", "Cache-Control": "no-store", ...headers });
      res.end(JSON.stringify(body));
    };
    if (url.pathname.startsWith("/api/")) asked.push(url.pathname + url.search);
    if (url.pathname === "/api/posts") {
      const pages = { 1: ["a", "b"], 2: ["c"], 3: [] };
      return json((pages[url.searchParams.get("page") || "1"] || []).map(post));
    }
    if (url.pathname === "/api/feed") {
      const feed = {
        "": { posts: ["f1", "f2"].map(post), nextCursor: "k2" },
        k2: { posts: ["f3"].map(post), nextCursor: "k3" },
        k3: { posts: ["f4"].map(post), nextCursor: null },
      };
      return json(feed[url.searchParams.get("after") || ""]);
    }
    if (url.pathname === "/api/search") {
      // The next page's address is the server's to write: relative here.
      if (url.searchParams.get("cursor") === "two") return json(["s3"].map(post));
      return json(["s1", "s2"].map(post), {
        Link: '</api/search?q=x&cursor=one>; rel="self", </api/search?q=x&cursor=two>; rel="next"',
      });
    }
    if (url.pathname === "/api/logs/7") {
      res.writeHead(200, { "Content-Type": "application/x-ndjson", "Cache-Control": "no-store" });
      const lines = [{ n: 1, text: "one" }, { n: 2, text: "two" }, { n: 3, text: "three" }];
      let i = 0;
      const next = () => {
        if (i === lines.length) return res.end();
        res.write(JSON.stringify(lines[i++]) + "\n");
        setTimeout(next, 400);
      };
      return next();
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
    start: () => new Promise((r) => srv.listen(port, "127.0.0.1", r)),
    stop: () => new Promise((r) => { srv.closeAllConnections?.(); srv.close(() => r()); }),
  };
}

async function scenario(label, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-paging-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const site = server(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const thrown = [];
  page.on("pageerror", (e) => thrown.push(e.message));
  // A file the site does not have is the browser's to ask for (a favicon);
  // anything of the API's that failed is the page's.
  page.on("response", (r) => { if (r.status() >= 400 && r.url().includes("/api/")) thrown.push(`${r.status()} ${r.url()}`); });
  const press = (label) => page.evaluate((label) =>
    [...document.querySelectorAll("button")].find((b) => b.textContent === label).click(), label);
  const asked = (prefix) => site.asked.filter((a) => a.startsWith(prefix));
  try {
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "load" });

    // A streamed read is loaded from its first line, while the rest come.
    check("a streamed read shows its first line while the answer is still arriving",
      await shows(page, "lines", "one") && (await text(page, "lines-state")) === "true true",
      `${await text(page, "lines")} / ${await text(page, "lines-state")}`);
    check("and grows to every line, then is done",
      await shows(page, "lines", "one,two,three") && await shows(page, "lines-state", "true false"),
      `${await text(page, "lines")} / ${await text(page, "lines-state")}`);

    // By number.
    check("the first numbered page", await shows(page, "list", "a,b"), await text(page, "list"));
    await press("More list");
    check("the next page is gathered below it", await shows(page, "list", "a,b,c"), await text(page, "list"));
    await press("More list");
    check("an empty page ends them", await shows(page, "list-more", "false"), await text(page, "list-more"));
    check("asked for pages 1, 2 and 3, the first as the call wrote it",
      JSON.stringify(asked("/api/posts")) === JSON.stringify(["/api/posts?page=1", "/api/posts?page=2", "/api/posts?page=3"])
        || JSON.stringify(asked("/api/posts")) === JSON.stringify(["/api/posts", "/api/posts?page=2", "/api/posts?page=3"]),
      JSON.stringify(asked("/api/posts")));
    const before = asked("/api/posts").length;
    await press("Refresh list");
    await until(page, () => true, null, 300);
    await new Promise((r) => setTimeout(r, 300));
    const again = asked("/api/posts").slice(before);
    check("a refresh asks again for every page gathered, in order", again.length === 3 && again[2] === "/api/posts?page=3", JSON.stringify(again));
    check("and shows them together", (await text(page, "list")) === "a,b,c", await text(page, "list"));

    // By cursor.
    check("the first page of a cursor's", await shows(page, "feed", "f1,f2"), await text(page, "feed"));
    await press("More feed");
    check("the next page starts where the last one said", await shows(page, "feed", "f1,f2,f3"), await text(page, "feed"));
    await press("More feed");
    check("until a page names no next one", await shows(page, "feed", "f1,f2,f3,f4") && await shows(page, "feed-more", "false"),
      `${await text(page, "feed")} / ${await text(page, "feed-more")}`);
    check("each asked with the cursor before it",
      JSON.stringify(asked("/api/feed")) === JSON.stringify(["/api/feed", "/api/feed?after=k2", "/api/feed?after=k3"]),
      JSON.stringify(asked("/api/feed")));

    // By the Link header.
    check("the first page of a linked search, with more to come",
      await shows(page, "found", "s1,s2") && await shows(page, "found-more", "true"),
      `${await text(page, "found")} / ${await text(page, "found-more")}`);
    await press("More found");
    check("the next page is the address its rel=\"next\" named",
      await shows(page, "found", "s1,s2,s3") && await shows(page, "found-more", "false"),
      `${await text(page, "found")} / ${await text(page, "found-more")}`);
    check("asked as the server wrote it",
      asked("/api/search").includes("/api/search?q=x&cursor=two"), JSON.stringify(asked("/api/search")));

    // `for await` in an action.
    await press("Walk");
    check("`for await` runs its body for each line as it arrives",
      await shows(page, "walked", "0:one"), await text(page, "walked"));
    check("and counts them to the end", await shows(page, "walked", "0:one,1:two,2:three"), await text(page, "walked"));
    check("nothing threw", thrown.length === 0, thrown.join(" | "));
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("A production build", 4961);

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
