//! `wf start` — a built project, served as a host serves it.
//!
//! `wf serve` is the development loop: it watches, rebuilds and reloads.
//! This is the other end: what `wf build` wrote, on `$PORT`, with nothing
//! added to the pages — the precompressed `.gz` files a build writes beside
//! each text file, the headers its `_headers` file names (the policy, the
//! frame rule), a single-page build's shell for every route a navigation
//! asks for, and a static build's `404.html` for a route it never wrote.
//!
//! `--proxy /api=http://localhost:9000/api` forwards every request under a
//! prefix to another server, so a page reaches an API that lives elsewhere
//! through its own origin: nothing to add to the policy, no CORS, and the
//! API's address chosen where the site is started rather than when it was
//! built. It speaks HTTP/1.1 to the server behind it, over TLS for an
//! `https://` one, which it trusts as this machine does — its certificate
//! store, and the certificates `SSL_CERT_FILE` or `NODE_EXTRA_CA_CERTS`
//! name (a private authority, a development one).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::diagnostics::error::{Result, WebFluentError};
use crate::project::config::ProjectConfig;

/// Requests under `prefix` go to `host:port`, the prefix replaced by
/// `path` (`/api=http://localhost:9000/v2` sends `/api/users` to
/// `http://localhost:9000/v2/users`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Proxy {
    pub prefix: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    /// Over TLS: the URL was `https://`.
    pub tls: bool,
}

impl Proxy {
    /// A `PREFIX=URL` as `--proxy` takes it.
    pub(crate) fn parse(spec: &str) -> Result<Proxy> {
        let bad = |why: &str| {
            WebFluentError::config(format!(
                "--proxy {spec}: {why}. Write it PREFIX=URL, as `--proxy /api=http://localhost:9000/api`"
            ))
        };
        let (prefix, url) = spec.split_once('=').ok_or_else(|| bad("no `=`"))?;
        let prefix = prefix.trim().trim_end_matches('/');
        if !prefix.starts_with('/') || prefix.len() < 2 {
            return Err(bad("the prefix is a path of the site, such as `/api`"));
        }
        let url = url.trim();
        let (tls, rest) = if let Some(rest) = url.strip_prefix("https://") {
            (true, rest)
        } else if let Some(rest) = url.strip_prefix("http://") {
            (false, rest)
        } else {
            return Err(bad("the URL starts with http:// or https://"));
        };
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, ""),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) => (
                h,
                p.parse::<u16>().map_err(|_| bad("the port is a number"))?,
            ),
            None => (authority, if tls { 443 } else { 80 }),
        };
        if host.is_empty() {
            return Err(bad("the URL names no host"));
        }
        Ok(Proxy {
            prefix: prefix.to_string(),
            host: host
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string(),
            port,
            path: path.trim_end_matches('/').to_string(),
            tls,
        })
    }

    /// `http://host:port/path`, as it was given.
    pub(crate) fn url(&self) -> String {
        format!(
            "{}://{}:{}{}",
            if self.tls { "https" } else { "http" },
            self.host,
            self.port,
            self.path
        )
    }

    /// The path on the server behind it for `path` under the prefix, or
    /// `None` when `path` is not under it.
    fn target(&self, path_and_query: &str) -> Option<String> {
        let rest = path_and_query.strip_prefix(&self.prefix)?;
        if !(rest.is_empty() || rest.starts_with('/') || rest.starts_with('?')) {
            return None;
        }
        let rest = if rest.is_empty() || rest.starts_with('?') {
            format!("/{rest}")
        } else {
            rest.to_string()
        };
        Some(format!("{}{rest}", self.path))
    }
}

/// A built directory on a port, for as long as it is held.
pub(crate) struct Served {
    /// `http://<address>:<port>`.
    pub origin: String,
    server: Arc<tiny_http::Server>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Served {
    pub(crate) fn close(mut self) {
        self.server.unblock();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    fn wait(mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Serve `dir` (a build's output, under `base_path`) on `bind`, with
/// `proxies` forwarding their prefixes.
pub(crate) fn serve_built(
    dir: PathBuf,
    base_path: &str,
    bind: &str,
    proxies: Vec<Proxy>,
) -> Result<Served> {
    let server = Arc::new(
        tiny_http::Server::http(bind)
            .map_err(|e| WebFluentError::io(format!("could not listen on {bind}: {e}")))?,
    );
    let addr = server
        .server_addr()
        .to_ip()
        .ok_or_else(|| WebFluentError::io("no address"))?;
    let origin = format!("http://{addr}");
    let headers = Arc::new(read_headers_file(&dir));
    let (dir, base, proxies) = (Arc::new(dir), base_path.to_string(), Arc::new(proxies));
    let serving = Arc::clone(&server);
    let thread = std::thread::spawn(move || {
        for request in serving.incoming_requests() {
            let (dir, base, proxies, headers) = (
                Arc::clone(&dir),
                base.clone(),
                Arc::clone(&proxies),
                Arc::clone(&headers),
            );
            // A request to the API behind a proxy may take its time; the
            // pages do not wait for it.
            std::thread::spawn(move || answer(request, &dir, &base, &proxies, &headers));
        }
    });
    Ok(Served {
        origin,
        server,
        thread: Some(thread),
    })
}

fn answer(
    mut request: tiny_http::Request,
    dir: &Path,
    base: &str,
    proxies: &[Proxy],
    headers: &[(String, Vec<(String, String)>)],
) {
    let full = request.url().to_string();
    for proxy in proxies {
        if let Some(target) = proxy.target(&full) {
            let response = forward(&mut request, proxy, &target);
            let _ = request.respond(response);
            return;
        }
    }
    let url = full.split('?').next().unwrap_or("/").to_string();
    let path = url.strip_prefix(base).unwrap_or(&url).to_string();
    if path == "/_headers" {
        let _ = request.respond(text(404, "Not Found"));
        return;
    }
    let Some(mut file) = super::preview::resolve(dir, &path) else {
        let _ = request.respond(text(404, "Not Found"));
        return;
    };
    if file.is_dir() {
        file = file.join("index.html");
    }
    let mut status = 200;
    if !file.is_file() {
        if path == "/favicon.ico" {
            let _ = request.respond(tiny_http::Response::empty(204));
            return;
        }
        // A fetch, a script or a file the build did not write is a 404 a
        // client can read as one, never a page of HTML.
        if !super::preview::is_page_request(&path, request.headers()) {
            let _ = request.respond(text(404, "Not Found"));
            return;
        }
        // A route no file answers: a static build's 404 page, else the
        // single-page build's shell, which draws every route itself.
        if dir.join("404.html").is_file() {
            file = dir.join("404.html");
            status = 404;
        } else {
            file = dir.join("index.html");
        }
    }
    let gzip = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Accept-Encoding"))
        .is_some_and(|h| h.value.as_str().contains("gzip"));
    let packed = PathBuf::from(format!("{}.gz", file.display()));
    let (body, encoded) = if gzip && packed.is_file() {
        (std::fs::read(&packed), true)
    } else {
        (std::fs::read(&file), false)
    };
    let Ok(body) = body else {
        let _ = request.respond(text(404, "Not Found"));
        return;
    };
    let mut response = tiny_http::Response::from_data(body)
        .with_status_code(status)
        .with_header(header("Content-Type", content_type(&file)));
    if encoded {
        response = response.with_header(header("Content-Encoding", "gzip"));
    }
    response = response.with_header(header("Vary", "Accept-Encoding"));
    // A page is asked for again each time; a file whose name carries its
    // hash never changes.
    let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let cache = if name.ends_with(".html") {
        "no-cache"
    } else if hashed(name) {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=0, must-revalidate"
    };
    response = response.with_header(header("Cache-Control", cache));
    response = response.with_header(header("X-Content-Type-Options", "nosniff"));
    for (pattern, set) in headers {
        if pattern_matches(pattern, &path) {
            for (name, value) in set {
                response = response.with_header(header(name, value));
            }
        }
    }
    let _ = request.respond(response);
}

/// Whether a file name carries a content hash: `hero.3f9a2c1b.960.webp`.
fn hashed(name: &str) -> bool {
    name.split('.').any(|part| {
        part.len() >= 8 && part.len() <= 16 && part.chars().all(|c| c.is_ascii_hexdigit())
    })
}

/// The blocks of a build's `_headers` file (the format Netlify and
/// Cloudflare read): a path pattern, then its headers, indented.
fn read_headers_file(dir: &Path) -> Vec<(String, Vec<(String, String)>)> {
    let Ok(text) = std::fs::read_to_string(dir.join("_headers")) else {
        return Vec::new();
    };
    let mut out: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with([' ', '\t']) {
            out.push((line.trim().to_string(), Vec::new()));
        } else if let (Some(last), Some((name, value))) = (out.last_mut(), line.split_once(':')) {
            last.1
                .push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    out
}

/// Whether a `_headers` pattern (`/*`, `/docs/*`, `/about`) covers `path`.
fn pattern_matches(pattern: &str, path: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => path.starts_with(prefix),
        None => pattern == path || pattern.trim_end_matches('/') == path.trim_end_matches('/'),
    }
}

/// Send `request` on to the server behind `proxy`, and hand back its
/// answer — or a 502 saying why there is none.
fn forward(
    request: &mut tiny_http::Request,
    proxy: &Proxy,
    target: &str,
) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    match try_forward(request, proxy, target) {
        Ok(response) => response,
        Err(why) => text(
            502,
            &format!(
                "wf start: {}://{}:{}{target} did not answer: {why}",
                if proxy.tls { "https" } else { "http" },
                proxy.host,
                proxy.port
            ),
        ),
    }
}

fn try_forward(
    request: &mut tiny_http::Request,
    proxy: &Proxy,
    target: &str,
) -> std::io::Result<tiny_http::Response<std::io::Cursor<Vec<u8>>>> {
    let mut body = Vec::new();
    request.as_reader().read_to_end(&mut body)?;
    let request: &tiny_http::Request = request;
    let stream = TcpStream::connect((proxy.host.as_str(), proxy.port))?;
    stream.set_read_timeout(Some(Duration::from_secs(60)))?;
    stream.set_write_timeout(Some(Duration::from_secs(60)))?;
    if proxy.tls {
        let name = rustls::pki_types::ServerName::try_from(proxy.host.clone())
            .map_err(|e| std::io::Error::other(format!("{}: {e}", proxy.host)))?;
        let connection =
            rustls::ClientConnection::new(tls_config()?, name).map_err(std::io::Error::other)?;
        exchange(
            rustls::StreamOwned::new(connection, stream),
            request,
            proxy,
            target,
            &body,
        )
    } else {
        exchange(stream, request, proxy, target, &body)
    }
}

/// What this machine trusts, read once: its certificate store, with
/// `SSL_CERT_FILE` (which the store reads itself) and `NODE_EXTRA_CA_CERTS`.
fn tls_config() -> std::io::Result<Arc<rustls::ClientConfig>> {
    static CONFIG: std::sync::OnceLock<std::result::Result<Arc<rustls::ClientConfig>, String>> =
        std::sync::OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let mut roots = rustls::RootCertStore::empty();
            let found = rustls_native_certs::load_native_certs();
            roots.add_parsable_certificates(found.certs);
            if let Some(extra) = std::env::var_os("NODE_EXTRA_CA_CERTS") {
                use rustls::pki_types::pem::PemObject;
                if let Ok(certs) = rustls::pki_types::CertificateDer::pem_file_iter(&extra) {
                    roots.add_parsable_certificates(certs.flatten());
                }
            }
            if roots.is_empty() {
                return Err("no certificates to trust: this machine's store is empty".to_string());
            }
            let provider = Arc::new(rustls::crypto::ring::default_provider());
            rustls::ClientConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .map(|b| Arc::new(b.with_root_certificates(roots).with_no_client_auth()))
                .map_err(|e| e.to_string())
        })
        .clone()
        .map_err(std::io::Error::other)
}

/// One request, written to `stream`, and the answer read back.
fn exchange<S: Read + Write>(
    mut stream: S,
    request: &tiny_http::Request,
    proxy: &Proxy,
    target: &str,
    body: &[u8],
) -> std::io::Result<tiny_http::Response<std::io::Cursor<Vec<u8>>>> {
    let default_port = if proxy.tls { 443 } else { 80 };
    let host = if proxy.port == default_port {
        proxy.host.clone()
    } else {
        format!("{}:{}", proxy.host, proxy.port)
    };
    let mut head = format!("{} {target} HTTP/1.1\r\nHost: {host}\r\n", request.method());
    for h in request.headers() {
        let name = h.field.as_str().as_str();
        // Hop-by-hop, or this connection's own.
        if [
            "host",
            "connection",
            "keep-alive",
            "transfer-encoding",
            "upgrade",
            "content-length",
            "accept-encoding",
        ]
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
        {
            continue;
        }
        head.push_str(&format!("{name}: {}\r\n", h.value.as_str()));
    }
    if !body.is_empty() {
        head.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    head.push_str("Connection: close\r\n\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;

    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line)?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| std::io::Error::other(format!("not an HTTP answer: {status_line:?}")))?;
    let mut headers = Vec::new();
    let mut chunked = false;
    let mut length: Option<usize> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let (name, value) = (name.trim(), value.trim());
        if name.eq_ignore_ascii_case("transfer-encoding") {
            chunked = value.to_ascii_lowercase().contains("chunked");
            continue;
        }
        if name.eq_ignore_ascii_case("content-length") {
            length = value.parse().ok();
            continue;
        }
        if ["connection", "keep-alive"]
            .iter()
            .any(|n| name.eq_ignore_ascii_case(n))
        {
            continue;
        }
        headers.push((name.to_string(), value.to_string()));
    }
    let mut body = Vec::new();
    if chunked {
        loop {
            let mut size = String::new();
            reader.read_line(&mut size)?;
            let size = usize::from_str_radix(size.trim().split(';').next().unwrap_or("0"), 16)
                .map_err(|_| std::io::Error::other("a bad chunk"))?;
            if size == 0 {
                break;
            }
            let mut chunk = vec![0; size];
            reader.read_exact(&mut chunk)?;
            body.extend_from_slice(&chunk);
            let mut end = String::new();
            reader.read_line(&mut end)?;
        }
    } else if let Some(n) = length {
        body.resize(n, 0);
        reader.read_exact(&mut body)?;
    } else {
        reader.read_to_end(&mut body)?;
    }
    let mut response = tiny_http::Response::from_data(body).with_status_code(status);
    for (name, value) in headers {
        if let Ok(h) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
            response = response.with_header(h);
        }
    }
    Ok(response)
}

fn text(status: u16, body: &str) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    tiny_http::Response::from_data(body.as_bytes().to_vec())
        .with_status_code(status)
        .with_header(header("Content-Type", "text/plain; charset=utf-8"))
}

fn header(name: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap_or_else(|()| {
        tiny_http::Header::from_bytes(&b"X-Ignored"[..], &b"1"[..])
            .expect("a fixed header is a valid one")
    })
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "webmanifest" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        _ => "application/octet-stream",
    }
}

/// The built project's output folder, or why there is none to serve.
pub(crate) fn built_output(
    project_dir: &Path,
    allow_output_outside: bool,
) -> Result<(PathBuf, ProjectConfig)> {
    let config = ProjectConfig::load(project_dir)?;
    let dir =
        crate::build::project_output(project_dir, &config.build.output, allow_output_outside)?;
    if !dir.join("index.html").is_file() && !dir.join("404.html").is_file() {
        return Err(WebFluentError::io(format!(
            "nothing to serve: {} holds no build. Run `wf build` first",
            dir.display()
        )));
    }
    Ok((dir, config))
}

pub(crate) fn run_start(
    project_dir: &Path,
    port: Option<u16>,
    host: &str,
    proxies: &[String],
    allow_output_outside: bool,
) -> Result<()> {
    let (dir, config) = built_output(project_dir, allow_output_outside)?;
    let proxies = proxies
        .iter()
        .map(|p| Proxy::parse(p))
        .collect::<Result<Vec<_>>>()?;
    let port = match port {
        Some(p) => p,
        None => match std::env::var("PORT") {
            Ok(p) => p
                .trim()
                .parse()
                .map_err(|_| WebFluentError::config(format!("PORT={p} is not a port")))?,
            Err(_) => 3000,
        },
    };
    let base = config.build.base_path.trim_end_matches('/').to_string();
    let bind = if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };
    let served = serve_built(dir, &base, &bind, proxies.clone())?;
    println!(
        "Serving {} on http://{host}:{port}{base}/",
        config.build.output.trim_start_matches("./")
    );
    for p in &proxies {
        println!("  {}/* → {}/*", p.prefix, p.url());
    }
    served.wait();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_proxy_is_a_prefix_and_an_http_url() {
        let p = Proxy::parse("/api=http://localhost:9000/v2/").unwrap();
        assert_eq!(p.prefix, "/api");
        assert_eq!(
            (p.host.as_str(), p.port, p.path.as_str()),
            ("localhost", 9000, "/v2")
        );
        assert_eq!(p.target("/api/users?x=1").as_deref(), Some("/v2/users?x=1"));
        assert_eq!(p.target("/api").as_deref(), Some("/v2/"));
        assert_eq!(p.target("/apiary"), None);
        assert_eq!(p.target("/about"), None);
        let bare = Proxy::parse("/media=http://127.0.0.1").unwrap();
        assert_eq!(
            (bare.port, bare.target("/media/a.png").as_deref()),
            (80, Some("/a.png"))
        );
        let secure = Proxy::parse("/api=https://api.example.com/v1").unwrap();
        assert_eq!(
            (secure.tls, secure.port, secure.path.as_str()),
            (true, 443, "/v1")
        );
        assert_eq!(secure.url(), "https://api.example.com:443/v1");
        for bad in [
            "/api",
            "api=http://x",
            "/api=ftp://x",
            "/api=http://:80",
            "/api=http://x:port",
        ] {
            assert!(Proxy::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn headers_file_blocks_apply_by_pattern() {
        let dir = std::env::temp_dir().join(format!("wf-start-headers-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("_headers"),
            "/*\n  X-Frame-Options: DENY\n  Content-Security-Policy: default-src 'self'\n/docs/*\n  X-Docs: 1\n",
        )
        .unwrap();
        let blocks = read_headers_file(&dir);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].1[1].0, "Content-Security-Policy");
        assert!(pattern_matches("/*", "/anything"));
        assert!(pattern_matches("/docs/*", "/docs/a"));
        assert!(!pattern_matches("/docs/*", "/blog"));
        assert!(pattern_matches("/about", "/about/"));
        assert!(hashed("hero.3f9a2c1b.960.webp"));
        assert!(!hashed("app.js"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The status, headers and body of `GET path` from `origin`.
    fn get(origin: &str, path: &str, headers: &str) -> String {
        let addr = origin.trim_start_matches("http://");
        let mut stream = TcpStream::connect(addr).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {addr}\r\n{headers}Connection: close\r\n\r\n"
        )
        .unwrap();
        let mut reply = String::new();
        stream.read_to_string(&mut reply).unwrap();
        reply
    }

    #[test]
    fn it_serves_the_shell_the_files_and_the_api_behind_it() {
        // The API: answers every request with the path it was asked for.
        let api = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let api_port = api.server_addr().to_ip().unwrap().port();
        std::thread::spawn(move || {
            for request in api.incoming_requests() {
                let body = format!("{{\"path\":\"{}\"}}", request.url());
                let _ = request.respond(
                    tiny_http::Response::from_string(body)
                        .with_header(header("Content-Type", "application/json")),
                );
            }
        });
        let dir = std::env::temp_dir().join(format!("wf-start-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            "<!doctype html><title>shell</title>",
        )
        .unwrap();
        std::fs::write(dir.join("app.js"), "1").unwrap();
        std::fs::write(dir.join("_headers"), "/*\n  X-Frame-Options: DENY\n").unwrap();
        let proxy = Proxy::parse(&format!("/api=http://127.0.0.1:{api_port}/v2")).unwrap();
        let served = serve_built(dir.clone(), "", "127.0.0.1:0", vec![proxy]).unwrap();
        let o = served.origin.clone();

        let shell = get(&o, "/pokemon/pikachu", "Sec-Fetch-Mode: navigate\r\n");
        assert!(shell.starts_with("HTTP/1.1 200"), "{shell}");
        assert!(shell.contains("<title>shell</title>"));
        assert!(shell.contains("X-Frame-Options: DENY"), "{shell}");
        assert!(get(&o, "/app.js", "").starts_with("HTTP/1.1 200"));
        assert!(get(&o, "/missing.js", "").starts_with("HTTP/1.1 404"));
        assert!(get(&o, "/_headers", "").starts_with("HTTP/1.1 404"));
        let api_reply = get(&o, "/api/pokemon?limit=2", "");
        assert!(api_reply.starts_with("HTTP/1.1 200"), "{api_reply}");
        assert!(
            api_reply.contains(r#"{"path":"/v2/pokemon?limit=2"}"#),
            "{api_reply}"
        );
        served.close();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
