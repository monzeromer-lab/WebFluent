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
            // A fetch, a script or a file is not a route: it gets the 404 a
            // host would send, so a missing API is a failed request rather
            // than a page of HTML handed to `JSON.parse`.
            // The browser asks for `/favicon.ico` on its own; the page did
            // not, so a site without one is answered with nothing.
            if !file.is_file() && path == "/favicon.ico" {
                let _ = request.respond(tiny_http::Response::empty(204));
                continue;
            }
            if !file.is_file() && !is_page_request(path, request.headers()) {
                let _ = request
                    .respond(tiny_http::Response::from_string("Not Found").with_status_code(404));
                continue;
            }
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

/// Whether a request is a page navigation, which a single-page build
/// answers with its shell: the browser says so (`Sec-Fetch-Mode:
/// navigate`), or — from a client that does not say — the address has no
/// file extension. A `fetch`, a script, an image or a stylesheet is not.
pub fn is_page_request(path: &str, headers: &[tiny_http::Header]) -> bool {
    if let Some(mode) = headers
        .iter()
        .find(|h| h.field.equiv("Sec-Fetch-Mode"))
        .map(|h| h.value.as_str().to_ascii_lowercase())
    {
        return mode == "navigate" || mode == "nested-navigate";
    }
    let last = path.rsplit('/').next().unwrap_or("");
    !last.contains('.')
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    /// The status line of `GET path` with `headers`, from the server.
    fn status(origin: &str, path: &str, headers: &str) -> String {
        let addr = origin.trim_start_matches("http://");
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {addr}\r\n{headers}Connection: close\r\n\r\n"
        )
        .unwrap();
        let mut reply = String::new();
        stream.read_to_string(&mut reply).unwrap();
        reply.lines().next().unwrap_or("").to_string()
    }

    #[test]
    fn only_a_page_navigation_gets_the_shell() {
        let dir = std::env::temp_dir().join(format!("wf-preview-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            "<!doctype html><title>shell</title>",
        )
        .unwrap();
        std::fs::write(dir.join("app.js"), "1").unwrap();
        let server = serve_directory(dir, "").unwrap();
        let o = &server.origin;
        assert!(status(o, "/about", "Sec-Fetch-Mode: navigate\r\n").contains("200"));
        // A client that does not say: an address with no extension is a page.
        assert!(status(o, "/about", "").contains("200"));
        assert!(status(o, "/app.js", "Sec-Fetch-Mode: no-cors\r\n").contains("200"));
        // A fetch for something the build did not write is a 404, not HTML.
        assert!(status(o, "/api/users", "Sec-Fetch-Mode: cors\r\n").contains("404"));
        assert!(status(o, "/data.json", "").contains("404"));
        assert!(status(o, "/favicon.ico", "Sec-Fetch-Mode: no-cors\r\n").contains("204"));
        server.close();
    }
}
