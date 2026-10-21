// Plural forms in a real Chrome, against `tests/fixtures/plurals` — an
// Arabic site — pre-rendered and as a single page: the page is in Arabic
// and right to left before its script runs; each count is painted in the
// form Arabic's rules give it, and the live page says the same; read in
// English, the page turns left to right and each count takes English's
// form, exactly none the file's `zero`; and the browser's own plural rules
// pick for every count what the compiler's do.
//
//   cd tests/browser && npm ci && node plurals.mjs

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
const fixture = path.join(repo, "tests/fixtures/plurals");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

const COUNTS = [0, 1, 2, 5, 11, 100];
const ARABIC = ["لا ملفات", "ملف واحد", "ملفان", "5 ملفات", "11 ملفًا", "100 ملف"];
const ENGLISH = ["No files", "1 file", "2 files", "5 files", "11 files", "100 files"];
const texts = (page) => page.evaluate((ns) => ns.map((n) => document.getElementById("n" + n)?.textContent.trim() ?? null), COUNTS);

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-plurals-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  try {
    const html = fs.readFileSync(path.join(dir, "build/index.html"), "utf8");
    check("before its script, the page is Arabic and right to left", /<html lang="ar" dir="rtl">/.test(html), html.slice(0, 120));
    if (ssg) {
      const painted = COUNTS.map((n) => (html.match(new RegExp(`id="n${n}">([^<]*)<`)) || [])[1]?.trim() ?? null);
      check("each count is painted in Arabic's form for it", JSON.stringify(painted) === JSON.stringify(ARABIC), painted.join(" | "));
    }
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    check("the live page says the same", JSON.stringify(await texts(page)) === JSON.stringify(ARABIC), (await texts(page)).join(" | "));
    check("…right to left", (await page.evaluate(() => document.documentElement.dir)) === "rtl");
    await page.click("#english");
    await new Promise((r) => setTimeout(r, 50));
    check("read in English, each count takes English's form, and exactly none the file's zero", JSON.stringify(await texts(page)) === JSON.stringify(ENGLISH), (await texts(page)).join(" | "));
    const after = await page.evaluate(() => [document.documentElement.lang, document.documentElement.dir]);
    check("…left to right", after[0] === "en" && after[1] === "ltr", after.join(" "));
    // The browser's rules and the compiler's, for every locale.
    const table = JSON.parse(execFileSync(WF, ["translations", "--plurals", "--json"], { encoding: "utf8" }));
    const differ = await page.evaluate((table) => {
      const out = [];
      let compared = 0;
      for (const [locale, picks] of Object.entries(table.picks)) {
        let rules;
        try { rules = new Intl.PluralRules(locale); } catch (e) { continue; }
        if (rules.resolvedOptions().locale.split("-")[0] !== locale.split("-")[0]) continue;
        table.counts.forEach((n, i) => { compared += 1; if (rules.select(n) !== picks[i]) out.push(`${locale} ${n}: ${rules.select(n)} vs ${picks[i]}`); });
      }
      return { out, compared };
    }, table);
    const chromeCldr = await page.evaluate(() => (typeof Intl !== "undefined" ? "yes" : "no"));
    check(`Chrome's plural rules pick what the compiler's do, ${differ.compared} counts (CLDR ${table.cldr})`, chromeCldr === "yes" && differ.compared > 10000 && differ.out.length === 0, differ.out.slice(0, 8).join(", "));
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4401);
await scenario("A single page", false, 4402);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
