//! A static server for a built directory, for as long as something needs
//! one.
//!
//! `wf serve` is the development loop — it watches, rebuilds and reloads.
//! This is the other case: `wf verify` needs the output on a port for a
//! minute while a browser walks it, and nothing more.

use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{Result, WebFluentError};

pub struct Preview {
    /// `http://127.0.0.1:<port>`, as a browser would ask for it.
    pub origin: String,
    server: Arc<tiny_http::Server>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Preview {
    /// Stop serving and wait for the thread to finish.
    ///
    /// `unblock` is what makes the wait end: the accept loop is inside
    /// `incoming_requests()`, and nothing short of telling the server to
    /// stop brings it back out.
    pub fn close(mut self) {
        self.server.unblock();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Serve `dir` on a port the operating system picks.
pub fn serve_directory(dir: PathBuf, base_path: &str) -> Result<Preview> {
    let server = Arc::new(
        tiny_http::Server::http("127.0.0.1:0")
            .map_err(|e| WebFluentError::IoError(format!("could not open a port: {e}")))?,
    );
    let origin = format!(
        "http://{}",
        server
            .server_addr()
            .to_ip()
            .ok_or_else(|| WebFluentError::IoError("no address".into()))?
    );
    let base = base_path.to_string();
    let serving = server.clone();
    let thread = std::thread::spawn(move || {
        for request in serving.incoming_requests() {
            let url = request.url().split('?').next().unwrap_or("/").to_string();
            let path = url.strip_prefix(&base).unwrap_or(&url);
            let mut file = dir.join(path.trim_start_matches('/'));
            if file.is_dir() {
                file = file.join("index.html");
            }
            // A route the build wrote no file for: the catch-all, else the
            // shell — which is how a single-page build serves every route.
            if !file.is_file() {
                file = ["404.html", "index.html"]
                    .iter()
                    .map(|name| dir.join(name))
                    .find(|p| p.is_file())
                    .unwrap_or(file);
            }
            let response = match std::fs::read(&file) {
                Ok(body) => tiny_http::Response::from_data(body).with_header(
                    tiny_http::Header::from_bytes("Content-Type", content_type(&file)).unwrap(),
                ),
                Err(_) => tiny_http::Response::from_string("Not Found").with_status_code(404),
            };
            let _ = request.respond(response);
        }
    });
    Ok(Preview {
        origin,
        server,
        thread: Some(thread),
    })
}

fn content_type(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}
