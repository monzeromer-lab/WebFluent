#!/usr/bin/env node
// The toolchain each framework hands a developer — and an agent, which runs these
// commands on every turn: what installing it puts on the disk, how long an install
// takes, how long a production build takes, how soon the dev server answers, and what
// the starter ships. Each is the framework's own starter, made as its docs say.
//
//   node bench/toolchain/run.mjs [--only=webfluent,astro] [--runs=3] [--wf=PATH]
//
// Writes bench/toolchain/results.json. Projects are made under
// ~/.cache/webfluent-bench/toolchain/, never in this repository.

import { spawn, spawnSync, execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const WORK = path.join(os.homedir(), '.cache/webfluent-bench/toolchain');
const args = Object.fromEntries(process.argv.slice(2).map((a) => a.replace(/^--/, '').split('=')));
const RUNS = Number(args.runs ?? 3);
const WF = path.resolve(args.wf ?? path.join(HERE, '../../target/release/wf'));
const log = (...a) => console.log(`[${new Date().toISOString().slice(11, 19)}]`, ...a);

// How each starter is made, built, served for development, and where its build lands.
const TOOLS = {
  webfluent: {
    label: 'WebFluent',
    create: (dir) => [[WF, ['init', 'app', '-t', 'spa'], dir]],
    install: null, // one binary, nothing to install per project
    build: [WF, ['build']],
    dev: [WF, ['serve'], 3000],
    out: 'build',
    version: () => spawnSync(WF, ['--version'], { encoding: 'utf8' }).stdout.trim(),
  },
  astro: {
    label: 'Astro',
    create: (dir) => [['npm', ['create', 'astro@latest', 'app', '--', '--template', 'minimal', '--no-install', '--no-git', '--skip-houston', '--yes'], dir]],
    install: true,
    build: ['npm', ['run', 'build']],
    dev: ['npx', ['astro', 'dev', '--port', '4321'], 4321],
    out: 'dist',
    version: (app) => pkgVersion(app, 'astro'),
  },
  next: {
    label: 'Next.js',
    create: (dir) => [['npx', ['--yes', 'create-next-app@latest', 'app', '--ts', '--app', '--eslint', '--no-tailwind', '--no-src-dir', '--use-npm', '--skip-install', '--disable-git', '--yes'], dir]],
    install: true,
    build: ['npm', ['run', 'build']],
    dev: ['npx', ['next', 'dev', '--port', '3001'], 3001],
    out: '.next',
    version: (app) => pkgVersion(app, 'next'),
  },
  sveltekit: {
    label: 'SvelteKit',
    create: (dir) => [['npx', ['--yes', 'sv@latest', 'create', 'app', '--template', 'minimal', '--types', 'ts', '--no-add-ons', '--no-install'], dir]],
    install: true,
    build: ['npm', ['run', 'build']],
    dev: ['npx', ['vite', 'dev', '--port', '5174', '--strictPort'], 5174],
    out: '.svelte-kit/output',
    version: (app) => pkgVersion(app, '@sveltejs/kit'),
  },
  nuxt: {
    label: 'Nuxt',
    create: (dir) => [['npx', ['--yes', 'nuxi@latest', 'init', 'app', '--packageManager', 'npm', '--gitInit', 'false', '--no-install', '--template', 'minimal'], dir]],
    install: true,
    build: ['npm', ['run', 'build']],
    dev: ['npx', ['nuxi', 'dev', '--port', '3002'], 3002],
    out: '.output',
    version: (app) => pkgVersion(app, 'nuxt'),
  },
  react: {
    label: 'React + Vite',
    create: (dir) => [['npm', ['create', 'vite@latest', 'app', '--', '--template', 'react-ts', '--no-interactive'], dir]],
    install: true,
    build: ['npm', ['run', 'build']],
    dev: ['npx', ['vite', '--port', '5175', '--strictPort'], 5175],
    out: 'dist',
    version: (app) => pkgVersion(app, 'react'),
  },
};

function pkgVersion(app, name) {
  try {
    return JSON.parse(fs.readFileSync(path.join(app, 'node_modules', name, 'package.json'), 'utf8')).version;
  } catch {
    return null;
  }
}

function run(cmd, argv, cwd, env = {}) {
  const t = performance.now();
  const r = spawnSync(cmd, argv, { cwd, env: { ...process.env, CI: '1', NEXT_TELEMETRY_DISABLED: '1', ASTRO_TELEMETRY_DISABLED: '1', NUXT_TELEMETRY_DISABLED: '1', ...env }, encoding: 'utf8', maxBuffer: 64 << 20, input: '' });
  const ms = performance.now() - t;
  if (r.status !== 0) throw new Error(`${cmd} ${argv.join(' ')} in ${cwd}: exit ${r.status}\n${(r.stderr || r.stdout || '').slice(-1500)}`);
  return ms;
}

const du = (p) => (fs.existsSync(p) ? Number(execSync(`du -sb "${p}"`).toString().split('\t')[0]) : 0);
const count = (dir) => (fs.existsSync(dir) ? Number(execSync(`find "${dir}" -name package.json -not -path "*/node_modules/*/node_modules/*/node_modules/*" | wc -l`).toString().trim()) : 0);
const median = (xs) => {
  const s = [...xs].sort((a, b) => a - b);
  return s.length ? (s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2) : null;
};

// Shipped to a browser by the starter's production build: its .js files (gzipped).
function shippedJs(dir) {
  if (!fs.existsSync(dir)) return null;
  let raw = 0;
  let gz = 0;
  const walk = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) {
        if (['server', 'cache', 'node_modules', 'types', 'standalone'].includes(e.name)) continue;
        walk(p);
      } else if (/\.m?js$/.test(e.name) && !/server|\.map$/.test(p)) {
        const buf = fs.readFileSync(p);
        raw += buf.length;
        gz += Number(execSync('gzip -9 -c | wc -c', { input: buf }).toString().trim());
      }
    }
  };
  walk(dir);
  return { raw, gzip: gz };
}

// Start the dev server and time it until it answers its first page. A port
// that already answers is a server left from an earlier run (a dev server that
// outlived its process group), which would answer at once: that run is refused
// rather than timed.
async function devReady([cmd, argv, port], cwd) {
  try {
    await fetch(`http://127.0.0.1:${port}/`, { signal: AbortSignal.timeout(1000) });
    throw new Error(`port ${port} already answers: stop what is serving it first`);
  } catch (e) {
    if (String(e.message).startsWith('port ')) throw e;
  }
  return new Promise((resolve) => {
    const t = performance.now();
    const child = spawn(cmd, argv, { cwd, env: { ...process.env, CI: '1', BROWSER: 'none', NEXT_TELEMETRY_DISABLED: '1', ASTRO_TELEMETRY_DISABLED: '1', NUXT_TELEMETRY_DISABLED: '1' }, stdio: 'ignore', detached: true });
    const stop = (ms) => {
      try { process.kill(-child.pid, 'SIGTERM'); } catch {}
      setTimeout(() => {
        try { process.kill(-child.pid, 'SIGKILL'); } catch {}
        // Astro's dev server leaves its process group; whatever still holds
        // the port goes too, so the next run starts from nothing.
        spawnSync('fuser', ['-k', '-KILL', `${port}/tcp`], { stdio: 'ignore' });
        setTimeout(() => resolve(ms), 300);
      }, 800);
    };
    const poll = async () => {
      if (performance.now() - t > 120_000) return stop(null);
      try {
        const r = await fetch(`http://127.0.0.1:${port}/`, { signal: AbortSignal.timeout(2000) });
        if (r.status < 500) { await r.arrayBuffer(); return stop(performance.now() - t); }
      } catch {}
      setTimeout(poll, 50);
    };
    poll();
  });
}

async function measure(key, tool) {
  const dir = path.join(WORK, key);
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
  const app = path.join(dir, 'app');
  log(`${tool.label}: creating the starter`);
  for (const [cmd, argv, cwd] of tool.create(dir)) run(cmd, argv, cwd);
  const r = { label: tool.label };
  if (tool.install) {
    // Once to fill the npm cache, then again from it: the install a developer re-runs.
    run('npm', ['install', '--no-audit', '--no-fund'], app);
    const times = [];
    for (let i = 0; i < RUNS; i++) {
      fs.rmSync(path.join(app, 'node_modules'), { recursive: true, force: true });
      times.push(run('npm', ['install', '--no-audit', '--no-fund', '--prefer-offline'], app));
    }
    r.installMs = median(times);
    r.nodeModulesBytes = du(path.join(app, 'node_modules'));
    r.packages = count(path.join(app, 'node_modules'));
  } else {
    r.installMs = 0;
    r.nodeModulesBytes = 0;
    r.packages = 0;
  }
  r.version = tool.version(app);
  const builds = [];
  for (let i = 0; i < RUNS; i++) {
    fs.rmSync(path.join(app, tool.out), { recursive: true, force: true });
    builds.push(run(tool.build[0], tool.build[1], app));
  }
  r.buildMs = median(builds);
  r.shippedJs = shippedJs(path.join(app, tool.out));
  const devs = [];
  for (let i = 0; i < RUNS; i++) devs.push(await devReady(tool.dev, app));
  r.devReadyMs = median(devs.filter((x) => x !== null));
  log(`${tool.label}: install ${Math.round(r.installMs)} ms, ${Math.round(r.nodeModulesBytes / 2 ** 20)} MiB in ${r.packages} packages, build ${Math.round(r.buildMs)} ms, dev ${Math.round(r.devReadyMs)} ms`);
  return r;
}

const only = args.only ? args.only.split(',') : Object.keys(TOOLS);
const results = { measuredAt: new Date().toISOString(), runs: RUNS, machine: { cpu: os.cpus()[0]?.model, cores: os.cpus().length, memoryGiB: Math.round(os.totalmem() / 2 ** 30), os: `${os.type()} ${os.release()}`, node: process.version, npm: spawnSync('npm', ['--version'], { encoding: 'utf8' }).stdout.trim() }, wfBinaryBytes: fs.statSync(WF).size, tools: {} };
const file = path.join(HERE, 'results.json');
if (fs.existsSync(file) && args.only) Object.assign(results.tools, JSON.parse(fs.readFileSync(file, 'utf8')).tools);
for (const key of only) {
  try {
    results.tools[key] = await measure(key, TOOLS[key]);
  } catch (e) {
    results.tools[key] = { label: TOOLS[key].label, error: String(e.message).slice(0, 1200) };
    log(`${TOOLS[key].label}: ${e.message.split('\n')[0]}`);
  }
  fs.writeFileSync(file, JSON.stringify(results, null, 2));
}
log(`wrote ${file}`);
