//! A program's names never take the generated code's own.
//!
//! Run: node --test tests/js/names.test.mjs (`tests/program_names.rs` builds
//! `wf` and runs it as part of `cargo test`).
//!
//! The generated code names its own things too: a component's props `_p`, a
//! handler's event `e`, a bind's value `v`, a store's `store`, the router's
//! `routes`. A program that called something by one of those names — a
//! `derived p`, a loop item `e` — used to compile to a script that did not
//! parse, or that read the generated code's value in place of its own. So
//! every name the generated code uses is put here in every place a program
//! can bind a name, each place a component that says `ok` when it reads its
//! own value and `bad` when it does not. Each project is built, run against
//! the fake DOM, clicked through and read back.
//!
//! A name the language refuses where it is written (`E118`, `params`) is
//! not a clash: that place is left out for it, and the rest still run.

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { makeDom } from "./dom.mjs";

/// The names the generated code declares or binds for itself, found by
/// building every fixture and listing what its output names that its
/// source does not, and by reading the code generator's templates.
const NAMES = [
  // Its own, beside a program's signals.
  "p", "slots", "frag", "root", "app", "routes", "routerEl", "error", "wfForm", "ev", "cf", "s",
  "idx", "n", "e1", "e2", "x_check", "x_pending",
  // The parameters of the functions it wraps around a program's code.
  "e", "v", "event", "store", "page", "old", "r", "a", "b",
  // What it declares at the top of the script.
  "container", "routes", "env", "WF", "document", "encodeURIComponent",
  // What a live binding and a component are called.
  "value", "item", "arm", "key", "Component_Box", "Page_Home",
  // The parameters a handler or a connection's handler gets unnamed.
  "message", "reason",
  // JavaScript's own words, and the globals a script cannot declare again.
  "class", "new", "delete", "default", "this", "function", "var", "typeof", "void", "with",
  "switch", "case", "do", "super", "export", "import", "extends", "debugger", "yield", "static",
  "arguments", "eval", "instanceof", "package", "private", "protected", "public", "implements",
  "interface", "undefined", "NaN", "Infinity",
  "top", "location", "window", "globalThis", "console", "Array", "Number", "Object", "String",
];

/// Each place a program binds a name: a component of its own that shows
/// `P<k> ok` when it reads back what it bound, and `P<k> bad …` when it
/// does not. `go P<k>`, where there is one, is clicked before reading.
/// `@` is the name.
const PLACES = [
  // A page's state, read and written.
  ["state", `component P1 {
    state @ = 1
    Text(if @ == 2 { "P1 ok" } else { "P1 bad {@}" })
    Button("go P1") { on click { @ = @ + 1 } }
}`],
  // A derived value beside a prop, as the editor's PullPage had it.
  ["derived", `component P2(label: String = "L") {
    state base = 2
    derived @ = base * 2
    Text(if @ == 4 && label == "L" { "P2 ok" } else { "P2 bad {@} {label}" })
}`],
  // A loop's item: read, handed to a handler, bound, and in a link.
  ["loop item", `component P3 {
    state rows = [{ id: "a", done: false }, { id: "b", done: false }]
    state picked = ""
    derived left = rows.filter(r2 => !r2.done).length
    for @ in rows by @.id {
        Button("pick P3 {@.id}") { on click { picked = @.id } }
        Checkbox(bind: @.done, label: "done P3 {@.id}")
    }
    Text(if picked == "b" && left == 1 { "P3 ok" } else { "P3 bad {picked} {left}" })
}`, ["pick P3 b", "done P3 a"]],
  // A loop's index, and an unkeyed loop.
  ["loop index", `component P4 {
    state rows = ["x", "y"]
    state seen = ""
    for it, @ in rows { Button("idx P4 {@}") { on click { seen = "{@}{it}" } } }
    Text(if seen == "1y" { "P4 ok" } else { "P4 bad {seen}" })
}`, ["idx P4 1"]],
  // A lambda's parameter.
  ["lambda", `component P5 {
    state rows = [{ id: "a" }, { id: "b" }]
    derived ids = rows.map(@ => @.id).join(",")
    Text(if ids == "a,b" { "P5 ok" } else { "P5 bad {ids}" })
}`],
  // An action's local.
  ["let", `component P6 {
    state out = 0
    action go() { let @ = 5  out = @ + 1 }
    Text(if out == 6 { "P6 ok" } else { "P6 bad {out}" })
    Button("go P6") { on click { go() } }
}`],
  // A caught error, named; and a nameless catch beside a state of the name.
  ["catch", `component P7 {
    state out = ""
    action go() { try { let bad = JSON.parse("{") } catch @ { out = "caught {@ != null}" } }
    Text(if out == "caught true" { "P7 ok" } else { "P7 bad {out}" })
    Button("go P7") { on click { go() } }
}`],
  ["nameless catch", `component P8 {
    state @ = "none"
    action go() { try { let bad = JSON.parse("{") } catch { @ = "caught" } }
    Text(if @ == "caught" { "P8 ok" } else { "P8 bad {@}" })
    Button("go P8") { on click { go() } }
}`],
  // An if-let binding.
  ["if let", `component P9 {
    state maybe: String? = "here"
    if let @ = maybe { Text(if @ == "here" { "P9 ok" } else { "P9 bad {@}" }) }
}`],
  // An action of the name, called from a handler.
  ["action", `component P10 {
    state out = 0
    action @() { out = out + 1 }
    Text(if out == 1 { "P10 ok" } else { "P10 bad {out}" })
    Button("go P10") { on click { @() } }
}`],
  // A handler's parameter.
  ["handler parameter", `component P11 {
    state out = ""
    Button("go P11") { on click(@) { out = @.type } }
    Text(if out == "click" { "P11 ok" } else { "P11 bad {out}" })
}`],
  // A state the handler reads, beside the handler's own event.
  ["read in a handler", `component P12 {
    state @ = 41
    state out = 0
    Button("go P12") { on click { out = @ + 1 } }
    Text(if out == 42 { "P12 ok" } else { "P12 bad {out}" })
}`],
  // An unkeyed loop's item — a plain name, not an accessor — read in a
  // link's address and in a bound field.
  ["loop item in a link", `component P13 {
    state rows = [{ id: "a", name: "A" }]
    for @ in rows {
        Input(bind: @.name, label: "name P13")
        Link("to P13 {@.id}", to: "/p13/{@.id}")
    }
    Text(if rows[0]?.name == "Z" { "P13 ok" } else { "P13 bad {rows[0]?.name}" })
}`, [{ type: "name P13", value: "Z" }]],
  // A scoped slot's fill, which reads the caller's state of the name.
  ["slot fill", `component P14Box {
    slot row(label: String)
    row(label: "in")
}
component P14 {
    state @ = "caller"
    P14Box { row(x) { Text(if x == "in" && @ == "caller" { "P14 ok" } else { "P14 bad {x} {@}" }) } }
}`],
  // A component's event, handled by a state of the name.
  ["component event", `component P15Box {
    event pick(n2: Number)
    Button("go P15") { on click { emit pick(7) } }
}
component P15 {
    state @ = 0
    P15Box { on pick(got) { @ = got } }
    Text(if @ == 7 { "P15 ok" } else { "P15 bad {@}" })
}`],
  // A prop of the name.
  ["prop", `component P16Box(@: String) {
    Text(if @ == "given" { "P16 ok" } else { "P16 bad {@}" })
}
component P16 { P16Box(@: "given") }`],
  // A store's state, derived value, lambda and action local.
  ["store", `store P17Store {
    state @ = 1
    state rows = [1, 2, 3]
    derived big = rows.filter(@ => @ > 1).length
    action go() { let @ = 2  @ = @ + 1 }
}
component P17 {
    use P17Store
    Text(if P17Store.big == 2 { "P17 ok" } else { "P17 bad {P17Store.big}" })
}`],
  ["store member read in a lambda", `store P18Store {
    state @ = 1
    state rows = [1, 2, 3]
    derived above = rows.filter(r2 => r2 > @).length
    action go() { @ = 2 }
}
component P18 {
    use P18Store
    Text(if P18Store.above == 1 { "P18 ok" } else { "P18 bad {P18Store.above}" })
    Button("go P18") { on click { P18Store.go() } }
}`],
  // A constant of the name, at the top of the script.
  ["const", `const @ = "konst"
component P19 { Text(if @ == "konst" { "P19 ok" } else { "P19 bad {@}" }) }`],
  // An enum payload bound by a match arm.
  ["match payload", `enum P20Tone { calm, loud(level: Number) }
component P20 {
    state tone: P20Tone = .loud(3)
    match tone {
        .loud(@) { Text(if @ == 3 { "P20 ok" } else { "P20 bad {@}" }) }
        else { Text("P20 bad calm") }
    }
}`],
  // A validated state.
  ["validated", `component P21 {
    state @ = ""
    validate @ { required }
    Input(bind: @, label: "field P21")
    Text(if @ == "" { "P21 ok" } else { "P21 bad {@}" })
}`],
  // An action's parameter.
  ["action parameter", `component P23 {
    state out = 0
    action go(@: Number) { out = @ + 1 }
    Text(if out == 2 { "P23 ok" } else { "P23 bad {out}" })
    Button("go P23") { on click { go(1) } }
}`],
  // A form's handle.
  ["form handle", `component P24 {
    state field = "x"
    Form(bind: @) {
        Input(bind: field, label: "form P24")
        Text(if @.valid { "P24 ok" } else { "P24 bad" })
    }
}`],
  // A store action's parameter, read in a loop of the action.
  ["store action parameter", `store P25Store {
    state total = 0
    action add(@: Number) { for k in [1, 2] { total = total + @ * k } }
}
component P25 {
    use P25Store
    Text(if P25Store.total == 6 { "P25 ok" } else { "P25 bad {P25Store.total}" })
    Button("go P25") { on click { P25Store.add(2) } }
}`],
  // An unkeyed loop's item whose field a date picker and a slider are
  // bound to: the handlers the generated code writes around the write read
  // the item.
  ["loop item bound to a date", `type P26Row { id: String, day: Date? = null }
component P26 {
    state rows: [P26Row] = [P26Row(id: "a")]
    derived shown = "{rows[0]?.day ?? "none"}"
    for @ in rows { DatePicker(bind: @.day, label: "day P26") }
    Text(if shown == "2026-03-14" { "P26 ok" } else { "P26 bad {shown}" })
}`, [{ change: "day P26", value: "2026-03-14" }]],
  ["loop item bound to a slider", `component P27 {
    state rows = [{ id: "a", n: 1 }]
    for @ in rows { Slider(bind: @.n, min: 0, max: 10, label: "slide P27") }
    Text(if rows[0]?.n == 7 { "P27 ok" } else { "P27 bad {rows[0]?.n}" })
}`, [{ type: "slide P27", value: "7" }]],
  // An effect and a timer that write a state of the name.
  ["effect", `component P22 {
    state @ = 0
    state seen = 0
    effect { seen = @ }
    Button("go P22") { on click { @ = 5 } }
    Text(if seen == 5 { "P22 ok" } else { "P22 bad {seen}" })
}`],
];

/// The `wf` to build with: `WF_BIN`, or the one this checkout built last.
function wfBinary() {
  if (process.env.WF_BIN) return process.env.WF_BIN;
  const built = ["release", "debug"]
    .map((p) => fileURLToPath(new URL(`../../target/${p}/wf`, import.meta.url)))
    .filter((p) => existsSync(p));
  built.sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs);
  return built[0] || null;
}

/// Each place of `PLACES` for `name`, in a file of its own so a place the
/// language refuses can be left out by its file.
function sources(name) {
  const files = {};
  for (const [, src] of PLACES) files[`src/components/${tagOf(src)}.wf`] = src.replaceAll("@", name);
  return files;
}

/// The component a place is: the one called `P<k>` and nothing more.
function tagOf(src) {
  return [...src.matchAll(/component (P\d+)\b/g)].pop()[1];
}

function writeProject(dir, files) {
  const kept = Object.keys(files).map((f) => f.match(/P(\d+)\.wf$/)[1]);
  mkdirSync(join(dir, "src/components"), { recursive: true });
  writeFileSync(join(dir, "webfluent.app.json"), JSON.stringify({ name: "Names", build: { output: "./build", ssg: false, minify: false, split: false } }));
  writeFileSync(
    join(dir, "src/App.wf"),
    `app { Router }\npage Home(path: "/", title: "Names", description: "Every name.") {\n    Heading("Names").h1\n${kept.map((k) => `    P${k}`).join("\n")}\n}\npage P13To(path: "/p13/:id", title: "To", description: "d", id: String) { Heading("arrived {id}").h1 }\n`,
  );
  for (const [file, src] of Object.entries(files)) writeFileSync(join(dir, file), src);
}

/// Build `name` in every place; the places the language refuses for it are
/// dropped and the rest built again. What a refusal is, is `E118` — any
/// other error is a failure of the test.
function build(name) {
  const wf = wfBinary();
  const dir = mkdtempSync(join(tmpdir(), "wf-names-"));
  let files = sources(name);
  const refused = [];
  for (;;) {
    rmSync(join(dir, "src"), { recursive: true, force: true });
    writeProject(dir, files);
    try {
      execFileSync(wf, ["build", "-d", dir], { stdio: "pipe", env: { ...process.env, NO_COLOR: "1" } });
      return { dir, refused };
    } catch (err) {
      const out = `${err.stdout}${err.stderr}`;
      const lines = out.split("\n");
      // Each error and the file it is in.
      const found = [];
      lines.forEach((line, i) => {
        const code = line.match(/^error\[(\w+)\]/);
        if (!code) return;
        const at = (lines[i + 1] || "").match(/components\/(P\d+)\.wf/);
        found.push({ code: code[1], file: at ? at[1] : null, line });
      });
      // A file the language refuses the name in is left out, with whatever
      // else that refusal brought with it there.
      const dropped = new Set(found.filter((f) => f.code === "E118" && f.file).map((f) => f.file));
      const errors = found.filter((f) => !dropped.has(f.file)).map((f) => f.line);
      if (errors.length || dropped.size === 0) {
        rmSync(dir, { recursive: true, force: true });
        throw new Error(`\`${name}\` did not build:\n${out}`);
      }
      for (const p of dropped) {
        delete files[`src/components/${p}.wf`];
        refused.push(p);
      }
    }
  }
}

function mount(dir) {
  let src = readFileSync(join(dir, "build/app.js"), "utf8");
  const pages = join(dir, "build/pages");
  if (existsSync(pages)) {
    for (const file of readdirSync(pages).sort()) {
      if (file.endsWith(".js")) src += "\n" + readFileSync(join(pages, file), "utf8");
    }
  }
  const { window, document, Node, Element, DocumentFragment } = makeDom();
  const app = document.createElement("div");
  app.id = "app";
  document.body.appendChild(app);
  window.location.pathname = "/";
  const timers = [];
  window.setTimeout = (fn) => { timers.push(fn); return timers.length; };
  window.clearTimeout = () => {};
  const errors = [];
  const report = { ...console, error: (...a) => errors.push(a.join(" ")) };
  const run = new Function(
    "window", "document", "Node", "Element", "DocumentFragment", "setTimeout", "clearTimeout", "URLSearchParams", "console",
    `${src}\nreturn typeof WF !== "undefined" ? WF : null;`,
  );
  globalThis.__wfNamesWindow = window;
  run(window, document, Node, Element, DocumentFragment, window.setTimeout, window.clearTimeout, URLSearchParams, report);
  const drain = () => { while (timers.length) timers.shift()(); };
  drain();
  return { app, drain, errors };
}

function control(app, label) {
  const button = app.querySelectorAll("button").find((b) => b.textContent.trim() === label);
  if (button) return button;
  // A checkbox, by the text of the label around it or beside it.
  const lab = app.querySelectorAll("label").find((l) => l.textContent.trim() === label);
  if (lab) {
    return lab.querySelectorAll("input")[0]
      || app.querySelectorAll("input").find((i) => i.id && i.id === lab.getAttribute("for"))
      || lab.parentNode?.querySelectorAll("input")[0];
  }
  return null;
}

for (const name of [...new Set(NAMES)]) {
  test(`\`${name}\` is the program's own in every place it can be bound`, () => {
    if (!wfBinary()) return;
    const { dir, refused } = build(name);
    try {
      const { app, drain, errors } = mount(dir);
      const wrong = [];
      PLACES.forEach(([place, src, clicks = []]) => {
        const tag = tagOf(src);
        if (refused.includes(tag)) return;
        try {
          for (const step of [...clicks, `go ${tag}`]) {
            const label = typeof step === "string" ? step : step.type ?? step.change;
            const el = control(app, label);
            if (!el && label === `go ${tag}`) continue;
            if (!el) throw new Error(`nothing is called "${label}"`);
            if (typeof step !== "string") {
              el.value = step.value;
              // What a reader does: types, then leaves the field, which
              // commits it (an unkeyed loop's field writes back then).
              for (const kind of step.change ? ["change"] : ["input", "change"]) {
                el.dispatchEvent({ type: kind, target: el, currentTarget: el, preventDefault() {} });
              }
            } else if (el.tagName === "INPUT") {
              el.checked = !el.checked;
              el.dispatchEvent({ type: "change", target: el, currentTarget: el, preventDefault() {} });
            } else {
              el.dispatchEvent({ type: "click", target: el, currentTarget: el, preventDefault() {} });
            }
            drain();
          }
        } catch (err) {
          wrong.push(`as a ${place}: ${err.message}`);
          return;
        }
        const shown = app.textContent;
        if (!shown.includes(`${tag} ok`)) {
          wrong.push(`as a ${place}: ${(shown.match(new RegExp(`${tag} bad.*?(?=P\\d|go |$)`)) || ["nothing drawn"])[0]}`);
        }
      });
      // Last, as it leaves the page: a link in a loop goes where its item
      // says, not where the click event would.
      if (!refused.includes("P13")) {
        const link = app.querySelectorAll("a").find((a) => a.textContent.trim() === "to P13 a");
        if (!link) wrong.push("as a loop item in a link: no link");
        else {
          link.dispatchEvent({ type: "click", target: link, currentTarget: link, button: 0, preventDefault() {} });
          drain();
          if (!app.textContent.includes("arrived a")) wrong.push(`as a loop item in a link: went to ${globalThis.__wfNamesWindow?.location?.pathname}`);
        }
      }
      if (errors.length) wrong.push(`the page reported: ${errors.join("; ")}`);
      assert.deepEqual(wrong, [], `\`${name}\` is read as something else`);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
}
