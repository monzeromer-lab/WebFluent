// Every page of the starters `wf init` writes, pre-rendered, taken over in a
// real Chrome: the runtime draws the page again beside the painted one and
// keeps the painted nodes in place (`hydrate.js`). This holds that it does,
// page by page, for the sites a new project starts from:
//
// - the painted page is taken over, not drawn again whole: the runtime
//   says so with a console error when it cannot match the trees, and the
//   painted page's root and its `<main>` stay the very nodes the reader
//   saw (an element something holds is still replaced with its subtree,
//   by design: those are listed, not failed);
// - what the reader sees afterwards is what the script drew: under the dev
//   flag (`window.__WF_DEV__`, as `wf serve` sets it) the runtime compares
//   the two trees and warns of any difference;
// - nothing threw, and nothing logged an error.
//
//   cd tests/browser && npm ci && node hydration.mjs
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

/// Every page a static build wrote, as the route that reaches it.
function routes(build) {
  const out = [];
  const walk = (dir, prefix) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (entry.isDirectory()) walk(path.join(dir, entry.name), `${prefix}${entry.name}/`);
      else if (entry.name === "index.html") out.push(prefix === "/" ? "/" : prefix.replace(/\/$/, ""));
    }
  };
  walk(build, "/");
  return out.sort();
}

async function starter(template, port) {
  console.log(`\nwf init -t ${template}, pre-rendered`);
  const work = fs.mkdtempSync(path.join(os.tmpdir(), `wf-hydration-${template}-`));
  const dir = path.join(work, "site");
  execFileSync(WF, ["init", dir, "-t", template], { stdio: "pipe" });
  // A single-page starter is pre-rendered too, so there is a page to take
  // over; the static one is already.
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build = { ...(config.build || {}), ssg: true };
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "-d", dir], { stdio: "pipe" });

  const build = path.join(dir, "build");
  const site = host(build, port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  try {
    const pages = routes(build);
    check(`${template}: the build wrote pages`, pages.length > 0, build);
    for (const route of pages) {
      const page = await browser.newPage();
      const problems = [];
      page.on("pageerror", (e) => problems.push(`threw: ${e.message}`));
      page.on("console", (m) => {
        const text = m.text();
        if (m.type() === "error" && !/^Failed to load resource/.test(text)) problems.push(`error: ${text}`);
        if (m.type() === "warn" && /^WF: /.test(text)) problems.push(`warning: ${text}`);
      });
      // Before any script runs — the parser is done, the deferred scripts
      // have not started — every painted element is noted, and the dev
      // flag that makes the runtime compare what it took over is set.
      await page.evaluateOnNewDocument(() => {
        window.__WF_DEV__ = true;
        document.addEventListener("readystatechange", () => {
          if (document.readyState !== "interactive" || window.__painted) return;
          const root = document.body;
          window.__painted = {
            root: root.firstElementChild,
            main: document.querySelector("main"),
            all: [...root.querySelectorAll("*")],
            text: (() => {
              const out = [];
              const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
              for (let n = walker.nextNode(); n; n = walker.nextNode()) if (n.textContent.trim()) out.push(n);
              return out;
            })(),
          };
        });
      });
      try {
        await page.goto(`http://127.0.0.1:${port}${route}`, { waitUntil: "networkidle0" });
        // Let the router draw the page's chunk and settle.
        await new Promise((r) => setTimeout(r, 300));
        const seen = await page.evaluate(() => {
          const p = window.__painted;
          if (!p) return null;
          const kept = p.all.filter((e) => e.isConnected).length;
          const words = p.text.filter((t) => t.isConnected).length;
          // The painted subtrees put aside for the drawn ones: an element
          // something holds (a handler, a widget, a form) is replaced with
          // everything under it, by design — listed, so a change that
          // replaces more shows here.
          const gone = p.all.filter((e) => !e.isConnected);
          const replaced = gone
            .filter((e) => !gone.includes(e.parentElement))
            .map((e) => `${e.tagName.toLowerCase()}${e.classList.length ? "." + [...e.classList].slice(0, 2).join(".") : ""}`);
          return {
            root: !!p.root && p.root.isConnected,
            main: !p.main || p.main.isConnected,
            kept,
            all: p.all.length,
            words,
            text: p.text.length,
            replaced,
          };
        });
        const label = `${template} ${route}`;
        if (!seen) {
          check(`${label}: the page was painted before its script ran`, false, "no painted page");
          continue;
        }
        check(
          `${label}: taken over in place (${seen.kept}/${seen.all} elements, ${seen.words}/${seen.text} texts kept)`,
          seen.root && seen.main,
          `root kept: ${seen.root}, main kept: ${seen.main}`,
        );
        check(`${label}: what was taken over is what was drawn, and nothing failed`, problems.length === 0, problems.join(" | "));
        if (seen.replaced.length) console.log(`        held, and drawn again: ${seen.replaced.join(", ")}`);
      } finally {
        await page.close();
      }
    }
  } finally {
    await browser.close();
    await site.stop();
    fs.rmSync(work, { recursive: true, force: true });
  }
}

await starter("spa", 47431);
await starter("static", 47432);
const passed = results.filter(Boolean).length;
console.log(`\n${passed} of ${results.length} passed`);
process.exit(passed === results.length ? 0 : 1);
