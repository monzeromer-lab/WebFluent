//! A reader who chose a theme sees it from the first frame. The choice is
//! kept in storage and `app.js`, deferred, applied it after the first paint,
//! so a site whose reader picked Day over its Night showed one frame of
//! Night on every page. A build whose program calls `setTheme` now writes
//! `theme.js` and links it, blocking and first, in each page's head.

use std::path::Path;
use std::process::Command;

fn build(name: &str, body: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-theme-boot-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("webfluent.app.json"),
        r#"{ "name": "Site", "build": { "output": "build", "ssg": true, "csp": true } }"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/App.wf"),
        format!(
            "app {{ Router }}\npage Home(path: \"/\", title: \"Home\", description: \"D.\") {{\n    Heading(\"Home\").h1\n    {body}\n}}\npage About(path: \"/about\", title: \"About\", description: \"D.\") {{ Heading(\"About\").h1 }}\n"
        ),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(["build", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    dir
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join("build").join(file)).unwrap_or_else(|e| panic!("{file}: {e}"))
}

#[test]
fn a_program_that_picks_a_theme_applies_the_choice_before_the_first_paint() {
    let dir = build(
        "picks",
        "Button(\"Day\") { on click { setTheme(\"light\") } }",
    );
    let boot = read(&dir, "theme.js");
    assert!(
        boot.contains("wf:theme") && boot.contains("data-theme"),
        "{boot}"
    );
    for page in ["index.html", "about/index.html"] {
        let html = read(&dir, page);
        let at = html
            .find("theme.js\"></script>")
            .unwrap_or_else(|| panic!("{page}: {html}"));
        // Not deferred, and ahead of the title, the sheets and app.js.
        assert!(!html[..at + 30].contains("theme.js\" defer"), "{html}");
        assert!(at < html.find("<title>").unwrap(), "{html}");
        assert!(at < html.find("app.js").unwrap(), "{html}");
        // After the policy, which governs only what follows it.
        if let Some(csp) = html.find("Content-Security-Policy") {
            assert!(csp < at, "{html}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_program_with_no_theme_choice_carries_nothing_for_one() {
    let dir = build("none", "Text(\"Plain\")");
    assert!(!dir.join("build/theme.js").exists());
    assert!(!read(&dir, "index.html").contains("theme.js"));
    let _ = std::fs::remove_dir_all(&dir);
}
