// The bundle VS Code loads: every module it needs is inside it, save `vscode`
// itself. The 0.2.0 package listed the language client as a dev dependency
// and left `node_modules` out, so it could not start — and nothing noticed,
// because nothing ever loaded what the package carried.

import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const bundle = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "out", "extension.js");

test("the bundle loads with nothing but `vscode` provided", { skip: !fs.existsSync(bundle) && "run `npm run build` first" }, () => {
  const require = createRequire(import.meta.url);
  const Module = require("module");
  const stub = new Proxy(function () {}, {
    get: (_, key) => (key === "__esModule" ? false : stub),
    apply: () => stub,
    construct: () => stub,
  });
  const resolve = Module._resolveFilename;
  Module._resolveFilename = function (request, ...rest) {
    return request === "vscode" ? "vscode" : resolve.call(this, request, ...rest);
  };
  require.cache.vscode = { id: "vscode", filename: "vscode", loaded: true, exports: stub };
  try {
    const extension = require(bundle);
    assert.equal(typeof extension.activate, "function");
    assert.equal(typeof extension.deactivate, "function");
  } finally {
    Module._resolveFilename = resolve;
  }
});
