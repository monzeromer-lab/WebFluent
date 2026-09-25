//! A headless browser, started and driven by the build.
//!
//! What a browser knows that a compiler cannot: whether the page loads,
//! whether anything threw on the way, whether every file it asked for
//! arrived, and how long it took to paint. `wf verify` asks it those
//! questions for every route a project serves.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

use serde_json::{Value, json};

use super::ws::Socket;
use crate::error::{Result, WebFluentError};

/// Where a browser usually is. The first one that exists is the one used.
const BINARIES: &[&str] = &[
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/opt/google/chrome/chrome",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
];

/// The browser this machine has, if it has one.
pub fn find() -> Option<String> {
    if let Ok(named) = std::env::var("WF_CHROME")
        && std::path::Path::new(&named).exists()
    {
        return Some(named);
    }
    BINARIES
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(|p| p.to_string())
}

/// What one page did.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Visit {
    pub url: String,
    /// Anything that went wrong: an exception, a `console.error`, a file
    /// that did not arrive.
    pub errors: Vec<String>,
    /// The title the tab showed.
    pub title: String,
    /// Milliseconds to the first content on screen, or -1 where the
    /// browser reported none.
    pub first_contentful_paint: i64,
    /// Every byte the page fetched, as the network counted them.
    pub transferred: u64,
    pub requests: u64,
    /// Elements in the document once it had settled.
    pub elements: u64,
    /// Characters of text in the main landmark.
    pub text: u64,
    /// The `wf-*` classes the page carried — what it actually drew.
    pub drew: Vec<String>,
}

/// One page a browser has open.
pub struct Page {
    target: String,
    session: String,
    visit: Visit,
}

impl Page {
    /// Anything the page said went wrong, in the order it said it.
    pub fn errors(&self) -> &[String] {
        &self.visit.errors
    }
}

pub struct Browser {
    child: Child,
    socket: Socket,
    next: u64,
    /// Events that arrived while waiting for the answer to something
    /// else. The protocol interleaves them on one socket, so a request
    /// reads past whatever the page was saying at the time — and what it
    /// was saying is exactly what a test needs to hear.
    heard: Vec<Value>,
}

impl Browser {
    /// Start one, or say plainly that there is none.
    pub fn start() -> Result<Self> {
        let binary = find().ok_or_else(|| {
            WebFluentError::IoError(
                "no Chrome or Chromium on this machine. Install one, or point `WF_CHROME` at it"
                    .to_string(),
            )
        })?;
        let profile = std::env::temp_dir().join(format!("wf-chrome-{}", std::process::id()));
        let mut child = Command::new(binary)
            .args([
                "--headless=new",
                "--remote-debugging-port=0",
                "--no-sandbox",
                "--disable-gpu",
                "--disable-dev-shm-usage",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-extensions",
                "--hide-scrollbars",
                &format!("--user-data-dir={}", profile.display()),
                "about:blank",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| WebFluentError::IoError(format!("could not start the browser: {e}")))?;

        // It prints its endpoint on stderr once it is listening.
        let stderr = child.stderr.take().expect("piped");
        let mut lines = BufReader::new(stderr).lines();
        let mut endpoint = None;
        for _ in 0..200 {
            let Some(Ok(line)) = lines.next() else { break };
            if let Some(at) = line.find("ws://") {
                endpoint = Some(line[at..].trim().to_string());
                break;
            }
        }
        let endpoint = endpoint.ok_or_else(|| {
            let _ = child.kill();
            WebFluentError::IoError("the browser started but never said where to talk to it".into())
        })?;
        let socket = Socket::connect(&endpoint)?;
        socket.deadline(30)?;
        Ok(Browser {
            child,
            socket,
            next: 1,
            heard: Vec::new(),
        })
    }

    fn call(&mut self, method: &str, params: Value, session: Option<&str>) -> Result<Value> {
        let id = self.next;
        self.next += 1;
        let mut message = json!({ "id": id, "method": method, "params": params });
        if let Some(session) = session {
            message["sessionId"] = json!(session);
        }
        self.socket.send(&message.to_string())?;
        // Everything else on the wire is an event; keep reading until the
        // answer to this one arrives.
        loop {
            let text = self.socket.receive()?;
            let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            if value.get("id").and_then(Value::as_u64) == Some(id) {
                if let Some(error) = value.get("error") {
                    return Err(WebFluentError::IoError(format!("browser: {error}")));
                }
                return Ok(value.get("result").cloned().unwrap_or(Value::Null));
            }
            if value.get("method").is_some() {
                self.heard.push(value);
            }
        }
    }

    /// Open `url` and wait for it to settle. The page stays open until
    /// it is closed, so a caller can act on it.
    pub fn open(&mut self, url: &str, settle_ms: u64) -> Result<Page> {
        let target = self.call("Target.createTarget", json!({ "url": "about:blank" }), None)?;
        let target_id = target["targetId"].as_str().unwrap_or_default().to_string();
        let attached = self.call(
            "Target.attachToTarget",
            json!({ "targetId": target_id, "flatten": true }),
            None,
        )?;
        let session = attached["sessionId"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        for method in [
            "Runtime.enable",
            "Log.enable",
            "Network.enable",
            "Page.enable",
        ] {
            self.call(method, json!({}), Some(&session))?;
        }
        let mut page = Page {
            target: target_id,
            session,
            visit: Visit {
                url: url.to_string(),
                ..Default::default()
            },
        };
        self.call("Page.navigate", json!({ "url": url }), Some(&page.session))?;
        self.settle(&mut page, settle_ms)?;
        Ok(page)
    }

    /// Read whatever the page has been saying for `ms`, and remember it.
    ///
    /// The events arrive on the same socket as the answers, so this is
    /// also how a page is given time: it is spent listening.
    pub fn settle(&mut self, page: &mut Page, ms: u64) -> Result<()> {
        // What was said while something else was being asked.
        for event in std::mem::take(&mut self.heard) {
            self.note(&event, &mut page.visit);
        }
        self.socket.deadline(2)?;
        let until = std::time::Instant::now() + std::time::Duration::from_millis(ms);
        while std::time::Instant::now() < until {
            let Ok(text) = self.socket.receive() else {
                break;
            };
            let Ok(event): std::result::Result<Value, _> = serde_json::from_str(&text) else {
                continue;
            };
            self.note(&event, &mut page.visit);
        }
        self.socket.deadline(30)
    }

    /// Run `expression` in the page and hand back what it gave.
    pub fn eval(&mut self, page: &Page, expression: &str) -> Result<Value> {
        let result = self.call(
            "Runtime.evaluate",
            json!({
                "expression": expression,
                "returnByValue": true,
                "awaitPromise": true,
            }),
            Some(&page.session),
        )?;
        if let Some(details) = result.get("exceptionDetails") {
            let text = details["exception"]["description"]
                .as_str()
                .or_else(|| details["text"].as_str())
                .unwrap_or("(no message)");
            return Err(WebFluentError::IoError(format!("in the page: {text}")));
        }
        Ok(result["result"]["value"].clone())
    }

    /// Close the page and hand back what it did.
    pub fn close(&mut self, page: Page) -> Result<Visit> {
        self.call(
            "Target.closeTarget",
            json!({ "targetId": page.target }),
            None,
        )?;
        Ok(page.visit)
    }

    /// Open `url`, let it settle, and say what happened.
    pub fn visit(&mut self, url: &str, settle_ms: u64) -> Result<Visit> {
        let mut page = self.open(url, settle_ms)?;
        let session = page.session.clone();

        // And what the page is, now that it has.
        let settled = self.call(
            "Runtime.evaluate",
            json!({ "expression": PAGE_REPORT, "returnByValue": true }),
            Some(&session),
        )?;
        let visit = &mut page.visit;
        if let Some(text) = settled["result"]["value"].as_str()
            && let Ok(report) = serde_json::from_str::<Value>(text)
        {
            visit.title = report["title"].as_str().unwrap_or_default().to_string();
            visit.first_contentful_paint = report["fcp"].as_i64().unwrap_or(-1);
            visit.elements = report["elements"].as_u64().unwrap_or(0);
            visit.text = report["text"].as_u64().unwrap_or(0);
            visit.drew = report["drew"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            for broken in report["images"].as_array().into_iter().flatten() {
                visit.errors.push(format!(
                    "an image did not load: {}",
                    broken.as_str().unwrap_or("")
                ));
            }
        }

        self.close(page)
    }

    /// The text a reader would see, for an `expect`.
    pub fn visible_text(&mut self, page: &Page) -> Result<String> {
        let value = self.eval(
            page,
            "((document.querySelector('main') || document.body).innerText || '')",
        )?;
        Ok(value.as_str().unwrap_or_default().to_string())
    }

    /// What one event says about the page.
    fn note(&self, event: &Value, visit: &mut Visit) {
        let method = event["method"].as_str().unwrap_or_default();
        let params = &event["params"];
        match method {
            "Runtime.exceptionThrown" => {
                let details = &params["exceptionDetails"];
                let text = details["exception"]["description"]
                    .as_str()
                    .or_else(|| details["text"].as_str())
                    .unwrap_or("(no message)");
                visit.errors.push(format!("uncaught: {text}"));
            }
            "Runtime.consoleAPICalled" if params["type"] == "error" => {
                let text: Vec<String> = params["args"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .map(|v| {
                                v["description"]
                                    .as_str()
                                    .or_else(|| v["value"].as_str())
                                    .unwrap_or("")
                                    .to_string()
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                visit
                    .errors
                    .push(format!("console.error: {}", text.join(" ")));
            }
            "Log.entryAdded" if params["entry"]["level"] == "error" => {
                let entry = &params["entry"];
                visit.errors.push(format!(
                    "{}: {}",
                    entry["source"].as_str().unwrap_or("log"),
                    entry["text"].as_str().unwrap_or("")
                ));
            }
            "Network.requestWillBeSent" => visit.requests += 1,
            "Network.loadingFinished" => {
                visit.transferred += params["encodedDataLength"].as_u64().unwrap_or(0);
            }
            "Network.responseReceived" => {
                let status = params["response"]["status"].as_u64().unwrap_or(0);
                if status >= 400 {
                    visit.errors.push(format!(
                        "{status} {}",
                        params["response"]["url"].as_str().unwrap_or("")
                    ));
                }
            }
            "Network.loadingFailed" if params["canceled"] != json!(true) => {
                visit.errors.push(format!(
                    "the browser could not fetch something: {}",
                    params["errorText"].as_str().unwrap_or("")
                ));
            }
            _ => {}
        }
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// What the page says about itself once it has settled.
const PAGE_REPORT: &str = r#"(() => {
  const paint = performance.getEntriesByType("paint");
  const fcp = paint.find((p) => p.name === "first-contentful-paint");
  return JSON.stringify({
    title: document.title,
    fcp: Math.round(fcp ? fcp.startTime : -1),
    elements: document.getElementsByTagName("*").length,
    text: (document.querySelector("main") || document.body).textContent.trim().length,
    drew: [...new Set([...document.querySelectorAll('[class*="wf-"]')]
      .flatMap((n) => [...n.classList].filter((c) => c.startsWith("wf-"))))].sort(),
    // An image that finished and has no size failed. One that has not
    // finished is lazy and off-screen, which is what it was asked to be.
    images: [...document.images]
      .filter((i) => i.complete && i.naturalWidth === 0 && i.getAttribute("src"))
      .map((i) => i.currentSrc || i.src),
  });
})()"#;
