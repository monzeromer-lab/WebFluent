// The live pieces: the compiler, in the page, when the reader asks for it.
//
// `/wasm/<release>/wf.js` is imported on the first edit (or on intent:
// hovering or focusing an Edit button), and it fetches the WebAssembly
// build then. Nothing here runs on page load. What a build found and wrote
// goes back to the page as a plain object, handed to one of its actions.

const wfLive = { module: null, timers: new Map(), titles: new Map(), slots: 0 };

/**
 * The compiler's module, imported once.
 * @param {string} base where the compiler lives, `/wasm/<release>/`
 */
function wfModule(base) {
  wfLive.module ??= import(base + "wf.js");
  return wfLive.module;
}

/**
 * Start loading the compiler before the first keystroke: on a hover or a
 * focus of Edit, or when an editor opens.
 * @param {string} base
 */
function wfWarm(base) {
  wfModule(base).then((m) => m.preload()).catch(() => {});
}

/**
 * A name for one preview on this page, so two examples never serve each
 * other's builds.
 * @returns {string}
 */
function wfSlot() {
  wfLive.slots += 1;
  return "p" + Date.now().toString(36) + wfLive.slots;
}

function wfEscape(text) {
  return text.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

function wfPaint(text, language) {
  // `WF` is the runtime's top-level `const`, not a property of `window`.
  const highlight = typeof WF !== "undefined" && typeof WF.highlight === "function" ? WF.highlight : null;
  return (highlight ? highlight(text, language) : wfEscape(text)) + "\n";
}

/**
 * An editor in a node: a text area over the same text, highlighted. Tab
 * indents; Escape, then Tab, leaves it.
 * @param {HTMLElement} node
 * @param {string} text what it starts with
 * @param {string} label its accessible name
 * @param {string} language how to highlight it: `wf`, `json`, `css`
 * @param {(text: string) => void} changed called with the text after every edit
 */
function wfEditor(node, text, label, language, changed) {
  const box = document.createElement("div");
  box.className = "bp-live";
  box.dir = "ltr";
  const paint = document.createElement("pre");
  paint.className = "bp-live__paint";
  paint.setAttribute("aria-hidden", "true");
  const area = document.createElement("textarea");
  area.className = "bp-live__input";
  area.spellcheck = false;
  area.setAttribute("autocapitalize", "off");
  area.setAttribute("autocomplete", "off");
  area.setAttribute("aria-label", label);
  area.value = text;
  let lang = language || "wf";
  let leaving = false;
  const redraw = () => { paint.innerHTML = wfPaint(area.value, lang); };
  area.addEventListener("input", () => { redraw(); changed(area.value); });
  area.addEventListener("keydown", (e) => {
    if (e.key === "Escape") { leaving = true; return; }
    if (e.key === "Tab" && !leaving && !e.shiftKey && !e.metaKey && !e.ctrlKey && !e.altKey) {
      e.preventDefault();
      document.execCommand("insertText", false, "    ");
    }
    leaving = false;
  });
  box.append(paint, area);
  node.replaceChildren(box);
  redraw();
  return {
    /** Show another text (a file switched to, a mistake put in, a fix applied). */
    set(next, nextLanguage) {
      if (nextLanguage && nextLanguage !== lang) { lang = nextLanguage; redraw(); }
      if (next !== area.value) { area.value = next; redraw(); }
    },
    focus() { area.focus(); },
  };
}

/** The finding as `wf` prints it, from what `wf check --format json` lists. */
function wfPrinted(d, file, source) {
  const out = [`${d.severity}[${d.code}]: ${d.message}`];
  if (d.line > 0) {
    const n = String(d.line);
    const pad = " ".repeat(n.length);
    const text = (source ?? "").split("\n")[d.line - 1] ?? "";
    const width = d.end_line === d.line && d.end_column > d.column ? d.end_column - d.column : 1;
    out.push(`${pad}--> ${file}:${d.line}:${d.column}`, `${pad} |`, `${n} | ${text}`,
      `${pad} | ${" ".repeat(Math.max(0, d.column - 1))}${"^".repeat(width)}`);
    if (d.hint) out.push(`${pad} = help: ${d.hint}`);
  } else if (d.hint) {
    out.push(`  = help: ${d.hint}`);
  }
  return out.join("\n");
}

/** The text of `file` with a fix's edits made, or "" when it has none there. */
function wfFixed(source, fix, file) {
  const edits = (fix?.edits ?? []).filter((e) => e.file.replace(/^\/?wf-project\//, "") === file);
  if (!edits.length || typeof source !== "string") return "";
  const starts = [0];
  for (let i = 0; i < source.length; i++) if (source[i] === "\n") starts.push(i + 1);
  const at = (line, column) => (starts[line - 1] ?? source.length) + column - 1;
  let text = source;
  for (const e of edits.slice().sort((a, b) => at(b.line, b.column) - at(a.line, a.column))) {
    const end = e.end_line > 0 ? at(e.end_line, e.end_column) : at(e.line, e.column);
    text = text.slice(0, at(e.line, e.column)) + e.text + text.slice(end);
  }
  return text;
}

async function wfTitle(wf, code) {
  if (!wfLive.titles.has(code)) wfLive.titles.set(code, wf.explain(code).then((x) => x?.title ?? "").catch(() => ""));
  return wfLive.titles.get(code);
}

/**
 * A project's config asking for a PDF: the same project, as a document
 * (a slide deck stays one).
 * @param {string} config the `webfluent.app.json` text
 * @returns {string}
 */
function wfAsDocument(config) {
  try {
    const object = JSON.parse(config || "{}");
    object.build = object.build ?? {};
    if (object.build.output_type !== "slides") object.build.output_type = "pdf";
    return JSON.stringify(object);
  } catch (e) {
    return config;
  }
}

/** What the page shows of a build is read, so it is built unminified. */
function wfReadable(config) {
  try {
    const object = JSON.parse(config || "{}");
    object.build = { minify: false, compress: false, ...object.build };
    return JSON.stringify(object);
  } catch (e) {
    return config;
  }
}

async function wfRun(base, slot, files, given) {
  const wf = await wfModule(base);
  const config = wfReadable(given);
  const paged = wf.wantsDocument(files, config);
  await wf.preload({ document: paged });
  const started = performance.now();
  const out = await wf.compile(files, wf.previewConfig(config, slot));
  const ms = Math.max(1, Math.round(performance.now() - started));
  const findings = [];
  for (const d of out.diagnostics ?? []) {
    const file = (d.file ?? "").replace(/^\/?wf-project\//, "");
    findings.push({
      key: `${d.code}:${file}:${d.line}:${d.column}`,
      code: d.code,
      severity: d.severity,
      title: await wfTitle(wf, d.code),
      printed: wfPrinted(d, file, files[file]),
      to: /^[A-Z]+\d+$/.test(d.code) ? `/errors/${d.code}` : "",
      file,
      fix: d.fixes?.[0]?.title ?? "",
      fixed: wfFixed(files[file], d.fixes?.[0], file),
    });
  }
  if (!out.ok && out.error && !findings.some((f) => f.severity === "error")) {
    findings.push({ key: "error", code: "", severity: "error", title: "", printed: out.error, to: "", file: "", fix: "", fixed: "" });
  }
  const written = Object.keys(out.files ?? {});
  const pdf = Object.keys(out.binary ?? {}).find((p) => p.endsWith(".pdf")) ?? "";
  if (out.ok) await wf.preview(out, "", slot);
  const chunks = written.filter((p) => p.startsWith("pages/") && p.endsWith(".js")).sort();
  return {
    ok: out.ok,
    ms,
    slot,
    errors: findings.filter((f) => f.severity === "error").length,
    warnings: findings.filter((f) => f.severity !== "error").length,
    findings,
    html: out.files?.["index.html"] ?? "",
    css: out.files?.["styles.css"] ?? "",
    js: chunks.length ? chunks.map((p) => `// ${p}\n${out.files[p]}`).join("\n\n") : out.files?.["app.js"] ?? "",
    pdf,
    document: paged,
  };
}

/**
 * Build a project in the page, a moment after the last edit, and hand what
 * came of it to `done`: `{ ok, ms, slot, errors, warnings, findings, html,
 * css, js, pdf, document }`, each finding `{ key, code, severity, title,
 * printed, to, file, fix, fixed }`. A build that compiles is published to
 * the preview worker under `/wasm/<release>/preview/<slot>/`; one that does
 * not leaves the last good one there.
 * @param {string} base `/wasm/<release>/`
 * @param {string} slot from `wfSlot()`
 * @param {Object} files path → text: `src/pages/Home.wf`, `webfluent.app.json`
 * @param {string} config the `webfluent.app.json` text
 * @param {(result: Object) => void} done
 * @param {number} [wait] milliseconds to wait for the next keystroke
 */
function wfBuild(base, slot, files, config, done, wait = 150) {
  clearTimeout(wfLive.timers.get(slot));
  wfLive.timers.set(slot, setTimeout(() => {
    wfRun(base, slot, files, config).then(done, (e) => done({
      ok: false, ms: 0, slot, errors: 1, warnings: 0, html: "", css: "", js: "", pdf: "", document: false,
      findings: [{ key: "load", code: "", severity: "error", title: "", printed: String(e?.message ?? e), to: "", file: "", fix: "", fixed: "" }],
    }));
  }, wait));
}

// ─── The Playground's project: shared in the address, downloaded as a zip ──

function wfBase64url(bytes) {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function wfUnbase64url(text) {
  const s = atob(text.replace(/-/g, "+").replace(/_/g, "/"));
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i);
  return out;
}

async function wfSqueeze(bytes, how) {
  const stream = new Blob([bytes]).stream().pipeThrough(how);
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

/**
 * The address of this page with the project in its hash, compressed — and
 * put on the clipboard. Nothing is sent anywhere.
 * @param {Object} files path → text
 * @returns {Promise<string>}
 */
async function wfShare(files) {
  const json = new TextEncoder().encode(JSON.stringify(files));
  const packed = await wfSqueeze(json, new CompressionStream("deflate-raw"));
  const url = location.origin + location.pathname + "#p=" + wfBase64url(packed);
  history.replaceState(null, "", url);
  try { await navigator.clipboard.writeText(url); } catch (e) { /* the address bar has it */ }
  return url;
}

/**
 * The project a shared address carries, handed to `found`; nothing when the
 * address has none, or one that cannot be read.
 * @param {(files: Object) => void} found
 */
function wfShared(found) {
  const m = /[#&]p=([A-Za-z0-9_-]+)/.exec(location.hash);
  if (!m) return;
  wfSqueeze(wfUnbase64url(m[1]), new DecompressionStream("deflate-raw"))
    .then((bytes) => {
      const files = JSON.parse(new TextDecoder().decode(bytes));
      if (files && typeof files === "object" && Object.values(files).every((v) => typeof v === "string")) found(files);
    })
    .catch(() => {});
}

const wfCrcTable = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function wfCrc(bytes) {
  let c = 0xffffffff;
  for (const b of bytes) c = wfCrcTable[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

/**
 * Save the project as a zip the reader can `wf build`: its files, stored
 * as they are, under a folder named `name`.
 * @param {string} name
 * @param {Object} files path → text
 */
function wfDownload(name, files) {
  const enc = new TextEncoder();
  const parts = [];
  const central = [];
  let offset = 0;
  const u16 = (v) => [v & 0xff, (v >>> 8) & 0xff];
  const u32 = (v) => [v & 0xff, (v >>> 8) & 0xff, (v >>> 16) & 0xff, (v >>> 24) & 0xff];
  for (const [path, text] of Object.entries(files)) {
    const fileName = enc.encode(`${name}/${path}`);
    const data = enc.encode(text);
    const crc = wfCrc(data);
    // version 20, flag 0x0800 (UTF-8 names), stored, no time
    const common = [...u16(20), ...u16(0x0800), ...u16(0), ...u16(0), ...u16(0x21), ...u32(crc), ...u32(data.length), ...u32(data.length), ...u16(fileName.length), ...u16(0)];
    const local = new Uint8Array([...u32(0x04034b50), ...common]);
    parts.push(local, fileName, data);
    central.push(new Uint8Array([...u32(0x02014b50), ...u16(20), ...common, ...u16(0), ...u16(0), ...u16(0), ...u32(0), ...u32(offset)]), fileName);
    offset += local.length + fileName.length + data.length;
  }
  const size = central.reduce((n, p) => n + p.length, 0);
  const count = Object.keys(files).length;
  const end = new Uint8Array([...u32(0x06054b50), ...u16(0), ...u16(0), ...u16(count), ...u16(count), ...u32(size), ...u32(offset), ...u16(0)]);
  const blob = new Blob([...parts, ...central, end], { type: "application/zip" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = `${name}.zip`;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}
