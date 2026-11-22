#!/usr/bin/env node
// Turns a batch (runs/<name>/) into results.json in the schema every benchmark here shares:
// the machine, the versions, one record per session, and per framework the median, the
// interquartile range and a 95% bootstrap interval of each measure, and the pass rate with
// its Wilson interval. Lawson's published medians sit beside ours for the frameworks he ran.
//
//   node summarize.mjs --name=B
//
// Only sessions the queue counted are read: a session set aside (a usage limit, a crash)
// lives under _lost/ and is not a result.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { readRunInfo } from './.upstream/bench/report.mjs';
import { summarizeTranscript } from './.upstream/bench/tokens.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const name = process.argv.find((a) => a.startsWith('--name='))?.slice(7);
if (!name) throw new Error('usage: node summarize.mjs --name=BATCH');
const RUN_DIR = path.join(HERE, 'runs', name);
const queue = JSON.parse(fs.readFileSync(path.join(RUN_DIR, 'queue.json'), 'utf8'));

// ---------- Statistics ----------

const sorted = (xs) => xs.filter((x) => typeof x === 'number' && Number.isFinite(x)).sort((a, b) => a - b);
function quantile(s, q) {
  if (!s.length) return null;
  const i = (s.length - 1) * q;
  const lo = Math.floor(i);
  return s[lo] + (s[Math.min(lo + 1, s.length - 1)] - s[lo]) * (i - lo);
}
// Seeded, so the same results give the same interval every time they are summarised.
function bootstrapMedian(xs, resamples = 10_000, seed = 42) {
  const s = sorted(xs);
  if (s.length < 2) return null;
  let r = seed;
  const rand = () => ((r = (r * 1664525 + 1013904223) >>> 0) / 2 ** 32);
  const meds = new Float64Array(resamples);
  for (let k = 0; k < resamples; k++) {
    const sample = Array.from({ length: s.length }, () => s[Math.floor(rand() * s.length)]).sort((a, b) => a - b);
    meds[k] = quantile(sample, 0.5);
  }
  meds.sort();
  return [quantile(meds, 0.025), quantile(meds, 0.975)];
}
function stat(xs) {
  const s = sorted(xs);
  return { n: s.length, median: quantile(s, 0.5), iqr: s.length ? [quantile(s, 0.25), quantile(s, 0.75)] : null, ci95: bootstrapMedian(s), min: s[0] ?? null, max: s.at(-1) ?? null };
}
function wilson(passed, n, z = 1.96) {
  if (!n) return null;
  const p = passed / n;
  const d = 1 + (z * z) / n;
  const c = (p + (z * z) / (2 * n)) / d;
  const h = (z * Math.sqrt((p * (1 - p)) / n + (z * z) / (4 * n * n))) / d;
  return [Math.max(0, c - h), Math.min(1, c + h)];
}

// ---------- Sessions ----------

// Non-blank lines of the app's own source, as Lawson's measure.mjs counts them (its
// extensions, .wf and .wfx added; tests, generated and vendored files left out), read
// from the agent's repo on this machine: the grader copies app/ without its git folder.
const SOURCE = /\.([cm]?[jt]sx?|vue|svelte|astro|html|css|scss|wf|wfx)$/;
const SKIP = new Set(['node_modules', 'build', 'dist', '.wf-cache', '.git', '.claude', '.agents', 'tests', 'test', '.wf', 'public']);
function sourceLines(dir) {
  if (!fs.existsSync(dir)) return null;
  let lines = 0;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (SKIP.has(entry.name)) continue;
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) lines += sourceLines(p) ?? 0;
    else if (SOURCE.test(entry.name) && !/\.(test|spec)\.[cm]?[jt]sx?$/.test(entry.name)) {
      const text = fs.readFileSync(p, 'utf8');
      if (/automatically generated|auto-generated|@generated|do not edit/i.test(text.slice(0, 500))) continue;
      lines += text.split('\n').filter((l) => l.trim()).length;
    }
  }
  return lines;
}

function session(item) {
  const dir = path.join(RUN_DIR, item.dir);
  const info = readRunInfo(path.join(dir, 'agent/run-info.txt'));
  let agent = null;
  try {
    agent = summarizeTranscript(path.join(dir, 'agent/transcript.jsonl'));
  } catch {}
  const m = fs.existsSync(path.join(dir, 'measure/results.json')) ? JSON.parse(fs.readFileSync(path.join(dir, 'measure/results.json'), 'utf8')) : null;
  const lh = (k) => m?.lighthouse?.['home/mobile']?.median?.[k] ?? null;
  return {
    id: item.id,
    framework: item.framework,
    dir: item.dir,
    attempts: item.attempts.length,
    passed: info.final_test_exit === '0',
    timedOut: fs.existsSync(path.join(dir, 'TIMED_OUT')),
    claudeCode: info.claude_code ?? null,
    tokens: agent?.tokens ?? null,
    costUSD: agent?.costUSD ?? null,
    wallMin: agent ? agent.wallMs / 60_000 : null,
    turns: agent?.turns ?? null,
    toolCalls: agent ? Object.values(agent.toolCalls).reduce((a, b) => a + b, 0) : null,
    lighthouse: m ? { performance: lh('performance'), accessibility: lh('accessibility'), lcp: lh('lcp'), fcp: lh('fcp'), tbt: lh('tbt'), cls: lh('cls'), scriptBytes: lh('scriptBytes'), bytes: lh('bytes'), requests: lh('requests') } : null,
    sourceLines: sourceLines(path.join(dir, 'repo/app')) || m?.code?.sourceLines || null,
    installed: m?.code?.installed ?? null,
  };
}

const sessions = queue.items.filter((i) => i.status === 'done').map(session);

// Lawson's published medians (his harness, his machine, 2026-10-25/26), for the frameworks
// he ran: what our calibration baselines are compared against.
let lawson = {};
try {
  const s = JSON.parse(fs.readFileSync(path.join(HERE, '.lawson-results/20261026/summary.json'), 'utf8'));
  lawson = Object.fromEntries(s.rows.map((r) => [r.framework, r]));
} catch {}

const frameworks = [...new Set(queue.items.map((i) => i.framework))];
const summary = Object.fromEntries(
  frameworks.map((fw) => {
    const ss = sessions.filter((s) => s.framework === fw);
    const pick = (f) => stat(ss.map(f));
    const passed = ss.filter((s) => s.passed).length;
    return [
      fw,
      {
        planned: queue.items.filter((i) => i.framework === fw).length,
        n: ss.length,
        passed,
        passRate: ss.length ? passed / ss.length : null,
        passRateCi95: wilson(passed, ss.length),
        setAside: queue.items.filter((i) => i.framework === fw).reduce((a, i) => a + i.attempts.filter((t) => t.kind !== 'done').length, 0),
        tokens: {
          total: pick((s) => s.tokens?.total),
          input: pick((s) => s.tokens?.input),
          cacheWrite: pick((s) => s.tokens?.cacheWrite),
          cacheRead: pick((s) => s.tokens?.cacheRead),
          output: pick((s) => s.tokens?.output),
        },
        costUSD: pick((s) => s.costUSD),
        wallMin: pick((s) => s.wallMin),
        turns: pick((s) => s.turns),
        toolCalls: pick((s) => s.toolCalls),
        sourceLines: pick((s) => s.sourceLines),
        lighthouse: {
          performance: pick((s) => s.lighthouse?.performance),
          accessibility: pick((s) => s.lighthouse?.accessibility),
          lcp: pick((s) => s.lighthouse?.lcp),
          scriptBytes: pick((s) => s.lighthouse?.scriptBytes),
        },
        lawson: lawson[fw] ?? null,
      },
    ];
  }),
);

const results = {
  benchmark: 'agents/pokedex',
  batch: name,
  summarizedAt: new Date().toISOString(),
  config: queue.config,
  harness: JSON.parse(fs.readFileSync(path.join(HERE, 'upstream.json'), 'utf8')),
  machine: { cpu: os.cpus()[0]?.model, cores: os.cpus().length, memoryGiB: Math.round(os.totalmem() / 2 ** 30), os: `${os.type()} ${os.release()}`, loadavg: os.loadavg() },
  definitions: {
    'tokens.total': 'input + output + cache reads + cache writes, as the harness\'s tokens.mjs adds them',
    costUSD: "Claude Code's own total_cost_usd: list prices, cache pricing included. API-equivalent; the runs were on a subscription",
    passed: "the harness's hidden Playwright tests, run against the app in a fresh container with changed API data",
  },
  summary,
  sessions,
};
fs.writeFileSync(path.join(RUN_DIR, 'results.json'), JSON.stringify(results, null, 2));

// A line per framework, to read at a glance.
const f = (x, d = 0) => (x === null || x === undefined ? '—' : x.toFixed(d));
console.log(`${name}: ${sessions.length} session(s) counted`);
for (const [fw, s] of Object.entries(summary)) {
  const l = s.lawson;
  console.log(
    `  ${fw.padEnd(12)} pass ${s.passed}/${s.n}  tokens ${f((s.tokens.total.median ?? NaN) / 1e6, 2)}M  cost $${f(s.costUSD.median, 2)}  ${f(s.wallMin.median, 1)} min` +
      (l ? `   | Lawson: pass ${l.passed}/${l.n}, ${f(l.tokens / 1e6, 2)}M, $${f(l.cost, 2)}, ${f(l.wallMin, 1)} min` : ''),
  );
}
console.log(`Wrote ${path.join(RUN_DIR, 'results.json')}`);
