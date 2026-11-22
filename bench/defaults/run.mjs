#!/usr/bin/env node
// What each framework's starter gets right with nothing configured: the starters
// `bench/toolchain/run.mjs` made (run that first), built for production and served
// by the framework's own production server, then read by Lighthouse (mobile, the
// median of RUNS) and checked for what a search engine, a link preview, a screen
// reader and a browser's security model look for.
//
//   node bench/defaults/run.mjs [--only=webfluent,astro] [--runs=3]
//
// `webfluent-site` and `astro-site` are the same starters with the one setting
// that names the site's address; the first is what bench/report.mjs holds
// capabilities/webfluent.json's "setting" items to. `--dump=DIR` saves what each
// server sent (the page, its headers, robots.txt, sitemap.xml), to check the
// checks by hand.
//
// Writes bench/defaults/results.json. Needs `npm install` in bench/defaults
// (Lighthouse) and a Chrome (CHROME_PATH, else /usr/bin/google-chrome).

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import puppeteer from 'puppeteer-core';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const WORK = path.join(os.homedir(), '.cache/webfluent-bench/toolchain');
const args = Object.fromEntries(process.argv.slice(2).map((a) => a.replace(/^--/, '').split('=')));
const RUNS = Number(args.runs ?? 3);
const WF = path.resolve(args.wf ?? path.join(HERE, '../../target/release/wf'));
const CHROME = process.env.CHROME_PATH || '/usr/bin/google-chrome';
const LH = path.join(HERE, 'node_modules/.bin/lighthouse');
const log = (...a) => console.log(`[${new Date().toISOString().slice(11, 19)}]`, ...a);

// Each starter's production server, as its docs start it.
const SERVE = {
  webfluent: { label: 'WebFluent', cmd: [WF, ['start', '--port', '4100']], port: 4100 },
  // `wf init -t static`: the starter for a site, pre-rendered. Made here when missing,
  // since the toolchain benchmark times the single-page starter.
  'webfluent-static': { label: 'WebFluent (static)', cmd: [WF, ['start', '--port', '4106']], port: 4106,
    make: ['init', 'app', '-t', 'static'] },
  // The same two starters with the one setting that names the site's address,
  // which a canonical link, JSON-LD and a sitemap need: WebFluent's
  // `meta.site_url`, Astro's `site`. Each is a copy of the starter above, the
  // setting changed and built again; nothing else is touched.
  'webfluent-site': { label: 'WebFluent (static, site_url)', cmd: [WF, ['start', '--port', '4107']], port: 4107,
    from: 'webfluent-static',
    setup: (app) => {
      spawnSync(WF, ['config', 'set', 'meta.site_url', 'https://example.com'], { cwd: app, stdio: 'ignore' });
      return spawnSync(WF, ['build'], { cwd: app, stdio: 'ignore' }).status === 0;
    } },
  'astro-site': { label: 'Astro (site)', cmd: ['npx', ['astro', 'preview', '--port', '4108']], port: 4108,
    from: 'astro',
    setup: (app) => {
      const file = path.join(app, 'astro.config.mjs');
      fs.writeFileSync(file, fs.readFileSync(file, 'utf8').replace('defineConfig({})', "defineConfig({ site: 'https://example.com' })"));
      return spawnSync('npm', ['run', 'build'], { cwd: app, stdio: 'ignore' }).status === 0;
    } },
  astro: { label: 'Astro', cmd: ['npx', ['astro', 'preview', '--port', '4101']], port: 4101 },
  next: { label: 'Next.js', cmd: ['npx', ['next', 'start', '-p', '4102']], port: 4102 },
  sveltekit: { label: 'SvelteKit', cmd: ['npx', ['vite', 'preview', '--port', '4103', '--strictPort']], port: 4103 },
  nuxt: { label: 'Nuxt', cmd: ['node', ['.output/server/index.mjs']], port: 4104, env: { PORT: '4104', NITRO_PORT: '4104' } },
  react: { label: 'React + Vite', cmd: ['npx', ['vite', 'preview', '--port', '4105', '--strictPort']], port: 4105 },
};

const median = (xs) => {
  const s = xs.filter((x) => x != null).sort((a, b) => a - b);
  return s.length ? (s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2) : null;
};

function start(key, app) {
  const { cmd: [c, a], port, env } = SERVE[key];
  const child = spawn(c, a, { cwd: app, env: { ...process.env, NODE_ENV: 'production', ...env }, stdio: 'ignore', detached: true });
  const stop = () => { try { process.kill(-child.pid, 'SIGTERM'); } catch {} };
  return new Promise((resolve) => {
    const t = Date.now();
    const poll = async () => {
      if (Date.now() - t > 60_000) { stop(); return resolve(null); }
      try {
        const r = await fetch(`http://127.0.0.1:${port}/`, { signal: AbortSignal.timeout(2000) });
        if (r.status < 500) return resolve({ stop, url: `http://127.0.0.1:${port}/` });
      } catch {}
      setTimeout(poll, 200);
    };
    poll();
  });
}

function lighthouse(url) {
  const out = path.join(os.tmpdir(), `lh-${process.pid}.json`);
  const r = spawnSync(LH, [url, '--quiet', '--output=json', `--output-path=${out}`, '--only-categories=performance,accessibility,best-practices,seo',
    `--chrome-flags=--headless=new --no-sandbox`], { env: { ...process.env, CHROME_PATH: CHROME }, encoding: 'utf8', timeout: 180_000 });
  if (r.status !== 0 || !fs.existsSync(out)) return null;
  const j = JSON.parse(fs.readFileSync(out, 'utf8'));
  fs.rmSync(out);
  const a = j.audits;
  return {
    performance: j.categories.performance.score, accessibility: j.categories.accessibility.score,
    bestPractices: j.categories['best-practices'].score, seo: j.categories.seo.score,
    lcp: a['largest-contentful-paint'].numericValue, fcp: a['first-contentful-paint'].numericValue,
    tbt: a['total-blocking-time'].numericValue, cls: a['cumulative-layout-shift'].numericValue,
    scriptBytes: (a['resource-summary']?.details?.items ?? []).find((i) => i.resourceType === 'script')?.transferSize ?? null,
    bytes: a['total-byte-weight'].numericValue,
  };
}

// Whether what the page shows is in the HTML the server sent, rather than
// drawn by a script once it runs: the body's text before any script (the
// HTML, scripts, styles and <title> taken out) against the text Chrome shows
// once the page has run. Most of it there is a yes, however little text the
// starter has — a page of one heading counts as much as a page of ten.
async function contentInHtml(url, html) {
  const words = (t) => t.replace(/\s+/g, ' ').trim();
  const body = (html.match(/<body[\s\S]*<\/body>/i) ?? [html])[0];
  const before = words(body.replace(/<script[\s\S]*?<\/script>|<style[\s\S]*?<\/style>|<noscript[\s\S]*?<\/noscript>|<[^>]+>/g, ' ')
    .replace(/&amp;/g, '&').replace(/&nbsp;/g, ' '));
  const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ['--no-sandbox'] });
  try {
    const page = await browser.newPage();
    await page.goto(url, { waitUntil: 'networkidle0' });
    const after = words(await page.evaluate(() => document.body.innerText));
    if (!after) return false;
    // Each word the page shows, counted once it is found in the HTML's text.
    const shown = after.split(' ');
    const found = shown.filter((w) => before.includes(w)).length;
    return found / shown.length >= 0.8;
  } finally {
    await browser.close();
  }
}

// What the page and the server say without anyone configuring it.
async function defaults(url) {
  const res = await fetch(url);
  const html = await res.text();
  const h = (n) => res.headers.get(n);
  const has = (re) => re.test(html);
  const get = async (p) => { try { const r = await fetch(new URL(p, url)); return r.ok && !(await r.text()).trimStart().startsWith('<!'); } catch { return false; } };
  return {
    csp: Boolean(h('content-security-policy') || has(/<meta[^>]+http-equiv=["']?content-security-policy/i)),
    frameOptions: Boolean(h('x-frame-options') || /frame-ancestors/.test(h('content-security-policy') ?? '')),
    htmlLang: has(/<html[^>]+lang=["'][a-z]/i),
    title: has(/<title>[^<]+<\/title>/i),
    description: has(/<meta[^>]+name=["']description["'][^>]+content=["'][^"']+/i),
    canonical: has(/<link[^>]+rel=["']canonical["']/i),
    openGraph: has(/<meta[^>]+property=["']og:title["']/i),
    jsonLd: has(/<script[^>]+application\/ld\+json/i),
    skipLink: has(/<a[^>]+href=["']#[^"']+["'][^>]*>[^<]*skip/i),
    sitemap: await get('/sitemap.xml'),
    robots: await get('/robots.txt'),
    contentInHtml: await contentInHtml(url, html),
  };
}

const only = args.only ? args.only.split(',') : Object.keys(SERVE);
const results = { measuredAt: new Date().toISOString(), runs: RUNS, chrome: spawnSync(CHROME, ['--version'], { encoding: 'utf8' }).stdout.trim(), tools: {} };
const file = path.join(HERE, 'results.json');
if (fs.existsSync(file) && args.only) Object.assign(results.tools, JSON.parse(fs.readFileSync(file, 'utf8')).tools);
for (const key of only) {
  const app = path.join(WORK, key, 'app');
  if (SERVE[key].from) {
    // A fresh copy each run, so the setting is the only difference.
    const source = path.join(WORK, SERVE[key].from, 'app');
    if (!fs.existsSync(source)) { log(`${key}: no ${SERVE[key].from} starter to copy`); continue; }
    fs.rmSync(path.dirname(app), { recursive: true, force: true });
    fs.mkdirSync(path.dirname(app), { recursive: true });
    fs.cpSync(source, app, { recursive: true, verbatimSymlinks: true });
    if (!SERVE[key].setup(app)) { results.tools[key] = { label: SERVE[key].label, error: 'the build failed' }; continue; }
  }
  if (SERVE[key].make && !fs.existsSync(app)) {
    fs.mkdirSync(path.dirname(app), { recursive: true });
    spawnSync(WF, SERVE[key].make, { cwd: path.dirname(app), stdio: 'inherit' });
    spawnSync(WF, ['build'], { cwd: app, stdio: 'inherit' });
  }
  if (!fs.existsSync(app)) { log(`${key}: no starter at ${app}; run bench/toolchain/run.mjs first`); continue; }
  log(`${SERVE[key].label}: serving the production build`);
  const server = await start(key, app);
  if (!server) { results.tools[key] = { label: SERVE[key].label, error: 'the production server did not answer' }; continue; }
  try {
    const d = await defaults(server.url);
    if (args.dump) {
      // What each check read, to check the checks by hand.
      const dir = path.join(args.dump, key);
      fs.mkdirSync(dir, { recursive: true });
      const res = await fetch(server.url);
      fs.writeFileSync(path.join(dir, 'index.html'), await res.text());
      fs.writeFileSync(path.join(dir, 'headers.txt'), [...res.headers].map(([k, v]) => `${k}: ${v}`).join('\n'));
      for (const f of ['robots.txt', 'sitemap.xml']) {
        const r = await fetch(new URL(f, server.url));
        fs.writeFileSync(path.join(dir, f), `${r.status} ${r.headers.get('content-type')}\n${(await r.text()).slice(0, 400)}`);
      }
    }
    const runs = [];
    for (let i = 0; i < RUNS; i++) { const r = lighthouse(server.url); if (r) runs.push(r); }
    const lh = Object.fromEntries(Object.keys(runs[0] ?? {}).map((k) => [k, median(runs.map((r) => r[k]))]));
    results.tools[key] = { label: SERVE[key].label, defaults: d, lighthouse: lh, lighthouseRuns: runs.length };
    log(`${SERVE[key].label}: perf ${lh.performance} a11y ${lh.accessibility} bp ${lh.bestPractices} seo ${lh.seo}; ${Object.values(d).filter(Boolean).length}/${Object.keys(d).length} defaults`);
  } finally {
    server.stop();
  }
  fs.writeFileSync(file, JSON.stringify(results, null, 2));
}
log(`wrote ${file}`);
