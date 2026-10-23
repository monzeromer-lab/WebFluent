// `wf test` and a component call's attributes, in a real Chrome.
//
// A project of its own, written here: a component that draws a name's head
// out of inline runs, a stylesheet under `src/`, one `meta.stylesheets`
// links from `public/`, and an image that stylesheet names. Its tests are
// run by `wf test` itself: a head reads as one line both rendered and in a
// browser; a test in a browser is drawn with the project's own stylesheets
// and `public/`; and `data-x: true` through a call is the word "true", which
// a stylesheet keyed on `[data-x="true"]` shows. The same page is then built
// pre-rendered and as a single page, and what a call and an element carry
// is the same on the page as built and the page running, taken over with
// no mismatch.
//
//   cd tests/browser && npm ci && node testing.mjs
//
// WF (default target/release/wf) and CHROME (default /usr/bin/google-chrome)
// say where they are.

import puppeteer from "puppeteer-core";
import { execFileSync, spawnSync } from "node:child_process";
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

function write(dir, files) {
  for (const [rel, text] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(dir, rel)), { recursive: true });
    fs.writeFileSync(path.join(dir, rel), text);
  }
}

const APP = `
component SymbolHead(_ name: String, word: String = "", detail: String = "") {
    Text(class: "head") {
        if word != "" { Code("{word} ") }
        Code(name)
        Code(": ")
        Code(detail)
    }
}

component Flag(_ label: String) {
    Text(label, class: "flag")
}

page Home(path: "/", title: "Home", description: "A head, a flag and an element.") {
    Heading("Home").h1
    SymbolHead("live", word: "derived", detail: "[Deploy]")
    Row(class: "runs") { Code("a") Code("b") }
    Flag("call", id: "call", data-x: true, data-y: false, aria-hidden: false, data-n: 3)
    Text("element", id: "element", data-x: true, data-y: false, aria-hidden: false, data-n: 3)
}
`;

const TESTS = `
test "rendered, a head reads as one line" {
    SymbolHead("live", word: "derived", detail: "[Deploy]")
    Row { Code("a") Code("b") }
    expect "derived live: [Deploy]"
    expect "a b"
    expect not "live :"
}

test "in a browser, a head reads the same" {
    state n = 0
    SymbolHead("live", word: "derived", detail: "[Deploy]")
    Row { Code("a") Code("b") }
    Button("Go") { on click { n = n + 1 } }
    click "Go"
    expect "derived live: [Deploy]"
    expect "a b"
    expect not "live :"
}

test "in a browser, the page is drawn with the project's own stylesheets" {
    state n = 0
    Text("quiet", class: "shout")
    Text("hidden words", class: "gone")
    Stack(class: "dot") { Text("dot") }
    Button("Go") { on click { n = n + 1 } }
    click "Go"
    expect "QUIET"
    expect not "hidden words"
}

test "in a browser, true through a call is the word" {
    state n = 0
    Flag("flagged", data-x: true)
    Flag("plain", data-x: false)
    Button("Go") { on click { n = n + 1 } }
    click "Go"
    expect "FLAGGED"
    expect "plain"
}
`;

function project() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-testing-site-"));
  write(dir, {
    "webfluent.app.json": JSON.stringify({
      name: "Testing",
      build: { output: "./build", ssg: true, minify: false },
      meta: { stylesheets: ["/extra.css"] },
    }, null, 2),
    "src/App.wf": APP,
    "src/look/parts.css": [
      ".shout { text-transform: uppercase; }",
      ".dot { background-image: url(\"/img/dot.svg\"); }",
      ".flag[data-x=\"true\"] { text-transform: uppercase; }",
      "",
    ].join("\n"),
    "public/extra.css": ".gone { display: none; }\n",
    "public/img/dot.svg": "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"4\"><circle cx=\"2\" cy=\"2\" r=\"2\"/></svg>\n",
    "tests/look.wf": TESTS,
  });
  return dir;
}

function wfTest(dir) {
  console.log("\nwf test");
  // From inside the project, as one usually runs it: `public/` is reached
  // from the test's site wherever that is.
  const run = spawnSync(path.resolve(WF), ["test", "."], { cwd: dir, encoding: "utf8" });
  const out = `${run.stdout}${run.stderr}`;
  const line = (name) => out.split("\n").find((l) => l.includes(name)) || "";
  for (const name of [
    "rendered, a head reads as one line",
    "in a browser, a head reads the same",
    "in a browser, the page is drawn with the project's own stylesheets",
    "in a browser, true through a call is the word",
  ]) {
    check(name, line(name).trim().startsWith("ok") || line(name).trim().startsWith("new"), out);
  }
  check("wf test passes", run.status === 0, out);
}

async function scenario(dir, label, ssg, port) {
  console.log(`\n${label}`);
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  fs.rmSync(path.join(dir, "build"), { recursive: true, force: true });
  execFileSync(WF, ["build", "-d", dir], { stdio: "ignore" });
  const built = fs.readFileSync(path.join(dir, "build/index.html"), "utf8");

  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  await page.evaluateOnNewDocument(() => { window.__WF_DEV__ = true; });
  const problems = [];
  const warnings = [];
  page.on("pageerror", (e) => problems.push(e.message));
  page.on("console", (m) => {
    // A missing file is caught by its response, below, which knows its
    // address; the browser's own line for it does not.
    if (m.type() === "error" && !/^Failed to load resource/.test(m.text())) problems.push(m.text());
    if (m.type() === "warn") warnings.push(m.text());
  });
  page.on("response", (r) => { if (r.status() >= 400 && !/favicon\.ico$/.test(r.url())) problems.push(`${r.status()} ${r.url()}`); });
  const carried = (id) => page.evaluate((id) => {
    const el = document.getElementById(id);
    return el && ["data-x", "data-y", "aria-hidden", "data-n"].map((k) => `${k}=${el.getAttribute(k)}`).join(" ");
  }, id);
  try {
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    const want = "data-x=true data-y=null aria-hidden=false data-n=3";
    check("an element carries true as the word, false as nothing but an ARIA state's", (await carried("element")) === want, await carried("element"));
    check("a call gives its root the same", (await carried("call")) === want, await carried("call"));
    check("the stylesheet keyed on the word shows it", await page.evaluate(() => getComputedStyle(document.getElementById("call")).textTransform === "uppercase"));
    check("a head reads as one line on the page",
      await page.evaluate(() => document.querySelector(".head").innerText === "derived live: [Deploy]"),
      await page.evaluate(() => document.querySelector(".head")?.innerText));
    if (ssg) {
      const tag = (id) => (built.match(new RegExp(`<p[^>]*id="${id}"[^>]*>`)) || [""])[0];
      const fromTag = (t) => ["data-x", "data-y", "aria-hidden", "data-n"]
        .map((k) => `${k}=${(t.match(new RegExp(` ${k}="([^"]*)"`)) || [null, null])[1]}`).join(" ");
      check("the pre-rendered element carries the same", fromTag(tag("element")) === want, tag("element"));
      check("and so does the pre-rendered call", fromTag(tag("call")) === want, tag("call"));
      const mismatch = warnings.filter((w) => w.includes("differs"));
      check("the page taken over is the page drawn", mismatch.length === 0, mismatch.join(" | "));
    }
    check("nothing threw, and nothing was missing", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
  }
}

const dir = project();
try {
  wfTest(dir);
  await scenario(dir, "Pre-rendered (build.ssg)", true, 4421);
  await scenario(dir, "A single page", false, 4422);
} finally {
  fs.rmSync(dir, { recursive: true, force: true });
}
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
