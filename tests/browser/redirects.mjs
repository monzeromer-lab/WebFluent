// Where a page may send its reader, in a real Chrome. A static build's
// `navigate(query.next ?? "/")` — the usual after a login — goes to a page
// of the site and nowhere else; a single-page build opened with a `?p=` that
// is not a path of the site still starts its router.
//
//   cd tests/browser && npm ci && node redirects.mjs
//
// WF (default target/release/wf) and CHROME (default /usr/bin/google-chrome)
// say where they are.

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

function project(ssg) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-redirects-site-"));
  const files = {
    "webfluent.app.json": JSON.stringify({
      name: "redirects", version: "1.0.0",
      build: { output: "./build", minify: false, ssg, csp: true },
      meta: { title: "redirects", description: "Where a page may send its reader.", lang: "en" },
    }, null, 2),
    "src/App.wf": "app {\n    Router\n}\n",
    "src/pages/Home.wf": `page Home(path: "/", title: "Home", description: "Home.") {\n    Heading("Home").h1\n}\n`,
    "src/pages/Next.wf": `page Next(path: "/next", title: "Next", description: "Next.") {\n    Heading("Next").h1\n}\n`,
    "src/pages/Login.wf": `page Login(path: "/login", title: "Login", description: "Login.") {\n    Heading("Login").h1\n    Button("Continue") { on click { navigate(query.next ?? "/") } }\n}\n`,
  };
  for (const [rel, text] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(dir, rel)), { recursive: true });
    fs.writeFileSync(path.join(dir, rel), text);
  }
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  return dir;
}

async function withSite(ssg, port, fn) {
  const dir = project(ssg);
  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  try {
    await fn(browser, `http://127.0.0.1:${port}`);
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

console.log("\nA static build's navigate()");
await withSite(true, 43100 + Math.floor(Math.random() * 400), async (browser, origin) => {
  const page = await browser.newPage();
  // Anything that leaves for another host is refused before it is made.
  await page.setRequestInterception(true);
  const offsite = [];
  page.on("request", (r) => {
    if (!r.url().startsWith(origin)) { offsite.push(r.url()); r.abort(); } else r.continue();
  });
  for (const next of ["//evil.example/", "https://evil.example/x", "/\\evil.example/"]) {
    await page.goto(`${origin}/login/?next=${encodeURIComponent(next)}`, { waitUntil: "networkidle0" });
    await page.click("button");
    await sleep(400);
    check(`?next=${next} stays on the site`, page.url().startsWith(origin) && offsite.length === 0, `${page.url()} ${offsite.join(" ")}`);
  }
  await page.goto(`${origin}/login/?next=${encodeURIComponent("/next")}`, { waitUntil: "networkidle0" });
  await Promise.all([page.waitForNavigation({ waitUntil: "networkidle0" }), page.click("button")]);
  check("?next=/next goes to the site's page", new URL(page.url()).pathname.replace(/\/$/, "") === "/next", page.url());
});

console.log("\nA single-page build opened with ?p=");
await withSite(false, 43500 + Math.floor(Math.random() * 400), async (browser, origin) => {
  const page = await browser.newPage();
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  for (const p of ["//evil.example", "/\\evil.example", "https://evil.example/"]) {
    errors.length = 0;
    await page.goto(`${origin}/?p=${encodeURIComponent(p)}`, { waitUntil: "networkidle0" });
    const drew = await page.evaluate(() => !!document.querySelector("main h1"));
    check(`?p=${p} leaves the router to start`, drew && errors.length === 0, errors.join(" | "));
  }
  await page.goto(`${origin}/?p=${encodeURIComponent("/next")}`, { waitUntil: "networkidle0" });
  const heading = await page.evaluate(() => document.querySelector("main h1") && document.querySelector("main h1").textContent);
  check("?p=/next still opens that page", heading === "Next" && new URL(page.url()).pathname === "/next", `${heading} ${page.url()}`);
});

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
