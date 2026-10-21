// A picture named by its path, in a real Chrome, against
// `tests/fixtures/pictures`, built twice — pre-rendered and as a single
// page: `Image(src: "/hero.png")` is a `<picture>` of the widths the build
// wrote; the browser takes the width the viewport needs; the box is the
// picture's shape before it arrives, so nothing below it moves; the blurred
// placeholder is allowed by the policy the build ships; an SVG is the file
// as it is.
//
//   cd tests/browser && npm ci && node pictures.mjs
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
const fixture = path.join(repo, "tests/fixtures/pictures");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-pictures-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  // 1200×630: the build writes 480, 960 and 1200.
  fs.copyFileSync(path.join(repo, "site/public/og.png"), path.join(dir, "public/hero.png"));
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const plan = JSON.parse(execFileSync(WF, ["assets", "-d", dir, "--plan", "/hero.png", "--json"]).toString());

  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const problems = [];
  try {
    for (const [width, want] of [[500, 960], [1280, 1200], [400, 480]]) {
      const page = await browser.newPage();
      page.on("pageerror", (e) => problems.push(e.message));
      page.on("console", (m) => { if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text()); });
      page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
      await page.setViewport({ width, height: 900, deviceScaleFactor: 1 });
      // Layout shifts from the start, and what the page fetched.
      await page.evaluateOnNewDocument(() => {
        window.__shift = 0;
        new PerformanceObserver((list) => {
          for (const e of list.getEntries()) if (!e.hadRecentInput) window.__shift += e.value;
        }).observe({ type: "layout-shift", buffered: true });
      });
      const fetched = [];
      page.on("request", (r) => fetched.push(new URL(r.url()).pathname));
      await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
      // Every picture on the page in view, and loaded.
      await page.evaluate(() => document.querySelectorAll("img").forEach((i) => { i.loading = "eager"; }));
      await page.waitForFunction(() => [...document.querySelectorAll("img")].every((i) => i.complete && i.naturalWidth > 0),
        { timeout: 5000 }).catch(() => {});
      const seen = await page.evaluate(async () => {
        const picture = document.querySelector("picture");
        const img = picture?.querySelector("img");
        const logo = [...document.querySelectorAll("img")].find((i) => /logo\.svg$/.test(i.getAttribute("src") || ""));
        return {
          pictures: document.querySelectorAll("picture").length,
          types: [...(picture?.querySelectorAll("source") || [])].map((s) => s.type),
          current: img?.currentSrc || "",
          // With a `srcset`, `naturalWidth` is corrected for density: the
          // file's own width is read from the file alone.
          natural: await new Promise((done) => {
            const one = new Image();
            one.onload = () => done(one.naturalWidth);
            one.onerror = () => done(0);
            one.src = img?.currentSrc || "";
          }),
          attrs: [img?.getAttribute("width"), img?.getAttribute("height")],
          box: img ? img.getBoundingClientRect().height / img.getBoundingClientRect().width : 0,
          alt: img?.alt,
          logo: logo ? { src: logo.getAttribute("src"), ok: logo.complete && logo.naturalWidth > 0 } : null,
          shift: window.__shift,
        };
      });
      const at = `at ${width}px`;
      check(`${at}: one <picture>, with a webp source`, seen.pictures === 1 && seen.types.join() === "image/webp", JSON.stringify(seen));
      check(`${at}: the browser takes the ${want}px file`, seen.current.endsWith(`.${want}.webp`) && seen.natural === want, seen.current);
      const name = plan.formats[0].files.find((f) => f.width === want).name;
      check(`${at}: it is the file the plan named`, seen.current.endsWith(`/img/${name}`), `${seen.current} vs ${name}`);
      check(`${at}: the box is the picture's shape from the file`, seen.attrs.join("x") === "1200x630" && Math.abs(seen.box - 630 / 1200) < 0.01, `${seen.attrs} ${seen.box}`);
      check(`${at}: nothing on the page moved`, seen.shift === 0, String(seen.shift));
      check(`${at}: only the one width was fetched`, fetched.filter((p) => p.includes("/img/hero.")).length === 1, fetched.filter((p) => p.includes("/img/")).join(" "));
      check(`${at}: the alt is the one written`, seen.alt === "The harbour at dawn", seen.alt);
      check(`${at}: an SVG is the file as it is`, seen.logo?.src === "/logo.svg" && seen.logo.ok, JSON.stringify(seen.logo));
      await page.close();
    }
    check("nothing threw, nothing 404'd, the policy refused nothing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4391);
await scenario("A single page", false, 4392);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
