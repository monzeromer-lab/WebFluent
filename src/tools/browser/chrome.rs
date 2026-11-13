//! A headless browser, started and driven by the build.
//!
//! What a browser knows that a compiler cannot: whether the page loads,
//! whether anything threw on the way, whether every file it asked for
//! arrived, and how long it took to paint. `wf verify` asks it those
//! questions for every route a project serves.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::ws::Socket;
use crate::diagnostics::error::{Result, WebFluentError};

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
        .map(std::string::ToString::to_string)
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
#[derive(Debug)]
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

#[derive(Debug)]
pub struct Browser {
    /// A script every page runs before its own: what `--returning-visitor`
    /// puts in storage.
    before_each: Option<String>,
    child: Child,
    socket: Socket,
    next: u64,
    /// Events that arrived while waiting for the answer to something
    /// else. The protocol interleaves them on one socket, so a request
    /// reads past whatever the page was saying at the time — and what it
    /// was saying is exactly what a test needs to hear.
    heard: Vec<Value>,
    /// The profile Chrome was started with: a directory of this browser's
    /// own, removed with it. A profile is ~150 MB, and where `/tmp` is
    /// memory, one left behind per run fills it and then the swap.
    profile: PathBuf,
}

/// Directories this process has made, so two made in the same instant
/// still differ.
static MADE: AtomicU64 = AtomicU64::new(0);

/// A new directory of the temporary directory's, named `<prefix>` and
/// something nobody can guess, made by this call and nobody else's: the
/// name is not reused if something is there already (`mkdir` fails rather
/// than open what another user put in its place), and only this user may
/// read or write it.
///
/// # Errors
///
/// The [`std::io::Error`] of making the directory when it cannot be made,
/// or an `AlreadyExists` one when 64 names in a row were taken.
pub fn private_dir(prefix: &str) -> std::io::Result<PathBuf> {
    use std::hash::{BuildHasher, Hasher};
    let tmp = std::env::temp_dir();
    let mut last = None;
    for _ in 0..64 {
        // A hasher seeded at random by the standard library, over the time
        // and a count: a name nobody can work out from the process id.
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos()),
        );
        hasher.write_u64(MADE.fetch_add(1, Ordering::Relaxed));
        let dir = tmp.join(format!("{prefix}{:016x}", hasher.finish()));
        #[cfg_attr(
            not(unix),
            allow(unused_mut, reason = "only a Unix system sets the directory's mode")
        )]
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        match builder.create(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => last = Some(e),
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("no name was free")))
}

/// What watches the browser on wf's behalf: a shell, in a process group of
/// its own with the browser, that starts it and then reads its input. wf
/// holds that input open for as long as it wants the browser; when wf ends —
/// done, killed, crashed — the input closes, and the shell stops the
/// browser and its helpers and removes the profile. Without it a `wf verify`
/// stopped by a signal left a dozen Chrome processes and their profile.
#[cfg(unix)]
const WATCHER: &str = r#"profile="$1"; shift
"$@" &
pid=$!
while read -r _; do :; done
trap '' TERM
kill -TERM "$pid" 2>/dev/null
n=0
while kill -0 "$pid" 2>/dev/null && [ $n -lt 50 ]; do sleep 0.1; n=$((n+1)); done
kill -KILL "$pid" 2>/dev/null
kill -TERM 0 2>/dev/null
sleep 0.2
rm -rf -- "$profile"
"#;

/// Whether to start the browser without its sandbox from the outset:
/// `WF_CHROME_NO_SANDBOX=1`, for a machine where it cannot have one.
fn sandbox_off_by_request() -> bool {
    std::env::var("WF_CHROME_NO_SANDBOX").is_ok_and(|v| v == "1")
}

/// The command that starts `binary` with `args`, watched where there is a
/// shell to watch it.
fn launch(binary: &str, args: &[String], profile: &Path) -> std::io::Result<Child> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Command::new("/bin/sh")
            .arg("-c")
            .arg(WATCHER)
            .arg("wf-chrome")
            .arg(profile)
            .arg(binary)
            .args(args)
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
    }
    #[cfg(not(unix))]
    {
        let _ = profile;
        Command::new(binary)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
    }
}

/// Stop a browser that will not be used, and remove its profile once
/// nothing writes to it.
fn abandon(child: &mut Child, profile: &Path) {
    // Closing the watcher's input is what stops it all; it gets a moment
    // to do so, then whatever is left is killed.
    child.stdin = None;
    let until = Instant::now() + Duration::from_secs(8);
    while Instant::now() < until && !matches!(child.try_wait(), Ok(Some(_))) {
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = child.kill();
    let _ = child.wait();
    // A helper process can outlive the browser by a moment and write a
    // file or two back; remove the profile until it stays removed.
    for _ in 0..20 {
        let _ = std::fs::remove_dir_all(profile);
        if !profile.exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// A browser, started, and the address of its protocol.
fn open_browser(binary: &str, sandbox: bool) -> Result<(Child, PathBuf, String)> {
    let profile = private_dir("wf-chrome-")
        .map_err(|e| WebFluentError::io(format!("could not make the browser's profile: {e}")))?;
    let mut args: Vec<String> = [
        "--headless=new",
        "--remote-debugging-port=0",
        "--disable-gpu",
        "--disable-dev-shm-usage",
        "--no-first-run",
        "--no-default-browser-check",
        "--disable-extensions",
        "--hide-scrollbars",
        // Nothing in the background: no component downloads (the
        // optimisation models made each profile ~150 MB), no sync,
        // no default apps — a page's numbers are the page's own.
        "--disable-background-networking",
        "--disable-component-update",
        "--disable-default-apps",
        "--disable-sync",
    ]
    .iter()
    .map(std::string::ToString::to_string)
    .collect();
    if !sandbox {
        args.push("--no-sandbox".to_string());
    }
    args.push(format!("--user-data-dir={}", profile.display()));
    args.push("about:blank".to_string());
    let mut child = match launch(binary, &args, &profile) {
        Ok(child) => child,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&profile);
            return Err(WebFluentError::io(format!(
                "could not start the browser: {e}"
            )));
        }
    };

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
    if let Some(endpoint) = endpoint {
        Ok((child, profile, endpoint))
    } else {
        abandon(&mut child, &profile);
        Err(WebFluentError::io(
            "the browser started but never said where to talk to it",
        ))
    }
}

impl Browser {
    /// Start one, or say plainly that there is none.
    ///
    /// It runs with its sandbox: what it loads is a project's pages and the
    /// scripts they name. Where the sandbox cannot start (a container that
    /// gives it none) it is started again without one, and that is said;
    /// `WF_CHROME_NO_SANDBOX=1` starts it so from the outset.
    ///
    /// # Errors
    ///
    /// [`WebFluentError::IoError`] when there is no Chrome or Chromium, when it
    /// will not start even without its sandbox, or when its debugging connection
    /// cannot be opened.
    pub fn start() -> Result<Self> {
        let binary = find().ok_or_else(|| {
            WebFluentError::io(
                "no Chrome or Chromium on this machine. Install one, or point `WF_CHROME` at it"
                    .to_string(),
            )
        })?;
        let sandboxed = !sandbox_off_by_request();
        let (mut child, profile, endpoint) = match open_browser(&binary, sandboxed) {
            Ok(started) => started,
            Err(first) if sandboxed => {
                eprintln!(
                    "note: the browser would not start with its sandbox ({first}); it runs without one. Set WF_CHROME_NO_SANDBOX=1 to skip the first try"
                );
                open_browser(&binary, false)?
            }
            Err(e) => return Err(e),
        };
        let socket = match Socket::connect(&endpoint).and_then(|s| s.deadline(30).map(|()| s)) {
            Ok(socket) => socket,
            Err(e) => {
                abandon(&mut child, &profile);
                return Err(e);
            }
        };
        Ok(Browser {
            before_each: None,
            child,
            socket,
            next: 1,
            heard: Vec::new(),
            profile,
        })
    }

    fn call(&mut self, method: &str, params: &Value, session: Option<&str>) -> Result<Value> {
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
                    return Err(WebFluentError::io(format!("browser: {error}")));
                }
                return Ok(value.get("result").cloned().unwrap_or(Value::Null));
            }
            if value.get("method").is_some() {
                self.heard.push(value);
            }
        }
    }

    /// Run `source` in every page opened from now on, before the page's own
    /// scripts; `None` stops it.
    pub fn before_each_page(&mut self, source: Option<String>) {
        self.before_each = source;
    }

    /// Open `url` and wait for it to settle. The page stays open until
    /// it is closed, so a caller can act on it.
    ///
    /// # Errors
    ///
    /// [`WebFluentError::IoError`] when the browser refuses a command or the
    /// connection to it fails.
    pub fn open(&mut self, url: &str, settle_ms: u64) -> Result<Page> {
        let target = self.call(
            "Target.createTarget",
            &json!({ "url": "about:blank" }),
            None,
        )?;
        let target_id = target["targetId"].as_str().unwrap_or_default().to_string();
        let attached = self.call(
            "Target.attachToTarget",
            &json!({ "targetId": target_id, "flatten": true }),
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
            self.call(method, &json!({}), Some(&session))?;
        }
        if let Some(source) = &self.before_each {
            self.call(
                "Page.addScriptToEvaluateOnNewDocument",
                &json!({ "source": source }),
                Some(&session),
            )?;
        }
        let mut page = Page {
            target: target_id,
            session,
            visit: Visit {
                url: url.to_string(),
                ..Default::default()
            },
        };
        self.call("Page.navigate", &json!({ "url": url }), Some(&page.session))?;
        self.settle(&mut page, settle_ms)?;
        Ok(page)
    }

    /// Read whatever the page has been saying for `ms`, and remember it.
    ///
    /// The events arrive on the same socket as the answers, so this is
    /// also how a page is given time: it is spent listening.
    ///
    /// # Errors
    ///
    /// [`WebFluentError::IoError`] when the connection to the browser fails.
    pub fn settle(&mut self, page: &mut Page, ms: u64) -> Result<()> {
        // What was said while something else was being asked.
        for event in std::mem::take(&mut self.heard) {
            self.note(&event, &mut page.visit);
        }
        // As long as was asked, not longer: a quiet page used to hold every
        // settle for the socket's two-second wait, whatever `ms` said.
        let until = std::time::Instant::now() + std::time::Duration::from_millis(ms);
        loop {
            let left = until.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() || !self.socket.ready_within(left)? {
                break;
            }
            self.socket.deadline(30)?;
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
    ///
    /// # Errors
    ///
    /// [`WebFluentError::IoError`] when `expression` throws in the page, the
    /// browser refuses it, or the connection to the browser fails.
    pub fn eval(&mut self, page: &Page, expression: &str) -> Result<Value> {
        let result = self.call(
            "Runtime.evaluate",
            &json!({
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
            return Err(WebFluentError::io(format!("in the page: {text}")));
        }
        Ok(result["result"]["value"].clone())
    }

    /// Close the page and hand back what it did.
    ///
    /// # Errors
    ///
    /// [`WebFluentError::IoError`] when the browser refuses to close the page or
    /// the connection to it fails.
    pub fn close(&mut self, page: Page) -> Result<Visit> {
        self.call(
            "Target.closeTarget",
            &json!({ "targetId": page.target }),
            None,
        )?;
        Ok(page.visit)
    }

    /// Open `url`, let it settle, and say what happened.
    ///
    /// # Errors
    ///
    /// What [`open`](Self::open) and [`close`](Self::close) return, and
    /// [`WebFluentError::IoError`] when the page's report cannot be asked for.
    pub fn visit(&mut self, url: &str, settle_ms: u64) -> Result<Visit> {
        let mut page = self.open(url, settle_ms)?;
        let session = page.session.clone();

        // And what the page is, now that it has.
        let settled = self.call(
            "Runtime.evaluate",
            &json!({ "expression": PAGE_REPORT, "returnByValue": true }),
            Some(&session),
        )?;
        if let Some(text) = settled["result"]["value"].as_str() {
            read_report(&mut page.visit, text);
        }

        self.close(page)
    }

    /// The text a reader would see, for an `expect`.
    ///
    /// # Errors
    ///
    /// What [`eval`](Self::eval) returns.
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
        // Asked to close, Chrome stops its helpers and removes its own
        // temporary files; killed, it leaves both behind. It gets a few
        // seconds to go by itself.
        // Gone, it closes the protocol's connection: read until it does.
        let close = json!({ "id": self.next, "method": "Browser.close" });
        if self.socket.send(&close.to_string()).is_ok() && self.socket.deadline(1).is_ok() {
            let until = Instant::now() + Duration::from_secs(5);
            while Instant::now() < until && self.socket.receive().is_ok() {}
        }
        abandon(&mut self.child, &self.profile);
    }
}

/// What [`PAGE_REPORT`] said, into `visit`: its title, first paint,
/// elements, text, an outlet the router left empty, the built-ins it drew
/// and the images that failed.
pub fn read_report(visit: &mut Visit, text: &str) {
    let Ok(report) = serde_json::from_str::<Value>(text) else {
        return;
    };
    visit.title = report["title"].as_str().unwrap_or_default().to_string();
    visit.first_contentful_paint = report["fcp"].as_i64().unwrap_or(-1);
    visit.elements = report["elements"].as_u64().unwrap_or(0);
    visit.text = report["text"].as_u64().unwrap_or(0);
    if report["outlet"].as_i64() == Some(0) {
        visit.errors.push(
            "the router drew nothing into <main>: no page's route matched, or the page threw before it drew"
                .to_string(),
        );
    }
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

/// What the page says about itself once it has settled: a JSON string.
pub const PAGE_REPORT: &str = r#"(() => {
  const paint = performance.getEntriesByType("paint");
  const fcp = paint.find((p) => p.name === "first-contentful-paint");
  return JSON.stringify({
    title: document.title,
    fcp: Math.round(fcp ? fcp.startTime : -1),
    elements: document.getElementsByTagName("*").length,
    text: (document.querySelector("main") || document.body).textContent.trim().length,
    // The router's outlet: -1 where there is none, else what it holds.
    outlet: (() => { const m = document.getElementById("wf-main"); return m ? m.childNodes.length : -1; })(),
    drew: [...new Set([...document.querySelectorAll('[class*="wf-"]')]
      .flatMap((n) => [...n.classList].filter((c) => c.startsWith("wf-"))))].sort(),
    // An image that finished and has no size failed. One that has not
    // finished is lazy and off-screen, which is what it was asked to be.
    images: [...document.images]
      .filter((i) => i.complete && i.naturalWidth === 0 && i.getAttribute("src"))
      .map((i) => i.currentSrc || i.src),
  });
})()"#;
