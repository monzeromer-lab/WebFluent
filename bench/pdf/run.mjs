// Runs the PDF benchmark and writes RESULTS.md and results.json.
//
//   just bench-pdf          (builds everything first, then runs this)
//   node bench/pdf/run.mjs  (expects the builds to be there)
//
// Environment: BENCH_COLD_RUNS (default 10), BENCH_WARM_RENDERS (100),
// BENCH_WARMUP (5), BENCH_OFFLINE=1 to skip the GitHub and Chrome for Testing
// size lookups.

import { execFileSync, spawn, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
  lstatSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import os from "node:os";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..");
const out = join(here, "out");
// Puppeteer finds its browsers through .puppeteerrc.cjs, which it looks for
// from the working directory up.
process.chdir(here);
mkdirSync(out, { recursive: true });

const COLD_RUNS = Number(process.env.BENCH_COLD_RUNS ?? 10);
const WARM_RENDERS = Number(process.env.BENCH_WARM_RENDERS ?? 100);
const WARMUP = Number(process.env.BENCH_WARMUP ?? 5);
const OFFLINE = process.env.BENCH_OFFLINE === "1";

const WF = join(repo, "target", "release", "wf");
const RUST_BENCH = join(here, "rust", "target", "release", "wf-pdf-bench");
const TEMPLATE = join(here, "invoice.wf");
const DATA = join(here, "invoice.json");
for (const f of [WF, RUST_BENCH, join(here, "node_modules", "puppeteer")]) {
  if (!existsSync(f)) {
    console.error(`missing ${relative(repo, f)} — run \`just bench-pdf\`, which builds it`);
    process.exit(2);
  }
}

const log = (...a) => console.error("  ", ...a);
const sh = (cmd, args, opts = {}) => {
  try {
    return execFileSync(cmd, args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], ...opts }).trim();
  } catch {
    return null;
  }
};

// ── Statistics ─────────────────────────────────────────────────────────

function stats(xs) {
  const s = [...xs].sort((a, b) => a - b);
  // Nearest-rank percentile.
  const pct = (p) => s[Math.min(s.length - 1, Math.max(0, Math.ceil((p / 100) * s.length) - 1))];
  const median = s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2;
  return {
    n: s.length,
    min: s[0],
    median,
    p95: pct(95),
    max: s[s.length - 1],
    mean: s.reduce((a, b) => a + b, 0) / s.length,
  };
}

// ── Memory: the whole process tree, sampled ────────────────────────────

/** Every pid under `root` (not `root` itself), read from /proc. */
function descendants(root) {
  const children = new Map();
  for (const name of readdirSync("/proc")) {
    if (!/^\d+$/.test(name)) continue;
    try {
      const stat = readFileSync(`/proc/${name}/stat`, "utf8");
      // The command may hold spaces and parentheses; the ppid follows the last ')'.
      const ppid = Number(stat.slice(stat.lastIndexOf(")") + 2).split(" ")[1]);
      if (!children.has(ppid)) children.set(ppid, []);
      children.get(ppid).push(Number(name));
    } catch {}
  }
  const out = [];
  const queue = [root];
  while (queue.length) {
    for (const c of children.get(queue.shift()) ?? []) {
      out.push(c);
      queue.push(c);
    }
  }
  return out;
}

function kb(file, field) {
  try {
    const m = readFileSync(file, "utf8").match(new RegExp(`^${field}:\\s+(\\d+) kB`, "m"));
    return m ? Number(m[1]) : 0;
  } catch {
    return 0;
  }
}

/**
 * Run a command and report its peak memory: the sum over every process it
 * starts (Chrome is a browser process, a GPU process, a network service, a
 * renderer, …), sampled every 10 ms, as RSS and as PSS (which divides a page
 * shared by several processes among them, so a shared library is counted
 * once). GNU time's max RSS of the largest single process is exact and is
 * kept too: for a one-process command it is the better number.
 */
function measureMemory(cmd, args) {
  return new Promise((resolve, reject) => {
    const timeFile = join(out, ".maxrss");
    const haveTime = existsSync("/usr/bin/time");
    const child = haveTime
      ? spawn("/usr/bin/time", ["-f", "%M", "-o", timeFile, cmd, ...args], { stdio: ["ignore", "ignore", "ignore"] })
      : spawn(cmd, args, { stdio: ["ignore", "ignore", "ignore"] });
    let peakRss = 0;
    let peakPss = 0;
    const sample = () => {
      const pids = haveTime ? descendants(child.pid) : [child.pid, ...descendants(child.pid)];
      let rss = 0;
      let pss = 0;
      for (const p of pids) {
        rss += kb(`/proc/${p}/status`, "VmRSS");
        pss += kb(`/proc/${p}/smaps_rollup`, "Pss");
      }
      peakRss = Math.max(peakRss, rss);
      peakPss = Math.max(peakPss, pss);
    };
    const timer = setInterval(sample, 10);
    child.on("error", reject);
    child.on("exit", (code) => {
      clearInterval(timer);
      if (code !== 0) return reject(new Error(`${cmd} exited with ${code}`));
      const single = haveTime ? Number(readFileSync(timeFile, "utf8").trim().split("\n").pop()) : 0;
      resolve({
        peak_rss_kb: Math.max(peakRss, single),
        peak_pss_kb: peakPss,
        max_single_rss_kb: single,
      });
    });
  });
}

// ── The scenarios ──────────────────────────────────────────────────────

const node = process.execPath;
const cold = {
  wf: { label: "`wf render` (CLI)", cmd: WF, args: (o) => ["render", TEMPLATE, "--data", DATA, "-f", "pdf", "-o", o] },
  chrome: { label: "Puppeteer + Chrome (default headless)", cmd: node, args: (o) => [join(here, "chrome", "cold.mjs"), DATA, o, "chrome"] },
  shell: { label: "Puppeteer + chrome-headless-shell", cmd: node, args: (o) => [join(here, "chrome", "cold.mjs"), DATA, o, "shell"] },
};
const warm = {
  wf: { label: "WebFluent Rust API (`Template::render_pdf`)", cmd: RUST_BENCH, args: (o) => [TEMPLATE, DATA, o, WARM_RENDERS, WARMUP] },
  chrome: { label: "Puppeteer + Chrome, one tab reused", cmd: node, args: (o) => [join(here, "chrome", "warm.mjs"), DATA, o, WARM_RENDERS, WARMUP, "chrome", "reuse"] },
  shell: { label: "Puppeteer + chrome-headless-shell, one tab reused", cmd: node, args: (o) => [join(here, "chrome", "warm.mjs"), DATA, o, WARM_RENDERS, WARMUP, "shell", "reuse"] },
  chromeNew: { label: "Puppeteer + Chrome, a new tab per render", cmd: node, args: (o) => [join(here, "chrome", "warm.mjs"), DATA, o, WARM_RENDERS, WARMUP, "chrome", "new"] },
};

const results = { cold: {}, warm: {}, memory: { cold: {}, warm: {} }, output: {}, sizes: {}, machine: {}, versions: {} };
const loadavg = () => os.loadavg().map((x) => +x.toFixed(2));
results.machine.load_before = loadavg();

// Cold: a fresh process per render. One priming run each (so every side
// starts with its files in the OS page cache), then the runs interleaved —
// wf, Chrome, shell, wf, … — so a busy moment on the machine lands on all
// three alike.
log(`cold renders: ${COLD_RUNS} each, interleaved`);
const coldTimes = Object.fromEntries(Object.keys(cold).map((k) => [k, []]));
for (const [k, s] of Object.entries(cold)) spawnSync(s.cmd, s.args(join(out, `cold-${k}.pdf`)), { stdio: "ignore" });
for (let i = 0; i < COLD_RUNS; i++) {
  for (const [k, s] of Object.entries(cold)) {
    const t = process.hrtime.bigint();
    const r = spawnSync(s.cmd, s.args(join(out, `cold-${k}.pdf`)), { stdio: "ignore" });
    const ms = Number(process.hrtime.bigint() - t) / 1e6;
    if (r.status !== 0) throw new Error(`${k} failed (${r.status})`);
    coldTimes[k].push(ms);
  }
}
for (const k of Object.keys(cold)) results.cold[k] = { ...stats(coldTimes[k]), times_ms: coldTimes[k].map((x) => +x.toFixed(2)) };

// Warm: one process, the template (or browser) set up once, then many renders.
for (const [k, s] of Object.entries(warm)) {
  log(`warm renders: ${s.label}`);
  const r = spawnSync(s.cmd, s.args(join(out, `warm-${k}.pdf`)).map(String), { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] });
  if (r.status !== 0) throw new Error(`${k} failed (${r.status})`);
  const j = JSON.parse(r.stdout);
  // Where the browser ran from, relative to here: no home directory in the results.
  if (j.executable) j.executable = relative(here, j.executable);
  results.warm[k] = { ...j, ...stats(j.times_ms) };
}

// Memory, in separate runs so the sampler's own work never touches a timing.
log("memory");
for (const [k, s] of Object.entries(cold)) {
  let worst = null;
  for (let i = 0; i < 3; i++) {
    const m = await measureMemory(s.cmd, s.args(join(out, `cold-${k}.pdf`)));
    worst = worst && worst.peak_pss_kb >= m.peak_pss_kb ? worst : m;
  }
  results.memory.cold[k] = worst;
}
for (const [k, s] of Object.entries(warm)) {
  results.memory.warm[k] = await measureMemory(s.cmd, s.args(join(out, `warm-${k}.pdf`)).map(String));
}
results.machine.load_after = loadavg();

// The documents themselves.
const pdfinfo = (f) => {
  const t = sh("pdfinfo", [f]);
  return t ? Number(t.match(/^Pages:\s+(\d+)/m)?.[1]) : null;
};
// Where the pages break: how many line items each page holds (every item
// has one detail line, "Sprint …", "Phase …" or "Change request …").
const itemsPerPage = (f, pages) => {
  if (!pages) return null;
  const counts = [];
  for (let p = 1; p <= pages; p++) {
    const t = sh("pdftotext", ["-f", String(p), "-l", String(p), f, "-"]);
    if (t == null) return null;
    counts.push(t.split("\n").filter((l) => /^(Change request|Sprint|Phase) /.test(l)).length);
  }
  return counts;
};
for (const k of Object.keys(cold)) {
  const f = join(out, `cold-${k}.pdf`);
  const pages = pdfinfo(f);
  results.output[k] = { bytes: statSync(f).size, pages, items_per_page: itemsPerPage(f, pages) };
}

// ── What each side installs ────────────────────────────────────────────

function du(path) {
  let total = 0;
  const st = lstatSync(path);
  if (st.isSymbolicLink()) return 0;
  if (!st.isDirectory()) return st.size;
  for (const e of readdirSync(path)) total += du(join(path, e));
  return total;
}

const wfVersion = sh(WF, ["--version"])?.split(" ").pop();
const pkg = (p) => JSON.parse(readFileSync(join(here, "node_modules", p, "package.json"), "utf8"));
results.versions = {
  wf: wfVersion,
  wf_commit: sh("git", ["-C", repo, "rev-parse", "--short", "HEAD"]),
  rustc: sh("rustc", ["--version"]),
  node: process.version,
  puppeteer: pkg("puppeteer").version,
  chrome: results.warm.chrome.browser,
  chrome_headless_shell: results.warm.shell.browser,
};

const cache = join(here, ".cache", "puppeteer");
const browserDir = (name) => {
  const d = join(cache, name);
  return existsSync(d) ? join(d, readdirSync(d)[0]) : null;
};
const chromeDir = browserDir("chrome");
const shellDir = browserDir("chrome-headless-shell");
const cftVersion = chromeDir?.split("-").pop();
results.sizes.puppeteer = {
  node_modules_bytes: du(join(here, "node_modules")),
  chrome_bytes: chromeDir ? du(chromeDir) : null,
  headless_shell_bytes: shellDir ? du(shellDir) : null,
  cft_version: cftVersion,
};

async function contentLength(url) {
  try {
    const r = await fetch(url, { method: "HEAD" });
    return r.ok ? Number(r.headers.get("content-length")) : null;
  } catch {
    return null;
  }
}

if (!OFFLINE) {
  log("download sizes");
  const base = `https://storage.googleapis.com/chrome-for-testing-public/${cftVersion}/linux64`;
  results.sizes.puppeteer.chrome_zip_bytes = await contentLength(`${base}/chrome-linux64.zip`);
  results.sizes.puppeteer.headless_shell_zip_bytes = await contentLength(`${base}/chrome-headless-shell-linux64.zip`);
  // npm's own downloads: every package's packed tarball, as the lock file names them.
  const lock = JSON.parse(readFileSync(join(here, "package-lock.json"), "utf8"));
  let npmBytes = 0;
  for (const [p, info] of Object.entries(lock.packages ?? {})) {
    if (!p || !info.resolved) continue;
    // The registry sends no Content-Length for a HEAD, so fetch each one.
    try {
      const res = await fetch(info.resolved);
      if (res.ok) npmBytes += (await res.arrayBuffer()).byteLength;
    } catch {}
  }
  results.sizes.puppeteer.npm_packed_bytes = npmBytes;

  // The `wf` release a Linux user downloads, for the version measured.
  const asset = `wf-v${wfVersion}-x86_64-linux.tar.gz`;
  try {
    const rel = await (await fetch(`https://api.github.com/repos/monzeromer-lab/WebFluent/releases/tags/v${wfVersion}`)).json();
    const a = rel.assets?.find((x) => x.name === asset);
    if (a) {
      const dir = join(here, ".cache", "wf-release", wfVersion);
      const tgz = join(dir, asset);
      if (!existsSync(tgz)) {
        mkdirSync(dir, { recursive: true });
        writeFileSync(tgz, Buffer.from(await (await fetch(a.browser_download_url)).arrayBuffer()));
      }
      const unpacked = join(dir, "unpacked");
      rmSync(unpacked, { recursive: true, force: true });
      mkdirSync(unpacked);
      execFileSync("tar", ["xzf", tgz, "-C", unpacked]);
      results.sizes.wf = { asset, tarball_bytes: a.size, unpacked_bytes: du(unpacked) };
    }
  } catch (e) {
    log(`could not read the wf release: ${e.message}`);
  }
}
results.sizes.wf ??= { asset: null, tarball_bytes: null, unpacked_bytes: null };

// ── The machine ────────────────────────────────────────────────────────

const osRelease = (() => {
  try {
    return readFileSync("/etc/os-release", "utf8").match(/^PRETTY_NAME="?(.*?)"?$/m)?.[1];
  } catch {
    return null;
  }
})();
Object.assign(results.machine, {
  cpu: os.cpus()[0]?.model.trim(),
  logical_cpus: os.cpus().length,
  ram_bytes: os.totalmem(),
  os: osRelease,
  kernel: os.release(),
  date: new Date().toLocaleDateString("en-CA"),
});
results.settings = { cold_runs: COLD_RUNS, warm_renders: WARM_RENDERS, warmup: WARMUP };

// A side-by-side picture of page 1 (WebFluent left, Chrome right), when
// poppler and ImageMagick are installed.
{
  try {
    for (const k of ["wf", "chrome"]) execFileSync("pdftoppm", ["-r", "60", "-png", "-f", "1", "-l", "1", "-singlefile", join(out, `cold-${k}.pdf`), join(out, `page1-${k}`)]);
    execFileSync("magick", [join(out, "page1-wf.png"), join(out, "page1-chrome.png"), "-bordercolor", "#999", "-border", "1", "+append", join(here, "side-by-side.png")]);
  } catch {}
}

writeFileSync(join(here, "results.json"), JSON.stringify(results, null, 2) + "\n");
writeFileSync(join(here, "RESULTS.md"), report(results));
log("wrote bench/pdf/RESULTS.md and bench/pdf/results.json");

// ── The report ─────────────────────────────────────────────────────────

function report(r) {
  const ms = (x) => (x >= 100 ? x.toFixed(0) : x.toFixed(1)) + " ms";
  const mib = (b) => (b == null ? "n/a" : `${(b / 1048576).toFixed(1)} MiB`);
  const mibKb = (k) => mib(k * 1024);
  const kib = (b) => `${(b / 1024).toFixed(0)} KiB`;
  // A ratio rounded down, so a quoted "N×" is never more than was measured.
  const times = (a, b) => `${(Math.floor((a / b) * 10) / 10).toFixed(1)}×`;
  const c = r.cold;
  const w = r.warm;
  const mc = r.memory.cold;
  const mw = r.memory.warm;
  const p = r.sizes.puppeteer;
  const wfs = r.sizes.wf;

  // The fastest and smallest Chrome configuration is what WebFluent is quoted against.
  const bestCold = c.chrome.median <= c.shell.median ? "chrome" : "shell";
  const bestWarm = ["chrome", "shell", "chromeNew"].reduce((a, b) => (w[a].median <= w[b].median ? a : b));
  const leanColdMem = mc.chrome.peak_pss_kb <= mc.shell.peak_pss_kb ? "chrome" : "shell";
  const leanWarmMem = ["chrome", "shell", "chromeNew"].reduce((a, b) => (mw[a].peak_pss_kb <= mw[b].peak_pss_kb ? a : b));
  const smallestChrome = Math.min(...[p.chrome_zip_bytes, p.headless_shell_zip_bytes].filter((x) => x));
  const puppeteerDownload = (p.chrome_zip_bytes ?? 0) + (p.headless_shell_zip_bytes ?? 0) + (p.npm_packed_bytes ?? 0);
  const puppeteerDisk = p.node_modules_bytes + (p.chrome_bytes ?? 0) + (p.headless_shell_bytes ?? 0);

  const coldRow = (k) =>
    `| ${cold[k].label} | ${ms(c[k].median)} | ${ms(c[k].min)} – ${ms(c[k].max)} | ${mibKb(mc[k].peak_pss_kb)} | ${mibKb(mc[k].peak_rss_kb)} |`;
  const warmRow = (k) =>
    `| ${warm[k].label} | ${(w[k].total_ms / 1000).toFixed(2)} s | ${ms(w[k].median)} | ${ms(w[k].p95)} | ${ms(w[k].setup_ms)} | ${mibKb(mw[k].peak_pss_kb)} | ${mibKb(mw[k].peak_rss_kb)} |`;

  return `# PDF benchmark: WebFluent vs. Puppeteer (headless Chrome)

<!-- Generated by bench/pdf/run.mjs (\`just bench-pdf\`). Do not edit by hand: re-run it. -->

One invoice, designed twice to look the same — once as a WebFluent template
([\`invoice.wf\`](invoice.wf)), once as the HTML and CSS a Puppeteer pipeline
would print ([\`chrome/template.mjs\`](chrome/template.mjs),
[\`chrome/invoice.css\`](chrome/invoice.css)) — rendered from the same data
([\`invoice.json\`](invoice.json), 46 line items, ${r.output.wf.pages ?? "?"} Letter pages)
with the same two font files ([\`fonts/\`](fonts): Manrope and JetBrains Mono,
both variable TrueType). WebFluent on the left, Chrome on the right:

![Page 1, WebFluent left, Chrome right](side-by-side.png)

Measured on ${r.machine.date}. Every number below comes from one run of
\`just bench-pdf\` on the machine described; raw timings are in
[\`results.json\`](results.json).

## Results

### Install

| | Download | On disk |
|---|---:|---:|
| \`wf\` ${r.versions.wf} (\`${wfs.asset ?? "release tarball"}\`) | ${mib(wfs.tarball_bytes)} | ${mib(wfs.unpacked_bytes)} |
| \`npm install puppeteer@${r.versions.puppeteer}\` (default: npm packages + Chrome for Testing + chrome-headless-shell ${p.cft_version}) | ${mib(puppeteerDownload || null)} | ${mib(puppeteerDisk)} |
| — of which Chrome for Testing | ${mib(p.chrome_zip_bytes)} | ${mib(p.chrome_bytes)} |
| — of which chrome-headless-shell | ${mib(p.headless_shell_zip_bytes)} | ${mib(p.headless_shell_bytes)} |
| — of which npm packages (puppeteer, qrcode and their dependencies) | ${mib(p.npm_packed_bytes)} | ${mib(p.node_modules_bytes)} |

### One cold render (a new process: start → PDF written → exit)

${r.settings.cold_runs} runs each, interleaved, after one priming run each.

| | Median | Range | Peak memory (PSS) | Peak memory (RSS) |
|---|---:|---:|---:|---:|
${["wf", "chrome", "shell"].map(coldRow).join("\n")}

### ${r.settings.warm_renders} warm renders (one long-lived process)

The template (WebFluent) or the browser (Puppeteer) is set up once — the
"setup" column, not counted in the renders — then ${r.settings.warmup} unmeasured
renders, then ${r.settings.warm_renders} measured ones, one after another.

| | Total | Median | p95 | Setup | Peak memory (PSS) | Peak memory (RSS) |
|---|---:|---:|---:|---:|---:|---:|
${["wf", "chrome", "shell", "chromeNew"].map(warmRow).join("\n")}

### The documents

| | Size | Pages | Line items on each page |
|---|---:|---:|---:|
${[["wf", "WebFluent"], ["chrome", "Chrome"], ["shell", "chrome-headless-shell"]]
  .map(([k, name]) => `| ${name} | ${kib(r.output[k].bytes)} | ${r.output[k].pages ?? "?"} | ${r.output[k].items_per_page?.join(" / ") ?? "?"} |`)
  .join("\n")}

## Numbers you may quote

Each is rounded in Chrome's favour and set against the *fastest* or
*smallest* Chrome configuration measured, not the default one. Say "on this
invoice, on this machine".

- **Install:** the \`wf\` Linux download is ${mib(wfs.tarball_bytes)}; a default \`npm install puppeteer\` downloads ${mib(puppeteerDownload || null)} (${times(smallestChrome, wfs.tarball_bytes)} more than \`wf\` even counting only its smaller browser, chrome-headless-shell).
- **Cold render:** \`wf render\` produced the invoice PDF in a median ${ms(c.wf.median)} from process start, against ${ms(c[bestCold].median)} for a fresh Node process launching ${bestCold === "shell" ? "chrome-headless-shell" : "Chrome"} through Puppeteer — ${times(c[bestCold].median, c.wf.median)} faster.
- **Warm renders:** rendering the same invoice ${r.settings.warm_renders} times in-process took ${(w.wf.total_ms / 1000).toFixed(2)} s with WebFluent's Rust API (median ${ms(w.wf.median)} each) and ${(w[bestWarm].total_ms / 1000).toFixed(2)} s with Puppeteer reusing one browser (median ${ms(w[bestWarm].median)}) — ${times(w[bestWarm].median, w.wf.median)} faster per render.
- **Memory:** a cold \`wf render\` peaked at ${mibKb(mc.wf.peak_pss_kb)}; Puppeteer and its browser processes together at ${mibKb(mc[leanColdMem].peak_pss_kb)} (PSS). Warm, ${mibKb(mw.wf.peak_pss_kb)} against ${mibKb(mw[leanWarmMem].peak_pss_kb)}.
- **Output:** the same ${r.output.wf.pages}-page invoice is ${kib(r.output.wf.bytes)} from WebFluent and ${kib(Math.min(r.output.chrome.bytes, r.output.shell.bytes))} from Chrome (see the caveat on fonts below).

Don't quote any ratio as true of PDFs in general: it is one document, and a
different one would move it. Don't quote the cold numbers without "cold",
or the warm ones without "warm".

## Method

- **Same document.** Both sides lay out the same data with the same fonts,
  the same colours, sizes and spacing; each is styled explicitly so neither
  leans on its engine's defaults. The WebFluent template uses \`Document\`,
  \`Footer\`, \`Watermark\` and \`QrCode\`; the HTML uses the CSS equivalents
  Chrome prints (\`@page\` margin boxes with \`counter(page)\`/\`counter(pages)\`,
  a \`position: fixed\` watermark, and the \`qrcode\` npm package at the same
  error-correction level). The table header repeats on every page on both.
- **Same work per render.** Each render starts from parsed data and ends
  with the PDF bytes in memory (cold: written to a file). WebFluent's render
  includes evaluating the template, reading and subsetting the fonts, layout
  and writing the PDF. Puppeteer's includes building the HTML string
  (template literal, number formatting, the QR code), \`page.setContent\`
  (\`waitUntil: "load"\`) and \`page.pdf()\`; the fonts are inlined in the
  stylesheet as data URIs, read from disk once per process.
- **Cold** is a fresh process each time (\`wf render …\` / \`node chrome/cold.mjs\`),
  timed from spawn to exit by the runner, after one priming run so every
  side's files are in the OS page cache. The very first launch of Chrome
  after a reboot or install is much slower than this (seconds) and is not
  measured.
- **Warm** follows the usual advice for Puppeteer on a server: launch the
  browser once and keep it. Both tab strategies are measured. WebFluent's
  side is a small Rust program ([\`rust/\`](rust)) that builds a
  \`Template\` once and calls \`render_pdf\` in a loop, with the release profile
  \`wf\` itself is built with.
- **Memory** is measured in separate runs from the timings: the runner
  sums the RSS and the PSS of every process the command starts, every 10 ms,
  and keeps the peak (for a one-process command, GNU time's exact max RSS
  when it is higher). PSS splits a page several processes share among them,
  so Chrome's shared libraries count once; RSS counts them per process and
  overstates Chrome. Cold: the highest of three runs.
- **Sizes.** \`wf\`: the GitHub release asset for the version measured, and
  its unpacked contents. Puppeteer: the Chrome for Testing archives it
  downloads, the packed npm tarballs its lock file names, and what all of
  it takes on disk afterwards.
- The \`wf\` measured is built from this checkout (\`${r.versions.wf_commit}\`) with
  \`cargo build --release\`, the same code the release binary is built from.

## Machine

| | |
|---|---|
| CPU | ${r.machine.cpu}, ${r.machine.logical_cpus} logical CPUs |
| RAM | ${(r.machine.ram_bytes / 1073741824).toFixed(1)} GiB |
| OS | ${r.machine.os}, Linux ${r.machine.kernel} |
| Load average (1/5/15 min) | before: ${r.machine.load_before.join(" / ")}; after: ${r.machine.load_after.join(" / ")} |
| wf | ${r.versions.wf} (${r.versions.wf_commit}), ${r.versions.rustc} |
| Node | ${r.versions.node} |
| Puppeteer | ${r.versions.puppeteer} |
| Chrome | ${r.versions.chrome} (Chrome for Testing), ${r.versions.chrome_headless_shell} (chrome-headless-shell) |

The machine was not otherwise idle: it is a desktop with a browser and
editors open. The load average shows how busy it was; a quieter machine
will give lower numbers on both sides.

## Caveats

- **One document.** An invoice: text, a long table, flex and grid boxes,
  borders, an SVG logo, a QR code. Documents heavy in pictures, or in CSS
  WebFluent's engine does not implement, would move the numbers, in either
  direction.
- **Chrome does more.** It is a complete browser engine: every CSS feature,
  JavaScript on the page, web fonts from anywhere, canvas, and years of
  print-path fixes. WebFluent's paged engine implements the CSS its
  documents need (block, flex, grid, tables, the box model, the cascade)
  and is much younger: it has had far less real-world exposure than Chrome's
  print path.
- **Not pixel-identical.** The two documents are as close as the two engines
  allow; see the picture above. They break pages in the same places (see
  "The documents") and wrap the same lines; small differences remain in
  the auto-sized table's column widths and a point or two of vertical
  spacing.
- **The file sizes are partly about fonts.** Chrome writes these variable
  fonts as Type 3 fonts (outlines drawn per glyph, as \`pdffonts\` shows);
  WebFluent embeds them as subset TrueType (CID) fonts. In a one-off side
  test with static font files (Liberation, on the Chrome side only) Chrome's
  file came out at about 200 KiB: fonts are part of the difference, not all
  of it.
- **Puppeteer adds a layer.** The Chrome numbers include Node and the
  DevTools protocol round trips, as every Puppeteer user's do. A pipeline
  that talks to Chrome directly, or keeps a pool of browsers, could do
  better; one that opens a new tab per render (also measured) does worse.
- **Parallelism is not measured.** Every render here runs one at a time.
  A server rendering many documents at once would use several threads with
  WebFluent (a \`Template\` is \`Send + Sync\`) or several tabs or browsers
  with Puppeteer.
- **WebFluent re-reads its font files on every render** (from the OS page
  cache), so its warm numbers include that work.

## Reproduce

\`\`\`sh
just bench-pdf
\`\`\`

Needs Rust (stable), Node ${">="}22 and npm, and network access for the first
run (npm and Puppeteer's browser download, ~${mib(puppeteerDownload || null)}).
Optional: \`pdfinfo\`/\`pdftoppm\` (poppler) for the page counts and the
picture, ImageMagick for the picture. Set \`BENCH_COLD_RUNS\`,
\`BENCH_WARM_RENDERS\` or \`BENCH_WARMUP\` to change the counts.
`;
}
