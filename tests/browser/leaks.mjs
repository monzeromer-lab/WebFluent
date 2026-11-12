// What a single-page app holds on to over a long session, in a real Chrome:
// a site of three pages — a store every page reads, links, timers, `now`
// ticking every second, a resource refreshed on an interval, a socket that
// cannot connect, a stream, a channel, an autoplaying carousel, a sidebar
// drawer, a popover, a menu, a long list — visited round and round. After a
// few rounds to warm up, the DOM nodes, the listeners, the live intervals
// and the document's and window's listeners must stay where they were: a
// page that leaves takes everything it made with it.
//
//   cd tests/browser && npm ci && node leaks.mjs
//
// WF (default target/release/wf) and CHROME (default /usr/bin/google-chrome)
// say where they are. ROUNDS (default 60) is how many times round.

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
const ROUNDS = Number(process.env.ROUNDS || 60);

const results = [];
function check(name, ok, detail = "") {
  results.push(ok);
  console.log(`  ${ok ? "PASS" : "FAIL"}  ${name}${detail ? `  (${detail})` : ""}`);
}

const FILES = {
  "webfluent.app.json": JSON.stringify({
    name: "leaks", version: "1.0.0",
    build: { output: "./build", minify: false, ssg: false, csp: true },
    meta: { title: "leaks", description: "A long session.", lang: "en" },
  }, null, 2),
  "public/api/items.json": JSON.stringify([{ id: "1", name: "one" }, { id: "2", name: "two" }]),
  "src/App.wf": `
store Counter {
    state n = 0
    action bump() { n = n + 1 }
}

app {
    use Counter
    Navbar {
        Navbar.Links {
            Link(to: "/") { Text("A") }
            Link(to: "/b") { Text("B") }
            Link(to: "/c") { Text("C") }
        }
    }
    Text("Count {Counter.n}")
    Router
}
`,
  "src/pages/A.wf": `
type Item { id: String, name: String }

page A(path: "/", title: "A", description: "A page.") {
    use Counter
    state tick = 0
    state open = false
    state rows: [Item] = [Item(id: "1", name: "x"), Item(id: "2", name: "y")]
    every(100) { tick = tick + 1 }
    on key("Escape") { open = false }
    resource items: [Item] = fetch("/api/items.json", on: .interval(60.seconds))
    Heading("Page A").h1
    Text("tick {tick} n {Counter.n} w {viewport.width} {now(every: 1.seconds)}")
    Button("Toggle", ref: trigger) { on click { open = !open } }
    Popover(visible: open, anchor: trigger, label: "Pop") { Text("Inside") }
    Tooltip("Tip") { Button("Hover me") }
    Menu(trigger: "Options") { Menu.Item { Text("One") } }
    match items {
        loading { Text("loading") }
        error(e) { Text(e.message) }
        ready(list) { for it in list by it.id { Text(it.name) } }
    }
    for r in rows by r.id { Card { Text(r.name) } }
    Button("Shuffle") { on click { rows = rows.map(r => Item(id: uuid(), name: r.name)) } }
}
`,
  "src/pages/B.wf": `
page B(path: "/b", title: "B", description: "B page.") {
    use Counter
    state draft = ""
    socket chat = ws("ws://127.0.0.1:1/none") {
        on message(m) { draft = "x" }
    }
    stream ev = sse("/missing-events")
    channel ch = broadcast("leak") { on message(m) { draft = "y" } }
    after(50) { draft = "z" }
    Heading("Page B").h1
    Text("{Counter.n} {draft} {query.tab ?? \\"\\"}")
    Input(bind: draft, label: "Draft")
    Sidebar {
        Sidebar.Header { Text("Side") }
        Sidebar.Item(to: "/", icon: "home") { Text("Home") }
    }
    Carousel(autoplay: true, interval: 1000) {
        Carousel.Slide { Text("one") }
        Carousel.Slide { Text("two") }
    }
}
`,
  "src/pages/C.wf": `
page C(path: "/c", title: "C", description: "C page.") {
    use Counter
    state xs = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    Heading("Page C").h1
    Text("{Counter.n} {now}")
    VirtualList(rowHeight: 24, label: "L") {
        style { height: 200px }
        for x in xs by x { Text("row {x}") }
    }
}
`,
};

const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-leaks-site-"));
for (const [rel, text] of Object.entries(FILES)) {
  fs.mkdirSync(path.dirname(path.join(dir, rel)), { recursive: true });
  fs.writeFileSync(path.join(dir, rel), text);
}
execFileSync(WF, ["build", "-d", dir], { stdio: "inherit" });

const port = 42000 + Math.floor(Math.random() * 1000);
const site = host(path.join(dir, "build"), port);
await site.start();
const browser = await puppeteer.launch({ executablePath: CHROME, headless: true, args: ["--no-sandbox"] });
try {
  const page = await browser.newPage();
  // Count what the page holds that a heap snapshot does not show plainly:
  // live intervals, and listeners on the document and the window.
  await page.evaluateOnNewDocument(() => {
    const C = (window.__count = { intervals: new Map(), doc: 0, win: 0 });
    const si = window.setInterval;
    const ci = window.clearInterval;
    window.setInterval = function (f, ms, ...a) { const id = si.call(window, f, ms, ...a); C.intervals.set(id, ms); return id; };
    window.clearInterval = function (id) { C.intervals.delete(id); return ci.call(window, id); };
    const add = EventTarget.prototype.addEventListener;
    const remove = EventTarget.prototype.removeEventListener;
    const seen = new WeakMap();
    const which = (t) => (t === window ? "win" : t === document ? "doc" : null);
    EventTarget.prototype.addEventListener = function (type, fn, opt) {
      const k = which(this);
      if (k && fn && !(opt && opt.once)) {
        let m = seen.get(this);
        if (!m) seen.set(this, (m = new Map()));
        const key = type + "|" + !!(opt === true || (opt && opt.capture));
        const s = m.get(key) || new Set();
        m.set(key, s);
        if (!s.has(fn)) { s.add(fn); C[k]++; }
      }
      return add.call(this, type, fn, opt);
    };
    EventTarget.prototype.removeEventListener = function (type, fn, opt) {
      const k = which(this);
      if (k) {
        const m = seen.get(this);
        const s = m && m.get(type + "|" + !!(opt === true || (opt && opt.capture)));
        if (s && s.delete(fn)) C[k]--;
      }
      return remove.call(this, type, fn, opt);
    };
  });
  const errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "networkidle0" });
  const cdp = await page.target().createCDPSession();
  const go = async (r) => {
    await page.evaluate((r) => WF.navigate(r), r);
    // A boolean, not the heading: a handle puppeteer is handed back keeps
    // the element it names alive, which would look like the page's leak.
    await page.waitForFunction((r) => location.pathname === r && !!document.querySelector("main h1"), { timeout: 5000 }, r).catch(() => {});
  };
  // ROUTES=/c narrows the round to the pages named (and back), to find
  // which one holds on to something.
  const visit = process.env.ROUTES ? process.env.ROUTES.split(",") : ["/b", "/c"];
  const round = async () => { for (const r of [...visit, "/"]) await go(r); };
  const snap = async () => {
    await cdp.send("HeapProfiler.collectGarbage");
    await cdp.send("HeapProfiler.collectGarbage");
    const m = await page.metrics();
    const c = await page.evaluate(() => ({ intervals: window.__count.intervals.size, doc: window.__count.doc, win: window.__count.win }));
    return { nodes: m.Nodes, listeners: m.JSEventListeners, heapMB: +(m.JSHeapUsedSize / 1e6).toFixed(2), ...c };
  };
  for (let i = 0; i < 5; i++) await round();
  await new Promise((r) => setTimeout(r, 300));
  const before = await snap();
  for (let i = 0; i < ROUNDS; i++) await round();
  await new Promise((r) => setTimeout(r, 500));
  const after = await snap();
  console.log(`\n  after 5 rounds:  ${JSON.stringify(before)}`);
  console.log(`  after ${ROUNDS + 5} rounds: ${JSON.stringify(after)}\n`);

  // A little slack for what the browser itself keeps (a style recalc's
  // nodes, a pooled listener); a leak grows with every round.
  const flat = (a, b, slack) => b <= a * 1.1 + slack;
  check("DOM nodes stay flat across rounds", flat(before.nodes, after.nodes, 100), `${before.nodes} → ${after.nodes}`);
  check("event listeners stay flat across rounds", flat(before.listeners, after.listeners, 40), `${before.listeners} → ${after.listeners}`);
  check("live intervals stay flat across rounds", after.intervals <= before.intervals, `${before.intervals} → ${after.intervals}`);
  check("document listeners stay flat across rounds", after.doc <= before.doc + 1, `${before.doc} → ${after.doc}`);
  check("window listeners stay flat across rounds", after.win <= before.win + 1, `${before.win} → ${after.win}`);
  check("the JS heap stays flat across rounds", after.heapMB <= before.heapMB * 1.25 + 0.5, `${before.heapMB} → ${after.heapMB} MB`);

  // `now(every: 1.seconds)` ticks while page A shows it, and not after.
  await go("/c");
  await new Promise((r) => setTimeout(r, 1300));
  const everySecond = () => page.evaluate(() => [...window.__count.intervals.values()].filter((ms) => ms === 1000).length);
  const onC = await everySecond();
  await go("/");
  const onA = await everySecond();
  check("the second-by-second clock ticks while its page shows it", onA === 1, `${onA} on /`);
  check("and stops when its page leaves", onC === 0, `${onC} on /c`);
  check("no page threw", errors.length === 0, errors.slice(0, 3).join(" | "));
} finally {
  await browser.close();
  await site.stop();
  fs.rmSync(dir, { recursive: true, force: true });
}

const failed = results.filter((ok) => !ok).length;
console.log(`\n${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
