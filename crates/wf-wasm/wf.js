// The WebFluent compiler in a page.
//
//   import { compile, check, explain } from "./wf.js";
//   const out = await compile({ "src/pages/Home.wf": "…", "public/a.png": bytes });
//   out.files["index.html"]   // text the build wrote
//   out.binary["img/a.….webp"] // Uint8Array: pictures, a PDF
//   out.diagnostics            // as `wf check --format json` lists them
//
// The compiler is loaded on the first call (`pkg/`, about 1.5 MB gzipped).
// A project whose config asks for a PDF or a slide deck is compiled by the
// build with the paged engine (`pkg-pdf/`), loaded then, with the fonts a
// document is set in fetched beside it — once each; the browser caches them.

const base = new URL(".", import.meta.url);

const FONTS = ["Sans", "Serif", "Mono"].flatMap((family) =>
  ["Regular", "Bold", "Italic", "BoldItalic"].map((style) => `Liberation${family}-${style}.ttf.z`));

// Faces a document falls back to for a script the Liberation faces lack,
// fetched the first time a document's text has a character of theirs.
const FALLBACKS = [
  { file: "NotoSansArabic.ttf.gz", script: /[\u0600-\u06FF\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/ },
];

let web = null;
let paged = null;
const fallbacks = new Map();

async function fetchFont(file) {
  const response = await fetch(new URL(`pkg-pdf/fonts/${file}`, base));
  if (!response.ok) throw new Error(`WebFluent: could not load the font ${file}`);
  return new Uint8Array(await response.arrayBuffer());
}

async function loadFallbacks(m, files) {
  const text = Object.values(files).filter((v) => typeof v === "string").join("\n");
  for (const { file, script } of FALLBACKS) {
    if (!script.test(text)) continue;
    if (!fallbacks.has(file)) fallbacks.set(file, fetchFont(file).then((bytes) => m.register_font(bytes)));
    await fallbacks.get(file);
  }
}

function loadWeb() {
  web ??= import(new URL("pkg/wf_wasm.js", base)).then(async (m) => {
    await m.default({ module_or_path: new URL("pkg/wf_wasm_bg.wasm", base) });
    return m;
  });
  return web;
}

function loadPaged() {
  paged ??= import(new URL("pkg-pdf/wf_wasm_pdf.js", base)).then(async (m) => {
    await m.default({ module_or_path: new URL("pkg-pdf/wf_wasm_pdf_bg.wasm", base) });
    const faces = await Promise.all(FONTS.map(fetchFont));
    for (const bytes of faces) m.register_font(bytes);
    return m;
  });
  return paged;
}

/** Whether `config` (or the project's own `webfluent.app.json`) asks for a document. */
export function wantsDocument(files, config) {
  const text = config ?? files["webfluent.app.json"];
  if (typeof text !== "string") return false;
  try {
    const type = JSON.parse(text)?.build?.output_type;
    return type === "pdf" || type === "slides";
  } catch {
    return false;
  }
}

function toBase64(bytes) {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) {
    s += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
  }
  return btoa(s);
}

function fromBase64(text) {
  const s = atob(text);
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i);
  return out;
}

function encode(files) {
  const out = {};
  for (const [path, value] of Object.entries(files)) {
    if (typeof value === "string") out[path] = value;
    else if (value instanceof ArrayBuffer) out[path] = { base64: toBase64(new Uint8Array(value)) };
    else if (ArrayBuffer.isView(value)) {
      out[path] = { base64: toBase64(new Uint8Array(value.buffer, value.byteOffset, value.byteLength)) };
    } else throw new TypeError(`WebFluent: ${path} is neither a string nor bytes`);
  }
  return JSON.stringify(out);
}

function decode(json) {
  const out = JSON.parse(json);
  for (const [path, b64] of Object.entries(out.binary ?? {})) out.binary[path] = fromBase64(b64);
  return out;
}

async function run(fn, files, config) {
  const document = wantsDocument(files, config);
  const m = document ? await loadPaged() : await loadWeb();
  if (document && fn === "compile") await loadFallbacks(m, files);
  return decode(m[fn](encode(files), config ?? null));
}

/**
 * Build a project. `files` maps a path under the project (`src/App.wf`,
 * `public/logo.png`) to its text, or to its bytes; `config` is the text of
 * `webfluent.app.json` (or put it in `files`). Resolves to
 * `{ ok, files, binary, diagnostics, error, lines }`.
 */
export const compile = (files, config) => run("compile", files, config);

/** Every finding, with nothing built: `wf check`. */
export const check = (files, config) => run("check", files, config);

/** What `wf explain CODE` says, or `null`. */
export async function explain(code) {
  return JSON.parse((await loadWeb()).explain(code));
}

/** The compiler's version. */
export async function version() {
  return (await loadWeb()).version();
}

/** Start loading ahead of the first edit; `document: true` loads the PDF build too. */
export function preload({ document = false } = {}) {
  return document ? Promise.all([loadWeb(), loadPaged()]) : loadWeb();
}

// ─── Preview ───────────────────────────────────────────────────────────

const previewScope = new URL("preview/", base);

/**
 * `config` (JSON text or an object) with `build.base_path` set to where
 * `preview()` serves a build, so its pages ask for their files there. A
 * `slot` (a name of letters, digits, `-` and `_`) keeps this build apart
 * from the others a page previews: each is served under `preview/<slot>/`.
 */
export function previewConfig(config, slot = "") {
  const object = typeof config === "string" ? JSON.parse(config) : structuredClone(config ?? {});
  object.name ??= "Playground";
  object.build = { ...object.build, base_path: slotScope(slot).pathname.replace(/\/$/, "") };
  return JSON.stringify(object);
}

function slotScope(slot) {
  if (slot && !/^[A-Za-z0-9_-]+$/.test(slot)) throw new TypeError(`WebFluent: "${slot}" is not a preview slot's name`);
  return slot ? new URL(`${slot}/`, previewScope) : previewScope;
}

let worker = null;

function previewWorker() {
  worker ??= navigator.serviceWorker
    .register(new URL("preview-sw.js", base), { scope: previewScope.href })
    .then((registration) => new Promise((resolve) => {
      const ready = () => registration.active && resolve(registration.active);
      if (registration.active) return ready();
      const installing = registration.installing ?? registration.waiting;
      installing?.addEventListener("statechange", ready);
    }));
  return worker;
}

/**
 * Serve what `compile()` built (with `previewConfig`) under `preview/`, as a
 * host would — every page, chunk and picture — and resolve to the address
 * to load in an iframe. With a `slot`, under `preview/<slot>/`, replacing
 * only what that slot served before. The iframe needs `allow-same-origin`
 * beside `allow-scripts` for the worker to answer it; give the preview an
 * origin of its own if the page around it holds anything worth protecting.
 */
export async function preview(out, route = "", slot = "") {
  const scope = slotScope(slot);
  const active = await previewWorker();
  const files = { ...out.files, ...out.binary };
  await new Promise((resolve) => {
    const channel = new MessageChannel();
    channel.port1.onmessage = resolve;
    active.postMessage({ type: "wf-publish", files, slot }, [channel.port2]);
  });
  return new URL(route.replace(/^\//, ""), scope).href;
}

/**
 * A face for documents to be set in or fall back to — a script the built-in
 * faces lack, a brand's family. TTF or OTF bytes, as they are, zlib or gzip.
 * Resolves to its family, or `""` when the bytes are not a font.
 */
export async function registerFont(bytes) {
  return (await loadPaged()).register_font(bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes));
}
