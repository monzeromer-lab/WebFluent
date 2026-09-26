"use strict";

const { execFileSync } = require("child_process");
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
function tmpFile(content, ext) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "wf-"));
  const file = path.join(dir, `data${ext}`);
  fs.writeFileSync(file, content);
  return file;
}

function cleanup(filepath) {
  try {
    fs.unlinkSync(filepath);
    fs.rmdirSync(path.dirname(filepath));
  } catch {}
}

class Template {
  /**
   * @param {string} source - The .wf template source code
   * @param {object} [options]
   * @param {string} [options.theme] - Which `theme` the template declares to
   *   render with; only needed when it declares more than one
   * @param {object} [options.tokens={}] - Design tokens over the theme's
   */
  constructor(source, options = {}) {
    this._source = source;
    this._theme = options.theme || null;
    this._tokens = options.tokens || {};
    this._templateFile = null;
  }

  /**
   * Create a Template from a .wf source string.
   * @param {string} source
   * @param {object} [options]
   * @returns {Template}
   */
  static fromString(source, options) {
    return new Template(source, options);
  }

  /**
   * Create a Template from a .wf file path.
   * @param {string} filePath
   * @param {object} [options]
   * @returns {Template}
   */
  static fromFile(filePath, options) {
    const tpl = new Template("", options);
    tpl._templateFile = path.resolve(filePath);
    return tpl;
  }

  /**
   * Render with one of the `theme` declarations the template makes.
   * @param {string} theme
   * @returns {Template}
   */
  withTheme(theme) {
    this._theme = theme;
    return this;
  }

  /**
   * Set custom design tokens.
   * @param {object} tokens - e.g. { "color-primary": "#8B5CF6" }
   * @returns {Template}
   */
  withTokens(tokens) {
    this._tokens = { ...this._tokens, ...tokens };
    return this;
  }

  /**
   * Render to a full HTML document string.
   * @param {object} data - JSON data context
   * @returns {string}
   */
  renderHtml(data) {
    return this._render(data, "html");
  }

  /**
   * Render to an HTML fragment string (no <html> wrapper).
   * @param {object} data - JSON data context
   * @returns {string}
   */
  renderHtmlFragment(data) {
    return this._render(data, "fragment");
  }

  /**
   * Render to a PDF Buffer.
   * @param {object} data - JSON data context
   * @returns {Buffer}
   */
  renderPdf(data) {
    return this._renderFile(data, "pdf");
  }

  /**
   * Render a `Presentation` to a PDF slide deck.
   * @param {object} data - JSON data context
   * @returns {Buffer}
   */
  renderSlides(data) {
    return this._renderFile(data, "slides");
  }

  /** @private */
  _renderFile(data, format) {
    const outFile = tmpFile("", ".pdf");
    try {
      this._render(data, format, outFile);
      return fs.readFileSync(outFile);
    } finally {
      cleanup(outFile);
    }
  }

  /** @private */
  _render(data, format, outputFile) {
    const tplFile = this._getTemplateFile();
    const dataFile = tmpFile(JSON.stringify(data === undefined ? {} : data), ".json");
    const args = ["render", tplFile, "--data", dataFile, "--format", format];
    if (this._theme) args.push("--theme", this._theme);
    for (const [name, value] of Object.entries(this._tokens)) {
      args.push("--token", `${name}=${value}`);
    }
    if (outputFile) args.push("-o", outputFile);
    try {
      const result = execFileSync(bin(), args, {
        encoding: "utf-8",
        maxBuffer: 50 * 1024 * 1024,
        stdio: ["pipe", "pipe", "pipe"],
      });
      return outputFile ? undefined : result;
    } catch (e) {
      // What `wf` said, not the child process's own report around it.
      const said = e.stderr ? String(e.stderr).trim() : "";
      throw new Error(said || e.message);
    } finally {
      cleanup(dataFile);
      if (!this._templateFile) cleanup(tplFile);
    }
  }

  /** @private */
  _getTemplateFile() {
    if (this._templateFile) return this._templateFile;
    const f = tmpFile(this._source, ".wf");
    return f;
  }
}

module.exports = { Template };
