import * as path from "path";
import * as fs from "fs";
import {
  workspace,
  ExtensionContext,
  window,
} from "vscode";
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

/**
 * Locate the `wf-lsp` binary. Search order:
 *   1. PATH (system-installed)
 *   2. Workspace root  -> target/release/wf-lsp
 *   3. Workspace root  -> target/debug/wf-lsp
 */
function findServerBinary(): string | undefined {
  // 1. Check PATH
  const envPath = process.env.PATH ?? "";
  const dirs = envPath.split(path.delimiter);
  for (const dir of dirs) {
    const candidate = path.join(dir, "wf-lsp");
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  // 2 & 3. Check workspace folders
  const folders = workspace.workspaceFolders;
  if (folders) {
    for (const folder of folders) {
      const root = folder.uri.fsPath;
      const release = path.join(root, "target", "release", "wf-lsp");
      if (fs.existsSync(release)) {
        return release;
      }
      const debug = path.join(root, "target", "debug", "wf-lsp");
      if (fs.existsSync(debug)) {
        return debug;
      }
    }
  }

  return undefined;
}

export async function activate(context: ExtensionContext): Promise<void> {
  const serverPath = findServerBinary();

  if (!serverPath) {
    window.showWarningMessage(
      "WebFluent: Could not find `wf-lsp` binary. " +
        "Install it or build it with `cargo build --release -p wf-lsp`, " +
        "then reload the window."
    );
    return;
  }

  const run: Executable = {
    command: serverPath,
    args: [],
  };

  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "webfluent" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.wf"),
    },
  };

  client = new LanguageClient(
    "webfluent",
    "WebFluent Language Server",
    serverOptions,
    clientOptions
  );

  await client.start();
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}
