"use strict";

const { execFileSync, spawn } = require("child_process");
const path = require("path");
const fs = require("fs");
const os = require("os");

/**
 * Find the `wf` binary. Checks:
 * 1. WF_BIN environment variable
 * 2. System PATH
 * 3. Common install locations
 */
function findBinary() {
  if (process.env.WF_BIN) return process.env.WF_BIN;

  // Try PATH
  try {
    const cmd = process.platform === "win32" ? "where" : "which";
    return execFileSync(cmd, ["wf"], { encoding: "utf-8" }).trim();
  } catch {}

  // Common locations
  const candidates = [
    // Where install.sh and install.ps1 put it.
    path.join(os.homedir(), ".webfluent", "bin", "wf"),
    path.join(os.homedir(), ".cargo", "bin", "wf"),
    "/usr/local/bin/wf",
    "/usr/bin/wf",
  ];
  if (process.platform === "win32") {
    candidates.push(path.join(os.homedir(), ".webfluent", "bin", "wf.exe"));
    candidates.push(path.join(os.homedir(), ".cargo", "bin", "wf.exe"));
  }
  for (const p of candidates) {
    if (fs.existsSync(p)) return p;
  }

  throw new Error(
    "WebFluent CLI (wf) not found. Install it with:\n" +
    "  curl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash\n" +
    "or: cargo install webfluent\n" +
    "Or set the WF_BIN environment variable to the binary path."
  );
}

let _bin = null;
function bin() {
  if (!_bin) _bin = findBinary();
  return _bin;
}

/**
 * Write data to a temp file and return the path.
 */
// A template given as a string is written to a file once, the first time
// it renders, and the file goes when the process does.
const written = new Set();
process.on("exit", () => {
  for (const file of written) {
    try {
      fs.rmSync(path.dirname(file), { recursive: true, force: true });
    } catch {}
  }
});

function writeSource(source) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-"));
  const file = path.join(dir, "template.wf");
  fs.writeFileSync(file, source);
  written.add(file);
  return file;
}

const FORMATS = { html: "html", fragment: "html-fragment", pdf: "pdf", slides: "slides" };

class Template {
  constructor(source, options = {}) {
    this._source = source;
    this._path = null;
    this._theme = options.theme || null;
    this._tokens = { ...(options.tokens || {}) };
    this._page = options.page || null;
    this._lang = options.lang || null;
  }

  /** A template from `.wf` source text. */
  static fromString(source, options) {
    return new Template(source, options);
  }

  /** A template from a `.wf` or `.wfx` file. */
  static fromFile(filePath, options) {
    const tpl = new Template(null, options);
    tpl._path = path.resolve(filePath);
    return tpl;
  }

  /**
   * Every `.wf` and `.wfx` file under a directory, as one template: the
   * components, themes and constants are shared, and `page(name)` picks a
   * page to render.
   */
  static fromDir(dirPath, options) {
    return Template.fromFile(dirPath, options);
  }

  /** The same template, rendering only the page called `name`. */
  page(name) {
    const copy = this._copy();
    copy._page = name;
    return copy;
  }

  withTheme(theme) {
    this._theme = theme;
    return this;
  }

  withTokens(tokens) {
    this._tokens = { ...this._tokens, ...tokens };
    return this;
  }

  /** The language `renderHtml` declares, `<html lang>`; `en` unless set. */
  withLang(lang) {
    this._lang = lang;
    return this;
  }

  renderHtml(data) {
    return this._run("html", data).toString("utf-8");
  }

  renderHtmlFragment(data) {
    return this._run("fragment", data).toString("utf-8");
  }

  renderPdf(data) {
    return this._run("pdf", data);
  }

  renderSlides(data) {
    return this._run("slides", data);
  }

  /** `renderHtml`, without blocking the event loop. */
  async renderHtmlAsync(data) {
    return (await this._runAsync("html", data)).toString("utf-8");
  }

  /** `renderHtmlFragment`, without blocking the event loop. */
  async renderHtmlFragmentAsync(data) {
    return (await this._runAsync("fragment", data)).toString("utf-8");
  }

  /** `renderPdf`, without blocking the event loop. */
  renderPdfAsync(data) {
    return this._runAsync("pdf", data);
  }

  /** `renderSlides`, without blocking the event loop. */
  renderSlidesAsync(data) {
    return this._runAsync("slides", data);
  }

  /** @private */
  _copy() {
    const copy = new Template(this._source, {
      theme: this._theme,
      tokens: this._tokens,
      page: this._page,
      lang: this._lang,
    });
    copy._path = this._path;
    return copy;
  }

  /** @private — the `wf render` arguments for `format`; the data goes on stdin. */
  _args(format) {
    if (!this._path) this._path = writeSource(this._source);
    const args = ["render", this._path, "--format", FORMATS[format]];
    if (this._theme) args.push("--theme", this._theme);
    if (this._page) args.push("--page", this._page);
    if (this._lang) args.push("--lang", this._lang);
    for (const [name, value] of Object.entries(this._tokens)) {
      args.push("--token", `${name}=${value}`);
    }
    return args;
  }

  /** @private */
  _run(format, data) {
    try {
      return execFileSync(bin(), this._args(format), {
        input: JSON.stringify(data === undefined ? {} : data),
        maxBuffer: 64 * 1024 * 1024,
        stdio: ["pipe", "pipe", "pipe"],
      });
    } catch (e) {
      throw failure(e.stderr, e.message);
    }
  }

  /** @private */
  _runAsync(format, data) {
    return new Promise((resolve, reject) => {
      let child;
      try {
        child = spawn(bin(), this._args(format), { stdio: ["pipe", "pipe", "pipe"] });
      } catch (e) {
        return reject(e);
      }
      const out = [];
      const err = [];
      child.stdout.on("data", (c) => out.push(c));
      child.stderr.on("data", (c) => err.push(c));
      child.on("error", reject);
      child.on("close", (code) => {
        if (code === 0) resolve(Buffer.concat(out));
        else reject(failure(Buffer.concat(err), `wf exited with code ${code}`));
      });
      child.stdin.end(JSON.stringify(data === undefined ? {} : data));
    });
  }
}

/** What `wf` said, not the child process's own report around it. */
function failure(stderr, fallback) {
  const said = stderr ? String(stderr).trim() : "";
  return new Error(said || fallback);
}

module.exports = { Template };
