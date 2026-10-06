//! The launch videos' projects under `examples/videos`. Video 5 shows the
//! compiler's answer to one typo; this holds that answer — the code, the
//! place, the message and the hint — to what is on screen, and holds the
//! project to having that one mistake and no other.
//! (`scripts/video-examples.sh` builds the rest.)

use std::path::{Path, PathBuf};
use std::process::Command;

fn typo_project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/videos/typo")
}

/// A copy of a project without what a build wrote, so the test never
/// touches the repository.
fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        let name = e.file_name();
        if name == "build" || name == ".wf-cache" || name == ".wf-sizes.json" {
            continue;
        }
        if p.is_dir() {
            copy(&p, &to.join(&name));
        } else {
            std::fs::copy(&p, to.join(&name)).unwrap();
        }
    }
}

/// `wf <args>` in `dir`: its exit code, standard output and standard error.
fn wf(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .env_remove("NO_COLOR")
        .output()
        .expect("wf runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn the_typo_video_shows_t05_with_a_did_you_mean() {
    let (code, out, err) = wf(&typo_project(), &["check"]);
    let said = format!("{out}{err}");
    assert_eq!(code, 1, "a check with an error exits 1:\n{said}");
    assert!(
        said.contains(
            "error[T05]: `User` has no field `nmae`\n \
             --> src/Profile.wf:7:22\n  |\n\
             7 |         Heading(user.nmae).h1\n  |                      ^^^^\n  \
             = help: Did you mean `name`? Its fields are `name`, `role`, `email`\n"
        ),
        "{said}"
    );
    assert!(
        said.ends_with("error: the build stopped: 1 error\n"),
        "{said}"
    );
    assert_eq!(said.matches("error[").count(), 1, "one mistake: {said}");
    assert!(!said.contains("warning["), "{said}");
}

#[test]
fn the_typo_has_one_fix_and_the_fix_checks_clean() {
    // The editor's quick fix, as `--format json` carries it.
    let (_, out, _) = wf(&typo_project(), &["check", "--format", "json"]);
    let report: serde_json::Value = serde_json::from_str(&out).expect("JSON");
    let found = report["diagnostics"].as_array().unwrap();
    assert_eq!(found.len(), 1, "{out}");
    assert_eq!(found[0]["code"], "T05");
    assert_eq!(found[0]["message"], "`User` has no field `nmae`");
    assert_eq!(
        found[0]["hint"],
        "Did you mean `name`? Its fields are `name`, `role`, `email`"
    );
    assert_eq!(found[0]["fixes"][0]["title"], "Change to `name`");
    assert_eq!(found[0]["fixes"][0]["edits"][0]["text"], ".name");

    // Spelled right, the project has nothing else to report.
    let tmp = std::env::temp_dir().join(format!("wf-video-typo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    copy(&typo_project(), &tmp);
    let page = tmp.join("src/Profile.wf");
    let src = std::fs::read_to_string(&page).unwrap();
    assert_eq!(src.matches("user.nmae").count(), 1, "{src}");
    std::fs::write(&page, src.replace("user.nmae", "user.name")).unwrap();
    let (code, out, err) = wf(&tmp, &["check", "--deny-warnings"]);
    assert_eq!(code, 0, "{out}{err}");
    let _ = std::fs::remove_dir_all(tmp);
}
