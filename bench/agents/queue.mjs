#!/usr/bin/env node
// Runs a benchmark batch on a Claude subscription, one agent session at a time, and
// survives the subscription's usage limits.
//
// The agents themselves run in the harness (`.upstream/bench/run.mjs`, Lawson's, patched by
// `setup.sh`): this file decides what runs next, in a shuffled order fixed when the batch is
// planned, keeps that queue on disk, and tells a session that the usage limit stopped (an
// infrastructure loss: set aside, waited out, run again) from one the agent failed (a
// result, counted).
//
//   node queue.mjs plan    --name=B --frameworks=webfluent:5,astro:3 [--runs=2] [--seed=1]
//                          [--model=claude-opus-5-5] [--effort=high] [--claude-version=2.1.310]
//                          [--wf-version=7.1.1] [--engine=workshop|podman|docker]
//   node queue.mjs images  --name=B           build the containers (once per batch)
//   node queue.mjs run     --name=B [--max-sessions=N] [--wait-min=30] [--max-waits=24]
//                          (touch runs/B/PAUSE to stop it before its next session)
//   node queue.mjs measure --name=B [--lighthouse-runs=5]   Lighthouse and lines of code, passing apps
//   node queue.mjs status  --name=B
//
// A batch lives in runs/<name>/: queue.json (the plan and every attempt), events.jsonl (a
// timestamped line per session and per limit hit: how many sessions fit in a usage window is
// read from it), and the harness's own <framework>-<n>/ directories.
//
// Credentials: CLAUDE_CODE_OAUTH_TOKEN from `claude setup-token`. ANTHROPIC_API_KEY is
// refused: it would bill per token.

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import * as ws from './workshop.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
// BENCH_HARNESS stands a stub in for the harness when testing this file.
const HARNESS = process.env.BENCH_HARNESS ?? path.join(HERE, '.upstream/bench/run.mjs');

const [command, ...rest] = process.argv.slice(2);
const opts = Object.fromEntries(
  rest.map((a) => {
    const m = a.match(/^--([a-z-]+)(?:=(.*))?$/);
    if (!m) die(`Unexpected argument: ${a}`);
    return [m[1], m[2] ?? true];
  }),
);

function die(msg) {
  console.error(`error: ${msg}`);
  process.exit(1);
}
const log = (...a) => console.log(`[${new Date().toISOString().slice(0, 19).replace('T', ' ')}]`, ...a);
const num = (k, d) => (opts[k] === undefined ? d : Number(opts[k]));

if (!opts.name) die('--name is required');
const RUN_DIR = path.join(HERE, 'runs', opts.name);
const QUEUE = path.join(RUN_DIR, 'queue.json');
const EVENTS = path.join(RUN_DIR, 'events.jsonl');

const readQueue = () => JSON.parse(fs.readFileSync(QUEUE, 'utf8'));
const writeQueue = (q) => fs.writeFileSync(QUEUE, JSON.stringify(q, null, 2));
const event = (e) => fs.appendFileSync(EVENTS, JSON.stringify({ at: new Date().toISOString(), ...e }) + '\n');

// Where the subscription token may be kept instead of the environment (mode 600).
const TOKEN_FILE = path.join(process.env.HOME ?? '', '.config/webfluent-bench/oauth-token');

// Every child gets the environment without an API key, whatever the shell had, and the
// subscription token from the environment or TOKEN_FILE.
function childEnv() {
  if (process.env.ANTHROPIC_API_KEY) die('ANTHROPIC_API_KEY is set: these runs go on the subscription only. Unset it.');
  const env = { ...process.env };
  if (!env.CLAUDE_CODE_OAUTH_TOKEN && fs.existsSync(TOKEN_FILE)) {
    if ((fs.statSync(TOKEN_FILE).mode & 0o077) !== 0) die(`${TOKEN_FILE} is readable by others: chmod 600 it.`);
    env.CLAUDE_CODE_OAUTH_TOKEN = fs.readFileSync(TOKEN_FILE, 'utf8').trim();
  }
  if (!env.CLAUDE_CODE_OAUTH_TOKEN) die(`No subscription token: run \`claude setup-token\` and put it in CLAUDE_CODE_OAUTH_TOKEN or ${TOKEN_FILE}.`);
  delete env.ANTHROPIC_API_KEY;
  return env;
}

function harness(args, env = process.env) {
  if (!fs.existsSync(HARNESS)) die('The harness is missing: run ./setup.sh first.');
  return new Promise((resolve) => {
    const child = spawn('node', [HARNESS, ...args], { stdio: 'inherit', env });
    child.on('exit', (code) => resolve(code ?? 1));
  });
}

function harnessFlags(cfg) {
  return [
    `--model=${cfg.model}`,
    `--effort=${cfg.effort}`,
    `--claude-version=${cfg.claudeVersion}`,
    `--wf-version=${cfg.wfVersion}`,
    `--engine=${cfg.engine}`,
  ];
}

// A seeded shuffle, so a batch's order is part of its plan and can be shown.
function shuffle(items, seed) {
  let s = seed >>> 0 || 1;
  const rand = () => ((s = (s * 1664525 + 1013904223) >>> 0) / 2 ** 32);
  const a = items.slice();
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

// ---------- What stopped a session ----------

const LIMIT = /usage limit|hit your (?:usage )?limit|limit reached|rate.?limit|resets? (?:at|in)|too many requests|\b429\b|overloaded/i;

// The last `result` event of a transcript, or null.
function lastResult(file) {
  if (!fs.existsSync(file)) return null;
  const lines = fs.readFileSync(file, 'utf8').split('\n').filter(Boolean);
  for (let i = lines.length - 1; i >= 0; i--) {
    try {
      const e = JSON.parse(lines[i]);
      if (e.type === 'result') return e;
    } catch {}
  }
  return null;
}

// "limit": the subscription stopped it (a loss, to run again). "auth": the token was refused
// (stop the batch). "done": the agent ended on its own, passing or not (a result).
function classify(iterDir) {
  const agent = path.join(iterDir, 'agent');
  const result = lastResult(path.join(agent, 'transcript.jsonl'));
  const stderr = fs.existsSync(path.join(agent, 'claude-stderr.log')) ? fs.readFileSync(path.join(agent, 'claude-stderr.log'), 'utf8') : '';
  const text = `${result?.result ?? ''}\n${stderr}`;
  // The harness stopped it at its time limit: the agent's failure, counted as Lawson counts it.
  if (fs.existsSync(path.join(iterDir, 'TIMED_OUT'))) return { kind: 'done', result: result ?? { is_error: true, subtype: 'timed_out' } };
  if (/invalid api key|authentication|unauthori[sz]ed|oauth token|please run \/login|401\b/i.test(text) && (!result || result.is_error)) return { kind: 'auth', text: text.trim().slice(0, 400) };
  if ((!result || result.is_error) && LIMIT.test(text)) return { kind: 'limit', text: text.trim().slice(0, 400), resetAt: resetTime(text) };
  if (!result) return { kind: 'limit-or-crash', text: text.trim().slice(0, 400) };
  return { kind: 'done', result };
}

// "…|1763466000" (seconds since the epoch), as Claude Code has written a limit's reset.
function resetTime(text) {
  const m = text.match(/\|(\d{10})\b/);
  return m ? new Date(Number(m[1]) * 1000).toISOString() : null;
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---------- Commands ----------

async function plan() {
  if (fs.existsSync(QUEUE)) die(`${QUEUE} exists: a batch is planned once. Pick another --name.`);
  // `webfluent:5,astro:3` gives each its own number of runs; a bare name takes --runs.
  const runs = num('runs', 1);
  const frameworks = String(opts.frameworks ?? '').split(',').filter(Boolean).map((f) => {
    const [name, n] = f.split(':');
    return { name, n: n === undefined ? runs : Number(n) };
  });
  if (!frameworks.length || frameworks.some((f) => !(f.n > 0))) die('--frameworks is required, each with a positive number of runs');
  const config = {
    model: opts.model ?? 'claude-opus-5-5',
    effort: opts.effort ?? 'high',
    claudeVersion: opts['claude-version'] ?? '2.1.310',
    wfVersion: opts['wf-version'] ?? '7.1.1',
    engine: opts.engine ?? 'workshop',
    seed: num('seed', Date.now() % 2 ** 31),
  };
  const items = shuffle(
    frameworks.flatMap(({ name, n }) => Array.from({ length: n }, () => ({ framework: name }))),
    config.seed,
  ).map((it, i) => ({ id: i + 1, ...it, status: 'pending', attempts: [] }));
  fs.mkdirSync(RUN_DIR, { recursive: true });
  // The harness appends to a run directory that has a config.json; give it one.
  fs.writeFileSync(path.join(RUN_DIR, 'config.json'), JSON.stringify({ iterations: 0, queue: opts.name, ...config }, null, 2));
  writeQueue({ name: opts.name, plannedAt: new Date().toISOString(), config, items });
  event({ type: 'planned', items: items.length });
  log(`Planned ${items.length} sessions in ${RUN_DIR}:`);
  log(items.map((i) => i.framework).join(' '));
}

async function images() {
  const q = readQueue();
  if (q.config.engine === 'workshop') return ws.setup({ wfVersion: q.config.wfVersion, claudeVersion: q.config.claudeVersion });
  const code = await harness(['--images-only', `--append-to=${RUN_DIR}`, ...harnessFlags(q.config)]);
  if (code) die(`image build failed (exit ${code})`);
}

const iterationDirs = () =>
  new Set(fs.readdirSync(RUN_DIR, { withFileTypes: true }).filter((d) => d.isDirectory() && /-\d+$/.test(d.name)).map((d) => d.name));

async function run() {
  const env = childEnv();
  const maxSessions = num('max-sessions', Infinity);
  const waitMs = num('wait-min', 30) * 60_000;
  const maxWaits = num('max-waits', 24);
  let sessions = 0;
  let waits = 0;
  for (;;) {
    const q = readQueue();
    const item = q.items.find((i) => i.status === 'pending');
    if (!item) return log('Nothing left to run.');
    if (sessions >= maxSessions) return log(`Stopped after ${sessions} session(s) (--max-sessions).`);
    // A file named PAUSE in the batch's folder stops it before the next session starts.
    if (fs.existsSync(path.join(RUN_DIR, 'PAUSE'))) return log(`Paused (${path.join(RUN_DIR, 'PAUSE')}); delete it and run again to go on.`);

    const before = iterationDirs();
    const startedAt = new Date().toISOString();
    log(`Session ${item.id}/${q.items.length}: ${item.framework} (attempt ${item.attempts.length + 1})`);
    event({ type: 'start', id: item.id, framework: item.framework });
    let code;
    if (q.config.engine === 'workshop') {
      // The next free <framework>-<n>, as the harness numbers them.
      let n = 1;
      while (before.has(`${item.framework}-${n}`)) n++;
      const { FRAMEWORKS, prompt } = await import('./.upstream/bench/frameworks.mjs');
      code = await ws.runAgent(path.join(RUN_DIR, `${item.framework}-${n}`), { framework: item.framework, prompt: prompt(FRAMEWORKS[item.framework]), model: q.config.model, effort: q.config.effort }, env);
    } else {
      code = await harness(
        [`--frameworks=${item.framework}`, '--iterations=1', `--append-to=${RUN_DIR}`, '--skip-image-build', '--no-measure', ...harnessFlags(q.config)],
        env,
      );
    }
    const made = [...iterationDirs()].filter((d) => !before.has(d));
    const attempt = { startedAt, endedAt: new Date().toISOString(), harnessExit: code, dir: made[0] ?? null };
    sessions++;

    const verdict = attempt.dir ? classify(path.join(RUN_DIR, attempt.dir)) : { kind: 'limit-or-crash', text: `harness exit ${code}, no iteration directory` };
    attempt.kind = verdict.kind;
    const fresh = readQueue();
    const it = fresh.items.find((i) => i.id === item.id);
    it.attempts.push(attempt);

    if (verdict.kind === 'done') {
      // The harness tests inside its own run; a Workshop session is graded here, and only
      // once it counts.
      if (q.config.engine === 'workshop') {
        const t = await ws.testApp(path.join(RUN_DIR, attempt.dir));
        log(`Tests: ${t === 0 ? 'pass' : `fail (exit ${t})`}`);
      }
      it.status = 'done';
      it.dir = attempt.dir;
      writeQueue(fresh);
      event({ type: 'done', id: item.id, framework: item.framework, dir: attempt.dir, isError: verdict.result.is_error ?? false, costUSD: verdict.result.total_cost_usd ?? null, durationMs: verdict.result.duration_ms ?? null });
      waits = 0;
      continue;
    }

    // Not a result: set the directory aside so it is never counted, and record why.
    if (attempt.dir) {
      const lost = path.join(RUN_DIR, '_lost');
      fs.mkdirSync(lost, { recursive: true });
      const to = path.join(lost, `${attempt.dir}-${Date.now()}`);
      fs.renameSync(path.join(RUN_DIR, attempt.dir), to);
      attempt.dir = path.relative(RUN_DIR, to);
    }
    attempt.reason = verdict.text;
    writeQueue(fresh);
    event({ type: verdict.kind, id: item.id, framework: item.framework, reason: verdict.text?.slice(0, 200), resetAt: verdict.resetAt ?? null });

    if (verdict.kind === 'auth') die(`the subscription token was refused; nothing more will run. ${verdict.text}`);
    if (++waits > maxWaits) die(`${maxWaits} waits in a row without a finished session; stopping. See ${EVENTS}.`);
    const until = verdict.resetAt ? Math.max(new Date(verdict.resetAt) - Date.now() + 60_000, 60_000) : waitMs;
    log(`Not a result (${verdict.kind}); set aside. Waiting ${Math.round(until / 60_000)} min before running it again.`);
    await sleep(until);
  }
}

async function measure() {
  const q = readQueue();
  if (q.config.engine === 'workshop') {
    for (const i of q.items.filter((x) => x.status === 'done')) {
      log(`Measure ${i.dir}`);
      await ws.measure(path.join(RUN_DIR, i.dir), { runs: num('lighthouse-runs', 5) });
    }
    return;
  }
  const code = await harness([`--measure-only=${RUN_DIR}`, '--skip-image-build', ...harnessFlags(q.config)]);
  if (code) die(`measurement failed (exit ${code})`);
}

function status() {
  const q = readQueue();
  const by = {};
  for (const i of q.items) {
    by[i.framework] ??= { pending: 0, done: 0, losses: 0 };
    by[i.framework][i.status]++;
    by[i.framework].losses += i.attempts.filter((a) => a.kind !== 'done').length;
  }
  console.log(`${q.name}: ${q.items.filter((i) => i.status === 'done').length}/${q.items.length} done (${JSON.stringify(q.config)})`);
  for (const [fw, c] of Object.entries(by)) console.log(`  ${fw.padEnd(16)} done ${c.done}  pending ${c.pending}  set aside ${c.losses}`);
}

const commands = { plan, images, run, measure, status };
if (!commands[command]) die(`usage: node queue.mjs ${Object.keys(commands).join('|')} --name=BATCH …`);
await commands[command]();
