//! A project built from memory.
//!
//! [`compile`] takes a project as a map of path → contents — `src/App.wf`,
//! `src/pages/Home.wf`, `public/logo.svg` — and builds it as `wf build`
//! does, with nothing read from a disk and nothing written to one. What it
//! wrote comes back as a map, and every finding with it, from the same
//! pipeline `wf check` runs. It is what the browser's playground calls,
//! through the `wf-wasm` crate; it runs natively too, which is how it is
//! tested.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;

use crate::build::{BuildOptions, BuildReport};
use crate::error::Diagnostic;
use crate::vfs::{Metadata, OverlayVfs, Vfs};

/// Where the project seems to sit. Nothing is read from or written there.
pub const PROJECT_DIR: &str = "/wf-project";

/// The config a project with none is built with.
pub const DEFAULT_CONFIG: &str = r#"{ "name": "Playground" }"#;

/// A file system with nothing in it, for buffers to lie over.
#[derive(Clone, Copy, Debug, Default)]
pub struct EmptyVfs;

impl Vfs for EmptyVfs {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not in the project", path.display()),
        ))
    }

    fn is_file(&self, _: &Path) -> bool {
        false
    }

    fn is_dir(&self, _: &Path) -> bool {
        false
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not in the project", path.display()),
        ))
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        self.read(path).map(|_| unreachable!())
    }
}

/// What to build, and how.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Report findings only, as `wf check` does: nothing is written.
    pub check_only: bool,
    /// A development build, as `wf serve` makes.
    pub dev: bool,
}

/// What a build in memory made and said.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Output {
    /// Whether the build finished. A build with an error in its findings,
    /// or one that could not run, did not.
    pub ok: bool,
    /// The site, by its path in the output (`index.html`, `app.js`,
    /// `pages/Home.js`). Text files as text; a file that is not UTF-8 is
    /// left out of this map and listed in [`Output::binary`].
    pub files: BTreeMap<String, String>,
    /// Files the build wrote that are not text (a processed image, a PDF),
    /// with their bytes.
    #[serde(skip)]
    pub binary: BTreeMap<String, Vec<u8>>,
    /// Every finding, in the order `wf check` gives them.
    pub diagnostics: Vec<Diagnostic>,
    /// The error a build that stopped ends with, as `wf build` prints it.
    pub error: Option<String>,
    /// What `wf build` printed, line by line.
    pub lines: Vec<String>,
}

/// Build `files` — paths relative to the project, as written under it — with
/// `config` as its `webfluent.app.json` (or the one in `files`, or
/// [`DEFAULT_CONFIG`]).
pub fn compile(files: &BTreeMap<String, String>, config: Option<&str>, options: &Options) -> Output {
    let root = PathBuf::from(PROJECT_DIR);
    let mut vfs = OverlayVfs::new(Arc::new(EmptyVfs));
    for (path, text) in files {
        let rel = path.trim_start_matches(['/', '.']).trim_start_matches('/');
        vfs.set(root.join(rel), text.as_str());
    }
    let config_path = root.join("webfluent.app.json");
    if let Some(config) = config {
        vfs.set(&config_path, config);
    } else if vfs.buffer(&config_path).is_none() {
        vfs.set(&config_path, DEFAULT_CONFIG);
    }

    let mut build_options = BuildOptions::new(&root, &vfs);
    build_options.dev = options.dev;
    let (outcome, written) = crate::outfs::capture(|| {
        if options.check_only {
            crate::build::check(&build_options)
        } else {
            crate::build::build(&build_options)
        }
    });

    let (report, error): (BuildReport, Option<String>) = match outcome {
        Ok(report) => (report, None),
        Err(failure) => {
            let message = match &failure.error {
                crate::error::WebFluentError::Diagnostics(_) => None,
                other => Some(other.to_string()),
            };
            (*failure.report, message.or_else(|| Some(failure.error.to_string())))
        }
    };
    let output_dir = if report.output_dir.as_os_str().is_empty() {
        root.join("build")
    } else {
        report.output_dir.clone()
    };

    let mut out = Output {
        ok: error.is_none(),
        error,
        diagnostics: report.diagnostics,
        lines: report.lines.into_iter().map(|l| l.text).collect(),
        ..Output::default()
    };
    for (path, bytes) in written {
        // The output, not the caches the build keeps beside the project.
        let Ok(rel) = path.strip_prefix(&output_dir) else {
            continue;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        // A gzipped copy is for a host that serves one; a page has no use
        // for it.
        if rel.ends_with(".gz") {
            continue;
        }
        match String::from_utf8(bytes) {
            Ok(text) => {
                out.files.insert(rel, text);
            }
            Err(e) => {
                out.binary.insert(rel, e.into_bytes());
            }
        }
    }
    out
}

/// What `wf explain CODE` says of a code: its title, severity, summary and
/// where the guide explains it; `None` for a code there is not.
pub fn explain(code: &str) -> Option<serde_json::Value> {
    let info = crate::diagnostics::codes::info(&code.to_uppercase())?;
    Some(serde_json::json!({
        "code": info.code,
        "severity": match info.severity {
            crate::diagnostics::Severity::Error => "error",
            crate::diagnostics::Severity::Warning => "warning",
            crate::diagnostics::Severity::Info => "note",
        },
        "title": info.title,
        "summary": info.summary,
        "docs": crate::diagnostics::codes::docs_url(info.code),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(files: &[(&str, &str)]) -> BTreeMap<String, String> {
        files
            .iter()
            .map(|(p, t)| (p.to_string(), t.to_string()))
            .collect()
    }

    #[test]
    fn a_project_of_several_files_builds_with_nothing_on_disk() {
        let files = project(&[
            ("src/App.wf", "app {\n    Router\n}\n"),
            (
                "src/pages/Home.wf",
                "page Home(path: \"/\", title: \"Home\", description: \"Count.\") {\n    state count = 0\n    Heading(\"Counter\").h1\n    Counter(start: count)\n}\n",
            ),
            (
                "src/components/Counter.wf",
                "component Counter(start: Number = 0) {\n    state n = start\n    Text(\"Count: {n}\")\n    Button(\"+1\").primary { on click { n = n + 1 } }\n}\n",
            ),
        ]);
        let out = compile(&files, None, &Options::default());
        assert!(out.ok, "{:?} {:#?}", out.error, out.diagnostics);
        assert!(out.files.contains_key("index.html"), "{:?}", out.files.keys());
        assert!(out.files.contains_key("app.js"));
        assert!(out.files.contains_key("styles.css"));
        let js: String = out.files.values().cloned().collect();
        assert!(js.contains("Count: "), "the component's text is compiled");
        assert!(!Path::new(PROJECT_DIR).exists(), "nothing reached the disk");
    }

    #[test]
    fn a_mistake_comes_back_as_the_finding_wf_check_gives() {
        let files = project(&[
            ("src/App.wf", "app {\n    Router\n}\n"),
            (
                "src/pages/Home.wf",
                "page Home(path: \"/\", title: \"Home\") {\n    Heading(\"Hi\").h1\n    Button(\"Go\").huge\n}\n",
            ),
        ]);
        let out = compile(&files, None, &Options::default());
        assert!(!out.ok);
        assert!(out.files.is_empty());
        let e103 = out
            .diagnostics
            .iter()
            .find(|d| d.code == "E103")
            .expect("E103 for an unknown flag");
        assert!(e103.file.ends_with("src/pages/Home.wf"), "{}", e103.file);

        let checked = compile(&files, None, &Options { check_only: true, dev: false });
        assert_eq!(
            checked.diagnostics.len(),
            out.diagnostics.len(),
            "a check finds what a build finds"
        );
    }

    #[test]
    fn explain_reads_the_registry() {
        let t05 = explain("t05").unwrap();
        assert_eq!(t05["code"], "T05");
        assert_eq!(t05["severity"], "error");
        assert!(explain("Z99").is_none());
    }
}
