use crate::config::ProjectConfig;
use crate::diagnostics::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// What the dev server knows about the build: how many times it has
/// rebuilt, and the error the last build stopped on, if any. A page polls
/// it and reloads on a new build, or shows the error over itself.
#[derive(Default)]
struct DevState {
    version: u64,
    error: Option<String>,
    /// Every finding of the last build — warnings too — as the overlay and
    /// `/__wf/status` read them.
    diagnostics: Vec<crate::diagnostics::error::Diagnostic>,
    /// Whether the project frames a page of its own (a `Frame` with a
    /// site-relative `src:`), so its pages are served `SAMEORIGIN` rather
    /// than `DENY`, as the built `_headers` asks.
    frames_itself: bool,
}

impl DevState {
    /// What a build that just ran leaves behind.
    fn record(&mut self, root: &Path, outcome: Result<()>) {
        // A source that does not read keeps the answer the last one gave.
        if let Ok((program, _)) = crate::build::read_project(root) {
            self.frames_itself = crate::codegen::csp::frames_itself(&program);
        }
        match outcome {
            Ok(()) => {
                self.version += 1;
                self.error = None;
                self.diagnostics.clear();
            }
            Err(e) => {
                // A build's findings were printed as they were found.
                if !matches!(e, crate::diagnostics::error::WebFluentError::Diagnostics(_)) {
                    eprintln!("{e}");
                }
                self.diagnostics = e.diagnostics();
                self.error = Some(e.to_string());
            }
        }
    }
}

/// The script every served page loads in development: it polls the
/// build's state, reloads the page when a build lands, and draws the
/// error of a build that failed over the page until the next one passes.
/// A file of its own, since the site's policy allows no inline script.
const DEV_SCRIPT: &str = r##"(() => {
  let version = null;
  let overlay = null;
  function show(error, findings) {
    if (!overlay) {
      overlay = document.createElement("div");
      overlay.id = "wf-error-overlay";
      overlay.setAttribute("role", "alert");
      const s = overlay.style;
      s.position = "fixed"; s.inset = "0"; s.zIndex = "2147483647"; s.background = "rgba(17, 24, 39, 0.96)";
      s.color = "#f9fafb"; s.font = "14px/1.5 ui-monospace, SFMono-Regular, Menlo, monospace"; s.padding = "32px"; s.overflow = "auto";
      const head = document.createElement("div");
      head.textContent = "wf serve — the build failed";
      head.style.color = "#fca5a5"; head.style.fontWeight = "600"; head.style.marginBottom = "16px"; head.style.fontSize = "16px";
      const pre = document.createElement("pre");
      pre.style.whiteSpace = "pre-wrap"; pre.style.margin = "0";
      const list = document.createElement("ol");
      list.style.listStyle = "none"; list.style.padding = "0"; list.style.margin = "0";
      const foot = document.createElement("div");
      foot.textContent = "Fix the source and save; the page reloads when the build passes.";
      foot.style.color = "#9ca3af"; foot.style.marginTop = "16px";
      overlay.append(head, list, pre, foot);
      document.body.appendChild(overlay);
    }
    const list = overlay.querySelector("ol");
    const pre = overlay.querySelector("pre");
    list.textContent = "";
    pre.textContent = "";
    if (!findings || findings.length === 0) { pre.textContent = error; return; }
    // One entry a finding: its code and message, where it is, and what to
    // do — errors first, the warnings the same build found after them.
    const order = (d) => (d.severity === "error" ? 0 : 1);
    for (const d of findings.slice().sort((a, b) => order(a) - order(b))) {
      const item = document.createElement("li");
      item.style.marginBottom = "18px";
      const title = document.createElement("div");
      title.style.fontWeight = "600";
      title.style.color = d.severity === "error" ? "#fca5a5" : "#fcd34d";
      title.textContent = d.severity + (d.code ? "[" + d.code + "]" : "") + ": " + d.message;
      const where = document.createElement("div");
      where.style.color = "#93c5fd";
      where.textContent = d.file + ":" + d.line + ":" + d.column;
      item.append(title, where);
      if (d.hint) {
        const help = document.createElement("div");
        help.style.color = "#d1d5db";
        help.textContent = d.hint;
        item.append(help);
      }
      list.append(item);
    }
  }
  function hide() { if (overlay) { overlay.remove(); overlay = null; } }

  // ─── Stores ─────────────────────────────────────────
  //
  // A store is a signal nobody can see. This panel shows what each one
  // holds, what every action did to it, and lets a step be put back — the
  // three things that make a bug in shared state findable rather than
  // guessed at.
  const log = [];
  let panel = null, list = null, tree = null;
  function css(node, text) { node.style.cssText = text; }
  function json(value) {
    try { return JSON.stringify(value); } catch (e) { return String(value); }
  }
  function changed(before, after) {
    if (!before) return "";
    return Object.keys(after || {})
      .filter((k) => json(before[k]) !== json(after[k]))
      .map((k) => k + ": " + json(before[k]) + " \u2192 " + json(after[k]))
      .join(", ");
  }
  function drawTree() {
    const held = window.WF.storeSnapshot();
    tree.textContent = "";
    const names = Object.keys(held);
    if (!names.length) {
      const empty = document.createElement("div");
      css(empty, "color:#9ca3af");
      empty.textContent = "No store has been read yet.";
      tree.appendChild(empty);
      return;
    }
    for (const name of names) {
      const row = document.createElement("div");
      css(row, "margin-bottom:6px");
      const title = document.createElement("b");
      title.textContent = name;
      title.style.color = "#93c5fd";
      const body = document.createElement("pre");
      css(body, "margin:2px 0 0;white-space:pre-wrap;color:#e5e7eb");
      try { body.textContent = JSON.stringify(held[name], null, 2); }
      catch (e) { body.textContent = String(held[name]); }
      row.append(title, body);
      tree.appendChild(row);
    }
  }
  function drawLog() {
    list.textContent = "";
    for (const entry of log.slice(-60).reverse()) {
      const row = document.createElement("button");
      css(row, "display:block;width:100%;text-align:left;background:none;border:0;border-bottom:1px solid #374151;color:inherit;font:inherit;padding:6px 0;cursor:pointer");
      const head = document.createElement("div");
      head.textContent = entry.action
        ? entry.store + "." + entry.action + "(" + entry.args.map(json).join(", ") + ")"
        : entry.store + " \u2014 built";
      head.style.color = "#a7f3d0";
      const diff = document.createElement("div");
      diff.style.color = "#9ca3af";
      diff.textContent = changed(entry.before, entry.after) || "nothing changed";
      row.append(head, diff);
      // Time travel: put the store back to what it held before this ran.
      row.title = entry.before ? "Put the store back to before this action" : "";
      row.onclick = () => {
        if (!entry.before) return;
        window.WF.restoreStore(entry.store, entry.before);
        drawTree();
      };
      list.appendChild(row);
    }
  }
  function devtools() {
    if (!window.WF || typeof window.WF.watchStores !== "function") return;
    const open = document.createElement("button");
    open.textContent = "stores";
    css(open, "position:fixed;right:12px;bottom:12px;z-index:2147483646;background:#111827;color:#e5e7eb;border:1px solid #374151;border-radius:6px;padding:4px 10px;font:12px ui-monospace,Menlo,monospace;cursor:pointer");
    panel = document.createElement("div");
    css(panel, "position:fixed;right:12px;bottom:48px;width:360px;max-height:60vh;overflow:auto;z-index:2147483646;background:#111827;color:#e5e7eb;border:1px solid #374151;border-radius:8px;padding:12px;font:12px/1.5 ui-monospace,Menlo,monospace;display:none");
    const head = document.createElement("div");
    head.textContent = "Stores \u2014 click an action to put it back";
    css(head, "color:#9ca3af;margin-bottom:8px");
    tree = document.createElement("div");
    list = document.createElement("div");
    css(list, "margin-top:10px;border-top:1px solid #374151");
    panel.append(head, tree, list);
    open.onclick = () => {
      const showing = panel.style.display === "block";
      panel.style.display = showing ? "none" : "block";
      if (!showing) { drawTree(); drawLog(); }
    };
    document.body.append(open, panel);
    window.WF.watchStores((entry) => {
      log.push(entry);
      if (panel.style.display === "block") { drawTree(); drawLog(); }
    });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", devtools);
  } else {
    devtools();
  }
  async function poll() {
    try {
      const r = await fetch("/__wf/status", { cache: "no-store" });
      const state = await r.json();
      if (version === null) version = state.version;
      if (state.error) show(state.error, state.diagnostics);
      else if (state.version !== version) location.reload();
      else hide();
    } catch (e) { /* the server is restarting */ }
    setTimeout(poll, 700);
  }
  poll();
})();
"##;

/// How many requests the dev server answers at once: one slow client — a
/// phone on a weak network, a stalled download — no longer holds up every
/// other request behind it.
const WORKERS: usize = 8;

/// The most compressed text the dev server keeps, in bytes, so a page
/// asked for again is not compressed again.
const GZIP_CACHE_BYTES: usize = 32 * 1024 * 1024;

/// A text response larger than this is sent as it is, streamed, rather
/// than read whole to be compressed.
const GZIP_LIMIT: u64 = 8 * 1024 * 1024;

/// What every worker of the dev server shares.
struct Shared {
    state: std::sync::Arc<std::sync::Mutex<DevState>>,
    output_dir: PathBuf,
    base_path: String,
    hot_reload: bool,
    /// The address it listens on, as given: what a `Host` may name.
    host: String,
    /// What was compressed, by file and its change time, so it is
    /// compressed once.
    gzipped: std::sync::Mutex<GzipCache>,
}

/// What a file is, to the cache: where it is, when it last changed and how
/// long it is, and — for a page with the dev script in it — the build.
type GzipKey = (PathBuf, Option<std::time::SystemTime>, u64, Option<u64>);

#[derive(Default)]
struct GzipCache {
    held: std::collections::HashMap<GzipKey, std::sync::Arc<Vec<u8>>>,
    bytes: usize,
}

impl Shared {
    fn state(&self) -> std::sync::MutexGuard<'_, DevState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// `content`, the text of `file` (with the dev script when `dev_page`),
    /// compressed — from the cache when the file has not changed since.
    fn gzip(&self, file: &Path, dev_page: bool, content: &[u8]) -> std::sync::Arc<Vec<u8>> {
        let meta = fs::metadata(file).ok();
        // A dev page is drawn again on every build: the build is its key.
        let key: GzipKey = (
            file.to_path_buf(),
            meta.as_ref().and_then(|m| m.modified().ok()),
            meta.as_ref().map_or(0, std::fs::Metadata::len),
            dev_page.then(|| self.state().version),
        );
        if let Some(gz) = self
            .gzipped
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .held
            .get(&key)
        {
            return std::sync::Arc::clone(gz);
        }
        // Compressed outside the lock, so other requests go on meanwhile.
        let gz = std::sync::Arc::new(crate::codegen::gzip::gzip(content));
        let mut cache = self
            .gzipped
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if cache.bytes + gz.len() > GZIP_CACHE_BYTES {
            cache.held.clear();
            cache.bytes = 0;
        }
        cache.bytes += gz.len();
        if let Some(old) = cache.held.insert(key, std::sync::Arc::clone(&gz)) {
            cache.bytes = cache.bytes.saturating_sub(old.len());
        }
        gz
    }
}

pub(crate) fn run_serve(project_dir: &Path, host: &str, allow_output_outside: bool) -> Result<()> {
    let config = ProjectConfig::load(project_dir)?;
    let output_dir =
        crate::build::project_output(project_dir, &config.build.output, allow_output_outside)?;
    let build_options = crate::cli::build::Options {
        dev: true,
        cache: true,
        allow_output_outside,
        ..Default::default()
    };

    // A build first, so what is served is what the source says; a build
    // that fails leaves the last output in place and its error on show.
    let state = std::sync::Arc::new(std::sync::Mutex::new(DevState::default()));
    {
        let mut s = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        s.record(
            project_dir,
            crate::cli::build::run_build_opts(project_dir, build_options),
        );
        // The first build is version 0, whether it passed or not.
        s.version = 0;
    }
    if !output_dir.exists() {
        return Err(crate::diagnostics::error::WebFluentError::io(format!(
            "nothing to serve: the build failed and {} does not exist",
            output_dir.display()
        )));
    }

    let port = config.dev.port;
    let ip: std::net::IpAddr = host.parse().map_err(|_| {
        crate::diagnostics::error::WebFluentError::io(format!(
            "`--host {host}` is not an address to listen on: give one such as 127.0.0.1 or 0.0.0.0"
        ))
    })?;
    let addr = std::net::SocketAddr::new(ip, port);

    // A site built for a sub-path (`build.base_path`, as on a GitHub Pages
    // project site) is served under that path, as its host will serve it:
    // its absolute links then resolve, and a visit to `/` is sent there.
    let base_path = config.build.base_path.trim_end_matches('/').to_string();

    let server = tiny_http::Server::http(addr).map_err(|e| {
        crate::diagnostics::error::WebFluentError::io(format!("Failed to start server: {e}"))
    })?;
    let port = server.server_addr().to_ip().map_or(port, |a| a.port());
    let shown = if ip.is_loopback() || ip.is_unspecified() {
        "localhost".to_string()
    } else if ip.is_ipv6() {
        format!("[{ip}]")
    } else {
        ip.to_string()
    };
    println!("Starting dev server at http://{shown}:{port}{base_path}/");
    if ip.is_unspecified() {
        match lan_address() {
            Some(lan) => println!("  On your network: http://{lan}:{port}{base_path}/"),
            None => println!("  Listening on every interface of this machine"),
        }
    }

    // Rebuild when a source file changes: the tree's newest change time is
    // read every half second, which needs no watcher and misses nothing.
    if config.dev.hot_reload {
        let root = project_dir.to_path_buf();
        let state = std::sync::Arc::clone(&state);
        std::thread::spawn(move || {
            let mut last = newest_change(&root);
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let now = newest_change(&root);
                if now == last {
                    continue;
                }
                last = now;
                println!("Change detected, rebuilding...");
                let outcome = crate::cli::build::run_build_opts(&root, build_options);
                state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .record(&root, outcome);
            }
        });
    }

    let shared = std::sync::Arc::new(Shared {
        state,
        output_dir,
        base_path,
        hot_reload: config.dev.hot_reload,
        host: host.to_string(),
        gzipped: std::sync::Mutex::default(),
    });
    let server = std::sync::Arc::new(server);
    let workers: Vec<_> = (0..WORKERS)
        .map(|_| {
            let server = std::sync::Arc::clone(&server);
            let shared = std::sync::Arc::clone(&shared);
            std::thread::spawn(move || {
                while let Ok(request) = server.recv() {
                    answer(&shared, request);
                }
            })
        })
        .collect();
    for worker in workers {
        let _ = worker.join();
    }
    Ok(())
}

/// This machine's address on its network, as a phone beside it would reach
/// it: the one a datagram to the outside would leave from. Nothing is sent.
fn lan_address() -> Option<std::net::IpAddr> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("192.0.2.1:9").ok()?;
    socket.local_addr().ok().map(|a| a.ip())
}

/// A header, built from a name and a value that are known to be fine.
fn header(name: &str, value: &str) -> Option<tiny_http::Header> {
    tiny_http::Header::from_bytes(name, value).ok()
}

fn respond<R: std::io::Read + Send + 'static>(
    request: tiny_http::Request,
    mut response: tiny_http::Response<R>,
    headers: &[(&str, &str)],
) {
    for (name, value) in headers {
        if let Some(h) = header(name, value) {
            response.add_header(h);
        }
    }
    let _ = request.respond(response.boxed());
}

fn not_found(request: tiny_http::Request) {
    respond(
        request,
        tiny_http::Response::from_string("Not Found").with_status_code(404),
        &[("Cache-Control", "no-store")],
    );
}

/// Answer one request.
fn answer(shared: &Shared, request: tiny_http::Request) {
    // A page on another site may point a name it owns at this machine, and
    // then read what is served here as its own (DNS rebinding): only a
    // name this server is reached by is answered.
    let host = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Host"))
        .map(|h| h.value.as_str().to_string());
    if !super::preview::host_allowed(host.as_deref(), &shared.host) {
        eprintln!(
            "  421 {}: refused a request for another host, {}",
            printable(request.url()),
            printable(host.as_deref().unwrap_or(""))
        );
        respond(
            request,
            tiny_http::Response::from_string("Misdirected Request").with_status_code(421),
            &[("Cache-Control", "no-store")],
        );
        return;
    }
    let base_path = shared.base_path.as_str();
    let url = request.url().to_string();
    // The query string names nothing on disk.
    let url = url.split('?').next().unwrap_or("").to_string();
    let url = if base_path.is_empty() {
        url
    } else if let Some(rest) = url.strip_prefix(base_path)
        && (rest.is_empty() || rest.starts_with('/'))
    {
        if rest.is_empty() {
            "/".to_string()
        } else {
            rest.to_string()
        }
    } else if url.starts_with("/__wf/") {
        url
    } else {
        let location = format!("{base_path}{url}");
        respond(
            request,
            tiny_http::Response::empty(302),
            &[("Location", &location)],
        );
        return;
    };
    // The dev server's own routes.
    if url == "/__wf/status" {
        let body = {
            let s = shared.state();
            serde_json::json!({
                "version": s.version,
                "error": s.error,
                "diagnostics": s.diagnostics,
            })
            .to_string()
        };
        respond(
            request,
            tiny_http::Response::from_string(body),
            &[
                ("Content-Type", "application/json"),
                ("Cache-Control", "no-store"),
            ],
        );
        return;
    }
    if url == "/__wf/mode.js" {
        respond(
            request,
            tiny_http::Response::from_string("window.__WF_DEV__ = true;\n"),
            &[("Content-Type", "application/javascript")],
        );
        return;
    }
    if url == "/__wf/dev.js" {
        respond(
            request,
            tiny_http::Response::from_string(DEV_SCRIPT),
            &[("Content-Type", "application/javascript")],
        );
        return;
    }
    let url_path = if url == "/" { "/index.html" } else { &url };
    let accepts_gzip = request
        .headers()
        .iter()
        .any(|h| h.field.equiv("Accept-Encoding") && h.value.as_str().contains("gzip"));

    // The file the address names, inside the output and nowhere else: an
    // address that climbs out (`..`, plain or percent-encoded), names the
    // root, or reaches a link out of the output names nothing.
    let Some(mut file_path) = super::preview::resolve(&shared.output_dir, url_path) else {
        eprintln!(
            "  404 {} {}: the address leads outside the build",
            printable(&request.method().to_string()),
            printable(&url)
        );
        not_found(request);
        return;
    };
    // A static build writes each route as `<route>/index.html`, which is
    // what a host serves for `/<route>`.
    if file_path.is_dir() {
        file_path = file_path.join("index.html");
    }

    // A fetch or a file the build did not write: a 404, as a host would
    // send, and a line saying so. The shell used to answer it, so a
    // missing API handed the page HTML to parse and nothing said why.
    if !file_path.is_file() && url == "/favicon.ico" {
        let _ = request.respond(tiny_http::Response::empty(204));
        return;
    }
    if !file_path.is_file() && !super::preview::is_page_request(&url, request.headers()) {
        eprintln!(
            "  404 {} {}: no file in the build answers it, and a fetch is not a page (an API the dev server does not serve?)",
            printable(&request.method().to_string()),
            printable(&url)
        );
        not_found(request);
        return;
    }
    let served = if file_path.is_file() {
        file_path.clone()
    } else {
        // A static build's catch-all page, else the SPA entry for all routes.
        let not_found = shared.output_dir.join("404.html");
        let index_path = shared.output_dir.join("index.html");
        if not_found.is_file() {
            not_found
        } else if index_path.is_file() {
            index_path
        } else {
            respond(
                request,
                tiny_http::Response::from_string("Not Found"),
                &[("Content-Type", "text/plain")],
            );
            return;
        }
    };
    let content_type = guess_content_type(&served);

    // The same headers the built output asks a host for, so a policy problem
    // surfaces in development rather than on the deployed site. A site that
    // frames its own pages lets its own origin do so.
    let frame_options = if shared.state().frames_itself {
        "SAMEORIGIN"
    } else {
        "DENY"
    };
    let common = [
        ("Vary", "Accept-Encoding"),
        ("X-Content-Type-Options", "nosniff"),
        ("Referrer-Policy", "strict-origin-when-cross-origin"),
        ("X-Frame-Options", frame_options),
    ];
    let with = |extra: &[(&'static str, &'static str)]| -> Vec<(&str, &str)> {
        let mut all: Vec<(&str, &str)> = vec![("Content-Type", content_type)];
        all.extend_from_slice(extra);
        all.extend_from_slice(&common);
        all
    };

    // A page carries the dev script, which reloads it on a new build and
    // shows a failed build's error over it.
    let dev_page = shared.hot_reload && content_type.starts_with("text/html");
    let size = fs::metadata(&served).map_or(0, |m| m.len());
    let compress = accepts_gzip && is_text(content_type);

    // What a host does: send the `.gz` the build wrote when the browser
    // takes it, or compress a text response on the way out, so the bytes
    // measured here are the bytes a deployed site sends.
    if compress && !dev_page && served == file_path {
        let prebuilt = PathBuf::from(format!("{}.gz", served.display()));
        if let Ok(gz) = fs::File::open(&prebuilt) {
            respond(
                request,
                tiny_http::Response::from_file(gz),
                &with(&[("Content-Encoding", "gzip")]),
            );
            return;
        }
    }
    // Anything that is not rewritten or compressed goes out as it is read,
    // never whole in memory.
    if !dev_page && (!compress || !(1024..=GZIP_LIMIT).contains(&size)) {
        match fs::File::open(&served) {
            Ok(file) => respond(request, tiny_http::Response::from_file(file), &with(&[])),
            Err(_) => not_found(request),
        }
        return;
    }
    let content = fs::read(&served).unwrap_or_default();
    let content = if dev_page {
        inject_dev_script(content)
    } else {
        content
    };
    if compress && content.len() >= 1024 {
        let gz = shared.gzip(&served, dev_page, &content);
        respond(
            request,
            tiny_http::Response::from_data(gz.as_slice().to_vec()),
            &with(&[("Content-Encoding", "gzip")]),
        );
    } else {
        respond(request, tiny_http::Response::from_data(content), &with(&[]));
    }
}

/// `text` as it may be printed to a terminal: a control character — an
/// escape sequence a request smuggled in — is written as its code.
pub(crate) fn printable(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}

/// The page with `<script src="/__wf/dev.js">` before `</body>`.
///
/// And, first thing in the head, `/__wf/mode.js`, which tells the runtime it
/// is under the dev server before the bundle runs — so a list item that
/// throws or a route nothing answers is shown on the page, not only logged.
fn inject_dev_script(content: Vec<u8>) -> Vec<u8> {
    let Ok(mut text) = String::from_utf8(content) else {
        return Vec::new();
    };
    let mode = "<script src=\"/__wf/mode.js\"></script>";
    match text.find("<head>") {
        Some(at) => text.insert_str(at + "<head>".len(), mode),
        None => text.insert_str(0, mode),
    }
    let tag = "<script src=\"/__wf/dev.js\" defer></script>\n";
    match text.rfind("</body>") {
        Some(at) => format!("{}{}{}", &text[..at], tag, &text[at..]).into_bytes(),
        None => format!("{text}{tag}").into_bytes(),
    }
}

/// The newest change time under `src/`, `public/` and the config.
///
/// The walk enters no directory twice, so a link that loops (`src/a -> .`)
/// cannot run it forever, and follows no link out of the project.
fn newest_change(root: &Path) -> Option<std::time::SystemTime> {
    fn walk(
        root: &Path,
        dir: &Path,
        seen: &mut std::collections::HashSet<PathBuf>,
        newest: &mut Option<std::time::SystemTime>,
    ) {
        match fs::canonicalize(dir) {
            Ok(real) if real.starts_with(root) && seen.insert(real.clone()) => {}
            _ => return,
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let linked = kind.is_symlink();
            if linked && fs::canonicalize(&path).map_or(true, |real| !real.starts_with(root)) {
                continue;
            }
            if path.is_dir() {
                walk(root, &path, seen, newest);
            } else if let Ok(meta) = fs::metadata(&path)
                && let Ok(modified) = meta.modified()
                && newest.is_none_or(|n| modified > n)
            {
                *newest = Some(modified);
            }
        }
    }
    let real_root = fs::canonicalize(root).ok()?;
    let mut newest = None;
    let mut seen = std::collections::HashSet::new();
    walk(&real_root, &root.join("src"), &mut seen, &mut newest);
    walk(&real_root, &root.join("public"), &mut seen, &mut newest);
    if let Ok(meta) = fs::metadata(root.join("webfluent.app.json"))
        && let Ok(modified) = meta.modified()
        && newest.is_none_or(|n| modified > n)
    {
        newest = Some(modified);
    }
    newest
}

/// Whether a response of this type is worth compressing.
fn is_text(content_type: &str) -> bool {
    content_type.starts_with("text/")
        || matches!(
            content_type.split(';').next().unwrap_or(""),
            "application/javascript" | "application/json" | "image/svg+xml" | "application/xml"
        )
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("gz") => "application/gzip",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_is_compressed_once_until_it_changes() {
        let dir = std::env::temp_dir().join(format!("wf-servegz-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("app.js");
        std::fs::write(&file, "x".repeat(4096)).unwrap();
        let shared = Shared {
            state: std::sync::Arc::default(),
            output_dir: dir.clone(),
            base_path: String::new(),
            hot_reload: true,
            host: "127.0.0.1".into(),
            gzipped: std::sync::Mutex::default(),
        };
        let first = shared.gzip(&file, false, &std::fs::read(&file).unwrap());
        let again = shared.gzip(&file, false, b"not what is on disk");
        assert!(std::sync::Arc::ptr_eq(&first, &again), "compressed twice");
        // A changed file is compressed again.
        std::fs::write(&file, "y".repeat(5000)).unwrap();
        let changed = shared.gzip(&file, false, &std::fs::read(&file).unwrap());
        assert!(!std::sync::Arc::ptr_eq(&first, &changed));
        // A page with the dev script is the build's: a new build, a new one.
        let page = shared.gzip(&file, true, b"<p>one</p>");
        assert!(std::sync::Arc::ptr_eq(
            &page,
            &shared.gzip(&file, true, b"<p>one</p>")
        ));
        shared.state().version += 1;
        assert!(!std::sync::Arc::ptr_eq(
            &page,
            &shared.gzip(&file, true, b"<p>two</p>")
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_control_character_is_printed_as_its_code() {
        assert_eq!(printable("/a\u{1b}[2Jb\u{7}"), "/a\\u{1b}[2Jb\\u{7}");
        assert_eq!(printable("/ünïcode"), "/ünïcode");
    }
}
