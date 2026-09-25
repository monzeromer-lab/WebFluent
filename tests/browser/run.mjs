//! Every page of every example, in a real browser — through `wf verify`.
//!
//! The command is the product's; this is the sweep across everything the
//! repository ships, so a change that breaks one example's pages is
//! caught here rather than by whoever opens it next. It runs the compiler
//! against its own fixtures, the templates `wf init` writes, the
//! documentation site, and a real project if one is on this machine.
//!
//! Run: node tests/browser/run.mjs [name…]
//! It reports nothing to do where there is no browser to drive.

import { execFileSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../..", import.meta.url));
const WF = ["target/release/wf", "target/debug/wf"].map((p) => join(ROOT, p)).find(existsSync);
const CHROME = [
  "/usr/bin/google-chrome",
  "/usr/bin/google-chrome-stable",
  "/usr/bin/chromium",
  "/usr/bin/chromium-browser",
].find(existsSync);

/// Everything worth loading. A PDF and a deck are not pages; the build
/// tests cover those.
function subjects() {
  const out = [];
  const fixtures = join(ROOT, "tests/fixtures");
  for (const name of readdirSync(fixtures)) {
    const config = join(fixtures, name, "webfluent.app.json");
    if (!existsSync(config)) continue;
    const kind = JSON.parse(readFileSync(config, "utf8")).build?.output_type ?? "spa";
    if (kind === "pdf" || kind === "slides") continue;
    out.push({ name: `fixture:${name}`, dir: join(fixtures, name) });
  }
  for (const template of ["spa", "static"]) out.push({ name: `init:${template}`, template });
  out.push({ name: "site", dir: join(ROOT, "site") });
  const halyard = "/home/monzer-omer/Code/halyard";
  if (existsSync(join(halyard, "webfluent.app.json"))) out.push({ name: "halyard", dir: halyard });
  return out;
}

/// A copy of the project, built where nothing else is, so a run cannot
/// leave anything behind and cannot read what a previous one did.
function prepare(subject, work) {
  const dir = join(work, subject.name.replace(":", "-"));
  if (subject.template) {
    execFileSync(WF, ["init", dir, "-t", subject.template], { stdio: "pipe" });
  } else {
    cpSync(subject.dir, dir, { recursive: true, filter: (p) => !p.includes("/.git") });
    rmSync(join(dir, "build"), { recursive: true, force: true });
  }
  execFileSync(WF, ["build", "-d", dir], { stdio: "pipe" });
  return dir;
}

function main() {
  if (!WF) throw new Error("build the compiler first: cargo build --release");
  if (!CHROME) {
    console.log("  no browser on this machine; nothing to sweep");
    return;
  }
  const only = process.argv.slice(2);
  const work = mkdtempSync(join(tmpdir(), "wf-browser-"));
  const failures = [];
  const drawn = new Set();
  let pages = 0;

  try {
    for (const subject of subjects()) {
      if (only.length && !only.some((n) => subject.name.includes(n))) continue;
      let dir;
      try {
        dir = prepare(subject, work);
      } catch (e) {
        failures.push(`${subject.name}: build failed — ${e.stdout?.toString() || e.message}`);
        continue;
      }
      let report;
      try {
        report = JSON.parse(
          execFileSync(WF, ["verify", dir, "--json"], { stdio: "pipe", maxBuffer: 64e6 }),
        );
      } catch (e) {
        // A failing verify still prints its report, and the report is
        // what says which page was wrong.
        try {
          report = JSON.parse(e.stdout.toString());
        } catch {
          failures.push(`${subject.name}: verify failed — ${e.stderr?.toString() || e.message}`);
          continue;
        }
      }
      console.log(`\n  ${subject.name}  (${report.routes.length} route(s))`);
      for (const route of report.routes) {
        pages += 1;
        for (const c of route.drew) drawn.add(c);
        const ok = route.errors.length === 0;
        console.log(
          `    ${ok ? "ok  " : "FAIL"} ${route.url.padEnd(34)}` +
            ` ${String(route.first_contentful_paint).padStart(5)}ms` +
            ` ${String(route.elements).padStart(6)} nodes` +
            ` ${(route.transferred / 1024).toFixed(1).padStart(8)} kB`,
        );
        for (const problem of route.errors) {
          failures.push(`${subject.name} ${route.url}: ${problem}`);
          console.log(`         ${problem}`);
        }
      }
    }
  } finally {
    rmSync(work, { recursive: true, force: true });
  }

  // The fixtures that declare tests: `wf test` runs the ones that only
  // look through the renderer and the ones that act in a browser, and
  // both are part of what a change must not break.
  for (const name of readdirSync(join(ROOT, "tests/fixtures"))) {
    const dir = join(ROOT, "tests/fixtures", name);
    if (!existsSync(join(dir, "tests"))) continue;
    if (only.length && !only.some((n) => `fixture:${name}`.includes(n))) continue;
    try {
      execFileSync(WF, ["test", dir], { stdio: "pipe" });
      console.log(`\n  fixture:${name}  tests pass`);
    } catch (e) {
      const out = (e.stdout?.toString() || "") + (e.stderr?.toString() || "");
      failures.push(`fixture:${name}: wf test failed`);
      console.log(`\n  fixture:${name}  TESTS FAILED\n${out}`);
    }
  }

  // Which built-ins nothing in the repository drew. One project need not
  // draw them all; between them they should.
  const registry = JSON.parse(execFileSync(WF, ["registry", "--json"], { stdio: "pipe" }));
  const NOT_IN_A_BROWSER = new Set([
    "Children", "Router", "Unsafe", "UnsafeHtml", "Host",
    "Document", "Header", "Footer", "PageBreak", "Paragraph",
    "Presentation", "Slide", "TitleSlide", "SectionSlide", "TwoColumn", "ImageSlide",
  ]);
  const web = registry.components.filter(
    (c) => !c.owner && c.class && !NOT_IN_A_BROWSER.has(c.name)
      && !/original (spelling|grammar)/.test(c.summary),
  );
  const missing = web
    .filter((c) => {
      const base = c.class.split(" ").pop();
      return ![...drawn].some((s) => s === base || s.startsWith(base + "--"));
    })
    .map((c) => c.name);
  console.log(
    missing.length
      ? `\n  ${web.length - missing.length}/${web.length} built-ins drawn; nothing drew: ${missing.join(", ")}`
      : `\n  every one of the ${web.length} built-ins was drawn`,
  );
  if (missing.length) failures.push(`nothing drew: ${missing.join(", ")}`);

  console.log(`\n  ${pages} page(s), ${failures.length} problem(s)`);
  for (const line of failures) console.log(`    ${line}`);
  if (failures.length) process.exitCode = 1;
}

main();
