// Every icon in a real Chrome: a page naming each one the registry lists,
// built pre-rendered and as a single page, draws each as a glyph — an
// `<svg>` with something in it (a line is all width), inside its box, in the colour around it —
// and an unknown name as its word.
//
//   cd tests/browser && npm ci && node icons.mjs

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

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

const icons = JSON.parse(execFileSync(WF, ["registry", "--json"], { encoding: "utf8" })).icons;

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-icons-site-"));
  fs.mkdirSync(path.join(dir, "src/pages"), { recursive: true });
  fs.writeFileSync(path.join(dir, "webfluent.app.json"), JSON.stringify({ name: "icons", build: { ssg } }));
  const rows = icons.map((n) => `        Icon("${n}", id: "i-${n}")`).join("\n");
  fs.writeFileSync(
    path.join(dir, "src/pages/Icons.wf"),
    `page Icons(path: "/", title: "Icons", description: "Every icon.") {\n    Heading("Icons").h1\n    Stack(id: "all") {\n        style { color: rgb(200, 30, 90) }\n${rows}\n        Icon("not-an-icon-at-all", id: "unknown")\n    }\n}\n`,
  );
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    const drawn = await page.evaluate((names) => names.map((n) => {
      const el = document.getElementById("i-" + n);
      const svg = el && (el.tagName.toLowerCase() === "svg" ? el : el.querySelector("svg"));
      if (!svg) return { n, ok: false, why: "no svg" };
      const box = svg.getBBox();
      const r = svg.getBoundingClientRect();
      const stroke = [...svg.querySelectorAll("*")].map((c) => getComputedStyle(c).stroke).find((s) => s && s !== "none");
      const fill = [...svg.querySelectorAll("*")].map((c) => getComputedStyle(c).fill).find((s) => s && s !== "none" && s !== "rgb(0, 0, 0)");
      const inside = box.x >= -1 && box.y >= -1 && box.x + box.width <= 25 && box.y + box.height <= 25;
      const coloured = stroke === "rgb(200, 30, 90)" || fill === "rgb(200, 30, 90)";
      return { n, ok: box.width + box.height > 0 && r.width > 0 && inside && coloured, why: JSON.stringify({ box: [box.x, box.y, box.width, box.height], stroke, fill }) };
    }), icons);
    const bad = drawn.filter((d) => !d.ok);
    check(`each of ${icons.length} icons draws a glyph inside its box in the colour around it`, icons.length >= 55 && bad.length === 0, bad.map((b) => `${b.n} ${b.why}`).join("; "));
    const unknown = await page.evaluate(() => document.getElementById("unknown")?.textContent.trim());
    check("an unknown name shows as its word", unknown === "not-an-icon-at-all", unknown);
    if (ssg) {
      const html = fs.readFileSync(path.join(dir, "build/index.html"), "utf8");
      check("the pre-rendered page names each icon", icons.every((n) => html.includes(`i-${n}`)));
    }
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4403);
await scenario("A single page", false, 4404);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
