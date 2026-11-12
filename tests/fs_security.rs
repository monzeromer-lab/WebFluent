//! A project builds inside itself: what it reads is its own, and what the
//! build writes, copies and deletes stays in its output. A project is
//! something a developer clones and builds, so none of this may be left to
//! the project's good intentions.

#![cfg(unix)]

use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const APP: &str = "app { Router }\npage Home(path: \"/\", title: \"H\", description: \"D\") { Heading(\"Home\").h1 }\n";

/// A scratch folder holding a project (`proj/`) and things outside it.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("wf-fssec-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("proj/src")).unwrap();
        std::fs::write(dir.join("secret.txt"), "TOP-SECRET-VALUE").unwrap();
        std::fs::write(
            dir.join("secret.json"),
            r#"{ "token": "TOP-SECRET-VALUE" }"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("outside")).unwrap();
        std::fs::write(dir.join("outside/inner.txt"), "TOP-SECRET-VALUE").unwrap();
        let s = Scratch(dir);
        s.config(r#"{ "name": "p" }"#);
        s.write("src/App.wf", APP);
        s
    }

    fn proj(&self) -> PathBuf {
        self.0.join("proj")
    }

    fn config(&self, json: &str) {
        self.write("webfluent.app.json", json);
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.proj().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn wf(&self, args: &[&str]) -> (i32, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_wf"))
            .args(args)
            .current_dir(self.proj())
            .output()
            .expect("wf runs");
        (
            out.status.code().unwrap_or(-1),
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

/// Every file under `dir`, read, joined: what a deploy would publish.
fn everything_under(dir: &Path) -> String {
    let mut all = String::new();
    let mut walk = vec![dir.to_path_buf()];
    while let Some(at) = walk.pop() {
        let Ok(entries) = std::fs::read_dir(&at) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                walk.push(path);
            } else {
                all.push_str(&String::from_utf8_lossy(
                    &std::fs::read(&path).unwrap_or_default(),
                ));
            }
        }
    }
    all
}

// ─── build.output ────────────────────────────────────────────────────

#[test]
fn an_output_outside_the_project_is_refused_unless_asked_for() {
    for output in [
        "../victim",
        "/tmp/wf-fssec-absolute-victim",
        "build/../../victim",
        "..\\victim",
    ] {
        let s = Scratch::new("out-outside");
        std::fs::create_dir_all(s.0.join("victim")).unwrap();
        std::fs::write(s.0.join("victim/keep.tar.gz"), "not ours").unwrap();
        s.write("public/planted.sh", "echo planted");
        s.config(&format!(
            r#"{{ "name": "p", "build": {{ "output": "{}" }} }}"#,
            output.replace('\\', "\\\\")
        ));
        let (code, out) = s.wf(&["build"]);
        assert_ne!(code, 0, "{output}: {out}");
        assert!(out.contains("build.output"), "{output}: {out}");
        assert!(out.contains("--allow-output-outside"), "{output}: {out}");
        assert!(!s.0.join("victim/planted.sh").exists(), "{output}");
        assert!(
            !Path::new("/tmp/wf-fssec-absolute-victim").exists(),
            "{output}"
        );
        assert!(s.0.join("victim/keep.tar.gz").exists(), "{output}");
        // `wf check` says so too, and `wf serve` and `wf verify` refuse to
        // serve it.
        let (code, out) = s.wf(&["check"]);
        assert_ne!(code, 0, "{output}: {out}");
        let (code, out) = s.wf(&["verify"]);
        assert_ne!(code, 0, "{output}: {out}");
        assert!(out.contains("build.output"), "{output}: {out}");
    }
}

#[test]
fn an_output_outside_the_project_is_written_when_the_command_line_allows_it() {
    let s = Scratch::new("out-allowed");
    std::fs::create_dir_all(s.0.join("site")).unwrap();
    std::fs::write(s.0.join("site/keep.tar.gz"), "not ours").unwrap();
    std::fs::write(s.0.join("site/orphan.js.gz"), "not ours either").unwrap();
    s.config(r#"{ "name": "p", "build": { "output": "../site" } }"#);
    let (code, out) = s.wf(&["build", "--allow-output-outside"]);
    assert_eq!(code, 0, "{out}");
    assert!(s.0.join("site/index.html").is_file());
    // A `.gz` the build did not write is not its to delete.
    assert!(s.0.join("site/keep.tar.gz").exists());
    assert!(s.0.join("site/orphan.js.gz").exists());
}

#[test]
fn an_output_that_is_a_link_out_of_the_project_is_refused() {
    let s = Scratch::new("out-link");
    symlink(s.0.join("outside"), s.proj().join("build")).unwrap();
    let (code, out) = s.wf(&["build"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("link"), "{out}");
    assert!(!s.0.join("outside/index.html").exists());
}

#[test]
fn a_link_inside_the_output_is_not_written_through() {
    let s = Scratch::new("out-inner-link");
    std::fs::create_dir_all(s.proj().join("build")).unwrap();
    symlink(s.0.join("secret.txt"), s.proj().join("build/index.html")).unwrap();
    let (code, out) = s.wf(&["build"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("index.html"), "{out}");
    assert_eq!(
        std::fs::read_to_string(s.0.join("secret.txt")).unwrap(),
        "TOP-SECRET-VALUE"
    );
}

#[test]
fn a_build_removes_only_the_gz_files_it_wrote() {
    let s = Scratch::new("gz");
    // Copied from `public/`: a `.gz` with no file beside it.
    s.write("public/archive.tar.gz", "an archive");
    s.write("public/data.json.gz", "precompressed by hand");
    // A file of the build's that gzip shrinks: it gets a `.gz` of its own.
    s.write("public/big.txt", &"compress me ".repeat(400));
    let (code, out) = s.wf(&["build"]);
    assert_eq!(code, 0, "{out}");
    let build = s.proj().join("build");
    assert!(build.join("archive.tar.gz").exists());
    assert!(build.join("data.json.gz").exists());
    assert!(build.join("big.txt.gz").exists());
    // Somebody else's in the output, beside nothing.
    std::fs::write(build.join("foreign.css.gz"), "theirs").unwrap();
    // The source of a `.gz` the build wrote goes: that `.gz` is stale.
    std::fs::remove_file(s.proj().join("public/big.txt")).unwrap();
    std::fs::remove_file(build.join("big.txt")).unwrap();
    let (code, out) = s.wf(&["build"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        !build.join("big.txt.gz").exists(),
        "a stale .gz of its own goes"
    );
    assert!(
        build.join("foreign.css.gz").exists(),
        "one it did not write stays"
    );
    assert!(build.join("archive.tar.gz").exists());
    assert!(build.join("data.json.gz").exists());
}

// ─── Links in the project ────────────────────────────────────────────

#[test]
fn a_link_out_of_the_project_is_not_copied_and_is_named() {
    let s = Scratch::new("links-out");
    s.write("public/ok.txt", "fine");
    s.write("shared/logo.txt", "inside the project");
    symlink(s.0.join("secret.txt"), s.proj().join("public/leak.txt")).unwrap();
    symlink(s.0.join("outside"), s.proj().join("public/dirlink")).unwrap();
    symlink(s.0.join("nowhere"), s.proj().join("public/dangling.txt")).unwrap();
    // A link that stays inside the project is followed, as it always was.
    symlink(
        s.proj().join("shared/logo.txt"),
        s.proj().join("public/logo.txt"),
    )
    .unwrap();
    symlink(s.0.join("secret.txt"), s.proj().join("src/leak.js")).unwrap();
    symlink(s.0.join("secret.txt"), s.proj().join("src/leak.css")).unwrap();
    let (code, out) = s.wf(&["build"]);
    assert_eq!(code, 0, "{out}");
    let build = s.proj().join("build");
    assert!(build.join("ok.txt").is_file());
    assert_eq!(
        std::fs::read_to_string(build.join("logo.txt")).unwrap(),
        "inside the project"
    );
    assert!(!build.join("leak.txt").exists());
    assert!(!build.join("dirlink").exists());
    assert!(!everything_under(&build).contains("TOP-SECRET-VALUE"));
    assert!(
        out.contains("leak.txt") && out.contains("outside the project"),
        "{out}"
    );
    assert!(out.contains("dangling.txt"), "{out}");
}

#[test]
fn a_link_loop_ends_the_walk_rather_than_the_build() {
    let s = Scratch::new("links-loop");
    s.write("src/pages/About.wf", "page About(path: \"/about\", title: \"A\", description: \"D\") { Heading(\"About\").h1 }\n");
    symlink(".", s.proj().join("src/a")).unwrap();
    symlink("..", s.proj().join("src/pages/up")).unwrap();
    // Two that point at each other's directory.
    std::fs::create_dir_all(s.proj().join("src/x")).unwrap();
    std::fs::create_dir_all(s.proj().join("src/y")).unwrap();
    symlink("../y", s.proj().join("src/x/l")).unwrap();
    symlink("../x", s.proj().join("src/y/l")).unwrap();
    std::fs::create_dir_all(s.proj().join("public")).unwrap();
    symlink(".", s.proj().join("public/self")).unwrap();
    let started = Instant::now();
    let (code, out) = s.wf(&["build"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "the walk went round"
    );
    assert!(out.contains("loop"), "{out}");
    assert!(s.proj().join("build/about/index.html").is_file() || out.contains("Build complete"));
}

// ─── Paths the project names ─────────────────────────────────────────

#[test]
fn a_data_file_outside_the_project_is_refused() {
    let s = Scratch::new("data-out");
    let abs = s.0.join("secret.json");
    for file in [
        "../secret.json",
        abs.to_str().unwrap(),
        "src/../../secret.json",
    ] {
        s.write(
            "src/App.wf",
            &format!("data leaked = \"{file}\"\n{APP}page L(path: \"/l\", title: \"L\", description: \"D\") {{ Text(leaked.token) }}\n"),
        );
        let (code, out) = s.wf(&["build"]);
        assert_ne!(code, 0, "{file}: {out}");
        assert!(
            out.contains("E111") && out.contains("outside the project"),
            "{file}: {out}"
        );
        assert!(!everything_under(&s.proj().join("build")).contains("TOP-SECRET-VALUE"));
    }
}

#[test]
fn a_data_file_reached_through_a_link_out_is_not_read() {
    let s = Scratch::new("data-link");
    symlink(s.0.join("secret.json"), s.proj().join("src/posts.json")).unwrap();
    s.write(
        "src/App.wf",
        &format!("data leaked = \"posts.json\"\n{APP}page L(path: \"/l\", title: \"L\", description: \"D\") {{ Text(leaked.token) }}\n"),
    );
    let (code, out) = s.wf(&["build"]);
    assert_ne!(code, 0, "{out}");
    assert!(!everything_under(&s.proj().join("build")).contains("TOP-SECRET-VALUE"));
}

#[test]
fn an_image_or_a_spec_outside_the_project_is_refused() {
    let s = Scratch::new("image-out");
    s.write("src/App.wf", &format!("image hero = \"../pic.png\"\n{APP}"));
    let (code, out) = s.wf(&["build"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("outside the project"), "{out}");
    s.write(
        "src/App.wf",
        &format!("api B from \"../../spec.json\" (base: \"/api\")\n{APP}"),
    );
    let (code, out) = s.wf(&["build"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("outside the project"), "{out}");
}

#[test]
fn config_paths_outside_the_project_are_refused() {
    for config in [
        r#"{ "name": "p", "i18n": { "default_locale": "en", "locales": ["en"], "dir": "../outside" } }"#,
        r#"{ "name": "p", "i18n": { "default_locale": "en", "locales": ["en"], "dir": "/etc" } }"#,
        r#"{ "name": "p", "build": { "pdf": { "fonts": ["/usr/share/fonts"] } } }"#,
        r#"{ "name": "p", "build": { "slides": { "fonts": ["../fonts"] } } }"#,
    ] {
        let s = Scratch::new("config-paths");
        s.config(config);
        let (code, out) = s.wf(&["build"]);
        assert_ne!(code, 0, "{config}: {out}");
        assert!(out.contains("outside the project"), "{config}: {out}");
    }
}

// ─── paths: ──────────────────────────────────────────────────────────

#[test]
fn a_paths_value_that_is_not_one_part_of_an_address_is_refused() {
    for value in ["../../../escaped", "..", ".", "", "a/b", "a\\\\b", "/abs"] {
        let s = Scratch::new("paths");
        s.config(r#"{ "name": "p", "build": { "ssg": true } }"#);
        s.write("src/slugs.json", &format!("[\"fine\", \"{value}\"]"));
        s.write(
            "src/App.wf",
            &format!("data slugs = \"slugs.json\"\n{APP}page Post(path: \"/p/:slug\", title: \"P\", description: \"D\", slug: String, paths: slugs) {{ Text(slug) }}\n"),
        );
        let (code, out) = s.wf(&["build"]);
        assert_ne!(code, 0, "{value:?}: {out}");
        assert!(out.contains("paths:"), "{value:?}: {out}");
        assert!(!s.0.join("escaped").exists(), "{value:?}");
        assert!(!s.proj().join("escaped").exists(), "{value:?}");
    }
}

#[test]
fn a_page_route_that_climbs_out_of_the_output_is_refused() {
    let s = Scratch::new("route-out");
    s.config(r#"{ "name": "p", "build": { "ssg": true } }"#);
    s.write(
        "src/App.wf",
        &format!("{APP}page Out(path: \"/../../escaped\", title: \"P\", description: \"D\") {{ Text(\"x\") }}\n"),
    );
    let (code, out) = s.wf(&["build"]);
    assert_ne!(code, 0, "{out}");
    assert!(!s.0.join("escaped").exists());
}

#[test]
fn paths_has_a_ceiling() {
    let s = Scratch::new("paths-many");
    s.config(r#"{ "name": "p", "build": { "ssg": true } }"#);
    let many: Vec<String> = (0..100_001).map(|i| format!("\"p{i}\"")).collect();
    s.write("src/slugs.json", &format!("[{}]", many.join(",")));
    s.write(
        "src/App.wf",
        &format!("data slugs = \"slugs.json\"\n{APP}page Post(path: \"/p/:slug\", title: \"P\", description: \"D\", slug: String, paths: slugs) {{ Text(slug) }}\n"),
    );
    let (code, out) = s.wf(&["build"]);
    assert_ne!(code, 0, "{out}");
    assert!(out.contains("100000"), "{out}");
    assert!(!s.proj().join("build/p/p0").exists());
}

// ─── Tools that write the sources ────────────────────────────────────

#[test]
fn fmt_fix_and_migrate_do_not_write_through_a_link_out_of_the_project() {
    let s = Scratch::new("fmt-link");
    let target = s.0.join("outside/target.wf");
    let original = "page   T(path: \"/t\", title: \"T\", description: \"D\") {\nText(\"x\")\n}\n";
    std::fs::write(&target, original).unwrap();
    symlink(&target, s.proj().join("src/evil.wf")).unwrap();
    for args in [
        &["fmt"][..],
        &["fix"],
        &["migrate"],
        &["fmt", "--to", "wfx"],
    ] {
        let _ = s.wf(args);
        assert_eq!(
            std::fs::read_to_string(&target).unwrap(),
            original,
            "`wf {}` wrote outside the project",
            args.join(" ")
        );
        assert!(target.exists());
        assert!(
            !s.0.join("outside/target.wfx").exists(),
            "`wf {}`",
            args.join(" ")
        );
    }
}
