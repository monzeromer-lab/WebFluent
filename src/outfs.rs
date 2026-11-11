//! Where a build writes.
//!
//! What a project says is read through [a `Vfs`](crate::vfs::Vfs); what a
//! build *writes* — the site, the image cache, `.wf-sizes.json` — goes
//! through these functions, which have the meaning of their `std::fs`
//! namesakes. Ordinarily they *are* those: `wf build` writes to the disk.
//!
//! Inside [`capture`] they write to memory instead, and the files come back
//! as a map. That is how the compiler builds where there is no disk — in a
//! browser, as WebAssembly — and how a test builds a project without one.
//! The capture belongs to the thread that opened it; the build writes from
//! that thread only.
//!
//! [`capture`]: crate::outfs::capture

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

/// The files a captured build wrote, by path.
pub type Files = BTreeMap<PathBuf, Vec<u8>>;

#[derive(Default)]
struct Memory {
    files: Files,
    dirs: BTreeSet<PathBuf>,
}

thread_local! {
    static CAPTURE: RefCell<Option<Memory>> = const { RefCell::new(None) };
}

/// Run `body` with every write kept in memory; what it wrote comes back
/// with its result. Captures do not nest: an inner one takes over until it
/// ends, then the outer one starts again from nothing.
pub fn capture<T>(body: impl FnOnce() -> T) -> (T, Files) {
    capture_from(Files::new(), body)
}

/// [`capture`], starting from `files` as though they had been written
/// already: what an earlier build left that this one may reuse (the image
/// cache).
pub fn capture_from<T>(files: Files, body: impl FnOnce() -> T) -> (T, Files) {
    let mut memory = Memory::default();
    for (path, bytes) in files {
        if let Some(parent) = path.parent() {
            memory.add_dirs(parent);
        }
        memory.files.insert(path, bytes);
    }
    let outer = CAPTURE.with(|c| c.replace(Some(memory)));
    let result = body();
    let memory = CAPTURE.with(|c| c.replace(outer)).unwrap_or_default();
    (result, memory.files)
}

/// Whether writes are going to memory.
pub fn capturing() -> bool {
    CAPTURE.with(|c| c.borrow().is_some())
}

fn with<T>(memory: impl FnOnce(&mut Memory) -> T, disk: impl FnOnce() -> T) -> T {
    CAPTURE
        .with(|c| c.borrow_mut().as_mut().map(memory))
        .unwrap_or_else(disk)
}

fn not_found(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("{} was not written", path.display()),
    )
}

impl Memory {
    fn add_dirs(&mut self, path: &Path) {
        for dir in path.ancestors() {
            if dir.as_os_str().is_empty() || !self.dirs.insert(dir.to_path_buf()) {
                break;
            }
        }
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains(path)
    }
}

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> io::Result<()> {
    let path = path.as_ref();
    let contents = contents.as_ref();
    with(
        |m| {
            if let Some(parent) = path.parent() {
                m.add_dirs(parent);
            }
            m.files.insert(path.to_path_buf(), contents.to_vec());
            Ok(())
        },
        || std::fs::write(path, contents),
    )
}

pub fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    with(
        |m| {
            m.add_dirs(path);
            Ok(())
        },
        || std::fs::create_dir_all(path),
    )
}

pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let path = path.as_ref();
    with(
        |m| m.files.get(path).cloned().ok_or_else(|| not_found(path)),
        || std::fs::read(path),
    )
}

pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    let bytes = read(path)?;
    String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not UTF-8"))
}

pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    with(
        |m| {
            m.files
                .remove(path)
                .map(|_| ())
                .ok_or_else(|| not_found(path))
        },
        || std::fs::remove_file(path),
    )
}

pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let (from, to) = (from.as_ref(), to.as_ref());
    if capturing() {
        write(to, read(from)?)
    } else {
        std::fs::copy(from, to).map(|_| ())
    }
}

pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let (from, to) = (from.as_ref(), to.as_ref());
    if capturing() {
        let bytes = read(from)?;
        remove_file(from)?;
        write(to, bytes)
    } else {
        std::fs::rename(from, to)
    }
}

/// The length of the file at `path`.
pub fn len(path: impl AsRef<Path>) -> io::Result<u64> {
    let path = path.as_ref();
    with(
        |m| {
            m.files
                .get(path)
                .map(|f| f.len() as u64)
                .ok_or_else(|| not_found(path))
        },
        || std::fs::metadata(path).map(|m| m.len()),
    )
}

pub fn exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    with(
        |m| m.files.contains_key(path) || m.is_dir(path),
        || path.exists(),
    )
}

pub fn is_file(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    with(|m| m.files.contains_key(path), || path.is_file())
}

pub fn is_dir(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    with(|m| m.is_dir(path), || path.is_dir())
}

/// The entries of the directory at `path`, each as `path` joined with its
/// name, sorted.
pub fn read_dir(path: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let path = path.as_ref();
    with(
        |m| {
            if !m.is_dir(path) {
                return Err(not_found(path));
            }
            let mut out: BTreeSet<PathBuf> = BTreeSet::new();
            let below = m
                .files
                .keys()
                .chain(m.dirs.iter())
                .filter_map(|p| p.strip_prefix(path).ok())
                .filter_map(|rest| rest.components().next());
            for first in below {
                out.insert(path.join(first));
            }
            Ok(out.into_iter().collect())
        },
        || {
            let mut out = std::fs::read_dir(path)?
                .map(|e| e.map(|e| e.path()))
                .collect::<io::Result<Vec<_>>>()?;
            out.sort();
            Ok(out)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_capture_keeps_what_was_written_off_the_disk() {
        let root = Path::new("/nowhere/wf-outfs");
        let ((), files) = capture(|| {
            create_dir_all(root.join("pages")).unwrap();
            write(root.join("index.html"), "<p>").unwrap();
            write(root.join("pages/A.js"), "a").unwrap();
            assert!(is_dir(root.join("pages")));
            assert_eq!(
                read_dir(root).unwrap(),
                [root.join("index.html"), root.join("pages")]
            );
            copy(root.join("pages/A.js"), root.join("pages/B.js")).unwrap();
            remove_file(root.join("pages/A.js")).unwrap();
            assert_eq!(len(root.join("pages/B.js")).unwrap(), 1);
        });
        assert!(!root.exists());
        assert_eq!(files.len(), 2);
        assert_eq!(files[&root.join("index.html")], b"<p>");
        assert!(!capturing());
    }
}
