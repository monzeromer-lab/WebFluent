//! Where a build writes: the disk, or — inside [`capture`] — memory,
//! which is how the compiler builds as WebAssembly in a browser.
//!
//! Where a build writes.
//!
//! What a project says is read through [a `Vfs`](crate::base::vfs::Vfs); what a
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
//! [`capture`]: crate::base::outfs::capture

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
    // What was capturing before comes back however `body` ends: a panic a
    // server or an editor catches must not leave this thread's writes
    // going to memory for good.
    let mut restore = Restore { outer: Some(outer) };
    let result = body();
    let memory = restore.finish().unwrap_or_default();
    (result, memory.files)
}

/// Puts back what was capturing before a [`capture_from`], when it ends or
/// when its body panics.
struct Restore {
    outer: Option<Option<Memory>>,
}

impl Restore {
    /// End the capture, returning what it kept.
    fn finish(&mut self) -> Option<Memory> {
        let outer = self.outer.take().flatten();
        CAPTURE.with(|c| c.replace(outer))
    }
}

impl Drop for Restore {
    fn drop(&mut self) {
        if self.outer.is_some() {
            self.finish();
        }
    }
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

/// `result`, its error naming `path`: "No such file or directory" alone
/// does not say which file. The kind is kept, so a caller that tells a
/// missing file from a refused one still can.
fn at<T>(path: &Path, result: io::Result<T>) -> io::Result<T> {
    result.map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
}

/// An I/O result whose error names the path it concerns, for code that
/// calls `std::fs` directly: `std::fs::write(&path, text).at(&path)?`.
pub(crate) trait At<T> {
    fn at(self, path: impl AsRef<Path>) -> io::Result<T>;
}

impl<T> At<T> for io::Result<T> {
    fn at(self, path: impl AsRef<Path>) -> io::Result<T> {
        at(path.as_ref(), self)
    }
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

/// Writes `contents` to the file at `path`, as [`std::fs::write`] does.
///
/// # Errors
///
/// The disk's [`io::Error`] when the file cannot be written, naming the
/// path. A capture never fails.
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
        || at(path, std::fs::write(path, contents)),
    )
}

/// Makes the directory at `path` and every one above it that is missing.
///
/// # Errors
///
/// The disk's [`io::Error`] when a directory cannot be made, naming the
/// path. A capture never fails.
pub fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    with(
        |m| {
            m.add_dirs(path);
            Ok(())
        },
        || at(path, std::fs::create_dir_all(path)),
    )
}

/// The bytes of the file at `path`.
///
/// # Errors
///
/// An [`io::Error`] naming the path when the file cannot be read; in a
/// capture, [`io::ErrorKind::NotFound`] when nothing was written there.
pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let path = path.as_ref();
    with(
        |m| m.files.get(path).cloned().ok_or_else(|| not_found(path)),
        || at(path, std::fs::read(path)),
    )
}

/// The text of the file at `path`.
///
/// # Errors
///
/// What [`read`] returns, and [`io::ErrorKind::InvalidData`] when the
/// file is not UTF-8.
pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    let path = path.as_ref();
    let bytes = read(path)?;
    String::from_utf8(bytes).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: not UTF-8", path.display()),
        )
    })
}

/// Removes the file at `path`.
///
/// # Errors
///
/// An [`io::Error`] naming the path when the file cannot be removed; in a
/// capture, [`io::ErrorKind::NotFound`] when nothing was written there.
pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    with(
        |m| {
            m.files
                .remove(path)
                .map(|_| ())
                .ok_or_else(|| not_found(path))
        },
        || at(path, std::fs::remove_file(path)),
    )
}

/// Copies the file at `from` to `to`.
///
/// # Errors
///
/// An [`io::Error`] naming `from` when the copy fails; in a capture, what
/// [`read`] returns for `from`.
pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let (from, to) = (from.as_ref(), to.as_ref());
    if capturing() {
        write(to, read(from)?)
    } else {
        at(from, std::fs::copy(from, to).map(|_| ()))
    }
}

/// Moves the file at `from` to `to`.
///
/// # Errors
///
/// An [`io::Error`] naming `from` when the move fails; in a capture, what
/// [`read`] and [`remove_file`] return for `from`.
pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    let (from, to) = (from.as_ref(), to.as_ref());
    if capturing() {
        let bytes = read(from)?;
        remove_file(from)?;
        write(to, bytes)
    } else {
        at(from, std::fs::rename(from, to))
    }
}

/// The length of the file at `path`.
///
/// # Errors
///
/// An [`io::Error`] naming the path when the file cannot be read; in a
/// capture, [`io::ErrorKind::NotFound`] when nothing was written there.
pub fn len(path: impl AsRef<Path>) -> io::Result<u64> {
    let path = path.as_ref();
    with(
        |m| {
            m.files
                .get(path)
                .map(|f| f.len() as u64)
                .ok_or_else(|| not_found(path))
        },
        || at(path, std::fs::metadata(path).map(|m| m.len())),
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
///
/// # Errors
///
/// An [`io::Error`] when the directory or one of its entries cannot be
/// read; in a capture, [`io::ErrorKind::NotFound`] when `path` is no
/// directory written there.
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
            let mut out = at(path, std::fs::read_dir(path))?
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
    fn a_capture_whose_body_panics_leaves_the_thread_writing_to_the_disk() {
        let panicked = std::panic::catch_unwind(|| {
            capture(|| {
                assert!(capturing());
                panic!("the build failed");
            })
        });
        assert!(panicked.is_err());
        assert!(!capturing(), "the capture outlived its panic");
    }

    #[test]
    fn an_error_on_the_disk_names_the_file_and_keeps_its_kind() {
        let missing = Path::new("/nowhere/wf-outfs-missing/page.wf");
        let e = read_to_string(missing).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::NotFound);
        assert!(
            e.to_string().contains("/nowhere/wf-outfs-missing/page.wf"),
            "{e}"
        );
        let e = std::fs::read(missing).at(missing).unwrap_err();
        assert!(
            e.to_string()
                .starts_with("/nowhere/wf-outfs-missing/page.wf: "),
            "{e}"
        );
    }

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
