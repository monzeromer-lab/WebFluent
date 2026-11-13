//! The tools that run for a while or start other processes: they clean up
//! after themselves, and what they are handed cannot crash them.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("wf-tooling-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("public")).unwrap();
        std::fs::write(dir.join("webfluent.app.json"), r#"{ "name": "t" }"#).unwrap();
        std::fs::write(
            dir.join("src/App.wf"),
            "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") { Heading(\"Home\").h1 }\n",
        )
        .unwrap();
        Scratch(dir)
    }

    fn wf(&self, args: &[&str]) -> (Option<i32>, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_wf"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("wf runs");
        (
            out.status.code(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
        )
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn an_svg_with_a_typographic_quote_does_not_crash_wf_assets() {
    let s = Scratch::new("svg-quote");
    for (name, svg) in [
        (
            "a.svg",
            "<svg width=\u{201c}10\u{201d} height=\u{201c}20\u{201d}></svg>",
        ),
        ("b.svg", "<svg viewBox=\u{201c}0 0 10 10\u{201d}></svg>"),
        (
            "c.svg",
            "<svg width=\u{00ab}10\u{00bb} viewBox=\u{2019}0 0 1 1\u{2019}></svg>",
        ),
        ("d.svg", "<svg width=\"10\" height='20'></svg>"),
    ] {
        std::fs::write(s.0.join("public").join(name), svg).unwrap();
    }
    let (code, out) = s.wf(&["assets", "--json"]);
    assert_eq!(code, Some(0), "{out}");
    let json: serde_json::Value = serde_json::from_str(&out).expect(&out);
    // The plain one is still read.
    assert!(json.to_string().contains("\"public/d.svg\""), "{json}");
}

/// A PNG `width` wide.
fn png(width: u32, shade: u8) -> Vec<u8> {
    let img = image::RgbImage::from_fn(width, 16, |x, y| {
        image::Rgb([(x % 256) as u8, (y * 9) as u8 ^ shade, shade])
    });
    let mut bytes = std::io::Cursor::new(Vec::new());
    img.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
    bytes.into_inner()
}

fn cached(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir.join(".wf-cache/media"))
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn the_image_cache_keeps_only_what_the_last_build_used() {
    let s = Scratch::new("media-prune");
    std::fs::write(s.0.join("src/hero.png"), png(1000, 1)).unwrap();
    std::fs::write(
        s.0.join("src/App.wf"),
        "image hero = \"hero.png\"\napp { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") { Heading(\"Home\").h1 Image(hero, alt: \"x\") }\n",
    )
    .unwrap();
    let (code, out) = s.wf(&["build"]);
    assert_eq!(code, Some(0), "{out}");
    let first = cached(&s.0);
    assert!(!first.is_empty(), "the build caches its encodings");
    // The same build again keeps them all.
    let (code, out) = s.wf(&["build"]);
    assert_eq!(code, Some(0), "{out}");
    assert_eq!(cached(&s.0), first);
    // Another picture: the old one's encodings go.
    std::fs::write(s.0.join("src/hero.png"), png(1000, 200)).unwrap();
    let (code, out) = s.wf(&["build"]);
    assert_eq!(code, Some(0), "{out}");
    let second = cached(&s.0);
    assert_ne!(second, [] as [String; 0]);
    assert!(
        first.iter().all(|f| !second.contains(f)),
        "{first:?} still beside {second:?}"
    );
}

// ─── The browser ─────────────────────────────────────────────────────

/// Every process whose command line mentions `needle`, with that line.
#[cfg(target_os = "linux")]
fn processes_mentioning(needle: &str) -> Vec<(u32, String)> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        let line = String::from_utf8_lossy(&cmdline).replace('\0', " ");
        if line.contains(needle) {
            found.push((pid, line));
        }
    }
    found
}

/// A project with enough routes that `wf verify` is still walking them a
/// few seconds in.
#[cfg(target_os = "linux")]
fn many_routes(name: &str) -> Scratch {
    let s = Scratch::new(name);
    let slugs: Vec<String> = (0..60).map(|i| format!("\"p{i}\"")).collect();
    std::fs::write(s.0.join("src/slugs.json"), format!("[{}]", slugs.join(","))).unwrap();
    std::fs::write(
        s.0.join("webfluent.app.json"),
        r#"{ "name": "t", "build": { "ssg": true } }"#,
    )
    .unwrap();
    std::fs::write(
        s.0.join("src/App.wf"),
        "data slugs = \"slugs.json\"\napp { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") { Heading(\"Home\").h1 }\npage P(path: \"/p/:slug\", title: \"P\", description: \"D\", slug: String, paths: slugs) { Heading(slug).h1 }\n",
    )
    .unwrap();
    let (code, out) = s.wf(&["build"]);
    assert_eq!(code, Some(0), "{out}");
    s
}

#[cfg(target_os = "linux")]
#[test]
fn a_killed_verify_leaves_no_browser_and_no_profile_behind() {
    let s = many_routes("kill-verify");
    let tmp = s.0.join("tmp");
    std::fs::create_dir_all(&tmp).unwrap();
    let tmp_text = tmp.to_string_lossy().to_string();
    let mut child = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("verify")
        .current_dir(&s.0)
        .env("TMPDIR", &tmp)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Wait for the browser to be up.
    let started = Instant::now();
    let mut browser = Vec::new();
    while started.elapsed() < Duration::from_secs(30) {
        browser = processes_mentioning(&tmp_text)
            .into_iter()
            .filter(|(_, line)| line.contains("--user-data-dir"))
            .collect();
        if !browser.is_empty() || matches!(child.try_wait(), Ok(Some(_))) {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    if browser.is_empty() {
        let out = child.wait_with_output().unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("no Chrome or Chromium")
                && std::env::var_os("WF_REQUIRE_CHROME").is_none(),
            "the browser never started: {err}"
        );
        eprintln!("\n  SKIPPED: no Chrome or Chromium.\n");
        return;
    }
    // Its profile is a private directory with a name nobody could guess.
    let profiles: Vec<PathBuf> = std::fs::read_dir(&tmp)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("wf-chrome-")
        })
        .collect();
    // A site of many routes is visited by a few browsers at once, each with
    // a profile of its own; however many have started, each is private.
    assert!(!profiles.is_empty(), "{profiles:?}");
    for profile in &profiles {
        let name = profile.file_name().unwrap().to_string_lossy().to_string();
        assert!(!name.contains(&child.id().to_string()), "{name}");
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(profile).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700, "{mode:o}");
    }
    // It runs with its sandbox, unless it said why not.
    let sandboxless = browser
        .iter()
        .any(|(_, line)| line.contains("--no-sandbox"));
    // Killed as a signal would: no Drop, no clean-up of its own.
    child.kill().unwrap();
    let out = child.wait_with_output().unwrap();
    if sandboxless {
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("sandbox"),
            "--no-sandbox without a word: {err}"
        );
    }
    let gone = Instant::now();
    loop {
        let left = processes_mentioning(&tmp_text);
        let profiles_left: Vec<String> = std::fs::read_dir(&tmp)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|name| name.starts_with("wf-"))
            .collect();
        if left.is_empty() && profiles_left.is_empty() {
            break;
        }
        assert!(
            gone.elapsed() < Duration::from_secs(20),
            "left behind: {left:?} and {profiles_left:?}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// A test that acts is served `public/` beside its page, through the links
/// its site makes to the project's files — and nothing a link inside
/// `public/` leads to beyond the project.
#[cfg(target_os = "linux")]
#[test]
fn an_acting_test_reads_public_but_not_past_it() {
    let s = Scratch::new("act-public");
    std::fs::write(s.0.join("public/data.json"), r#"{ "word": "hello" }"#).unwrap();
    let secret = s.0.join("secret.json");
    std::fs::write(&secret, r#"{ "word": "SECRET" }"#).unwrap();
    std::os::unix::fs::symlink(&secret, s.0.join("public/leak.json")).unwrap();
    std::fs::create_dir_all(s.0.join("tests")).unwrap();
    std::fs::write(
        s.0.join("tests/public.wf"),
        "test \"public is served\" {\n    state got = \"\"\n    Button(\"Load\") { on click { let r = await fetch(\"/data.json\")  got = r.word } }\n    Text(\"got {got}\")\n    click \"Load\"\n    expect \"got hello\"\n}\ntest \"a link out is not\" {\n    state got = \"none\"\n    Button(\"Load\") { on click { try { let r = await fetch(\"/leak.json\")  got = r.word } catch e { got = \"refused\" } } }\n    Text(\"got {got}\")\n    click \"Load\"\n    expect not \"SECRET\"\n}\n",
    )
    .unwrap();
    let (_, out) = s.wf(&["test"]);
    if out.contains("no Chrome or Chromium") {
        assert!(std::env::var_os("WF_REQUIRE_CHROME").is_none(), "{out}");
        return;
    }
    assert!(
        out.contains("ok    tests/public.wf — public is served"),
        "{out}"
    );
    // The link out is a 404, which fails that test: what matters is that
    // nothing past `public/` reached the page.
    assert!(out.contains("404") && out.contains("/leak.json"), "{out}");
    assert!(!out.contains("SECRET"), "{out}");
}

#[cfg(target_os = "linux")]
#[test]
fn wf_test_leaves_no_work_directory_behind() {
    let s = Scratch::new("act-work");
    std::fs::create_dir_all(s.0.join("tests")).unwrap();
    std::fs::write(
        s.0.join("tests/counter.wf"),
        "test \"a click counts\" {\n    state n = 0\n    Button(\"Add\") { on click { n = n + 1 } }\n    Text(\"{n} clicks\")\n    click \"Add\"\n    expect \"1 clicks\"\n}\n",
    )
    .unwrap();
    let tmp = s.0.join("tmp");
    std::fs::create_dir_all(&tmp).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("test")
        .current_dir(&s.0)
        .env("TMPDIR", &tmp)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    if err.contains("no Chrome or Chromium") {
        assert!(std::env::var_os("WF_REQUIRE_CHROME").is_none(), "{err}");
        return;
    }
    assert!(
        out.status.success(),
        "{}{err}",
        String::from_utf8_lossy(&out.stdout)
    );
    // What wf made: its sites' directory and the browser's profile. (The
    // browser keeps a temporary file of its own, which is its business.)
    let left: Vec<String> = std::fs::read_dir(&tmp)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.starts_with("wf-"))
        .collect();
    assert!(left.is_empty(), "left behind: {left:?}");
}
