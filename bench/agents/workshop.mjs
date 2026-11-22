#!/usr/bin/env node
// Lawson's containers, as Ubuntu Workshops (https://ubuntu.com/workshop): the engine
// `queue.mjs` uses when a batch is planned with --engine=workshop.
//
// Two workshops, both Ubuntu 26.04 LXD containers (the wf release binary needs glibc 2.43,
// which 24.04 lacks), each rolled back to its launch snapshot
// (`workshop restore`) before every use, so every session starts from the same state:
//
//   pokedex-agent   what Lawson's pokedex-agent image holds: Node, Playwright's Chromium, the
//                   exported template with its data, Claude Code, agent-browser, wf
//   pokedex-grader  what his pokedex-base and pokedex-measure hold: the same, without the
//                   agent's tools, plus Caddy and Lighthouse. The tests and the measurement
//                   run here, on the agent's app/ alone
//
// Workshop mounts its project folder read-write at /project in every workshop, so the
// project is a folder of its own, outside this repository (PROJECT below), holding only the
// definitions (the template each workshop copies sits beside its hooks). Between sessions
// it is checked for anything an agent left there, which is removed and recorded.
// Files go in and out as tar streams over `workshop exec`, never through /project.
//
//   node workshop.mjs setup [--wf-version=7.0.1] [--claude-version=2.1.310] [--rebuild]
//   node workshop.mjs check

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const UP = path.join(HERE, '.upstream');
export const PROJECT = process.env.BENCH_WORKSHOP_PROJECT ?? path.join(os.homedir(), '.local/share/webfluent-bench/workshop');
export const AGENT = 'pokedex-agent';
export const GRADER = 'pokedex-grader';
const W = '/home/workshop';
export const WORK = `${W}/work`;
// The agent's own user, as in Lawson's container: `agent`, no sudo, its work in ~/work.
// (Workshop's own user keeps its sudo: `workshop exec` runs through it.)
const A = '/home/agent';
const AWORK = `${A}/work`;

const NODE_VERSION = () => fs.readFileSync(path.join(UP, 'template/.nvmrc'), 'utf8').trim();
const PLAYWRIGHT_VERSION = '1.64.1';
const AGENT_BROWSER_VERSION = '0.39.2';

const log = (...a) => console.log(`[${new Date().toISOString().slice(11, 19)}]`, ...a);

// ---------- Running things in a workshop ----------

// `workshop exec` in the benchmark's project. `env` is a map: a value of `true` passes the
// variable by name, so its value comes from this process's environment and is never on a
// command line. `stdin` and `stdout` may be file paths (for tar streams); `log` appends both
// streams to a file instead.
export function exec(workshop, command, { env = {}, cwd = WORK, uid, timeout, stdin, stdout, log: logFile, childEnv = process.env } = {}) {
  const args = ['exec', '-p', PROJECT, '-I', '-w', cwd];
  if (uid !== undefined) args.push('--uid', String(uid), '--gid', String(uid));
  if (timeout) args.push('--timeout', timeout);
  for (const [k, v] of Object.entries(env)) args.push('--env', v === true ? k : `${k}=${v}`);
  args.push(workshop, '--', ...command);
  return new Promise((resolve) => {
    const out = logFile ? fs.openSync(logFile, 'a') : stdout ? fs.openSync(stdout, 'w') : 'inherit';
    const err = logFile ? out : 'inherit';
    const child = spawn('workshop', args, { stdio: [stdin ? 'pipe' : 'ignore', out, err], env: childEnv });
    if (stdin) fs.createReadStream(stdin).pipe(child.stdin);
    child.on('exit', (code) => {
      if (typeof out === 'number') fs.closeSync(out);
      resolve(code ?? 1);
    });
  });
}

function workshop(args, opts = {}) {
  const r = spawnSync('workshop', [...args, '-p', PROJECT], { encoding: 'utf8', ...opts });
  if (r.status !== 0) throw new Error(`workshop ${args.join(' ')}: ${(r.stderr || r.stdout || '').trim()}`);
  return r.stdout;
}

export function restore(name) {
  const t = Date.now();
  workshop(['restore', name]);
  return Date.now() - t;
}

// Anything in the project folder but the definitions was put there from inside a workshop:
// removed, so no session can read what an earlier one left, and reported.
export function cleanProject() {
  const found = [];
  for (const entry of fs.readdirSync(PROJECT)) {
    if (entry.startsWith('.workshop')) continue; // the definitions, and Workshop's lock
    found.push(entry);
    fs.rmSync(path.join(PROJECT, entry), { recursive: true, force: true });
  }
  return found;
}

// ---------- One session ----------

// Runs the agent in a freshly restored pokedex-agent and copies what it made to
// <iterDir>/agent: the same files Lawson's container writes to /out.
export async function runAgent(iterDir, { framework, prompt, model, effort, timeoutMin = 240 }, childEnv) {
  const agentDir = path.join(iterDir, 'agent');
  fs.mkdirSync(agentDir, { recursive: true });
  const leftovers = cleanProject();
  const restoreMs = restore(AGENT);
  const logFile = path.join(iterDir, 'container.log');
  fs.writeFileSync(logFile, `restore ${AGENT}: ${restoreMs} ms${leftovers.length ? `; removed from the project folder: ${leftovers.join(', ')}` : ''}\n`);
  await useCompiler(AGENT, compilerFor(framework), logFile);
  const env = {
    FRAMEWORK: framework,
    PROMPT: prompt,
    MODEL: model,
    EFFORT: effort,
    OUT: `${A}/out`,
    WORK: AWORK,
    IS_SANDBOX: '1',
    DISABLE_AUTOUPDATER: '1',
    NPM_CONFIG_UPDATE_NOTIFIER: 'false',
    CLAUDE_CODE_OAUTH_TOKEN: true,
  };
  // `timeout` inside, so the agent is stopped where it runs; Lawson's limit, 240 minutes.
  // Root starts it and `runuser` drops to `agent` (no sudo involved); the environment passes through.
  const code = await exec(AGENT, ['runuser', '-u', 'agent', '--', 'env', `HOME=${A}`, 'timeout', '-k', '60', `${timeoutMin}m`, `${A}/bench-agent/run-agent.sh`], { env, uid: 0, cwd: AWORK, log: logFile, childEnv });
  fs.appendFileSync(logFile, `run-agent.sh exit ${code}\n`);
  if (code === 124 || code === 137) {
    fs.writeFileSync(path.join(iterDir, 'TIMED_OUT'), `${timeoutMin} minutes\n`);
    // run-agent.sh did not get to save the repo: save it as it would have.
    await exec(AGENT, ['bash', '-c', `echo claude_exit=124 >> ${A}/out/run-info.txt; tar --exclude=node_modules --exclude=./data --exclude=./.data-cache --exclude=./test-results -czf ${A}/out/repo.tar.gz -C ${AWORK} .`], { uid: 0, cwd: '/', log: logFile });
  }
  const tgz = path.join(iterDir, 'agent-out.tar.gz');
  await exec(AGENT, ['tar', '-czf', '-', '-C', `${A}/out`, '.'], { uid: 0, cwd: '/', stdout: tgz });
  spawnSync('tar', ['-xzf', tgz, '-C', agentDir], { stdio: 'inherit' });
  fs.rmSync(tgz, { force: true });
  const after = cleanProject();
  if (after.length) fs.appendFileSync(logFile, `removed from the project folder after the session: ${after.join(', ')}\n`);
  return code;
}

// The framework a session ran, from its run-info.txt.
function frameworkOf(iterDir) {
  const f = path.join(iterDir, 'agent/run-info.txt');
  return fs.existsSync(f) ? fs.readFileSync(f, 'utf8').match(/^framework=(.*)$/m)?.[1] : null;
}

// Unpacks the agent's repo into <iterDir>/repo, as run.mjs does.
function extractRepo(iterDir) {
  const repo = path.join(iterDir, 'repo');
  fs.rmSync(repo, { recursive: true, force: true });
  fs.mkdirSync(repo);
  spawnSync('tar', ['-xzf', path.join(iterDir, 'agent/repo.tar.gz'), '-C', repo], { stdio: 'inherit' });
  return repo;
}

// The original tests, against the agent's app/ alone, in a freshly restored grader.
// Appends final_test_exit to agent/run-info.txt, as run.mjs does.
export async function testApp(iterDir) {
  if (!fs.existsSync(path.join(iterDir, 'agent/repo.tar.gz'))) return null;
  const repo = extractRepo(iterDir);
  fs.mkdirSync(path.join(repo, 'app'), { recursive: true });
  const logFile = path.join(iterDir, 'agent/final-test.log');
  fs.writeFileSync(logFile, `restore ${GRADER}: ${restore(GRADER)} ms\n`);
  await useCompiler(GRADER, compilerFor(frameworkOf(iterDir)), logFile);
  const appTgz = path.join(iterDir, 'app.tar.gz');
  spawnSync('tar', ['--exclude=node_modules', '-czf', appTgz, '-C', repo, 'app']);
  await exec(GRADER, ['tar', '-xzf', '-', '-C', WORK], { stdin: appTgz, log: logFile });
  fs.rmSync(appTgz, { force: true });
  const code = await exec(GRADER, ['npm', 'test'], { env: { NPM_CONFIG_UPDATE_NOTIFIER: 'false' }, timeout: '60m', log: logFile });
  fs.appendFileSync(path.join(iterDir, 'agent/run-info.txt'), `final_test_exit=${code}\n`);
  return code;
}

// Lighthouse and lines of code for a passing app, in a freshly restored grader.
export async function measure(iterDir, { runs = 5, measureFailed = false } = {}) {
  const results = path.join(iterDir, 'measure');
  fs.rmSync(results, { recursive: true, force: true });
  fs.mkdirSync(results, { recursive: true });
  const fail = (msg) => fs.writeFileSync(path.join(results, 'error.txt'), `${msg}\n`);
  const info = fs.existsSync(path.join(iterDir, 'agent/run-info.txt')) ? fs.readFileSync(path.join(iterDir, 'agent/run-info.txt'), 'utf8') : '';
  if (!fs.existsSync(path.join(iterDir, 'agent/repo.tar.gz'))) return fail('no repo.tar.gz (the agent run did not finish)');
  if (!/final_test_exit=0\b/.test(info) && !measureFailed) return fail('skipped: final tests failed');
  const repo = extractRepo(iterDir);
  if (!fs.existsSync(path.join(repo, 'app/package.json'))) return fail('no app/package.json');
  const logFile = path.join(results, 'measure.log');
  fs.writeFileSync(logFile, `restore ${GRADER}: ${restore(GRADER)} ms\n`);
  await useCompiler(GRADER, compilerFor(frameworkOf(iterDir)), logFile);
  const appTgz = path.join(iterDir, 'app.tar.gz');
  spawnSync('tar', ['--exclude=node_modules', '-czf', appTgz, '-C', repo, 'app']);
  await exec(GRADER, ['bash', '-c', `mkdir -p ${W}/repo && tar -xzf - -C ${W}/repo`], { stdin: appTgz, log: logFile });
  fs.rmSync(appTgz, { force: true });
  const code = await exec(GRADER, ['node', `${W}/bench/measure.mjs`, `${W}/repo/app`, `${W}/results`, '--runs', String(runs)], {
    env: { TEMPLATE_DIR: WORK, NPM_CONFIG_UPDATE_NOTIFIER: 'false' },
    cwd: `${W}/bench`,
    log: logFile,
  });
  if (code !== 0) return fail(`measure exited with ${code}; see measure/measure.log`);
  const tgz = path.join(iterDir, 'measure-out.tar.gz');
  await exec(GRADER, ['tar', '-czf', '-', '-C', `${W}/results`, '.'], { stdout: tgz });
  spawnSync('tar', ['-xzf', tgz, '-C', results]);
  fs.rmSync(tgz, { force: true });
}

// ---------- Building the workshops ----------

const sh = (lines) => `#!/bin/bash\nset -euo pipefail\n${lines.join('\n')}\n`;

// A command as another user, from a root hook.
const asUser = (user, home, cmd) => `runuser -u ${user} -- env HOME=${home} bash -c ${JSON.stringify(`set -euo pipefail; ${cmd}`)}`;
const asWorkshop = (cmd) => asUser('workshop', W, cmd);

// Two compilers in each image, one put on PATH per session (useCompiler): the release
// (`--wf-version`, checked against its SHA256SUMS) and one built from a local checkout,
// for an A/B of a change before it is released (the framework `webfluent-lean`).
const compilers = (wfVersion) => [
  'cd "$(dirname "$0")"',
  'mkdir -p /opt/wf/release /opt/wf/lean /tmp/wfr',
  `WF_TARBALL="wf-v${wfVersion}-x86_64-linux.tar.gz"`,
  `WF_BASE="https://github.com/monzeromer-lab/WebFluent/releases/download/v${wfVersion}"`,
  '(cd /tmp/wfr && curl -fsSLO "${WF_BASE}/${WF_TARBALL}" && curl -fsSL "${WF_BASE}/SHA256SUMS" | grep " ${WF_TARBALL}$" | sha256sum -c - && tar -xzf "${WF_TARBALL}")',
  'install -m 0755 "$(find /tmp/wfr -name wf -type f | head -n 1)" /opt/wf/release/wf && rm -rf /tmp/wfr',
  'if [ -f seed/wf-lean ]; then install -m 0755 seed/wf-lean /opt/wf/lean/wf; fi',
  'install -m 0755 /opt/wf/release/wf /usr/local/bin/wf',
  '/opt/wf/release/wf --version',
];

// Which compiler a framework's session uses.
export const compilerFor = (framework) => (framework?.endsWith('-lean') ? 'lean' : 'release');

// The local checkout's compiler (and, for the agent, its skill), copied into a workshop
// just restored: what `lean` means is whatever this checkout is now, with no rebuild of
// the workshop between one change and its A/B.
async function pushLocal(name, withSkill, log) {
  const repo = path.resolve(HERE, '../..');
  const bin = path.join(repo, 'target/release/wf');
  if (!fs.existsSync(bin)) throw new Error('no local compiler: cargo build --release first');
  const stage = fs.mkdtempSync(path.join(os.tmpdir(), 'wf-lean-'));
  fs.mkdirSync(path.join(stage, 'opt/wf/lean'), { recursive: true });
  fs.copyFileSync(bin, path.join(stage, 'opt/wf/lean/wf'));
  const parts = ['opt'];
  if (withSkill) {
    fs.mkdirSync(path.join(stage, 'home/agent'), { recursive: true });
    fs.cpSync(path.join(repo, 'skills/writing-webfluent'), path.join(stage, 'home/agent/skill-lean'), { recursive: true });
    const head = spawnSync('git', ['-C', repo, 'rev-parse', '--short', 'HEAD'], { encoding: 'utf8' }).stdout.trim();
    const dirty = spawnSync('git', ['-C', repo, 'status', '--porcelain', 'skills', 'src', 'AGENTS.md'], { encoding: 'utf8' }).stdout.trim() !== '';
    fs.writeFileSync(path.join(stage, 'home/agent/skill-lean/.checkout'), `${head}${dirty ? '+changes' : ''} ${new Date().toISOString()}\n`);
    parts.push('home');
  }
  const tgz = path.join(stage, 'push.tar.gz');
  spawnSync('tar', ['-czf', tgz, '-C', stage, ...parts]);
  const code = await exec(name, ['bash', '-c', 'rm -rf /home/agent/skill-lean && tar -xzf - -C / && chmod 0755 /opt/wf/lean/wf && (chown -R agent:agent /home/agent/skill-lean 2>/dev/null || true)'], { uid: 0, cwd: '/', stdin: tgz, log });
  fs.rmSync(stage, { recursive: true, force: true });
  if (code !== 0) throw new Error(`could not copy the local compiler into ${name}`);
}

// Puts that compiler on PATH, as root, after a restore and before anything runs.
export async function useCompiler(name, which, log) {
  if (which === 'lean') await pushLocal(name, name === AGENT, log);
  const code = await exec(name, ['bash', '-c', `test -x /opt/wf/${which}/wf && install -m 0755 /opt/wf/${which}/wf /usr/local/bin/wf && wf --version`], { uid: 0, cwd: '/', log });
  if (code !== 0) throw new Error(`no ${which} compiler in ${name}`);
}
const asAgent = (cmd) => asUser('agent', A, cmd);

// The exported template at ~/work, owned by the workshop user, with its dependencies and
// Playwright's Chromium, the data linked in read-only: what Lawson's base image holds.
// The seed sits beside the hook (the hook folder is read-only; it is only read).
const workDir = (user, home) => [
  'cd "$(dirname "$0")"',
  `rm -rf ${home}/work && cp -r seed/work ${home}/work`,
  `chmod +x ${home}/work/scripts/*.sh`,
  `rm -rf ${home}/work/data && ln -s /opt/pokedex/data ${home}/work/data`,
  `chown -R ${user}:${user} ${home}/work`,
  asUser(user, home, `cd ${home}/work && npm install --no-audit --no-fund && for i in 1 2 3 4 5; do npx playwright install chromium && break; echo retrying >&2; sleep 10; done; npx playwright install chromium`),
];

function definitions({ wfVersion, claudeVersion }) {
  const node = NODE_VERSION();
  return {
    // What every workshop shares: the system, Node, Chromium's libraries, wf, and the
    // exported template with its data and dependencies.
    'pokedex/sdk.yaml': 'name: pokedex\n',
    'pokedex/hooks/setup-base': sh([
      'export DEBIAN_FRONTEND=noninteractive',
      'apt-get update',
      'apt-get install -y --no-install-recommends ca-certificates curl git xz-utils build-essential python3 jq',
      `NODE_VERSION=${node}`,
      'case "$(uname -m)" in x86_64) ARCH=x64 ;; aarch64) ARCH=arm64 ;; *) echo "unsupported arch" >&2; exit 1 ;; esac',
      'TARBALL="node-v${NODE_VERSION}-linux-${ARCH}.tar.xz"',
      'cd /tmp',
      'curl -fsSLO "https://nodejs.org/dist/v${NODE_VERSION}/${TARBALL}"',
      'curl -fsSL "https://nodejs.org/dist/v${NODE_VERSION}/SHASUMS256.txt" | grep " ${TARBALL}$" | sha256sum -c -',
      'tar -xJf "${TARBALL}" -C /usr/local --strip-components=1 --no-same-owner',
      'rm -f "${TARBALL}"',
      `npx -y playwright@${PLAYWRIGHT_VERSION} install-deps chromium`,
      `WF_VERSION=${wfVersion}`,
      'WF_TARBALL="wf-v${WF_VERSION}-x86_64-linux.tar.gz"',
      'WF_BASE="https://github.com/monzeromer-lab/WebFluent/releases/download/v${WF_VERSION}"',
      'mkdir -p /tmp/wf && cd /tmp/wf',
      'curl -fsSLO "${WF_BASE}/${WF_TARBALL}"',
      'curl -fsSL "${WF_BASE}/SHA256SUMS" | grep " ${WF_TARBALL}$" | sha256sum -c -',
      'tar -xzf "${WF_TARBALL}"',
      'install -m 0755 "$(find /tmp/wf -name wf -type f | head -n 1)" /usr/local/bin/wf',
      'rm -rf /tmp/wf',
      // Lawson's --add-host: the agent must use the local API.
      "grep -q 'pokeapi.co' /etc/hosts || printf '127.0.0.1 pokeapi.co www.pokeapi.co\\n' >> /etc/hosts",
      // The Pokémon data (about 1.7 GB), fetched here rather than per launch: this hook's
      // result is snapshotted, so every workshop and every rebuild reuses it. Lawson's own
      // script, at its pinned commits; read-only to the agent.
      'cd "$(dirname "$0")"',
      'if [ ! -f /opt/pokedex/data/.done ]; then',
      // The hook folder is read-only and the script writes beside itself: run a copy.
      '  rm -rf /var/tmp/pokedata && mkdir -p /var/tmp/pokedata/scripts && cp data/scripts/* /var/tmp/pokedata/scripts/ && chmod +x /var/tmp/pokedata/scripts/setup-data.sh',
      '  for i in 1 2 3 4; do /var/tmp/pokedata/scripts/setup-data.sh && break; echo "data fetch failed, retrying" >&2; sleep 15; done',
      '  test -f /var/tmp/pokedata/data/.done',
      '  mkdir -p /opt/pokedex && rm -rf /opt/pokedex/data && mv /var/tmp/pokedata/data /opt/pokedex/data && rm -rf /var/tmp/pokedata',
      'fi',
      'node --version && wf --version && du -sh /opt/pokedex/data',
    ]),
    // Nothing per project: `workshop restore` runs this hook again after every rollback, so
    // everything a session needs is made in the setup-base hooks below, which are snapshotted.
    'pokedex/hooks/setup-project': sh(['true']),

    // The agent's user and tools, the template, its dependencies and the browsers, all in
    // the snapshot.
    'pokedex-agent/sdk.yaml': 'name: pokedex-agent\n',
    'pokedex-agent/hooks/setup-base': sh([
      // npm 11 skips install scripts it has not been told to allow; these two need theirs.
      `npm install -g --allow-scripts=@anthropic-ai/claude-code,agent-browser @anthropic-ai/claude-code@${claudeVersion} agent-browser@${AGENT_BROWSER_VERSION}`,
      ...compilers(wfVersion),
      'id agent >/dev/null 2>&1 || useradd --create-home --shell /bin/bash agent',
      'gpasswd -d agent sudo >/dev/null 2>&1 || true',
      ...workDir('agent', A),
      `rm -rf ${A}/bench-agent ${A}/out && mkdir -p ${A}/bench-agent ${A}/out`,
      `cp seed/agent/* ${A}/bench-agent/ && chmod +x ${A}/bench-agent/*.sh`,
      `chown -R agent:agent ${A}/bench-agent ${A}/out`,
      // The local checkout's skill, for webfluent-lean's setup-tooling.sh.
      `rm -rf ${A}/skill-lean && if [ -d seed/skill-lean ]; then cp -r seed/skill-lean ${A}/skill-lean && chown -R agent:agent ${A}/skill-lean; fi`,
      // Its Chrome download times out on a slow connection; it is retried, not given up on.
      asAgent('for i in 1 2 3 4 5 6; do agent-browser install && break; echo retrying >&2; sleep 15; done; agent-browser install'),
    ]),
    'pokedex-agent/hooks/setup-project': sh(['true']),

    // The grader: the same template and browsers, plus Caddy and Lighthouse, as
    // pokedex-base and pokedex-measure have them.
    'pokedex-grader/sdk.yaml': 'name: pokedex-grader\n',
    'pokedex-grader/hooks/setup-base': sh([
      'export DEBIAN_FRONTEND=noninteractive',
      'apt-get update',
      'apt-get install -y --no-install-recommends caddy',
      ...compilers(wfVersion),
      ...workDir('workshop', W),
      `rm -rf ${W}/bench && mkdir -p ${W}/bench && cp seed/measure/* ${W}/bench/ && chown -R workshop:workshop ${W}/bench`,
      asWorkshop(`cd ${W}/bench && npm ci --no-audit --no-fund`),
    ]),
    'pokedex-grader/hooks/setup-project': sh(['true']),

    [`${AGENT}.yaml`]: `name: ${AGENT}\nbase: ubuntu@26.04\nsdks:\n  - name: project-pokedex\n  - name: project-pokedex-agent\n`,
    [`${GRADER}.yaml`]: `name: ${GRADER}\nbase: ubuntu@26.04\nsdks:\n  - name: project-pokedex\n  - name: project-pokedex-grader\n`,
  };
}

// Lawson's data scripts, beside the hook that runs them.
function dataScripts() {
  const dir = path.join(PROJECT, '.workshop/pokedex/hooks/data/scripts');
  fs.mkdirSync(dir, { recursive: true });
  for (const f of ['setup-data.sh', 'scrub-data.mjs']) fs.copyFileSync(path.join(UP, 'template/scripts', f), path.join(dir, f));
  fs.chmodSync(path.join(dir, 'setup-data.sh'), 0o755);
}

function seed() {
  for (const [sdk, extra] of [['pokedex-agent', ['agent', 'bench/container', ['run-agent.sh', 'setup-tooling.sh', 'mcp-probe.sh']]], ['pokedex-grader', ['measure', 'bench', ['package.json', 'package-lock.json', 'stack.mjs', 'measure.mjs']]]]) {
    const s = path.join(PROJECT, '.workshop', sdk, 'hooks/seed');
    fs.rmSync(s, { recursive: true, force: true });
    spawnSync(path.join(UP, 'bench/container/export-template.sh'), [path.join(s, 'work')], { stdio: 'inherit' });
    const [dir, from, files] = extra;
    fs.mkdirSync(path.join(s, dir), { recursive: true });
    for (const f of files) fs.copyFileSync(path.join(UP, from, f), path.join(s, dir, f));
    // The local checkout's compiler and skill, when it has been built (cargo build --release).
    const repo = path.resolve(HERE, '../..');
    const lean = path.join(repo, 'target/release/wf');
    if (fs.existsSync(lean)) fs.copyFileSync(lean, path.join(s, 'wf-lean'));
    if (sdk === 'pokedex-agent') {
      fs.cpSync(path.join(repo, 'skills/writing-webfluent'), path.join(s, 'skill-lean'), { recursive: true });
      const head = spawnSync('git', ['-C', repo, 'rev-parse', '--short', 'HEAD'], { encoding: 'utf8' }).stdout.trim();
      const dirty = spawnSync('git', ['-C', repo, 'status', '--porcelain', 'skills', 'src', 'AGENTS.md'], { encoding: 'utf8' }).stdout.trim() !== '';
      fs.writeFileSync(path.join(s, 'skill-lean/.checkout'), `${head}${dirty ? '+changes' : ''} ${new Date().toISOString()}\n`);
    }
  }
}

export async function setup({ wfVersion = '7.1.1', claudeVersion = '2.1.310', rebuild = false } = {}) {
  if (!fs.existsSync(UP)) throw new Error('The harness is missing: run ./setup.sh first.');
  fs.mkdirSync(path.join(PROJECT, '.workshop'), { recursive: true });
  for (const [file, text] of Object.entries(definitions({ wfVersion, claudeVersion }))) {
    const p = path.join(PROJECT, '.workshop', file);
    fs.mkdirSync(path.dirname(p), { recursive: true });
    fs.writeFileSync(p, text, { mode: file.includes('/hooks/') ? 0o755 : 0o644 });
  }
  dataScripts();
  seed();
  // A workshop is launched once it has a status other than "Off" (defined, never launched).
  const launched = (n) => new RegExp(`^${n}\\s+(?!Off\\b)\\S`, 'm').test(workshop(['list']));
  if (rebuild) for (const n of [AGENT, GRADER]) if (launched(n)) workshop(['remove', n], { stdio: 'inherit' });
  for (const n of [AGENT, GRADER]) {
    const verb = launched(n) ? 'refresh' : 'launch';
    log(`${verb} ${n} (the first launch downloads about 2 GB)`);
    const r = spawnSync('workshop', [verb, n, '-p', PROJECT], { stdio: 'inherit' });
    if (r.status !== 0) throw new Error(`workshop ${verb} ${n} failed`);
  }
  await check();
}

// What a session can rely on, checked in each workshop as it stands after a restore.
export async function check() {
  for (const n of [AGENT, GRADER]) {
    log(`${n}: restored in ${restore(n)} ms`);
    const home = n === AGENT ? A : W;
    const who = n === AGENT ? ['runuser', '-u', 'agent', '--', 'env', `HOME=${A}`] : [];
    await exec(n, [...who, 'bash', '-c', [
      'echo "node $(node --version), npm $(npm --version), wf: $(wf --version)"',
      'echo "claude: $(claude --version 2>/dev/null || echo none)"',
      'echo "pokeapi.co resolves to: $(getent hosts pokeapi.co | cut -d" " -f1)"',
      'echo "sudo: $(sudo -n true 2>/dev/null && echo yes || echo no)"',
      'echo "user: $(id -un), groups: $(id -Gn)"',
      `echo "work: $(ls ${'${HOME}'}/work | tr '\\n' ' ')"`,
      `echo "data: $(du -shL ${'${HOME}'}/work/data 2>/dev/null | cut -f1)"`,
      'echo "/project: $(ls -A /project | tr \'\\n\' \' \')"',
      'echo "cpus $(nproc), memory $(free -g | awk \'/Mem/ {print $2}\') GiB"',
    ].join('; ')], n === AGENT ? { uid: 0, cwd: A } : { cwd: home });
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [cmd, ...rest] = process.argv.slice(2);
  const opts = Object.fromEntries(rest.map((a) => a.replace(/^--/, '').split('=')).map(([k, v]) => [k, v ?? true]));
  if (cmd === 'setup') await setup({ wfVersion: opts['wf-version'], claudeVersion: opts['claude-version'], rebuild: Boolean(opts.rebuild) });
  else if (cmd === 'check') await check();
  else {
    console.error('usage: node workshop.mjs setup|check');
    process.exit(1);
  }
}
