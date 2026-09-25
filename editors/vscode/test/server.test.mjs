// `src/server.ts` in plain Node: finding `wf-lsp`, naming the release asset
// for each platform, and — with WF_NETWORK_TESTS=1 — downloading the latest
// release's server and talking LSP to it.

import { test } from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const here = path.dirname(fileURLToPath(import.meta.url));
const bundle = path.join(os.tmpdir(), `wf-vscode-server-${process.pid}.cjs`);
await build({
  entryPoints: [path.join(here, "..", "src", "server.ts")],
  bundle: true,
  platform: "node",
  format: "cjs",
  outfile: bundle,
  logLevel: "silent",
});
const server = createRequire(import.meta.url)(bundle);

const tmp = () => fs.mkdtempSync(path.join(os.tmpdir(), "wf-vscode-"));
const touch = (p) => {
  fs.mkdirSync(path.dirname(p), { recursive: true });
  fs.writeFileSync(p, "");
  return p;
};

test("the asset is named as release.yml names it, for every platform it builds", () => {
  const t = "v4.0.1";
  assert.equal(server.assetName(t, "linux", "x64"), "wf-lsp-v4.0.1-x86_64-linux.tar.gz");
  assert.equal(server.assetName(t, "darwin", "arm64"), "wf-lsp-v4.0.1-aarch64-macos.tar.gz");
  assert.equal(server.assetName(t, "darwin", "x64"), "wf-lsp-v4.0.1-x86_64-macos.tar.gz");
  assert.equal(server.assetName(t, "win32", "x64"), "wf-lsp-v4.0.1-x86_64-windows.zip");
});

test("no asset is invented for a platform the release does not build", () => {
  assert.equal(server.assetName("v4.0.1", "linux", "arm64"), undefined);
  assert.equal(server.assetName("v4.0.1", "win32", "arm64"), undefined);
  assert.equal(server.assetName("v4.0.1", "freebsd", "x64"), undefined);
});

test("on Windows the server is wf-lsp.exe — it used to look for wf-lsp and never find it", () => {
  const dir = tmp();
  touch(path.join(dir, "wf-lsp.exe"));
  assert.equal(server.onPath(dir, "win32"), path.join(dir, "wf-lsp.exe"));
  assert.equal(server.onPath(dir, "linux"), undefined);
});

test("PATH is searched in order, and a directory named wf-lsp is not the server", () => {
  const [a, b] = [tmp(), tmp()];
  fs.mkdirSync(path.join(a, "wf-lsp"));
  touch(path.join(b, "wf-lsp"));
  assert.equal(server.onPath([a, b].join(path.delimiter), "linux"), path.join(b, "wf-lsp"));
  assert.equal(server.onPath("", "linux"), undefined);
});

test("a server built in the workspace is found, release before debug", () => {
  const root = tmp();
  const debug = touch(path.join(root, "target", "debug", "wf-lsp"));
  assert.equal(server.inFolders([root], "linux"), debug);
  const release = touch(path.join(root, "target", "release", "wf-lsp"));
  assert.equal(server.inFolders([root], "linux"), release);
});

test("the newest download is picked by version, not by spelling", () => {
  const storage = tmp();
  for (const tag of ["v4.0.9", "v4.0.10", "v4.0.2"]) touch(path.join(storage, tag, server.executableName()));
  fs.mkdirSync(path.join(storage, "v4.1.0")); // a download that never finished
  assert.equal(server.newestDownloaded(storage), path.join(storage, "v4.0.10", server.executableName()));
  assert.equal(server.newestDownloaded(path.join(storage, "nowhere")), undefined);
});

// Speaks the first message of the protocol and waits for the answer.
function initialize(binary) {
  return new Promise((resolve, reject) => {
    const child = spawn(binary, [], { stdio: ["pipe", "pipe", "ignore"] });
    const body = JSON.stringify({ jsonrpc: "2.0", id: 1, method: "initialize", params: { processId: null, rootUri: null, capabilities: {} } });
    let out = "";
    const timer = setTimeout(() => {
      child.kill();
      reject(new Error("no answer to initialize"));
    }, 10_000);
    child.stdout.on("data", (d) => {
      out += d;
      const at = out.indexOf("\r\n\r\n");
      if (at >= 0) {
        const length = Number(/Content-Length: (\d+)/i.exec(out.slice(0, at))?.[1]);
        if (out.length >= at + 4 + length) {
          clearTimeout(timer);
          child.kill();
          resolve(JSON.parse(out.slice(at + 4, at + 4 + length)));
        }
      }
    });
    child.on("error", reject);
    child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  });
}

test("the latest release's server downloads, starts and answers initialize", { skip: !process.env.WF_NETWORK_TESTS }, async () => {
  const storage = tmp();
  const release = await server.latestRelease();
  const binary = await server.fetchLatest(storage, release);
  assert.equal(binary, path.join(storage, release.tag, server.executableName()));
  assert.ok(!fs.existsSync(`${path.dirname(binary)}.partial`), "the partial download was moved into place");

  const reply = await initialize(binary);
  assert.equal(reply.id, 1);
  assert.ok(reply.result?.capabilities, "the server announced its capabilities");

  // A second start serves the copy already downloaded.
  const mtime = fs.statSync(binary).mtimeMs;
  assert.equal(await server.fetchLatest(storage, release), binary);
  assert.equal(fs.statSync(binary).mtimeMs, mtime);
});
