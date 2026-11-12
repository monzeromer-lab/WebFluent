//! `wf serve` serves the build and nothing else, to this machine unless it
//! is told otherwise, and only to a name it is reached by.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// A scratch folder holding a project (`proj/`) and a secret beside it.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str, port: u16) -> Self {
        let dir = std::env::temp_dir().join(format!("wf-servesec-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("proj/src")).unwrap();
        std::fs::create_dir_all(dir.join("proj/public")).unwrap();
        std::fs::write(dir.join("secret.txt"), "TOP-SECRET-VALUE").unwrap();
        std::fs::write(
            dir.join("proj/webfluent.app.json"),
            format!(r#"{{ "name": "s", "dev": {{ "port": {port}, "hot_reload": false }} }}"#),
        )
        .unwrap();
        std::fs::write(
            dir.join("proj/src/App.wf"),
            "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") { Heading(\"Home\").h1 }\n",
        )
        .unwrap();
        Scratch(dir)
    }

    fn proj(&self) -> PathBuf {
        self.0.join("proj")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A port nothing is listening on.
fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// A `wf serve` that is killed when the test ends, passing or not.
struct Serving(Option<Child>);

impl Serving {
    fn start(dir: &Path, port: u16, args: &[&str]) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_wf"))
            .arg("serve")
            .args(args)
            .current_dir(dir)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("wf serve starts");
        let serving = Serving(Some(child));
        let started = Instant::now();
        while raw(
            port,
            "GET /__wf/status HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )
        .is_none_or(|r| !r.starts_with("HTTP/1.1 200"))
        {
            assert!(
                started.elapsed() < Duration::from_secs(60),
                "wf serve never answered"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
        serving
    }

    /// Stop it, and hand back what it printed to standard error.
    fn stop(&mut self) -> Vec<u8> {
        let Some(mut child) = self.0.take() else {
            return Vec::new();
        };
        let _ = child.kill();
        let _ = child.wait();
        let mut err = Vec::new();
        if let Some(mut pipe) = child.stderr.take() {
            let _ = pipe.read_to_end(&mut err);
        }
        err
    }
}

impl Drop for Serving {
    fn drop(&mut self) {
        self.stop();
    }
}

/// `request`, sent as it is, and the whole reply.
fn raw(port: u16, request: &str) -> Option<String> {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .ok()?;
    stream.write_all(request.as_bytes()).ok()?;
    let mut reply = Vec::new();
    stream.read_to_end(&mut reply).ok()?;
    Some(String::from_utf8_lossy(&reply).to_string())
}

fn get(port: u16, path: &str) -> String {
    raw(
        port,
        &format!("GET {path} HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n\r\n"),
    )
    .unwrap_or_default()
}

#[test]
fn nothing_outside_the_build_is_served() {
    let port = free_port();
    let s = Scratch::new("traversal", port);
    let mut serving = Serving::start(&s.proj(), port, &[]);
    assert!(get(port, "/").starts_with("HTTP/1.1 200"));
    for path in [
        "/../secret.txt",
        "/../../secret.txt",
        "/%2e%2e/%2e%2e/secret.txt",
        "/%2E%2E/%2E%2E/secret.txt",
        "/.%2e/.%2e/secret.txt",
        "/..%2f..%2fsecret.txt",
        "/build/..%2f..%2f..%2fsecret.txt",
        "/..\\..\\secret.txt",
        "/%5c..%5c..%5csecret.txt",
        "//etc/passwd",
        "/../../../../../../etc/passwd",
        "/%2fetc%2fpasswd",
        "/index.html%00.txt",
        "/%ff%fe",
        "/ünïcode/../../secret.txt",
    ] {
        let reply = get(port, path);
        assert!(!reply.contains("TOP-SECRET-VALUE"), "{path}: {reply}");
        assert!(!reply.contains("root:"), "{path}: {reply}");
        // An address that names the root is a route of the site, and gets
        // its shell; one that climbs is refused outright.
        if !path.contains("etc") {
            assert!(!reply.starts_with("HTTP/1.1 200"), "{path}: {reply}");
        }
    }
    // A link in the output that leads out of it names nothing.
    #[cfg(unix)]
    std::os::unix::fs::symlink(s.0.join("secret.txt"), s.proj().join("build/leak.txt")).unwrap();
    let reply = get(port, "/leak.txt");
    assert!(!reply.contains("TOP-SECRET-VALUE"), "{reply}");
    // A file with a space in its name is found by its encoded address.
    std::fs::write(s.proj().join("build/a b.txt"), "spaced").unwrap();
    assert!(get(port, "/a%20b.txt").contains("spaced"));
    serving.stop();
}

#[test]
fn only_a_name_this_server_is_reached_by_is_answered() {
    let port = free_port();
    let s = Scratch::new("host", port);
    let _serving = Serving::start(&s.proj(), port, &[]);
    let with_host = |host: &str| {
        raw(
            port,
            &format!("GET / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"),
        )
        .unwrap_or_default()
    };
    for host in [
        "localhost",
        &format!("localhost:{port}"),
        "127.0.0.1",
        &format!("127.0.0.1:{port}"),
        "[::1]:1",
        "app.localhost",
    ] {
        assert!(with_host(host).starts_with("HTTP/1.1 200"), "{host}");
    }
    for host in [
        "evil.example",
        &format!("evil.example:{port}"),
        "localhost.evil.example",
        "attacker.local",
    ] {
        let reply = with_host(host);
        assert!(reply.starts_with("HTTP/1.1 421"), "{host}: {reply}");
        assert!(!reply.contains("<html"), "{host}: {reply}");
    }
    let status = with_host("evil.example");
    assert!(!status.contains("version"), "{status}");
}

#[test]
fn it_listens_on_this_machine_only_unless_told() {
    let port = free_port();
    let s = Scratch::new("bind", port);
    let _serving = Serving::start(&s.proj(), port, &[]);
    // This machine's address on its network, where it has one.
    let lan = std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|u| u.connect("192.0.2.1:9").map(|()| u))
        .and_then(|u| u.local_addr())
        .map(|a| a.ip())
        .ok()
        .filter(|ip| !ip.is_loopback() && !ip.is_unspecified());
    if let Some(ip) = lan {
        assert!(
            std::net::TcpStream::connect_timeout(&(ip, port).into(), Duration::from_secs(2))
                .is_err(),
            "reachable at {ip}"
        );
    }
}

#[test]
fn told_to_listen_on_every_interface_it_answers_a_phone_but_no_other_site() {
    let port = free_port();
    let s = Scratch::new("everywhere", port);
    let _serving = Serving::start(&s.proj(), port, &["--host", "0.0.0.0"]);
    let with_host = |host: &str| {
        raw(
            port,
            &format!("GET / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"),
        )
        .unwrap_or_default()
    };
    assert!(with_host(&format!("192.168.1.20:{port}")).starts_with("HTTP/1.1 200"));
    assert!(with_host("my-laptop.local").starts_with("HTTP/1.1 200"));
    assert!(with_host("evil.example").starts_with("HTTP/1.1 421"));
}

#[test]
fn a_slow_client_does_not_hold_up_the_rest() {
    let port = free_port();
    let s = Scratch::new("slow", port);
    // A file far bigger than a socket's buffers.
    std::fs::write(s.proj().join("public/big.bin"), vec![7u8; 64 * 1024 * 1024]).unwrap();
    let _serving = Serving::start(&s.proj(), port, &[]);
    // Ask for it, and read nothing.
    let mut stalled = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(
        stalled,
        "GET /big.bin HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let started = Instant::now();
    let reply = get(port, "/");
    assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "a page waited {:?} behind a stalled download",
        started.elapsed()
    );
    drop(stalled);
}

#[test]
fn a_logged_address_cannot_write_to_the_terminal() {
    let port = free_port();
    let s = Scratch::new("log", port);
    let mut serving = Serving::start(&s.proj(), port, &[]);
    let _ = raw(
        port,
        "GET /api/\x1b]0;owned\x07\x1b[2J HTTP/1.1\r\nHost: localhost\r\nSec-Fetch-Mode: cors\r\nConnection: close\r\n\r\n",
    );
    let _ = raw(
        port,
        "GET /../\x1b[31mred HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    let err = serving.stop();
    // Said, and written as its code.
    assert!(
        String::from_utf8_lossy(&err).contains("\\u{1b}"),
        "{}",
        String::from_utf8_lossy(&err)
    );
    assert!(!err.contains(&0x1b), "{}", String::from_utf8_lossy(&err));
    assert!(!err.contains(&0x07), "{}", String::from_utf8_lossy(&err));
}
