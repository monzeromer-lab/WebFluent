use crate::config::ProjectConfig;
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_serve(project_dir: &Path) -> Result<()> {
    let config = ProjectConfig::load(project_dir)?;
    let output_dir = project_dir.join(&config.build.output);

    if !output_dir.exists() {
        println!("Build directory not found. Running build first...");
        crate::cli::build::run_build(project_dir)?;
    }

    let port = config.dev.port;
    let addr = format!("0.0.0.0:{}", port);

    println!("Starting dev server at http://localhost:{}", port);

    let server = tiny_http::Server::http(&addr).map_err(|e| {
        crate::error::WebFluentError::IoError(format!("Failed to start server: {}", e))
    })?;

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        // The query string names nothing on disk.
        let url = url.split('?').next().unwrap_or("").to_string();
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

        // What a host does: send the `.gz` the build wrote when the browser
        // takes it, or compress a text response on the way out, so the bytes
        // measured here are the bytes a deployed site sends.
        let (content, encoding) = if accepts_gzip && is_text(content_type) {
            let prebuilt = PathBuf::from(format!("{}.gz", file_path.display()));
            match fs::read(&prebuilt) {
                Ok(gz) if file_path.is_file() => (gz, Some("gzip")),
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
