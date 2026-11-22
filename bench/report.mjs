#!/usr/bin/env node
// Every benchmark's raw results, gathered into the one file the docs site's
// /benchmarks page and its home page draw: site/src/benchmarks.json.
//
//   node bench/report.mjs
//
// Reads, as each benchmark wrote them:
//   bench/agents/runs/<batch>/results.json and Lawson's published medians
//     (bench/agents/.lawson-results, which bench/agents/setup.sh fetches)
//   bench/toolchain/results.json, bench/defaults/results.json
//   bench/runtime/.upstream/webdriver-ts/results/*.json (js-framework-benchmark)
//   bench/pdf/results.json
// and writes display-ready numbers: the page formats nothing itself, so what
// it shows is what this script printed, and a reader can check one against
// the other. Row labels are translation keys; the page says them.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.join(HERE, '../site/src/benchmarks.json');
const read = (p) => JSON.parse(fs.readFileSync(path.join(HERE, p), 'utf8'));
const has = (p) => fs.existsSync(path.join(HERE, p));
const args = Object.fromEntries(process.argv.slice(2).map((a) => a.replace(/^--/, '').split('=')));
const AGENT_BATCH = args.batch ?? 'tools2-ab';

const ms = (v) => (v == null ? '—' : v >= 1000 ? `${(v / 1000).toFixed(1)} s` : `${Math.round(v)} ms`);
const ms1 = (v) => (v == null ? '—' : `${v.toFixed(1)} ms`);
const kb = (b) => (b == null ? '—' : `${(b / 1024).toFixed(1)} kB`);
const mib = (b) => (b == null ? '—' : b === 0 ? '0' : `${Math.round(b / 1048576)} MiB`);
const score = (s) => (s == null ? '—' : String(Math.round(s * 100)));
const millions = (n) => `${(n / 1e6).toFixed(1)}M`;
const tick = (b) => (b ? '✓' : '—');

/// A table: the frameworks across the top, one row a measure, each cell a
/// string; `best` marks the cells that win their row (lowest unless `high`).
function table(columns, ours, rows) {
  return { columns, ours, rows };
}
function row(key, values, show, { high = false, mark = true } = {}) {
  const nums = values.map((v) => (typeof v === 'number' ? v : null));
  const known = nums.filter((v) => v != null);
  const target = known.length ? (high ? Math.max(...known) : Math.min(...known)) : null;
  return {
    key,
    cells: values.map((v) => (v == null ? '—' : show(v))),
    best: mark ? nums.map((v) => v != null && v === target) : nums.map(() => false),
  };
}

const report = { generatedAt: new Date().toISOString() };

// ── What each tool provides, and Lighthouse on its starter ─────────────────
const CHECKS = ['htmlLang', 'title', 'description', 'contentInHtml', 'openGraph', 'skipLink', 'csp', 'frameOptions', 'robots', 'sitemap', 'canonical', 'jsonLd'];
// What each framework gives, by its own docs, with no third-party package:
// bench/defaults/capabilities/<tool>.json, one level an item, each with the
// page that says so. The levels, from most to least done for you.
const LEVELS = ['default', 'setting', 'api', 'official', 'manual'];
const CAP_ORDER = ['webfluent', 'astro', 'next', 'sveltekit', 'nuxt', 'react'];
if (has('defaults/capabilities')) {
  const caps = CAP_ORDER.filter((k) => has(`defaults/capabilities/${k}.json`)).map((k) => read(`defaults/capabilities/${k}.json`));
  for (const c of caps) for (const k of CHECKS) {
    const it = c.items[k];
    if (!it || !LEVELS.includes(it.level) || !it.source) throw new Error(`${c.framework}: ${k} has no level or source`);
  }
  report.capabilities = {
    columns: caps.map((c) => c.framework),
    ours: 0,
    rows: CHECKS.map((k) => ({
      key: `bench.check.${k}`,
      cells: caps.map((c) => ({ level: c.items[k].level, how: c.items[k].how, source: c.items[k].source })),
    })),
    // Built in, on or by one setting, out of twelve.
    counts: caps.map((c) => CHECKS.filter((k) => ['default', 'setting'].includes(c.items[k].level)).length),
    checked: caps.map((c) => c.checked).sort().at(-1),
  };
  const ours = caps[0];
  // WebFluent's own claims, held to what the defaults benchmark measured:
  // what it says is built in, its static starter sends untouched; what it
  // says takes one setting, the same starter sends with `meta.site_url` set.
  if (has('defaults/results.json')) {
    const m = read('defaults/results.json').tools;
    for (const k of CHECKS) {
      const level = ours.items[k].level;
      const seen = level === 'default' ? m['webfluent-static']?.defaults?.[k] : level === 'setting' ? m['webfluent-site']?.defaults?.[k] : true;
      if (seen === false) throw new Error(`capabilities/webfluent.json says ${k} is "${level}", but the measured starter does not send it`);
    }
  }
  const on = (c, levels) => CHECKS.filter((k) => levels.includes(c.items[k].level)).length;
  report.headline = {
    ...(report.headline ?? {}),
    defaults: `${on(ours, ['default', 'setting'])} / ${CHECKS.length}`,
    defaultsUntouched: `${on(ours, ['default'])} / ${CHECKS.length}`,
    defaultsBest: `${Math.max(...caps.slice(1).map((c) => on(c, ['default', 'setting'])))} / ${CHECKS.length}`,
  };
}

if (has('defaults/results.json')) {
  const d = read('defaults/results.json');
  // Lighthouse on each starter as its own command makes it.
  const keys = ['webfluent-static', 'webfluent', 'astro', 'next', 'sveltekit', 'nuxt', 'react'].filter((k) => d.tools[k] && !d.tools[k].error);
  const tools = keys.map((k) => d.tools[k]);
  const SHORT = { 'webfluent-static': 'WebFluent (static)', webfluent: 'WebFluent (SPA)' };
  for (const [i, k] of keys.entries()) if (SHORT[k]) tools[i] = { ...tools[i], label: SHORT[k] };
  const lh = (k) => tools.map((t) => t.lighthouse?.[k] ?? null);
  report.lighthouse = table(tools.map((t) => t.label), 0, [
    row('bench.lh.performance', lh('performance'), score, { high: true }),
    row('bench.lh.accessibility', lh('accessibility'), score, { high: true }),
    row('bench.lh.bestPractices', lh('bestPractices'), score, { high: true }),
    row('bench.lh.seo', lh('seo'), score, { high: true }),
    row('bench.lh.lcp', lh('lcp'), ms),
    row('bench.lh.scriptBytes', lh('scriptBytes'), kb),
  ]);
  report.defaultsMeta = { measuredAt: d.measuredAt.slice(0, 10), runs: d.runs, chrome: d.chrome };
  const ours = d.tools['webfluent-static'] ?? d.tools.webfluent;
  report.headline = {
    ...(report.headline ?? {}),
    // The four categories, as one figure when they agree.
    lighthouse: [...new Set(['performance', 'accessibility', 'bestPractices', 'seo'].map((k) => score(ours.lighthouse[k])))].join(' / '),
  };
}

// ── Toolchain: from nothing to a running starter ───────────────────────────
if (has('toolchain/results.json')) {
  const t = read('toolchain/results.json');
  const keys = ['webfluent', 'astro', 'next', 'sveltekit', 'nuxt', 'react'].filter((k) => t.tools[k] && !t.tools[k].error);
  const tools = keys.map((k) => t.tools[k]);
  const col = (k) => tools.map((x) => x[k] ?? null);
  report.toolchain = table(tools.map((x) => x.label), 0, [
    row('bench.tc.installMs', col('installMs'), ms),
    row('bench.tc.packages', col('packages'), String),
    row('bench.tc.nodeModules', col('nodeModulesBytes'), mib),
    row('bench.tc.buildMs', col('buildMs'), ms),
    row('bench.tc.devReadyMs', col('devReadyMs'), ms),
    row('bench.tc.shippedJs', tools.map((x) => x.shippedJs?.gzip ?? null), kb),
  ]);
  report.toolchainMeta = {
    measuredAt: t.measuredAt.slice(0, 10), runs: t.runs,
    machine: `${t.machine.cpu}, ${t.machine.memoryGiB} GiB, Node ${t.machine.node.replace(/^v/, '')}`,
    versions: tools.map((x) => `${x.label} ${String(x.version).replace(/^wf /, '')}`).join(', '),
    binary: mib(t.wfBinaryBytes),
  };
  const wf = t.tools.webfluent;
  const others = tools.filter((x) => x !== wf);
  report.headline = {
    ...(report.headline ?? {}),
    build: ms(wf.buildMs),
    buildOthers: `${ms(Math.min(...others.map((x) => x.buildMs)))}–${ms(Math.max(...others.map((x) => x.buildMs)))}`,
    nodeModulesOthers: `${mib(Math.min(...others.map((x) => x.nodeModulesBytes)))}–${mib(Math.max(...others.map((x) => x.nodeModulesBytes)))}`,
  };
}

// ── Runtime: js-framework-benchmark, keyed ─────────────────────────────────
// Copied out of the upstream checkout after a run, so the checkout can go.
const RT = 'runtime/results';
if (has(RT)) {
  const by = {};
  for (const f of fs.readdirSync(path.join(HERE, RT))) {
    if (!f.endsWith('.json')) continue;
    const j = read(`${RT}/${f}`);
    const fw = j.framework.replace(/-keyed$/, '');
    const v = j.type === 'cpu' ? j.values.total.median : (j.values.DEFAULT ?? j.values.total ?? Object.values(j.values)[0]).median;
    (by[fw] ??= {})[j.benchmark] = v;
  }
  const order = ['webfluent', 'vanillajs', 'solid', 'svelte', 'vue', 'preact-hooks', 'react-hooks'];
  const fws = order.map((o) => Object.keys(by).find((k) => k === o || k.startsWith(`${o}-v`))).filter(Boolean);
  const name = (k) => {
    const [n, v] = k.split('-v');
    const label = { webfluent: 'WebFluent', vanillajs: 'Vanilla JS', solid: 'Solid', svelte: 'Svelte', vue: 'Vue', 'preact-hooks': 'Preact', 'react-hooks': 'React' }[n] ?? n;
    return v ? `${label} ${v}` : label;
  };
  const cpu = ['01_run1k', '02_replace1k', '03_update10th1k_x16', '04_select1k', '05_swap1k', '06_remove-one-1k', '07_create10k', '08_create1k-after1k_x2', '09_clear1k_x8'];
  const geo = (fw) => Math.exp(cpu.reduce((s, b) => s + Math.log(by[fw][b] / by[fws.find((f) => f.startsWith('vanillajs'))][b]), 0) / cpu.length);
  report.runtime = table(fws.map(name), 0, [
    ...cpu.map((b) => row(`bench.rt.${b}`, fws.map((f) => by[f][b] ?? null), ms1)),
    row('bench.rt.geomean', fws.map(geo), (v) => `${v.toFixed(2)}×`),
    row('bench.rt.firstPaint', fws.map((f) => by[f]['43_first-paint'] ?? null), ms1),
    row('bench.rt.size', fws.map((f) => by[f]['42_size-compressed'] ?? null), (v) => `${v.toFixed(1)} kB`),
    row('bench.rt.memory', fws.map((f) => by[f]['22_run-memory'] ?? null), (v) => `${v.toFixed(1)} MB`),
  ]);
  report.headline = { ...(report.headline ?? {}), runtime: `${geo(fws[0]).toFixed(2)}×` };
}

// ── PDF: one invoice, WebFluent against Puppeteer ──────────────────────────
if (has('pdf/results.json')) {
  const p = read('pdf/results.json');
  report.pdf = table(['WebFluent', 'Puppeteer + Chrome', 'Puppeteer + headless shell'], 0, [
    row('bench.pdf.cold', [p.cold.wf.median, p.cold.chrome.median, p.cold.shell?.median ?? null], ms),
    row('bench.pdf.warm', [p.warm.wf.median, p.warm.chrome.median, p.warm.shell?.median ?? null], ms),
    row('bench.pdf.bytes', [p.warm.wf.bytes, p.warm.chrome.bytes, p.warm.shell?.bytes ?? null], kb),
  ]);
  report.headline = { ...(report.headline ?? {}), pdf: `${(p.cold.chrome.median / p.cold.wf.median).toFixed(1)}×` };
}

// ── Agents: what it costs an agent to build the same app ───────────────────
const batch = `agents/runs/${AGENT_BATCH}/results.json`;
const lawsonFile = 'agents/.lawson-results/20261026/summary.json';
if (has(batch) && has(lawsonFile)) {
  const b = read(batch);
  const s = Object.values(b.summary)[0];
  const lawson = read(lawsonFile);
  const LABEL = { next: 'Next.js', nuxt: 'Nuxt', 'tanstack-start': 'TanStack Start', 'react-router': 'React Router', solidstart: 'SolidStart', sveltekit: 'SvelteKit', angular: 'Angular', astro: 'Astro', qwik: 'Qwik', gracile: 'Gracile', 'hono-htmx': 'Hono + htmx', vanilla: 'Vanilla' };
  const rows = [
    { label: `WebFluent ${b.config.wfVersion}`, tokens: s.tokens.total.median, cost: s.costUSD.median, wall: s.wallMin.median, turns: s.turns.median, loc: s.sourceLines.median, passed: `${s.passed}/${s.n}`, ours: true },
    ...lawson.rows.map((r) => ({ label: LABEL[r.framework] ?? r.framework, tokens: r.tokens, cost: r.cost, wall: r.wallMin, turns: r.turns, loc: r.loc, passed: `${r.passed}/${r.n}`, ours: false })),
  ].sort((x, y) => x.tokens - y.tokens);
  const most = Math.max(...rows.map((r) => r.tokens));
  report.agents = rows.map((r) => ({ label: r.label, share: Math.round((r.tokens / most) * 1000) / 10, shown: millions(r.tokens), ours: r.ours }));
  // One row a framework, in the chart's order.
  report.agentsTable = rows.map((r) => ({
    label: r.label, ours: r.ours, tokens: millions(r.tokens), cost: `$${r.cost.toFixed(2)}`,
    wall: `${r.wall.toFixed(1)} min`, turns: String(Math.round(r.turns)), loc: String(Math.round(r.loc)), passed: r.passed,
  }));
  const others = rows.filter((r) => !r.ours).map((r) => r.tokens);
  report.agentsMeta = {
    model: b.config.model, effort: b.config.effort, claudeCode: b.config.claudeVersion,
    runs: s.n, lawsonRuns: lawson.rows[0].n,
    ratio: `${(s.tokens.total.median / Math.max(...others)).toFixed(1)}–${(s.tokens.total.median / Math.min(...others)).toFixed(1)}×`,
  };
  report.headline = { ...(report.headline ?? {}), agentTokens: millions(s.tokens.total.median), agentTokensOthers: `${millions(Math.min(...others))}–${millions(Math.max(...others))}` };
}

// A figure spliced into a sentence (`434 ms–4.5 s`) is its own left-to-right
// run (LRI \u2026 PDI), so an Arabic sentence around it does not reorder its
// parts, `4 / 12` included, which has no letter to say which way it reads.
const isolate = (v) => (typeof v === 'string' && /[0-9]/.test(v) ? `\u2066${v}\u2069` : v);
for (const k of ['defaultsBest', 'defaultsUntouched', 'buildOthers', 'nodeModulesOthers', 'agentTokensOthers']) report.headline[k] = isolate(report.headline[k]);
if (report.agentsMeta) report.agentsMeta.ratio = isolate(report.agentsMeta.ratio);
if (report.toolchainMeta) report.toolchainMeta.machine = isolate(report.toolchainMeta.machine);

fs.writeFileSync(OUT, `${JSON.stringify(report, null, 2)}\n`);
console.log(`wrote ${path.relative(process.cwd(), OUT)}`);
console.log(JSON.stringify(report.headline, null, 2));
