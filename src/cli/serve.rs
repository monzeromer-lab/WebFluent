use crate::config::ProjectConfig;
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// What the dev server knows about the build: how many times it has
/// rebuilt, and the error the last build stopped on, if any. A page polls
/// it and reloads on a new build, or shows the error over itself.
#[derive(Default)]
struct DevState {
    version: u64,
    error: Option<String>,
}

/// The script every served page loads in development: it polls the
/// build's state, reloads the page when a build lands, and draws the
/// error of a build that failed over the page until the next one passes.
/// A file of its own, since the site's policy allows no inline script.
const DEV_SCRIPT: &str = r##"(() => {
  let version = null;
  let overlay = null;
  function show(error) {
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
      const foot = document.createElement("div");
      foot.textContent = "Fix the source and save; the page reloads when the build passes.";
      foot.style.color = "#9ca3af"; foot.style.marginTop = "16px";
      overlay.append(head, pre, foot);
      document.body.appendChild(overlay);
    }
    overlay.querySelector("pre").textContent = error;
  }
  function hide() { if (overlay) { overlay.remove(); overlay = null; } }
  async function poll() {
    try {
      const r = await fetch("/__wf/status", { cache: "no-store" });
      const state = await r.json();
      if (version === null) version = state.version;
      if (state.error) show(state.error);
      else if (state.version !== version) location.reload();
      else hide();
    } catch (e) { /* the server is restarting */ }
    setTimeout(poll, 700);
  }
  poll();
})();
"##;

pub fn run_serve(project_dir: &Path) -> Result<()> {
    let config = ProjectConfig::load(project_dir)?;
    let output_dir = project_dir.join(&config.build.output);

    // A build first, so what is served is what the source says; a build
    // that fails leaves the last output in place and its error on show.
    let state = std::sync::Arc::new(std::sync::Mutex::new(DevState::default()));
    if let Err(e) = crate::cli::build::run_build(project_dir) {
        eprintln!("{e}");
        state.lock().unwrap().error = Some(e.to_string());
    }
    if !output_dir.exists() {
        return Err(crate::error::WebFluentError::IoError(format!(
            "nothing to serve: the build failed and {} does not exist",
            output_dir.display()
        )));
    }

    let port = config.dev.port;
    let addr = format!("0.0.0.0:{}", port);

    println!("Starting dev server at http://localhost:{}", port);

    let server = tiny_http::Server::http(&addr).map_err(|e| {
        crate::error::WebFluentError::IoError(format!("Failed to start server: {}", e))
    })?;

    // Rebuild when a source file changes: the tree's newest change time is
    // read every half second, which needs no watcher and misses nothing.
    if config.dev.hot_reload {
        let root = project_dir.to_path_buf();
        let state = state.clone();
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
                let outcome = crate::cli::build::run_build(&root);
                let mut s = state.lock().unwrap();
                match outcome {
                    Ok(()) => {
                        s.version += 1;
                        s.error = None;
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        s.error = Some(e.to_string());
                    }
                }
            }
        });
    }

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        // The query string names nothing on disk.
        let url = url.split('?').next().unwrap_or("").to_string();
        // The dev server's own routes.
        if url == "/__wf/status" {
            let s = state.lock().unwrap();
            let body = serde_json::json!({ "version": s.version, "error": s.error }).to_string();
            drop(s);
            let response = tiny_http::Response::from_string(body)
                .with_header(
                    tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap(),
                )
                .with_header(tiny_http::Header::from_bytes("Cache-Control", "no-store").unwrap());
            let _ = request.respond(response);
            continue;
        }
        if url == "/__wf/dev.js" {
            let response = tiny_http::Response::from_string(DEV_SCRIPT).with_header(
                tiny_http::Header::from_bytes("Content-Type", "application/javascript").unwrap(),
            );
            let _ = request.respond(response);
            continue;
        }
        let url_path = if url == "/" { "/index.html" } else { &url };
        let accepts_gzip = request
            .headers()
            .iter()
            .any(|h| h.field.equiv("Accept-Encoding") && h.value.as_str().contains("gzip"));

        // Try to serve the file. A static build writes each route as
        // `<route>/index.html`, which is what a host serves for `/<route>`;
        // the server used to fall through to the root index for those, so a
        // pre-rendered page was never the one seen in development.
        let mut file_path = output_dir.join(url_path.trim_start_matches('/'));
        if file_path.is_dir() {
            file_path = file_path.join("index.html");
        }

        let (content, content_type) = if file_path.exists() && file_path.is_file() {
            let content = fs::read(&file_path).unwrap_or_default();
            let ct = guess_content_type(&file_path);
            (content, ct)
        } else {
            // A static build's catch-all page, else the SPA entry for all routes.
            let not_found = output_dir.join("404.html");
            let index_path = output_dir.join("index.html");
            if not_found.exists() {
                let content = fs::read(&not_found).unwrap_or_default();
                (content, "text/html")
            } else if index_path.exists() {
                let content = fs::read(&index_path).unwrap_or_default();
                (content, "text/html")
            } else {
                (b"Not Found".to_vec(), "text/plain")
            }
        };

        // A page carries the dev script, which reloads it on a new build and
        // shows a failed build's error over it.
        let dev_page = config.dev.hot_reload && content_type.starts_with("text/html");
        let content = if dev_page {
            inject_dev_script(content)
        } else {
            content
        };

        // What a host does: send the `.gz` the build wrote when the browser
        // takes it, or compress a text response on the way out, so the bytes
        // measured here are the bytes a deployed site sends.
        let (content, encoding) = if accepts_gzip && is_text(content_type) {
            let prebuilt = PathBuf::from(format!("{}.gz", file_path.display()));
            match fs::read(&prebuilt) {
                Ok(gz) if file_path.is_file() && !dev_page => (gz, Some("gzip")),
                _ if content.len() >= 1024 => (crate::codegen::gzip::gzip(&content), Some("gzip")),
                _ => (content, None),
            }
        } else {
            (content, None)
        };

        // The same headers the built output asks a host for, so a policy problem
        // surfaces in development rather than on the deployed site.
        let mut response = tiny_http::Response::from_data(content)
            .with_header(tiny_http::Header::from_bytes("Content-Type", content_type).unwrap());
        if let Some(encoding) = encoding {
            response
                .add_header(tiny_http::Header::from_bytes("Content-Encoding", encoding).unwrap());
        }
        for (name, value) in [
            ("Vary", "Accept-Encoding"),
            ("X-Content-Type-Options", "nosniff"),
            ("Referrer-Policy", "strict-origin-when-cross-origin"),
            ("X-Frame-Options", "DENY"),
        ] {
            if let Ok(header) = tiny_http::Header::from_bytes(name, value) {
                response.add_header(header);
            }
        }

        let _ = request.respond(response);
    }

    Ok(())
}

/// The page with `<script src="/__wf/dev.js">` before `</body>`.
fn inject_dev_script(content: Vec<u8>) -> Vec<u8> {
    let Ok(text) = String::from_utf8(content) else {
        return Vec::new();
    };
    let tag = "<script src=\"/__wf/dev.js\" defer></script>\n";
    match text.rfind("</body>") {
        Some(at) => format!("{}{}{}", &text[..at], tag, &text[at..]).into_bytes(),
        None => format!("{text}{tag}").into_bytes(),
    }
}

/// The newest change time under `src/`, the config and the translations.
fn newest_change(root: &Path) -> Option<std::time::SystemTime> {
    fn walk(dir: &Path, newest: &mut Option<std::time::SystemTime>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, newest);
            } else if let Ok(meta) = entry.metadata()
                && let Ok(modified) = meta.modified()
                && newest.is_none_or(|n| modified > n)
            {
                *newest = Some(modified);
            }
        }
    }
    let mut newest = None;
    walk(&root.join("src"), &mut newest);
    walk(&root.join("public"), &mut newest);
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
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("gz") => "application/gzip",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    }
}
