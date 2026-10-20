//! `BuildCache`: a build that reuses what an earlier one rendered writes
//! exactly what a build from nothing writes — after an edit to any file of
//! a project, and after it is undone.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use webfluent::build::{BuildCache, BuildOptions, build};
use webfluent::vfs::{FsVfs, OverlayVfs, Vfs};

/// A copy of a tutorial, for one test (`label`) to build and change.
fn tutorial(name: &str, label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-build-cache-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tutorials")
            .join(name),
        &dir,
    );
    dir
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

/// Every file a build wrote, by its path in the output.
fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                out.insert(rel, std::fs::read(&path).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// A build of `project` through `vfs` into `out`, with `cache` or without:
/// what it wrote, or that it failed.
fn built(
    project: &Path,
    vfs: &dyn Vfs,
    out: &Path,
    cache: Option<&BuildCache>,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let _ = std::fs::remove_dir_all(out);
    let options = BuildOptions {
        output_dir: Some(out.to_path_buf()),
        cache,
        ..BuildOptions::new(project, vfs)
    };
    match build(&options) {
        Ok(_) => Ok(tree(out)),
        Err(failure) => Err(failure.error.to_string()),
    }
}

/// The byte ranges of the string literals in `text`, in order.
fn literals(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'"' => {
                let start = i + 1;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                out.push((start, i));
            }
            _ => {}
        }
        i += 1;
    }
    out
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path
            .extension()
            .is_some_and(|e| e == "wf" || e == "wfx" || e == "md")
        {
            out.push(path);
        }
    }
}

#[test]
fn an_edit_anywhere_builds_with_the_cache_what_it_builds_without() {
    edits_agree(tutorial("medium-field-notes", "edits"));
}

/// The same over any project — Halyard, `site/` — too slow for every run:
/// `WF_BUILD_CACHE_PROJECT=<dir> cargo test --release --test build_cache -- --ignored`.
#[test]
#[ignore]
fn an_edit_anywhere_in_a_named_project_builds_the_same() {
    let from = PathBuf::from(std::env::var("WF_BUILD_CACHE_PROJECT").expect("a project to try"));
    let dir = std::env::temp_dir().join(format!("wf-build-cache-named-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    copy(&from, &dir);
    edits_agree(dir);
}

fn edits_agree(project: PathBuf) {
    let cached_out = project.with_extension("cached");
    let full_out = project.with_extension("full");
    let cache = BuildCache::in_memory();

    let original = built(&project, &FsVfs, &cached_out, Some(&cache)).expect("the tutorial builds");
    assert_eq!(built(&project, &FsVfs, &full_out, None).unwrap(), original);

    let mut files = Vec::new();
    sources(&project.join("src"), &mut files);
    files.sort();
    let mut edits = 0;
    let mut reused = 0;
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let spans = literals(&text);
        // The first, middle and last literal of each file: a title, a
        // label, an argument; whatever the file holds.
        let mut picks: Vec<usize> = vec![0, spans.len() / 2, spans.len().saturating_sub(1)];
        picks.dedup();
        for pick in picks {
            let Some(&(start, end)) = spans.get(pick) else {
                continue;
            };
            let mut edited = text.clone();
            edited.replace_range(start..end, &format!("{}·", &text[start..end]));
            let mut vfs = OverlayVfs::over_disk();
            vfs.set(file, edited);
            let with = built(&project, &vfs, &cached_out, Some(&cache));
            let without = built(&project, &vfs, &full_out, None);
            assert_eq!(
                with.is_ok(),
                without.is_ok(),
                "{}, literal {pick}: {with:?} / {without:?}",
                file.display()
            );
            if let (Ok(with), Ok(without)) = (&with, &without) {
                for (path, bytes) in without {
                    assert_eq!(
                        with.get(path),
                        Some(bytes),
                        "{} differs after an edit to {}, literal {pick}",
                        path,
                        file.display()
                    );
                }
                assert_eq!(with.len(), without.len(), "{}", file.display());
            }
            edits += 1;
        }
        // Undone, the cache gives back the site as it was.
        assert_eq!(
            built(&project, &FsVfs, &cached_out, Some(&cache)).unwrap(),
            original,
            "after undoing an edit to {}",
            file.display()
        );
        reused += 1;
    }
    assert!(
        edits >= files.len(),
        "{edits} edits over {} files",
        files.len()
    );
    assert!(reused > 0);

    // The config: every page reads it.
    let config = project.join("webfluent.app.json");
    let text = std::fs::read_to_string(&config).unwrap();
    let edited = text.replacen("\"name\": \"", "\"name\": \"Edited ", 1);
    assert_ne!(edited, text, "the config names the project");
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(&config, edited);
    assert_eq!(
        built(&project, &vfs, &cached_out, Some(&cache)).unwrap(),
        built(&project, &vfs, &full_out, None).unwrap()
    );

    for dir in [&project, &cached_out, &full_out] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn an_unchanged_build_reuses_every_page_and_an_edited_page_is_rendered_alone() {
    let project = tutorial("medium-field-notes", "count");
    let out = project.with_extension("out");
    let cache = BuildCache::in_memory();
    let options = |vfs| BuildOptions {
        output_dir: Some(out.clone()),
        cache: Some(&cache),
        ..BuildOptions::new(&project, vfs)
    };

    let first = build(&options(&FsVfs)).expect("builds");
    assert_eq!(first.pages_reused, 0);
    assert!(first.pages_rendered > 1, "{}", first.pages_rendered);
    let pages = first.pages_rendered;

    let again = build(&options(&FsVfs)).expect("builds");
    assert_eq!((again.pages_reused, again.pages_rendered), (pages, 0));

    // A page's own text, changed — in its body, not its header, which
    // every page reads (breadcrumbs name their ancestors): that page is
    // rendered, and only it.
    let mut files = Vec::new();
    sources(&project.join("src/pages"), &mut files);
    files.sort();
    let page = &files[0];
    let text = std::fs::read_to_string(page).unwrap();
    let (start, end) = *literals(&text).last().unwrap();
    let mut edited = text.clone();
    edited.replace_range(start..end, &format!("{}·", &text[start..end]));
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(page, edited);
    let edited = build(&options(&vfs)).expect("builds");
    assert_eq!(
        (edited.pages_reused, edited.pages_rendered),
        (pages - 1, 1),
        "an edit to {}",
        page.display()
    );

    for dir in [&project, &out] {
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn a_cache_on_disk_outlives_the_value() {
    let project = tutorial("medium-field-notes", "disk");
    let out = project.with_extension("out");
    let dir = project.join(".wf-cache/build");
    let run = || {
        let cache = BuildCache::on_disk(&dir);
        let report = build(&BuildOptions {
            output_dir: Some(out.clone()),
            cache: Some(&cache),
            ..BuildOptions::new(&project, &FsVfs)
        })
        .expect("builds");
        (report.pages_reused, report.pages_rendered, tree(&out))
    };
    let (reused, rendered, first) = run();
    assert_eq!(reused, 0);
    // A new cache over the same directory: a restart.
    let (reused_again, rendered_again, second) = run();
    assert_eq!((reused_again, rendered_again), (rendered, 0));
    assert_eq!(first, second);
    for dir in [&project, &out] {
        let _ = std::fs::remove_dir_all(dir);
    }
}
