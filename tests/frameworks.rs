//! A template served by axum and by Actix Web, as their users serve one:
//! the template the server's state, a handler returning
//! `Result<Rendered, WebFluentError>`, and a PDF laid out on the blocking
//! pool. Each server runs on a free port and is asked over a real
//! connection.

use serde::Serialize;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use webfluent::{Format, Rendered, Template, WebFluentError};

#[derive(Serialize)]
struct Line {
    name: String,
    price: f64,
}

#[derive(Serialize)]
struct Invoice {
    number: String,
    lines: Vec<Line>,
}

const SOURCE: &str = r#"type Line { name: String, price: Number }
page Invoice(path: "/", title: "Invoice {number}", number: String, lines: [Line]) {
    Document {
        Heading("Invoice {number}").h1
        for l in lines { Text("{l.name}: {l.price}") }
    }
}
"#;

fn invoice(number: &str) -> Invoice {
    Invoice {
        number: number.to_string(),
        lines: vec![Line {
            name: "Widget".into(),
            price: 9.5,
        }],
    }
}

/// The status line, the headers (lower-cased names) and the body of a
/// `GET` to `path`.
fn get(addr: SocketAddr, path: &str) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let mut stream = TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).unwrap();
    let split = raw.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let head = String::from_utf8_lossy(&raw[..split]).to_string();
    let mut body = raw[split + 4..].to_vec();
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(": "))
        .map(|(k, v)| (k.to_ascii_lowercase(), v.to_string()))
        .collect();
    if headers
        .iter()
        .any(|(k, v)| k == "transfer-encoding" && v == "chunked")
    {
        body = unchunk(&body);
    }
    (status, headers, body)
}

fn unchunk(mut raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let end = raw.windows(2).position(|w| w == b"\r\n").unwrap();
        let size =
            usize::from_str_radix(std::str::from_utf8(&raw[..end]).unwrap().trim(), 16).unwrap();
        if size == 0 {
            return out;
        }
        out.extend_from_slice(&raw[end + 2..end + 2 + size]);
        raw = &raw[end + 4 + size..];
    }
}

fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

/// What every server must answer: the page, the PDF as a download, and a
/// `500` for data the page does not take.
fn holds(addr: SocketAddr) {
    let (status, headers, body) = get(addr, "/invoices/1042");
    assert_eq!(status, 200);
    assert_eq!(
        header(&headers, "content-type"),
        Some("text/html; charset=utf-8")
    );
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("<title>Invoice 1042</title>"), "{html}");
    assert!(html.contains("Widget: 9.5"), "{html}");

    let (status, headers, body) = get(addr, "/invoices/1042.pdf");
    assert_eq!(status, 200);
    assert_eq!(header(&headers, "content-type"), Some("application/pdf"));
    assert_eq!(
        header(&headers, "content-disposition"),
        Some("attachment; filename=\"invoice-1042.pdf\"")
    );
    assert!(body.starts_with(b"%PDF"), "a PDF");

    let (status, headers, body) = get(addr, "/broken");
    assert_eq!(status, 500);
    assert_eq!(
        header(&headers, "content-type"),
        Some("text/plain; charset=utf-8")
    );
    // A debug build says what is wrong.
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("lines"), "{text}");
}

#[test]
fn an_axum_server_sends_pages_pdfs_and_a_500() {
    use axum::{
        Router,
        extract::{Path, State},
        routing::get as route,
    };

    async fn page(
        State(tpl): State<Template>,
        Path(number): Path<String>,
    ) -> Result<Rendered, WebFluentError> {
        if let Some(n) = number.strip_suffix(".pdf") {
            let pdf = webfluent::axum::render(&tpl, &invoice(n), Format::Pdf).await?;
            return Ok(pdf.download(&format!("invoice-{n}.pdf")));
        }
        tpl.respond(&invoice(&number), Format::Html)
    }
    async fn broken(State(tpl): State<Template>) -> Result<Rendered, WebFluentError> {
        webfluent::axum::render(&tpl, &serde_json::json!({ "number": "1" }), Format::Html).await
    }

    let tpl = Template::from_str(SOURCE).unwrap();
    let app = Router::new()
        .route("/invoices/{number}", route(page))
        .route("/broken", route(broken))
        .with_state(tpl);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let listener = runtime
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let addr = listener.local_addr().unwrap();
    runtime.spawn(async move { axum::serve(listener, app).await.unwrap() });
    holds(addr);
}

#[test]
fn an_actix_server_sends_pages_pdfs_and_a_500() {
    use actix_web::{App, HttpServer, web};

    async fn page(
        tpl: web::Data<Template>,
        number: web::Path<String>,
    ) -> Result<Rendered, WebFluentError> {
        let number = number.into_inner();
        if let Some(n) = number.strip_suffix(".pdf") {
            let pdf = webfluent::actix::render(&tpl, &invoice(n), Format::Pdf).await?;
            return Ok(pdf.download(&format!("invoice-{n}.pdf")));
        }
        tpl.respond(&invoice(&number), Format::Html)
    }
    async fn broken(tpl: web::Data<Template>) -> Result<Rendered, WebFluentError> {
        webfluent::actix::render(&tpl, &serde_json::json!({ "number": "1" }), Format::Html).await
    }

    let tpl = web::Data::new(Template::from_str(SOURCE).unwrap());
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        actix_web::rt::System::new().block_on(async move {
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(tpl.clone())
                    .route("/invoices/{number}", web::get().to(page))
                    .route("/broken", web::get().to(broken))
            })
            .workers(1)
            .bind(("127.0.0.1", 0))
            .unwrap();
            tx.send(server.addrs()[0]).unwrap();
            server.run().await.unwrap();
        });
    });
    let addr = rx.recv().unwrap();
    holds(addr);
}
