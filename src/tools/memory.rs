//! A project built from memory, with nothing read from or written to a disk:
//! what the browser's playground runs.
//!
//! A project built from memory.
//!
//! [`compile`] takes a project as a map of path → contents — `src/App.wf`,
//! `src/pages/Home.wf`, `public/logo.svg` — and builds it as `wf build`
//! does, with nothing read from a disk and nothing written to one. What it
//! wrote comes back as a map, and every finding with it, from the same
//! pipeline `wf check` runs. It is what the browser's playground calls,
//! through the `wf-wasm` crate; it runs natively too, which is how it is
//! tested.
//!
//! [`compile`]: crate::tools::memory::compile

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;

use crate::base::vfs::{Metadata, OverlayVfs, Vfs};
use crate::build::{BuildOptions, BuildReport};
use crate::diagnostics::error::Diagnostic;

/// Where the project seems to sit. Nothing is read from or written there.
pub const PROJECT_DIR: &str = "/wf-project";

thread_local! {
    /// What the image pipeline encoded, kept from one build to the next.
    static MEDIA_CACHE: std::cell::RefCell<crate::base::outfs::Files> =
        const { std::cell::RefCell::new(crate::base::outfs::Files::new()) };
}

/// Past this many bytes of encoded pictures the cache starts again empty.
const MEDIA_CACHE_LIMIT: usize = 64 << 20;

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
    /// with their bytes — base64 when serialised.
    #[serde(serialize_with = "base64_map")]
    pub binary: BTreeMap<String, Vec<u8>>,
    /// Every finding, in the order `wf check` gives them.
    pub diagnostics: Vec<Diagnostic>,
    /// The error a build that stopped ends with, as `wf build` prints it.
    pub error: Option<String>,
    /// What `wf build` printed, line by line.
    pub lines: Vec<String>,
}

fn base64_map<S: serde::Serializer>(
    map: &BTreeMap<String, Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    let mut out = serializer.serialize_map(Some(map.len()))?;
    for (path, bytes) in map {
        out.serialize_entry(path, &crate::project::media::probe::base64(bytes))?;
    }
    out.end()
}

/// What a file of the project holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Contents {
    Text(String),
    /// A file that is not text: a picture, a font.
    Bytes(Vec<u8>),
}

/// Build `files` — paths relative to the project, as written under it — with
/// `config` as its `webfluent.app.json` (or the one in `files`, or
/// [`DEFAULT_CONFIG`]).
pub fn compile(
    files: &BTreeMap<String, String>,
    config: Option<&str>,
    options: &Options,
) -> Output {
    compile_files(
        files
            .iter()
            .map(|(p, t)| (p.clone(), Contents::Text(t.clone()))),
        config,
        options,
    )
}

/// [`compile`], with files that need not be text: a picture under
/// `public/`, a font under `fonts/`.
pub fn compile_files(
    files: impl IntoIterator<Item = (String, Contents)>,
    config: Option<&str>,
    options: &Options,
) -> Output {
    let root = PathBuf::from(PROJECT_DIR);
    let mut vfs = OverlayVfs::new(Arc::new(EmptyVfs));
    for (path, contents) in files {
        let rel = path.trim_start_matches(['/', '.']).trim_start_matches('/');
        match contents {
            Contents::Text(text) => vfs.set(root.join(rel), text),
            Contents::Bytes(bytes) => vfs.set_bytes(root.join(rel), bytes),
        }
    }
    let config_path = root.join("webfluent.app.json");
    if let Some(config) = config {
        vfs.set(&config_path, config);
    } else if vfs.buffer(&config_path).is_none() {
        vfs.set(&config_path, DEFAULT_CONFIG);
    }

    let mut build_options = BuildOptions::new(&root, &vfs);
    build_options.dev = options.dev;
    // The pictures earlier builds encoded, so an edit that leaves a picture
    // alone does not encode it again.
    let media_cache = root.join(".wf-cache/media");
    let seed = MEDIA_CACHE.with(|c| c.take());
    let (outcome, written) = crate::base::outfs::capture_from(seed, || {
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
                crate::diagnostics::error::WebFluentError::Diagnostics(_) => None,
                other => Some(other.to_string()),
            };
            (
                *failure.report,
                message.or_else(|| Some(failure.error.to_string())),
            )
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
    let mut kept = crate::base::outfs::Files::new();
    let mut kept_bytes = 0;
    for (path, bytes) in &written {
        if path.starts_with(&media_cache) {
            kept_bytes += bytes.len();
            kept.insert(path.clone(), bytes.clone());
        }
    }
    if kept_bytes <= MEDIA_CACHE_LIMIT {
        MEDIA_CACHE.with(|c| c.replace(kept));
    }
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
        assert!(
            out.files.contains_key("index.html"),
            "{:?}",
            out.files.keys()
        );
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

        let checked = compile(
            &files,
            None,
            &Options {
                check_only: true,
                dev: false,
            },
        );
        assert_eq!(
            checked.diagnostics.len(),
            out.diagnostics.len(),
            "a check finds what a build finds"
        );
    }

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbImage::from_fn(width, height, |x, y| {
            image::Rgb([
                (x * 7 % 256) as u8,
                (y * 5 % 256) as u8,
                ((x ^ y) % 256) as u8,
            ])
        });
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(image)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn a_picture_is_sized_in_memory_and_its_encoding_kept_for_the_next_build() {
        let files = || {
            vec![
                (
                    "src/pages/Home.wf".to_string(),
                    Contents::Text(
                        "page Home(path: \"/\", title: \"Pic\", description: \"A picture.\") {\n    Heading(\"Pic\").h1\n    Image(src: \"/hero.png\", alt: \"Noise\")\n}\n"
                            .into(),
                    ),
                ),
                ("public/hero.png".to_string(), Contents::Bytes(png(1000, 500))),
            ]
        };
        let first = compile_files(files(), None, &Options::default());
        assert!(first.ok, "{:?} {:#?}", first.error, first.diagnostics);
        let sized: Vec<&String> = first
            .binary
            .keys()
            .filter(|p| p.starts_with("img/") && p.ends_with(".webp"))
            .collect();
        assert!(!sized.is_empty(), "{:?}", first.binary.keys());
        assert!(
            first.files.values().any(|f| f.contains("image/webp")),
            "the page offers the WebP"
        );
        // The cache now holds the encodings: the next build writes the same.
        assert!(MEDIA_CACHE.with(|c| !c.borrow().is_empty()));
        let second = compile_files(files(), None, &Options::default());
        assert_eq!(first.binary, second.binary);
    }

    #[cfg(wf_pdf)]
    #[test]
    fn a_document_is_rendered_in_memory() {
        let files = project(&[(
            "src/pages/Report.wf",
            "page Report(path: \"/\", title: \"Report\") {\n    Document(size: \"A4\") {\n        Heading(\"Q1\").h1\n        Text(\"Revenue grew.\")\n    }\n}\n",
        )]);
        let out = compile(
            &files,
            Some(r#"{ "name": "Report", "build": { "output_type": "pdf" } }"#),
            &Options::default(),
        );
        assert!(out.ok, "{:?} {:#?}", out.error, out.diagnostics);
        assert!(out.binary["Report.pdf"].starts_with(b"%PDF"));
    }

    #[test]
    fn explain_reads_the_registry() {
        let t05 = explain("t05").unwrap();
        assert_eq!(t05["code"], "T05");
        assert_eq!(t05["severity"], "error");
        assert!(explain("Z99").is_none());
    }
}
