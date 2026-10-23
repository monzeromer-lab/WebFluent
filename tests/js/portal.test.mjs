//! `Portal(into: node) { … }`: a block drawn into a node somebody else's
//! code owns — an editor's tooltip, a map's popup. The box follows `into:`,
//! is moved rather than drawn again, is kept aside with its nodes and state
//! when `into:` names nothing, and goes with the scope it is written in;
//! what is inside belongs to where it is written.
//!
//! The first tests drive the runtime as the compiler's output does; the
//! rest build small projects with the compiler (skipped when none is built)
//! and run what it wrote.
//!
//! Run: node --test tests/js/portal.test.mjs
import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { makeDom } from "./dom.mjs";
import { fullRuntime } from "./runtime.mjs";

function load() {
  const dom = makeDom();
  const fn = new Function(
    "window", "document", "Node", "Element", "DocumentFragment",
    `${fullRuntime()}\nreturn WF;`,
  );
  return { WF: fn(dom.window, dom.document, dom.Node, dom.Element, dom.DocumentFragment), ...dom };
}

/// Whether `node` is in the document: the fake DOM has no `isConnected`.
function inDocument(document, node) {
  for (let n = node; n; n = n.parentNode) if (n === document.body) return true;
  return false;
}

/// What `Portal(into: read()) { block }` compiles to, written into `parent`:
/// the box, the marker, the box placed, then the block drawn into it at the
/// same level — never inside a block of its own — and what it held let go.
function portalIn(WF, document, parent, read, block) {
  const box = WF.el("div", { className: "wf-portal" });
  parent.appendChild(document.createComment("wf-portal"));
  const drawn = WF.portal(box, read);
  block(box);
  drawn();
  return box;
}

/// Somebody else's node: an element outside the app's root.
function foreign(document, id) {
  const el = document.createElement("div");
  el.id = id;
  document.body.appendChild(el);
  return el;
}

test("a portal draws its block into somebody else's node, and leaves only a marker where it is written", () => {
  const { WF, document } = load();
  const app = document.createElement("div");
  document.body.appendChild(app);
  const tip = foreign(document, "tip");
  WF.scoped(() => {
    portalIn(WF, document, app, () => tip, (box) => box.appendChild(WF.el("p", {}, ["inside"])));
  });
  assert.equal(tip.children.length, 1);
  assert.equal(tip.children[0].className, "wf-portal");
  assert.equal(tip.textContent, "inside");
  assert.equal(app.children.length, 0, "nothing is drawn where it is written");
  assert.deepEqual(app.childNodes.map((n) => n.nodeType), [8], "but a marker is");
});

test("a portal moves when into: changes, with its nodes and their state", () => {
  const { WF, document } = load();
  const a = foreign(document, "a");
  const b = foreign(document, "b");
  const where = WF.signal(a);
  const count = WF.signal(0);
  // Something else it reads, so it is read again with no change of node.
  const tick = WF.signal(0);
  let p;
  WF.scoped(() => {
    portalIn(WF, document, document.createDocumentFragment(), () => { tick(); return where(); }, (box) => {
      p = WF.el("p", {}, [() => `count ${count()}`]);
      box.appendChild(p);
    });
  });
  count.set(1);
  where.set(b);
  assert.equal(a.children.length, 0, "it left the first node");
  assert.equal(b.children[0].children[0], p, "the same paragraph, moved");
  assert.equal(p.textContent, "count 1");
  count.set(2);
  assert.equal(p.textContent, "count 2", "and what it shows still follows its state");
  // Read again and naming the node it is in, it is not moved: the node's
  // own children after it stay after it.
  const box = b.children[0];
  b.appendChild(WF.el("span", {}, ["after"]));
  tick.set(1);
  assert.equal(b.children[0], box, "still first: it was not appended again");
});

test("null keeps the block aside, and the same nodes come back", () => {
  const { WF, document } = load();
  const host = foreign(document, "host");
  const where = WF.signal(host);
  const typed = WF.signal("a");
  let input;
  WF.scoped(() => {
    portalIn(WF, document, document.createDocumentFragment(), () => where(), (box) => {
      input = WF.el("input", { value: () => typed() });
      box.appendChild(input);
    });
  });
  const box = host.children[0];
  where.set(null);
  assert.equal(host.children.length, 0, "parked");
  assert.equal(box.parentNode, null);
  assert.equal(box.children[0], input, "its nodes kept");
  typed.set("ab");
  assert.equal(input.value, "ab", "and kept current while parked");
  where.set(undefined);
  where.set(host);
  assert.equal(host.children[0], box, "the same box came back");
  assert.equal(box.children[0], input, "with the same input in it");
});

test("a handle's element is somewhere to draw, even one drawn after the portal", async () => {
  const { WF, document } = load();
  const spot = WF.ref();
  const page = document.createDocumentFragment();
  WF.scoped(() => {
    portalIn(WF, document, page, () => spot, (box) => box.appendChild(WF.el("p", {}, ["in"])));
    // The element the handle names is written after the portal.
    page.appendChild(WF.el("div", { className: "spot", ref: spot }));
  });
  await Promise.resolve();
  const el = spot.el;
  assert.equal(el.children.length, 1, "looked at again once the drawing was done");
  assert.equal(el.textContent, "in");
});

test("the end of its scope takes the box away, and nothing brings it back", () => {
  const { WF, document } = load();
  const host = foreign(document, "host");
  const where = WF.signal(host);
  const gone = [];
  const [, dispose] = WF.scoped(() => {
    portalIn(WF, document, document.createDocumentFragment(), () => where(), (box) => {
      box.appendChild(WF.el("p", {}, ["in"]));
      WF.onCleanup(() => gone.push("block"));
    });
  });
  assert.equal(host.children.length, 1);
  dispose();
  assert.equal(host.children.length, 0, "the box went with its scope");
  assert.deepEqual(gone, ["block"], "and the block's cleanups ran");
  where.set(foreign(document, "other"));
  assert.equal(document.getElementById("other").children.length, 0, "a later into: is not read");
});

test("a portal in a branch goes when the branch does, and its marker is the branch's node", () => {
  const { WF, document } = load();
  const host = foreign(document, "host");
  const open = WF.signal(true);
  const app = document.createElement("div");
  WF.scoped(() => {
    WF.when(app, () => open(), () => {
      const f = document.createDocumentFragment();
      portalIn(WF, document, f, () => host, (box) => box.appendChild(WF.el("p", {}, ["tip"])));
      return f;
    });
  });
  assert.equal(host.textContent, "tip");
  open.set(false);
  assert.equal(host.children.length, 0, "the branch left, and took the box");
  open.set(true);
  assert.equal(host.textContent, "tip", "and a new one comes with the branch");
  assert.equal(host.children.length, 1);
});

test("a mount inside a portal meets its node in the page when the target is", () => {
  const { WF, document } = load();
  const host = foreign(document, "host");
  const seen = [];
  WF.scoped(() => {
    portalIn(WF, document, document.createDocumentFragment(), () => host, (box) => {
      // A Host as the compiler draws one: its lifetime asked for before it
      // is appended to the box.
      const node = WF.el("div", { className: "wf-host" });
      WF.attach(node, (n) => { seen.push(["mount", inDocument(document, n)]); return "h"; }, null, (h) => seen.push(["cleanup", h]));
      const inner = WF.el("section", {});
      const deeper = WF.el("span", {});
      WF.attach(deeper, (n) => { seen.push(["deep", inDocument(document, n)]); }, null, null);
      inner.appendChild(deeper);
      box.appendChild(node);
      box.appendChild(inner);
      seen.push(["drawn"]);
    });
  });
  assert.deepEqual(seen, [["drawn"], ["mount", true], ["deep", true]], "after the block is whole, in the page");
});

test("a lifetime held by a portal belongs to the branch it was asked for in", () => {
  const { WF, document } = load();
  const host = foreign(document, "host");
  const shown = WF.signal(true);
  const seen = [];
  WF.scoped(() => {
    portalIn(WF, document, document.createDocumentFragment(), () => host, (box) => {
      WF.when(box, () => shown(), () => {
        const n = WF.el("canvas", {});
        WF.attach(n, () => { seen.push("mount"); return 1; }, null, () => seen.push("cleanup"));
        return n;
      });
    });
  });
  assert.deepEqual(seen, ["mount"]);
  shown.set(false);
  assert.deepEqual(seen, ["mount", "cleanup"], "the branch's cleanup, not the page's");
  shown.set(true);
  assert.deepEqual(seen, ["mount", "cleanup", "mount"], "drawn again later, it mounts at once");
});

test("a block that throws holds nothing for good: what it and the page after it asked for runs once the drawing is over", async () => {
  const { WF, document } = load();
  const host = foreign(document, "host");
  const seen = [];
  const [, dispose] = WF.scoped(() => {
    try {
      portalIn(WF, document, document.createDocumentFragment(), () => host, () => {
        WF.attach(WF.el("div", {}), () => seen.push("before the throw"), null, () => seen.push("its cleanup"));
        throw new Error("boom");
      });
    } catch { /* a list draws the rest of its items */ }
    // A portal drawn after it still lets its own go once it is whole.
    portalIn(WF, document, document.createDocumentFragment(), () => host, () => {
      WF.attach(WF.el("div", {}), () => seen.push("the next portal"), null, null);
    });
    WF.attach(WF.el("div", {}), () => seen.push("later"), null, null);
  });
  assert.deepEqual(seen, ["the next portal"]);
  await Promise.resolve();
  assert.deepEqual(seen, ["the next portal", "before the throw", "later"], "let go once the drawing is over");
  dispose();
  assert.deepEqual(seen.at(-1), "its cleanup", "in the scope it was asked for in");
  WF.scoped(() => WF.attach(WF.el("div", {}), () => seen.push("after"), null, null));
  assert.equal(seen.at(-1), "after", "and nothing is held after it");
});

// ─── What the compiler writes ──────────────────────────────────

const WF_BIN = ["target/release/wf", "target/debug/wf"].find((p) => existsSync(p));

/// Build `files` as a project, and the bundle it wrote with its scripts in
/// front, as a page links them. `null` when no compiler is built.
function build(files, config = {}) {
  if (!WF_BIN) return null;
  const dir = mkdtempSync(join(tmpdir(), "wf-portal-"));
  for (const [path, text] of Object.entries(files)) {
    mkdirSync(join(dir, path, ".."), { recursive: true });
    writeFileSync(join(dir, path), text);
  }
  writeFileSync(
    join(dir, "webfluent.app.json"),
    JSON.stringify({ name: "portal", build: { output: "./build", split: false, minify: false, ...config } }),
  );
  execFileSync(WF_BIN, ["build", "-d", dir], { stdio: "pipe" });
  const out = join(dir, "build");
  let scripts = "";
  if (existsSync(join(out, "js"))) {
    for (const f of readdirSync(join(out, "js")).sort()) scripts += readFileSync(join(out, "js", f), "utf8") + "\n";
  }
  const bundle = scripts + readFileSync(join(out, "app.js"), "utf8");
  const html = readFileSync(join(out, "index.html"), "utf8");
  rmSync(dir, { recursive: true, force: true });
  return { bundle, html };
}

/// Run a bundle against a fresh DOM: `#app`, filled with `painted` markup
/// when a pre-rendered page is being taken over; `before(dom)` sets up what
/// somebody else's code would have made.
function run(bundle, { painted = null, before = () => {}, dev = false, quiet = false } = {}) {
  const dom = makeDom();
  const { window, document } = dom;
  const app = document.createElement("div");
  app.id = "app";
  document.body.appendChild(app);
  if (painted) parseInto(document, app, painted);
  window.__WF_DEV__ = dev;
  before(dom);
  const warned = [];
  // `quiet`: what the page reports as an error is kept, not printed — for
  // a test that makes something throw on purpose.
  const errors = [];
  const console_ = {
    ...console,
    warn: (...a) => warned.push(a.map(String).join(" ")),
    ...(quiet ? { error: (...a) => errors.push(a.map(String).join(" ")) } : {}),
  };
  const timers = [];
  window.setTimeout = (fn) => { timers.push(fn); return timers.length; };
  window.clearTimeout = () => {};
  new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout", "clearTimeout",
    "URLSearchParams", "console", bundle,
  )(window, document, dom.Node, dom.Element, dom.DocumentFragment, window.setTimeout, window.clearTimeout, URLSearchParams, console_);
  const drain = () => { while (timers.length) timers.shift()(); };
  drain();
  return { ...dom, app, warned, errors, drain };
}

const click = (el) => el.dispatchEvent({ type: "click", target: el, preventDefault() {} });
const buttonIn = (root, text) => root.querySelectorAll("button").find((b) => b.textContent === text);

/// The markup of a pre-rendered `#app`, parsed into `app` as a browser
/// would: elements, attributes, text (indentation included) and comments.
function parseInto(document, app, html) {
  const start = html.indexOf('<div id="app">');
  assert.ok(start >= 0, "a pre-rendered #app");
  const VOID = new Set(["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"]);
  const decode = (t) => t.replace(/&(amp|lt|gt|quot|#39|#x27);/g, (_, e) => ({ amp: "&", lt: "<", gt: ">", quot: '"', "#39": "'", "#x27": "'" })[e]);
  const stack = [app];
  let i = start + '<div id="app">'.length;
  while (i < html.length && stack.length) {
    if (html.startsWith("<!--", i)) {
      const end = html.indexOf("-->", i);
      stack.at(-1).appendChild(document.createComment(html.slice(i + 4, end)));
      i = end + 3;
    } else if (html.startsWith("</", i)) {
      stack.pop();
      i = html.indexOf(">", i) + 1;
    } else if (html[i] === "<") {
      const m = /^<([a-zA-Z0-9-]+)((?:\s+[^\s=>/]+(?:="[^"]*")?)*)\s*(\/?)>/.exec(html.slice(i));
      assert.ok(m, `markup at ${html.slice(i, i + 40)}`);
      const el = document.createElement(m[1]);
      for (const a of m[2].matchAll(/([^\s=>/]+)(?:="([^"]*)")?/g)) el.setAttribute(a[1], decode(a[2] ?? ""));
      stack.at(-1).appendChild(el);
      if (!VOID.has(m[1].toLowerCase()) && !m[3]) stack.push(el);
      i += m[0].length;
    } else {
      const next = html.indexOf("<", i);
      stack.at(-1).appendChild(document.createTextNode(decode(html.slice(i, next))));
      i = next;
    }
  }
}

const HOSTS = `/**
 * The element the editor made for a popup, or null when it has none: the
 * test makes them, a new one for every write, as an editor makes its
 * tooltips anew.
 * @param {string} id
 * @returns {HTMLElement|null}
 */
function editorHost(id) {
  return (window.hosts && window.hosts[id]) || null;
}
`;

test("a keyed for of portals into hosts made anew on every write keeps one box per key", (t) => {
  const built = build({
    "src/hosts.js": HOSTS,
    "src/App.wf": `store Popups {
    state rows = [{ id: "a", name: "alpha" }, { id: "b", name: "beta" }]
    state round = 0
    action write() {
        round = round + 1
        rows = [{ id: "a", name: "alpha {round}" }, { id: "b", name: "beta {round}" }]
    }
}
page Home(path: "/", title: "H", description: "D") {
    Heading("Rows").h1
    Button("Write") { on click { Popups.write() } }
    for r, i in Popups.rows by i {
        Portal(into: editorHost(r.id), data-popup: "row") { Text(r.name) }
    }
}
`,
  });
  if (!built) return t.skip("no compiler built");
  let made = 0;
  const hostsFor = (window, document) => {
    made++;
    window.hosts = {};
    for (const id of ["a", "b"]) {
      const el = document.createElement("div");
      el.className = `host-${id}-${made}`;
      document.body.appendChild(el);
      window.hosts[id] = el;
    }
    return window.hosts;
  };
  const page = run(built.bundle, { before: ({ window, document }) => hostsFor(window, document) });
  const { window, document, app } = page;
  const boxes = () => document.querySelectorAll(".wf-portal");
  const first = window.hosts;
  assert.equal(boxes().length, 2);
  assert.equal(first.a.textContent, "alpha");
  assert.equal(first.b.textContent, "beta");
  const [boxA, boxB] = [first.a.children[0], first.b.children[0]];
  assert.equal(boxA.getAttribute("data-popup"), "row", "a data-* argument is the box's");
  for (let round = 1; round <= 3; round++) {
    const before = window.hosts;
    const now = hostsFor(window, document);
    click(buttonIn(app, "Write"));
    page.drain();
    assert.equal(boxes().length, 2, `round ${round}: one box per key`);
    assert.equal(now.a.children.length, 1);
    assert.equal(now.a.children[0], boxA, `round ${round}: the same box for a, moved`);
    assert.equal(now.b.children[0], boxB);
    assert.equal(before.a.children.length, 0, "the old host is left empty");
    assert.equal(now.a.textContent, `alpha ${round}`, "and it shows the item's new value");
  }
  assert.equal(app.querySelectorAll(".wf-portal").length, 0, "nothing in the app's own tree");
});

test("handlers and emit inside a portal reach the component it is written in", (t) => {
  const built = build({
    "src/hosts.js": HOSTS,
    "src/App.wf": `component Chip(_ label: String) {
    event picked(label: String)
    Button(label) { on click { emit picked(label) } }
}
component Panel {
    state chosen = "none"
    state count = 0
    Text("chosen {chosen}, count {count}")
    Portal(into: editorHost("tip")) {
        Chip("alpha") { on picked(n) { chosen = n } }
        Button("More") { on click { count = count + 1 } }
    }
}
page Home(path: "/", title: "H", description: "D") {
    Heading("Panel").h1
    Panel
}
`,
  });
  if (!built) return t.skip("no compiler built");
  const page = run(built.bundle, {
    before: ({ window, document }) => {
      const tip = document.createElement("div");
      document.body.appendChild(tip);
      window.hosts = { tip };
    },
  });
  const tip = page.window.hosts.tip;
  assert.ok(page.app.textContent.includes("chosen none, count 0"), page.app.textContent);
  click(buttonIn(tip, "alpha"));
  click(buttonIn(tip, "More"));
  click(buttonIn(tip, "More"));
  page.drain();
  assert.ok(page.app.textContent.includes("chosen alpha, count 2"), page.app.textContent);
  assert.equal(buttonIn(page.app, "alpha"), undefined, "the buttons are in the host, not the app");
});

test("a pre-rendered page with a portal is taken over with no mismatch, and the block drawn where it goes", (t) => {
  const source = {
    "src/hosts.js": HOSTS,
    "src/App.wf": `page Home(path: "/", title: "H", description: "D") {
    state count = 0
    Heading("Taken over").h1
    Stack(ref: spot) { Text("spot") }
    Portal(into: spot) { Text("in the spot") }
    Stack(id: "side") { Text("side") }
    Portal(into: document.getElementById("side")) { Text("in the painted side") }
    Portal(into: editorHost("tip")) {
        Button("Count {count}") { on click { count = count + 1 } }
    }
    Text("after")
}
`,
  };
  const built = build(source, { ssg: true });
  if (!built) return t.skip("no compiler built");
  assert.ok(built.html.includes("<!--wf-portal-->"), "the paint has the portal's place");
  assert.ok(!built.html.includes("in the spot"), "and not its block");
  let heading;
  const page = run(built.bundle, {
    painted: built.html,
    dev: true,
    before: ({ window, document }) => {
      heading = document.querySelector("h1");
      const tip = document.createElement("div");
      document.body.appendChild(tip);
      window.hosts = { tip };
    },
  });
  const mismatch = page.warned.filter((w) => w.includes("differs"));
  assert.deepEqual(mismatch, [], "the page taken over is the page drawn");
  assert.equal(page.document.querySelector("h1"), heading, "the painted heading is kept");
  const spot = page.app.querySelector(".wf-stack");
  assert.ok(spot.textContent.includes("spot") && spot.textContent.includes("in the spot"), spot.textContent);
  // A portal that found a painted node — the page's own, read from the
  // document while the page was drawn beside it — keeps its box there,
  // and the painted node is the one kept.
  const side = page.document.getElementById("side");
  assert.ok(side.textContent.includes("in the painted side"), side.textContent);
  assert.equal(side.querySelectorAll(".wf-portal").length, 1);
  const tip = page.window.hosts.tip;
  click(buttonIn(tip, "Count 0"));
  page.drain();
  assert.ok(buttonIn(tip, "Count 1"), "the block in the host is live");
  assert.ok(page.app.textContent.includes("after"));
});

test("what a portal's block declares is read after it, as what a Stack's block declares is", (t) => {
  const built = build({
    "src/hosts.js": HOSTS,
    "src/App.wf": `page Home(path: "/", title: "H", description: "D") {
    state n = 3
    Heading("Declared").h1
    Portal(into: editorHost("tip")) {
        derived twice = n * 2
        Text("in {twice}")
    }
    Text("out {twice}")
}
`,
  });
  if (!built) return t.skip("no compiler built");
  // The block is written at the level the portal is: inside a JavaScript
  // block of its own, `twice` was not defined after it, and the page threw.
  const page = run(built.bundle, {
    before: ({ window, document }) => {
      const tip = document.createElement("div");
      document.body.appendChild(tip);
      window.hosts = { tip };
    },
  });
  assert.ok(page.app.textContent.includes("out 6"), page.app.textContent);
  assert.equal(page.window.hosts.tip.textContent, "in 6");
});

test("an item whose portal's block throws is one item: the rest of the page draws and mounts", async (t) => {
  const built = build({
    "src/hosts.js": HOSTS + `
/**
 * A row's text, which throws for one row as bad data would.
 * @param {string} id
 * @returns {string}
 */
function label(id) {
  if (id === "b") throw new Error("a row that cannot be drawn");
  return "row " + id;
}

/**
 * What a mount saw.
 * @param {HTMLElement} node
 * @returns {number}
 */
function sawMount(node) {
  window.seen = (window.seen || []).concat([node.className]);
  return 1;
}
`,
    "src/App.wf": `page Home(path: "/", title: "H", description: "D") {
    state rows = [{ id: "a" }, { id: "b" }, { id: "c" }]
    Heading("Rows").h1
    for r in rows by r.id {
        Portal(into: editorHost(r.id)) {
            Host(mount: (n) => sawMount(n), class: "in-{r.id}")
            Text(label(r.id))
        }
    }
    Host(mount: (n) => sawMount(n), class: "after")
}
`,
  });
  if (!built) return t.skip("no compiler built");
  const page = run(built.bundle, {
    quiet: true,
    before: ({ window, document }) => {
      window.hosts = {};
      for (const id of ["a", "b", "c"]) {
        const el = document.createElement("div");
        document.body.appendChild(el);
        window.hosts[id] = el;
      }
    },
  });
  await Promise.resolve();
  const { hosts } = page.window;
  assert.equal(hosts.a.textContent, "row a");
  assert.equal(hosts.c.textContent, "row c", "the rows after it draw");
  assert.ok(page.errors.some((e) => e.includes("could not be drawn")), page.errors.join("\n"));
  const seen = (page.window.seen || []).map((c) => c.split(" ").find((n) => n !== "wf-host"));
  for (const which of ["in-a", "in-c", "after"]) {
    assert.ok(seen.includes(which), `${which} mounted: ${seen}`);
  }
});

test("a portal into a node of the page's own that the takeover puts a drawn one in place of follows it", async (t) => {
  const built = build({
    "src/App.wf": `page Home(path: "/", title: "H", description: "D") {
    state n = 0
    Heading("Taken over").h1
    Stack(id: "side") {
        on click { n = n + 1 }
        Text("side")
    }
    Portal(into: document.getElementById("side")) { Text("in the side") }
    Text("after {n}")
}
`,
  }, { ssg: true });
  if (!built) return t.skip("no compiler built");
  const page = run(built.bundle, { painted: built.html, dev: true });
  await Promise.resolve();
  // The painted `#side` is what `into:` found while the page was drawn
  // beside it; the takeover puts the drawn one, which its handler holds,
  // in its place, and once the drawing is over the box goes there.
  const side = page.document.getElementById("side");
  assert.ok(side.textContent.includes("in the side"), side.textContent);
  assert.equal(page.document.querySelectorAll(".wf-portal").length, 1);
  assert.deepEqual(page.warned.filter((w) => w.includes("differs")), []);
});

test("a single page finds its own element by id once it is in the document, as a pre-rendered one does", async (t) => {
  const source = {
    "src/App.wf": `page Home(path: "/", title: "H", description: "D") {
    state open = false
    Heading("Found").h1
    Stack(id: "side") { Text("side") }
    Portal(into: document.getElementById("side")) { Text("in the side") }
    Button("Open") { on click { open = true } }
    if open {
        Stack(id: "later") { Text("later") }
        Portal(into: document.getElementById("later")) { Text("in the later one") }
    }
}
`,
  };
  for (const ssg of [false, true]) {
    const built = build(source, { ssg });
    if (!built) return t.skip("no compiler built");
    const page = run(built.bundle, ssg ? { painted: built.html } : {});
    // The page is drawn apart from the document, where a lookup finds
    // nothing; once it is in, where the portal goes is read again.
    await Promise.resolve();
    const side = page.document.getElementById("side");
    assert.ok(side.textContent.includes("in the side"), `ssg ${ssg}: ${side.textContent}`);
    click(buttonIn(page.app, "Open"));
    page.drain();
    await Promise.resolve();
    const later = page.document.getElementById("later");
    assert.ok(later.textContent.includes("in the later one"), `ssg ${ssg}: ${later.textContent}`);
  }
});
