//! `output_type: "static"` is a static site: every page pre-rendered to its
//! own HTML, as `ssg: true` builds it. It used to be accepted and then
//! built as a single-page app with nothing pre-rendered.

use std::path::Path;
use std::process::Command;

fn build(dir: &Path) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .arg("build")
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("wf runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A project of two pages in a folder of its own.
fn project(n: usize, config: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-static-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src/pages")).unwrap();
    std::fs::write(dir.join("webfluent.app.json"), config).unwrap();
    std::fs::write(
        dir.join("src/pages/Home.wf"),
        "page Home(path: \"/\", title: \"Home\", description: \"The home page.\") {\n    Text(\"Hello there\")\n}\n\npage About(path: \"/about\", title: \"About\", description: \"About us.\") {\n    Text(\"About us\")\n}\n",
    )
    .unwrap();
    dir
}

#[test]
fn a_static_output_type_pre_renders_every_page_as_ssg_does() {
    for (n, config) in [
        r#"{ "name": "st", "build": { "output_type": "static" } }"#,
        r#"{ "name": "st", "build": { "ssg": true } }"#,
    ]
    .into_iter()
    .enumerate()
    {
        let dir = project(n, config);
        build(&dir);
        let home = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
        assert!(home.contains("Hello there"), "{config}: {home}");
        let about = std::fs::read_to_string(dir.join("build/about/index.html")).unwrap();
        assert!(about.contains("About us"), "{config}: {about}");
        let _ = std::fs::remove_dir_all(&dir);
    }
    let spa = project(2, r#"{ "name": "st" }"#);
    build(&spa);
    assert!(
        !spa.join("build/about/index.html").exists(),
        "a single-page app has one shell"
    );
    let _ = std::fs::remove_dir_all(&spa);
}
