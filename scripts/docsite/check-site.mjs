// The built docs site, held to what a reader sees, in headless Chrome:
//
// - every built-in's reference page draws its example in the preview — the
//   preview is not empty, nothing in it has collapsed to no size, and the
//   page threw and logged nothing — unless its example is marked as one a
//   web page cannot draw (a PDF's or a deck's element);
// - no table cell on any page breaks a word across lines: a column too
//   narrow for its words is drawn a letter a line (`overflow-wrap:
//   anywhere` once did that).
//
//   node scripts/docsite/check-site.mjs [docs] [--only /reference]
//
// Needs `npm ci` in tests/browser (puppeteer-core) and a Chrome, at
// CHROME or /usr/bin/google-chrome.
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import http from "node:http";

const here = path.dirname(new URL(import.meta.url).pathname);
const repo = path.resolve(here, "../..");
const require = createRequire(path.join(repo, "tests/browser/package.json"));
const puppeteer = require("puppeteer-core");

const args = process.argv.slice(2);
const root = path.resolve(args.find((a) => !a.startsWith("--")) || path.join(repo, "docs"));
const only = args.includes("--only") ? args[args.indexOf("--only") + 1] : "";

const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".css": "text/css", ".json": "application/json", ".svg": "image/svg+xml", ".png": "image/png", ".webp": "image/webp", ".woff2": "font/woff2", ".wasm": "application/wasm", ".md": "text/markdown", ".txt": "text/plain" };
const file = (p) => {
  const base = path.join(root, decodeURIComponent(p.split("?")[0].split("#")[0]));
  if (!base.startsWith(root)) return null;
  for (const c of [base, base + ".html", path.join(base, "index.html")]) {
    if (fs.existsSync(c) && fs.statSync(c).isFile()) return c;
  }
  return null;
};
const server = http.createServer((req, res) => {
  const f = file(req.url) || path.join(root, "404.html");
  res.writeHead(file(req.url) ? 200 : 404, { "Content-Type": TYPES[path.extname(f)] || "application/octet-stream" });
  res.end(fs.readFileSync(f));
});
await new Promise((done) => server.listen(0, "127.0.0.1", done));
const origin = `http://127.0.0.1:${server.address().port}`;

// Every page the build wrote, as a route.
const routes = [];
(function walk(dir) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) { if (!["wasm", "fonts", "pages", "studio"].includes(e.name)) walk(p); continue; }
    if (!e.name.endsWith(".html") || e.name === "404.html") continue;
    let r = "/" + path.relative(root, p).replace(/\\/g, "/").replace(/(index)?\.html$/, "");
    r = r.replace(/\/$/, "") || "/";
    if (!routes.includes(r)) routes.push(r);
  }
})(root);

const data = JSON.parse(fs.readFileSync(path.join(repo, "site/src/data/components.json"), "utf8"));
const drawn = new Set(data.items.filter((i) => !i.owner && !i.legacy && i.drawn).map((i) => "/reference/" + i.slug));

const browser = await puppeteer.launch({ executablePath: process.env.CHROME || "/usr/bin/google-chrome", headless: true, args: ["--no-sandbox"] });
const page = await browser.newPage();
// A laptop's width: tables are tightest beside the sidebar and the rail.
await page.setViewport({ width: +(process.env.CHECK_SITE_WIDTH || 1100), height: 900 });
if (process.env.CHECK_SITE_CSS) await page.setBypassCSP(true);
const problems = [];
let errors = [];
page.on("console", (m) => { if (m.type() === "error" && !/Failed to load resource/.test(m.text())) errors.push(m.text()); });
page.on("pageerror", (e) => errors.push(String(e)));

for (const route of routes.filter((r) => r.startsWith(only)).sort()) {
  errors = [];
  await page.goto(origin + route, { waitUntil: "networkidle0" });
  // CHECK_SITE_CSS: a rule laid over every page, to show the check fails
  // when it should (`td, td code { overflow-wrap: anywhere }`).
  if (process.env.CHECK_SITE_CSS) await page.addStyleTag({ content: process.env.CHECK_SITE_CSS });
  const found = await page.evaluate((isDrawn) => {
    const out = [];
    // A word split across lines in a table cell.
    for (const cell of document.querySelectorAll("td, th")) {
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT);
      let node;
      while ((node = walker.nextNode())) {
        // A line may end where Unicode allows one to (after a hyphen, a
        // slash, a `?`); a run of letters and digits must stay whole.
        const re = /[\p{L}\p{N}_]{3,}/gu;
        let m;
        while ((m = re.exec(node.data))) {
          const range = document.createRange();
          range.setStart(node, m.index);
          range.setEnd(node, m.index + m[0].length);
          const tops = new Set([...range.getClientRects()].map((r) => Math.round(r.top)));
          if (tops.size > 1) { out.push(`a table cell breaks "${m[0]}" across lines`); break; }
        }
        if (out.length) break;
      }
      if (out.length > 3) break;
    }
    // The preview of a built-in's page.
    const subject = document.querySelector(".bp-stage__subject");
    if (subject && isDrawn) {
      const visible = [...subject.querySelectorAll("*")].filter((e) => { const b = e.getBoundingClientRect(); return b.width > 1 && b.height > 1; });
      if (!visible.length) out.push("the preview draws nothing");
      const collapsed = [...subject.children].filter((e) => {
        const b = e.getBoundingClientRect();
        return e.tagName !== "DIALOG" && getComputedStyle(e).display !== "none" && (b.width < 2 || b.height < 2) && !e.childElementCount && !e.textContent.trim();
      });
      if (collapsed.length) out.push("in the preview, nothing shows of " + collapsed.map((e) => e.className || e.tagName).join(", "));
    }
    return out;
  }, drawn.has(route));
  if (drawn.has(route) && errors.length) found.push("the page logged: " + errors[0].slice(0, 160));
  for (const f of found) problems.push(`${route}: ${f}`);
}
await browser.close();
server.close();
console.log(`${routes.filter((r) => r.startsWith(only)).length} page(s), ${drawn.size} drawn previews, ${problems.length} problem(s)`);
for (const p of problems) console.log("  " + p);
process.exit(problems.length ? 1 : 0);
