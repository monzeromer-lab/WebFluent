use std::path::Path;
use std::fs;
use crate::config::ProjectConfig;
use crate::error::Result;

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
        let url_path = if url == "/" { "/index.html" } else { &url };

        // Try to serve the file
        let file_path = output_dir.join(url_path.trim_start_matches('/'));

        let (content, content_type) = if file_path.exists() && file_path.is_file() {
            let content = fs::read(&file_path).unwrap_or_default();
            let ct = guess_content_type(&file_path);
            (content, ct)
        } else {
            // SPA fallback — serve index.html for all routes
            let index_path = output_dir.join("index.html");
            if index_path.exists() {
                let content = fs::read(&index_path).unwrap_or_default();
                (content, "text/html")
            } else {
                (b"Not Found".to_vec(), "text/plain")
            }
        };

        let response = tiny_http::Response::from_data(content)
            .with_header(
                tiny_http::Header::from_bytes("Content-Type", content_type).unwrap()
            );

        let _ = request.respond(response);
    }

    Ok(())
}

fn guess_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
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
