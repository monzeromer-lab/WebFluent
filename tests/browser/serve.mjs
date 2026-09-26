// A build served as a static host would, for the browser suites: a
// directory is its `index.html`, a single-page build answers every route with
// its shell, and a `POST` is recorded rather than refused.
import fs from "node:fs";
import http from "node:http";
import path from "node:path";

const TYPES = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json" };

/// The build served as a static host would, and `POST /api/save` recorded.
export function host(root, port) {
  const received = [];
  const server = http.createServer((req, res) => {
    if (req.method === "POST") {
      let body = "";
      req.on("data", (c) => { body += c; });
      req.on("end", () => {
        received.push(`${req.url} ${body}`);
        res.writeHead(200, { "Content-Type": "application/json" }).end('{"ok":true}');
      });
      return;
    }
    let file = path.join(root, decodeURIComponent(new URL(req.url, "http://x").pathname));
    if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");
    if (!fs.existsSync(file)) {
      // A single-page build answers every route with its shell.
      const shell = path.join(root, "index.html");
      if (path.extname(file) === "" && fs.existsSync(shell)) file = shell;
      else return res.writeHead(404).end("not found");
    }
    res.writeHead(200, { "Content-Type": TYPES[path.extname(file)] || "application/octet-stream" });
    fs.createReadStream(file).pipe(res);
  });
  return {
    received,
    start: () => new Promise((r) => server.listen(port, "127.0.0.1", r)),
    stop: () => new Promise((r) => { server.closeAllConnections?.(); server.close(() => r()); }),
  };
}

