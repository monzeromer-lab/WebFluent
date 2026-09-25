// Finding `wf-lsp`, and fetching it when it is nowhere to be found.
//
// Nothing here imports `vscode`, so it runs — and is tested — in plain Node.
// The download mirrors the Zed extension (editors/zed/src/webfluent.rs): the
// latest GitHub release, the asset named as `.github/workflows/release.yml`
// names it, kept per release so a failed lookup still has a copy to serve.

import * as fs from "fs";
import * as https from "https";
import * as path from "path";
import { execFile } from "child_process";
import { promisify } from "util";

export const REPO = "monzeromer-lab/WebFluent";
const SERVER = "wf-lsp";
const USER_AGENT = "webfluent-vscode (https://github.com/monzeromer-lab/WebFluent)";

/** The server's file name on this platform. */
export function executableName(platform: NodeJS.Platform = process.platform): string {
  return platform === "win32" ? `${SERVER}.exe` : SERVER;
}

/**
 * The release asset built for a platform, or `undefined` where the release
 * builds none: x86_64 on Linux, macOS and Windows, and arm64 on macOS.
 */
export function assetName(
  tag: string,
  platform: NodeJS.Platform = process.platform,
  arch: string = process.arch,
): string | undefined {
  const os =
    platform === "linux" ? "linux" : platform === "darwin" ? "macos" : platform === "win32" ? "windows" : undefined;
  const cpu = arch === "x64" ? "x86_64" : arch === "arm64" ? "aarch64" : undefined;
  if (!os || !cpu || (cpu === "aarch64" && os !== "macos")) {
    return undefined;
  }
  return `${SERVER}-${tag}-${cpu}-${os}.${os === "windows" ? "zip" : "tar.gz"}`;
}

/** The server on `PATH`, as a shell would find it. */
export function onPath(
  envPath: string = process.env.PATH ?? "",
  platform: NodeJS.Platform = process.platform,
): string | undefined {
  const name = executableName(platform);
  for (const dir of envPath.split(path.delimiter)) {
    if (!dir) continue;
    const candidate = path.join(dir, name);
    if (isFile(candidate)) return candidate;
  }
  return undefined;
}

/** A server built in one of these folders — the compiler's own checkout. */
export function inFolders(folders: string[], platform: NodeJS.Platform = process.platform): string | undefined {
  const name = executableName(platform);
  for (const root of folders) {
    for (const profile of ["release", "debug"]) {
      const candidate = path.join(root, "target", profile, name);
      if (isFile(candidate)) return candidate;
    }
  }
  return undefined;
}

function isFile(p: string): boolean {
  try {
    return fs.statSync(p).isFile();
  } catch {
    return false;
  }
}

/** A GET that follows GitHub's redirects to where a release asset lives. */
export function get(url: string, redirects = 5): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const request = https.get(url, { headers: { "User-Agent": USER_AGENT, Accept: "*/*" } }, (res) => {
      const status = res.statusCode ?? 0;
      if (status >= 300 && status < 400 && res.headers.location) {
        res.resume();
        if (redirects <= 0) return reject(new Error(`too many redirects fetching ${url}`));
        return resolve(get(new URL(res.headers.location, url).toString(), redirects - 1));
      }
      if (status !== 200) {
        res.resume();
        return reject(new Error(`${url} answered ${status}`));
      }
      const chunks: Buffer[] = [];
      res.on("data", (c: Buffer) => chunks.push(c));
      res.on("end", () => resolve(Buffer.concat(chunks)));
      res.on("error", reject);
    });
    request.setTimeout(60_000, () => request.destroy(new Error(`timed out fetching ${url}`)));
    request.on("error", reject);
  });
}

export interface Release {
  tag: string;
  assets: { name: string; url: string }[];
}

export async function latestRelease(repo: string = REPO): Promise<Release> {
  const body = await get(`https://api.github.com/repos/${repo}/releases/latest`);
  const json = JSON.parse(body.toString("utf8"));
  return {
    tag: json.tag_name,
    assets: (json.assets ?? []).map((a: { name: string; browser_download_url: string }) => ({
      name: a.name,
      url: a.browser_download_url,
    })),
  };
}

/** The server downloaded for `tag`, if an earlier start fetched it. */
export function downloadedFor(storage: string, tag: string): string | undefined {
  const candidate = path.join(storage, tag, executableName());
  return isFile(candidate) ? candidate : undefined;
}

/** The newest release downloaded before, for when the lookup fails. */
export function newestDownloaded(storage: string): string | undefined {
  let tags: string[];
  try {
    tags = fs.readdirSync(storage);
  } catch {
    return undefined;
  }
  const byVersion = (t: string) => t.replace(/^v/, "").split(".").map((n) => parseInt(n, 10) || 0);
  tags.sort((a, b) => {
    const [x, y] = [byVersion(a), byVersion(b)];
    for (let i = 0; i < Math.max(x.length, y.length); i++) {
      if ((x[i] ?? 0) !== (y[i] ?? 0)) return (y[i] ?? 0) - (x[i] ?? 0);
    }
    return 0;
  });
  for (const tag of tags) {
    const found = downloadedFor(storage, tag);
    if (found) return found;
  }
  return undefined;
}

/**
 * The server for the latest release: the copy already downloaded for it, or
 * a fresh download. `tar` unpacks both archive kinds — Windows has shipped
 * one that reads `.zip` since Windows 10.
 */
export async function fetchLatest(storage: string, release?: Release): Promise<string> {
  const latest = release ?? (await latestRelease());
  const have = downloadedFor(storage, latest.tag);
  if (have) return have;

  const name = assetName(latest.tag);
  if (!name) {
    throw new Error(
      `no wf-lsp is built for ${process.platform}/${process.arch}; build it with ` +
        "`cargo build --release -p wf-lsp` and set `webfluent.server.path`",
    );
  }
  const asset = latest.assets.find((a) => a.name === name);
  if (!asset) throw new Error(`release ${latest.tag} has no asset named ${name}`);

  const dir = path.join(storage, latest.tag);
  const partial = `${dir}.partial`;
  fs.rmSync(partial, { recursive: true, force: true });
  fs.mkdirSync(partial, { recursive: true });
  const archive = path.join(partial, name);
  fs.writeFileSync(archive, await get(asset.url));
  await promisify(execFile)("tar", ["-xf", archive, "-C", partial]);
  fs.rmSync(archive);
  const binary = path.join(partial, executableName());
  if (!isFile(binary)) throw new Error(`${name} did not contain ${executableName()}`);
  if (process.platform !== "win32") fs.chmodSync(binary, 0o755);
  // Moved into place whole, so an interrupted download never looks finished.
  fs.rmSync(dir, { recursive: true, force: true });
  fs.renameSync(partial, dir);
  return path.join(dir, executableName());
}
