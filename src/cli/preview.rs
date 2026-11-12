//! A static server for a built directory, for as long as something needs
//! one.
//!
//! `wf serve` is the development loop — it watches, rebuilds and reloads.
//! This is the other case: `wf verify` needs the output on a port for a
//! minute while a browser walks it, and nothing more.

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use crate::diagnostics::error::{Result, WebFluentError};

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
    serve_directory_within(dir, base_path, Vec::new())
}

/// [`serve_directory`], where a link in `dir` may also lead into one of
/// `also`: a test's site, whose `public/` files are links to the project's.
pub fn serve_directory_within(
    dir: PathBuf,
    base_path: &str,
    also: Vec<PathBuf>,
) -> Result<Preview> {
    let server = Arc::new(
        tiny_http::Server::http("127.0.0.1:0")
            .map_err(|e| WebFluentError::io(format!("could not open a port: {e}")))?,
    );
    let origin = format!(
        "http://{}",
        server
            .server_addr()
            .to_ip()
            .ok_or_else(|| WebFluentError::io("no address"))?
    );
    let base = base_path.to_string();
    let serving = server.clone();
    let thread = std::thread::spawn(move || {
        for request in serving.incoming_requests() {
            let host = request
                .headers()
                .iter()
                .find(|h| h.field.equiv("Host"))
                .map(|h| h.value.as_str().to_string());
            if !host_allowed(host.as_deref(), "127.0.0.1") {
                let _ = request.respond(
                    tiny_http::Response::from_string("Misdirected Request").with_status_code(421),
                );
                continue;
            }
            let url = request.url().split('?').next().unwrap_or("/").to_string();
            let path = url.strip_prefix(&base).unwrap_or(&url);
            let Some(mut file) = resolve_within(&dir, &also, path) else {
                let _ = request
                    .respond(tiny_http::Response::from_string("Not Found").with_status_code(404));
                continue;
            };
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

/// The file under `root` that the path of a request names — or `None`, for
/// a path that names nothing a server may send: one that climbs out of
/// `root` (`..`, written plainly or percent-encoded as `%2e%2e`), one with a
/// backslash or a NUL in it, one that is not text once decoded, or one
/// that reaches, through a link, somewhere outside `root`. A path that
/// names nothing on the disk is returned as it is, for the caller's 404 or
/// its shell.
pub fn resolve(root: &Path, url_path: &str) -> Option<PathBuf> {
    resolve_within(root, &[], url_path)
}

/// [`resolve`], where a link may also lead into one of `also` (each
/// resolved already).
pub fn resolve_within(root: &Path, also: &[PathBuf], url_path: &str) -> Option<PathBuf> {
    let decoded = percent_decode(url_path)?;
    for text in [url_path, decoded.as_str()] {
        if text.contains(['\\', '\0']) || text.split('/').any(|segment| segment == "..") {
            return None;
        }
    }
    let rel = decoded.trim_start_matches('/');
    if !Path::new(rel)
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return None;
    }
    let file = root.join(rel);
    if std::fs::symlink_metadata(&file).is_ok() {
        let real_root = std::fs::canonicalize(root).ok()?;
        let real = std::fs::canonicalize(&file).ok()?;
        if !real.starts_with(&real_root) && !also.iter().any(|a| real.starts_with(a)) {
            return None;
        }
    }
    Some(file)
}

/// `text` with each `%XX` read as the byte it stands for, or `None` when
/// the bytes are not UTF-8. A `%` that starts no escape is kept.
fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2]))
        {
            out.push((h * 16 + l) as u8);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).ok()
}

/// Whether a request's `Host` names this server, as it listens on `bound`
/// (`127.0.0.1`, `0.0.0.0`, a name). A web page elsewhere can point a name
/// of its own at this machine and read what is served as its own (DNS
/// rebinding); it cannot make the browser send a name it does not own. So
/// answered are: `localhost` and names under it, an address written as
/// numbers (no page can rebind one), the name the server was told to listen
/// on, and — when it listens on every interface, for a phone — a `.local`
/// name of this network. A request with no `Host` (HTTP/1.0) is no
/// browser's.
pub fn host_allowed(host: Option<&str>, bound: &str) -> bool {
    let Some(host) = host else {
        return true;
    };
    let host = host.trim().to_ascii_lowercase();
    // Without the port: `[::1]:3000`, `localhost:3000`.
    let name = if let Some(rest) = host.strip_prefix('[') {
        match rest.split_once(']') {
            Some((inside, _)) => inside.to_string(),
            None => return false,
        }
    } else {
        match host.rsplit_once(':') {
            Some((name, port)) if port.chars().all(|c| c.is_ascii_digit()) => name.to_string(),
            _ => host.clone(),
        }
    };
    let name = name.trim_end_matches('.');
    if name.is_empty() {
        return false;
    }
    if name == "localhost" || name.ends_with(".localhost") {
        return true;
    }
    if name.parse::<std::net::IpAddr>().is_ok() {
        return true;
    }
    let bound = bound.trim_start_matches('[').trim_end_matches(']');
    if name.eq_ignore_ascii_case(bound) {
        return true;
    }
    let everywhere = bound
        .parse::<std::net::IpAddr>()
        .is_ok_and(|ip| ip.is_unspecified());
    everywhere && name.ends_with(".local")
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
    fn a_path_resolves_inside_the_root_or_not_at_all() {
        let dir = std::env::temp_dir().join(format!("wf-resolve-{}", std::process::id()));
        let root = dir.join("build");
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::write(dir.join("secret.txt"), "x").unwrap();
        std::fs::write(root.join("docs/ü b.txt"), "y").unwrap();
        assert_eq!(resolve(&root, "/index.html"), Some(root.join("index.html")));
        assert_eq!(
            resolve(&root, "/docs/%C3%BC%20b.txt"),
            Some(root.join("docs/ü b.txt"))
        );
        assert_eq!(
            resolve(&root, "/docs/ü b.txt"),
            Some(root.join("docs/ü b.txt"))
        );
        assert_eq!(resolve(&root, "/./docs"), Some(root.join("./docs")));
        for bad in [
            "/../secret.txt",
            "/docs/../../secret.txt",
            "/%2e%2e/secret.txt",
            "/%2E%2e%2fsecret.txt",
            "/..%5csecret.txt",
            "/..\\secret.txt",
            "/a%00b",
            "/%ff",
        ] {
            assert_eq!(resolve(&root, bad), None, "{bad}");
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.join("secret.txt"), root.join("out.txt")).unwrap();
            std::os::unix::fs::symlink(&dir, root.join("up")).unwrap();
            std::os::unix::fs::symlink(root.join("docs"), root.join("in")).unwrap();
            assert_eq!(resolve(&root, "/out.txt"), None);
            assert_eq!(resolve(&root, "/up/secret.txt"), None);
            assert_eq!(resolve(&root, "/in/ü b.txt"), Some(root.join("in/ü b.txt")));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_host_is_answered_only_by_a_name_it_is_reached_by() {
        for host in [
            None,
            Some("localhost"),
            Some("LOCALHOST:3000"),
            Some("app.localhost:3000"),
            Some("127.0.0.1:3000"),
            Some("[::1]:3000"),
            Some("10.0.0.5"),
        ] {
            assert!(host_allowed(host, "127.0.0.1"), "{host:?}");
        }
        for host in [
            "evil.example",
            "evil.example:3000",
            "localhost.evil.example",
            "laptop.local",
            "",
            "[::1",
        ] {
            assert!(!host_allowed(Some(host), "127.0.0.1"), "{host}");
        }
        assert!(host_allowed(Some("laptop.local:3000"), "0.0.0.0"));
        assert!(host_allowed(Some("192.168.1.4:3000"), "0.0.0.0"));
        assert!(!host_allowed(Some("evil.example"), "0.0.0.0"));
        assert!(host_allowed(Some("dev.box:3000"), "dev.box"));
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
