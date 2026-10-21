// The node map in a real Chrome, against `tests/fixtures/nodes` built for an
// editor's preview (`wf build --studio`) twice — pre-rendered and as a
// single page: every element carries the id `nodes.json` gives it; the
// pre-rendered page and the page once live stamp the same ids; two
// components written alike in two files keep their own; a loop's one node
// is drawn once per item; the branch not taken is in the map and not on
// the page; an instance's root names the call that made it; and a click at
// the middle of any element lands on that node or one inside it.
//
//   cd tests/browser && npm ci && node nodes.mjs
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
const fixture = path.join(repo, "tests/fixtures/nodes");

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail && !ok ? `  (${detail})` : ""}`);
}

async function scenario(label, ssg, port) {
  console.log(`\n${label}`);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-nodes-site-"));
  fs.cpSync(fixture, dir, { recursive: true, filter: (p) => !p.includes(`${path.sep}build`) });
  const configPath = path.join(dir, "webfluent.app.json");
  const config = JSON.parse(fs.readFileSync(configPath, "utf8"));
  config.build.ssg = ssg;
  fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  execFileSync(WF, ["build", "--studio", "-d", dir], { stdio: "ignore" });
  const nodes = JSON.parse(fs.readFileSync(path.join(dir, "build/nodes.json"), "utf8")).nodes;
  const byId = new Map(nodes.map((n) => [n.path, n]));
  const ancestors = (id) => {
    const out = [];
    for (let n = byId.get(id); n; n = n.parent ? byId.get(n.parent) : null) out.push(n.path);
    return out;
  };

  const site = host(path.join(dir, "build"), port);
  await site.start();
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  const problems = [];
  page.on("pageerror", (e) => problems.push(e.message));
  try {
    // What the pre-rendered page says, before any script runs.
    let painted = [];
    if (ssg) {
      const html = fs.readFileSync(path.join(dir, "build/index.html"), "utf8");
      painted = [...html.matchAll(/data-wf-node="([^"]+)"/g)].map((m) => m[1]).sort();
    }
    await page.goto(`http://localhost:${port}/`, { waitUntil: "networkidle0" });
    const live = await page.evaluate(() => [...document.querySelectorAll("[data-wf-node]")].map((e) => e.getAttribute("data-wf-node")));
    check("every stamped id is one the map has", live.length > 10 && live.every((id) => byId.has(id)), live.filter((id) => !byId.has(id)).join(" "));
    if (ssg) {
      check("the pre-rendered page stamps what the live page does", JSON.stringify(painted) === JSON.stringify([...live].sort()), `${painted.length} vs ${live.length}`);
    }
    check("two components written alike keep their own ids", live.includes("Left:0") && live.includes("Rite:0"), live.join(" "));
    const left = byId.get("Left:0"), rite = byId.get("Rite:0");
    check("…though they are written at the same place in their files", left.start === rite.start && left.end === rite.end && left.file !== rite.file, `${left.file} ${rite.file}`);
    const item = nodes.find((n) => n.element === "Badge");
    check("a loop's node is drawn once per item", live.filter((id) => id === item.path).length === 3 && item.within[0].kind === "for");
    const shut = nodes.find((n) => n.within.some((w) => w.kind === "else"));
    check("the branch not taken is in the map and not on the page", shut && !live.includes(shut.path) && live.includes(nodes.find((n) => n.within.some((w) => w.kind === "if")).path));
    const calls = await page.evaluate(() => [...document.querySelectorAll("[data-wf-call]")].map((e) => [e.getAttribute("data-wf-node"), e.getAttribute("data-wf-call")]));
    const panelCall = nodes.find((n) => n.element === "Panel");
    check("an instance's root names the call that made it", calls.some(([node, call]) => node === "Panel:0" && call === panelCall.path), JSON.stringify(calls));
    check("…and every call it names is in the map", calls.every(([, call]) => call.split(" ").every((c) => byId.has(c))));
    // A click at the middle of each element lands on it, or on what is
    // drawn inside it — its own children, or a fill the caller put there.
    const taps = await page.evaluate(() => [...document.querySelectorAll("[data-wf-node]")].map((e) => {
      const r = e.getBoundingClientRect();
      if (!(r.width > 0 && r.height > 0)) return null;
      const hit = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2)?.closest("[data-wf-node]");
      return { id: e.getAttribute("data-wf-node"), hit: hit?.getAttribute("data-wf-node") ?? null, inside: !!hit && e.contains(hit) };
    }).filter(Boolean));
    const missed = taps.filter((t) => !t.inside || !byId.has(t.hit)).map((t) => `${t.id} → ${t.hit}`);
    check(`a click at the middle of each of ${taps.length} elements lands on it or inside it`, taps.length > 8 && missed.length === 0, missed.join(", "));
    // And what it lands on is the innermost: the node itself, unless one of
    // its own is under the point.
    const own = taps.filter((t) => t.hit === t.id || ancestors(t.hit).includes(t.id) || byId.get(t.hit).component !== byId.get(t.id).component);
    check("…on the innermost node there", own.length === taps.length, taps.filter((t) => !own.includes(t)).map((t) => `${t.id} → ${t.hit}`).join(", "));
    check("nothing threw", problems.length === 0, problems.join(" | "));
  } catch (e) {
    check(`the run (${e.message})`, false);
  } finally {
    await browser.close();
    await site.stop().catch(() => {});
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

await scenario("Pre-rendered (build.ssg)", true, 4395);
await scenario("A single page", false, 4396);
const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} of ${results.length} passed`);
process.exit(failed ? 1 : 0);
