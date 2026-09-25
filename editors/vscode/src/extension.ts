import * as fs from "fs";
import { ExtensionContext, ProgressLocation, window, workspace } from "vscode";
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import { fetchLatest, inFolders, latestRelease, newestDownloaded, onPath } from "./server";

let client: LanguageClient | undefined;

/**
 * Where the language server is, in this order: the `webfluent.server.path`
 * setting; `wf-lsp` on PATH; one built in the workspace (the compiler's own
 * checkout); and otherwise the latest release's, downloaded once and kept.
 */
async function findServer(context: ExtensionContext): Promise<string | undefined> {
  const configured = workspace.getConfiguration("webfluent").get<string>("server.path")?.trim();
  if (configured) {
    if (fs.existsSync(configured)) return configured;
    window.showErrorMessage(`WebFluent: \`webfluent.server.path\` names ${configured}, which does not exist.`);
    return undefined;
  }

  const local =
    onPath() ?? inFolders((workspace.workspaceFolders ?? []).map((f) => f.uri.fsPath));
  if (local) return local;

  if (!workspace.getConfiguration("webfluent").get<boolean>("server.download", true)) {
    window.showWarningMessage(
      "WebFluent: `wf-lsp` is not on PATH and downloading it is turned off " +
        "(`webfluent.server.download`). Highlighting works; diagnostics and completion need the server.",
    );
    return undefined;
  }

  const storage = context.globalStorageUri.fsPath;
  try {
    const release = await latestRelease();
    return await window.withProgress(
      { location: ProgressLocation.Window, title: `WebFluent: fetching wf-lsp ${release.tag}` },
      () => fetchLatest(storage, release),
    );
  } catch (error) {
    // No network, a rate limit: a server downloaded before still serves.
    const cached = newestDownloaded(storage);
    if (cached) return cached;
    window.showWarningMessage(
      `WebFluent: could not get wf-lsp (${(error as Error).message}). ` +
        "Highlighting works; for diagnostics and completion, install wf-lsp on PATH or set `webfluent.server.path`.",
    );
    return undefined;
  }
}

export async function activate(context: ExtensionContext): Promise<void> {
  const serverPath = await findServer(context);
  if (!serverPath) return;

  const run: Executable = { command: serverPath, args: [] };
  const serverOptions: ServerOptions = { run, debug: run };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "webfluent" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.{wf,wfx}"),
    },
  };

  client = new LanguageClient("webfluent", "WebFluent Language Server", serverOptions, clientOptions);
  await client.start();
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}
