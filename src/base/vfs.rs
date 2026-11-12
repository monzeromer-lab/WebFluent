//! Where a project is read from: the disk ([`FsVfs`]), or the disk with
//! an editor's unsaved buffers over it ([`OverlayVfs`]).
//!
//! Where a project is read from.
//!
//! Everything that reads a project — `wf build`, the language server, an
//! editor that builds what its buffers say before they are saved — reads it
//! through [a `Vfs`](crate::base::vfs::Vfs). [The `FsVfs`](crate::base::vfs::FsVfs) is
//! the disk, and what every command uses. [An
//! `OverlayVfs`](crate::base::vfs::OverlayVfs) is another file system with an
//! editor's open buffers laid over it: a buffer is read instead of the file
//! it names, a buffer for a file that is not on disk yet is listed in its
//! directory, and a file can be hidden as though it were deleted.
//!
//! What a build *writes* — the output, the image cache, `.wf-sizes.json` —
//! goes to the disk either way: an overlay changes what the project says,
//! not where its output lands.

use std::collections::BTreeMap;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

/// What a cache needs to know about a file to tell whether it changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub len: u64,
    /// When it last changed, if the file system keeps the time. A buffer has
    /// none — its text changes without one — so a cache keyed by this must
    /// not keep what it read when this is `None`.
    pub modified: Option<SystemTime>,
}

/// A file system a project is read from.
///
/// The methods are the few that reading a project takes, each with the
/// meaning of its `std::fs` or [`Path`] namesake.
pub trait Vfs: Send + Sync {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        String::from_utf8(self.read(path)?).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            )
        })
    }

    fn is_file(&self, path: &Path) -> bool;

    fn is_dir(&self, path: &Path) -> bool;

    fn exists(&self, path: &Path) -> bool {
        self.is_file(path) || self.is_dir(path)
    }

    /// The entries of the directory at `path`, each as `path` joined with
    /// its name, in no particular order: a reader that needs an order — and
    /// a build does, so it is the same wherever the project sits — sorts.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>>;

    fn metadata(&self, path: &Path) -> io::Result<Metadata>;

    /// Copy the file at `from` to `to` on the disk: how a build copies
    /// `public/` into its output.
    fn copy_out(&self, from: &Path, to: &Path) -> io::Result<()> {
        crate::base::outfs::write(to, self.read(from)?)
    }

    /// Whether `path` is itself a symbolic link. A file system with no
    /// links has none.
    fn is_link(&self, _path: &Path) -> bool {
        false
    }

    /// Where `path` really is — every link on the way resolved — when it is
    /// on a disk and there; `None` for a file system with no links, or a
    /// path that is not there.
    fn canonical(&self, _path: &Path) -> Option<PathBuf> {
        None
    }
}

/// The disk.
#[derive(Clone, Copy, Debug, Default)]
pub struct FsVfs;

/// `result`, its error naming `path`, its kind kept.
fn at<T>(path: &Path, result: io::Result<T>) -> io::Result<T> {
    result.map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))
}

impl Vfs for FsVfs {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        at(path, std::fs::read(path))
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        at(path, std::fs::read_to_string(path))
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        at(path, std::fs::read_dir(path))?
            .map(|entry| entry.map(|e| e.path()))
            .collect()
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let meta = at(path, std::fs::metadata(path))?;
        Ok(Metadata {
            len: meta.len(),
            modified: meta.modified().ok(),
        })
    }

    fn copy_out(&self, from: &Path, to: &Path) -> io::Result<()> {
        // `fs::copy` keeps the file's permissions, as `wf build` always has.
        crate::base::outfs::copy(from, to)
    }

    fn is_link(&self, path: &Path) -> bool {
        std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
    }

    fn canonical(&self, path: &Path) -> Option<PathBuf> {
        std::fs::canonicalize(path).ok()
    }
}

/// A file system with buffers laid over it.
///
/// Paths are matched as written, less any `.` component: lay a buffer over
/// the path the reader will ask for — `project_dir.join("src/App.wf")` for
/// a build of `project_dir`.
#[derive(Clone)]
pub struct OverlayVfs {
    base: Arc<dyn Vfs>,
    layers: BTreeMap<PathBuf, Layer>,
}

#[derive(Clone)]
enum Layer {
    Text(Arc<str>),
    /// A file that is not text: a picture, a font.
    Bytes(Arc<[u8]>),
    Deleted,
}

impl Layer {
    /// Whether it is a file, of text or not.
    fn is_file(&self) -> bool {
        !matches!(self, Layer::Deleted)
    }

    fn bytes(&self) -> Option<&[u8]> {
        match self {
            Layer::Text(text) => Some(text.as_bytes()),
            Layer::Bytes(bytes) => Some(bytes),
            Layer::Deleted => None,
        }
    }
}

impl OverlayVfs {
    pub fn new(base: Arc<dyn Vfs>) -> Self {
        Self {
            base,
            layers: BTreeMap::new(),
        }
    }

    /// Buffers over the disk.
    pub fn over_disk() -> Self {
        Self::new(Arc::new(FsVfs))
    }

    /// Lay `text` over `path`: it is read instead of the file, and listed
    /// in its directory whether the file exists or not.
    pub fn set(&mut self, path: impl AsRef<Path>, text: impl Into<Arc<str>>) {
        self.layers
            .insert(key(path.as_ref()), Layer::Text(text.into()));
    }

    /// Lay `bytes` over `path`, as [`set`](Self::set) lays text: for a
    /// file that is not text, a picture or a font.
    pub fn set_bytes(&mut self, path: impl AsRef<Path>, bytes: impl Into<Arc<[u8]>>) {
        self.layers
            .insert(key(path.as_ref()), Layer::Bytes(bytes.into()));
    }

    /// Hide `path` — and, for a directory, everything under it — as though
    /// it were deleted. A buffer set under it later shows.
    pub fn delete(&mut self, path: impl AsRef<Path>) {
        let gone = key(path.as_ref());
        self.layers.retain(|at, _| !at.starts_with(&gone));
        self.layers.insert(gone, Layer::Deleted);
    }

    /// Take away whatever lay over `path`, so the file shows through again.
    /// Whether anything did.
    pub fn clear(&mut self, path: impl AsRef<Path>) -> bool {
        self.layers.remove(&key(path.as_ref())).is_some()
    }

    /// The buffer over `path`, if there is one.
    pub fn buffer(&self, path: impl AsRef<Path>) -> Option<&Arc<str>> {
        match self.layers.get(&key(path.as_ref())) {
            Some(Layer::Text(text)) => Some(text),
            _ => None,
        }
    }

    /// Every buffer, by path.
    pub fn buffers(&self) -> impl Iterator<Item = (&Path, &Arc<str>)> {
        self.layers.iter().filter_map(|(path, layer)| match layer {
            Layer::Text(text) => Some((path.as_path(), text)),
            Layer::Bytes(_) | Layer::Deleted => None,
        })
    }

    /// Whether `key`, or a directory above it, was deleted.
    fn hidden(&self, key: &Path) -> bool {
        key.ancestors()
            .any(|at| matches!(self.layers.get(at), Some(Layer::Deleted)))
    }

    /// Whether a buffer lies under `key`, which makes it a directory.
    fn holds_buffer(&self, key: &Path) -> bool {
        self.layers
            .iter()
            .any(|(at, layer)| layer.is_file() && at != key && at.starts_with(key))
    }

    fn gone(path: &Path) -> io::Error {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} was deleted", path.display()),
        )
    }
}

/// `path` without its `.` components, so `./src/App.wf` and `src/App.wf`
/// name one buffer.
fn key(path: &Path) -> PathBuf {
    path.components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect()
}

impl Vfs for OverlayVfs {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let at = key(path);
        match self.layers.get(&at).and_then(Layer::bytes) {
            Some(bytes) => Ok(bytes.to_vec()),
            _ if self.hidden(&at) => Err(Self::gone(path)),
            _ => self.base.read(path),
        }
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        let at = key(path);
        match self.layers.get(&at) {
            Some(Layer::Text(text)) => Ok(text.to_string()),
            Some(Layer::Bytes(bytes)) => String::from_utf8(bytes.to_vec()).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "stream did not contain valid UTF-8",
                )
            }),
            _ if self.hidden(&at) => Err(Self::gone(path)),
            _ => self.base.read_to_string(path),
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        let at = key(path);
        match self.layers.get(&at) {
            Some(layer) if layer.is_file() => true,
            _ if self.hidden(&at) => false,
            _ => self.base.is_file(path),
        }
    }

    fn is_dir(&self, path: &Path) -> bool {
        let at = key(path);
        if self.layers.get(&at).is_some_and(Layer::is_file) {
            return false;
        }
        if self.holds_buffer(&at) {
            return true;
        }
        !self.hidden(&at) && self.base.is_dir(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let at = key(path);
        let implied = self.holds_buffer(&at);
        let mut entries = if self.hidden(&at) {
            if !implied {
                return Err(Self::gone(path));
            }
            Vec::new()
        } else {
            match self.base.read_dir(path) {
                Ok(entries) => entries,
                // A directory only buffers are in is not on disk yet.
                Err(_) if implied => Vec::new(),
                Err(e) => return Err(e),
            }
        };
        entries.retain(|entry| {
            let entry = key(entry);
            !self.hidden(&entry)
                || self.layers.get(&entry).is_some_and(Layer::is_file)
                || self.holds_buffer(&entry)
        });
        // Each buffer under `path` is listed by the name of its first
        // component below it: the file itself, or the directory it is in.
        for (buffered, layer) in &self.layers {
            if !layer.is_file() {
                continue;
            }
            let Some(Component::Normal(name)) = buffered
                .strip_prefix(&at)
                .ok()
                .and_then(|rest| rest.components().next())
            else {
                continue;
            };
            let entry = path.join(name);
            let entry_key = key(&entry);
            if !entries.iter().any(|e| key(e) == entry_key) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let at = key(path);
        match self.layers.get(&at) {
            Some(layer) if layer.is_file() => Ok(Metadata {
                len: layer.bytes().map_or(0, |b| b.len()) as u64,
                modified: None,
            }),
            _ if self.hidden(&at) => Err(Self::gone(path)),
            _ => self.base.metadata(path),
        }
    }

    fn copy_out(&self, from: &Path, to: &Path) -> io::Result<()> {
        let at = key(from);
        match self.layers.get(&at) {
            Some(layer) if layer.is_file() => {
                crate::base::outfs::write(to, layer.bytes().unwrap_or_default())
            }
            _ if self.hidden(&at) => Err(Self::gone(from)),
            _ => self.base.copy_out(from, to),
        }
    }

    fn is_link(&self, path: &Path) -> bool {
        let at = key(path);
        !self.layers.contains_key(&at) && !self.hidden(&at) && self.base.is_link(path)
    }

    fn canonical(&self, path: &Path) -> Option<PathBuf> {
        let at = key(path);
        if self.layers.contains_key(&at) || self.hidden(&at) {
            return None;
        }
        self.base.canonical(path)
    }
}

/// How many links one path may pass through below the project's root: a
/// walk that follows more is going round, or near enough.
const MAX_LINKS: usize = 8;

/// A project read through another file system, kept inside it.
///
/// What a build reads is the project's: a file, or a directory it walks,
/// that leads out of the project through a symbolic link (`public/x`
/// pointing at a key in the home folder), or a path that climbs out of it,
/// is not there; and a walk enters no directory a link brings it back to
/// (`src/a -> .`), so a loop cannot run it forever. A link that stays
/// inside is followed, as it always was. What a walk left out is kept, for
/// the build to say.
///
/// A file system with no links (a test's, the browser's) is read as it is.
pub struct Confined<'a> {
    inner: &'a dyn Vfs,
    root: PathBuf,
    /// The root, resolved: `None` where it is not on a disk.
    real_root: Option<PathBuf>,
    skipped: std::sync::Mutex<BTreeMap<PathBuf, &'static str>>,
}

impl<'a> Confined<'a> {
    pub fn new(inner: &'a dyn Vfs, root: &Path) -> Self {
        Confined {
            inner,
            root: root.to_path_buf(),
            real_root: inner.canonical(root),
            skipped: Default::default(),
        }
    }

    /// The links a walk left out, relative to the root where they are
    /// under it, each with why.
    pub fn skipped(&self) -> Vec<(PathBuf, &'static str)> {
        let skipped = self.skipped.lock().unwrap_or_else(|e| e.into_inner());
        skipped
            .iter()
            .map(|(path, why)| {
                let shown = path.strip_prefix(&self.root).unwrap_or(path);
                (shown.to_path_buf(), *why)
            })
            .collect()
    }

    fn skip(&self, path: &Path, why: &'static str) {
        self.skipped
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(path.to_path_buf(), why);
    }

    /// Whether `path` leads out of the root on the disk: the deepest part
    /// of it that is there, resolved, is somewhere else.
    pub fn escapes(&self, path: &Path) -> bool {
        let Some(root) = &self.real_root else {
            return false;
        };
        let mut at = path.to_path_buf();
        loop {
            if let Some(real) = self.inner.canonical(&at) {
                return !real.starts_with(root);
            }
            if self.inner.is_link(&at) {
                // A link to nothing: whatever it names is not the project's.
                return true;
            }
            if !at.pop() || at.as_os_str().is_empty() {
                return false;
            }
        }
    }

    /// Whether a walk may go on into `entry`, read from a directory.
    fn admits(&self, entry: &Path) -> bool {
        let Some(root) = &self.real_root else {
            return true;
        };
        if !self.inner.is_link(entry) {
            return true;
        }
        let Some(target) = self.inner.canonical(entry) else {
            self.skip(entry, "a link to nothing");
            return false;
        };
        if !target.starts_with(root) {
            self.skip(entry, "a link that leads outside the project");
            return false;
        }
        if self.inner.is_dir(entry) {
            // Back into a directory the walk is already inside: a loop.
            let mut links = 0;
            for above in entry.ancestors().skip(1) {
                if self.inner.canonical(above).as_ref() == Some(&target) {
                    self.skip(entry, "a link back into a directory above it, a loop");
                    return false;
                }
                if self.inner.is_link(above) {
                    links += 1;
                }
                if above == self.root || above.as_os_str().is_empty() {
                    break;
                }
            }
            if links >= MAX_LINKS {
                self.skip(entry, "a link under too many others");
                return false;
            }
        }
        true
    }

    fn refused(path: &Path) -> io::Error {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "{} leads outside the project, so it is not read",
                path.display()
            ),
        )
    }
}

impl Vfs for Confined<'_> {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        if self.escapes(path) {
            return Err(Self::refused(path));
        }
        self.inner.read(path)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        if self.escapes(path) {
            return Err(Self::refused(path));
        }
        self.inner.read_to_string(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        !self.escapes(path) && self.inner.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        !self.escapes(path) && self.inner.is_dir(path)
    }

    fn exists(&self, path: &Path) -> bool {
        !self.escapes(path) && self.inner.exists(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        if self.escapes(path) {
            return Err(Self::refused(path));
        }
        let mut entries = self.inner.read_dir(path)?;
        entries.retain(|entry| self.admits(entry));
        Ok(entries)
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        if self.escapes(path) {
            return Err(Self::refused(path));
        }
        self.inner.metadata(path)
    }

    fn copy_out(&self, from: &Path, to: &Path) -> io::Result<()> {
        if self.escapes(from) {
            return Err(Self::refused(from));
        }
        self.inner.copy_out(from, to)
    }

    fn is_link(&self, path: &Path) -> bool {
        self.inner.is_link(path)
    }

    fn canonical(&self, path: &Path) -> Option<PathBuf> {
        self.inner.canonical(path)
    }
}

/// Whether `path`, inside the project at `dir` by its words, leads out of
/// it on the disk: through a link, anywhere on the way, to somewhere else
/// or to nothing. What an agent reads, writes or is shown must not.
pub fn escapes(dir: &Path, path: &Path) -> bool {
    let Ok(root) = std::fs::canonicalize(dir) else {
        return false;
    };
    // The deepest part of the path that is there, resolved.
    let mut at = path.to_path_buf();
    loop {
        if std::fs::symlink_metadata(&at).is_ok() {
            return match std::fs::canonicalize(&at) {
                Ok(real) => !real.starts_with(&root),
                // A link to nothing: writing through it would make its
                // target, wherever that is.
                Err(_) => true,
            };
        }
        if !at.pop() {
            return false;
        }
    }
}

/// Whether `path` is a link that leads out of the project at `dir`.
pub(crate) fn leaves(dir: &Path, path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) && escapes(dir, path)
}

/// Whether `file`, a path a project names (a `data` file, an image, the
/// translations directory, a font directory, a specification), stays
/// inside the project by its words: not absolute, no `..`, no NUL. Where it
/// leads on the disk — through a link — is the reader's to check
/// ([`crate::base::vfs::Confined`]).
pub fn stays_inside(file: &str) -> bool {
    let path = Path::new(file);
    !file.is_empty()
        && !path.is_absolute()
        && !file.starts_with(['/', '\\'])
        && !file.contains('\0')
        // A drive, as Windows writes one: `C:\x`, `C:x`.
        && !(file.len() >= 2 && file.as_bytes()[1] == b':' && file.as_bytes()[0].is_ascii_alphabetic())
        && !file.split(['/', '\\']).any(|part| part == "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reading_a_missing_file_names_it() {
        let e = FsVfs
            .read_to_string(Path::new("/nowhere/wf-vfs-missing.wf"))
            .unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::NotFound);
        assert!(e.to_string().contains("/nowhere/wf-vfs-missing.wf"), "{e}");
    }

    /// A project on disk: `src/App.wf`, `src/pages/Home.wf`, `public/a.txt`.
    struct Tree(PathBuf);

    impl Tree {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir().join(format!("wf-vfs-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(root.join("src/pages")).unwrap();
            std::fs::create_dir_all(root.join("public")).unwrap();
            std::fs::write(root.join("src/App.wf"), "app").unwrap();
            std::fs::write(root.join("src/pages/Home.wf"), "home").unwrap();
            std::fs::write(root.join("public/a.txt"), "a").unwrap();
            Tree(root)
        }
    }

    impl Drop for Tree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn names(vfs: &dyn Vfs, dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = vfs
            .read_dir(dir)
            .unwrap()
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn the_disk_reads_as_the_disk() {
        let tree = Tree::new("disk");
        let fs = FsVfs;
        assert_eq!(
            fs.read_to_string(&tree.0.join("src/App.wf")).unwrap(),
            "app"
        );
        assert!(fs.is_file(&tree.0.join("src/App.wf")));
        assert!(fs.is_dir(&tree.0.join("src/pages")));
        assert!(!fs.exists(&tree.0.join("src/Nope.wf")));
        assert_eq!(names(&fs, &tree.0.join("src")), ["App.wf", "pages"]);
        assert_eq!(fs.metadata(&tree.0.join("src/App.wf")).unwrap().len, 3);
        let entries = fs.read_dir(&tree.0.join("src")).unwrap();
        assert!(entries.iter().all(|e| e.starts_with(tree.0.join("src"))));
    }

    #[test]
    fn a_buffer_is_read_instead_of_the_file() {
        let tree = Tree::new("buffer");
        let mut vfs = OverlayVfs::over_disk();
        vfs.set(tree.0.join("src/App.wf"), "unsaved");
        assert_eq!(
            vfs.read_to_string(&tree.0.join("src/App.wf")).unwrap(),
            "unsaved"
        );
        assert_eq!(vfs.read(&tree.0.join("src/App.wf")).unwrap(), b"unsaved");
        // The rest is the disk's.
        assert_eq!(
            vfs.read_to_string(&tree.0.join("src/pages/Home.wf"))
                .unwrap(),
            "home"
        );
        // Listed once, not beside the file it covers.
        assert_eq!(names(&vfs, &tree.0.join("src")), ["App.wf", "pages"]);
        // With no time, so no cache keeps it.
        let meta = vfs.metadata(&tree.0.join("src/App.wf")).unwrap();
        assert_eq!((meta.len, meta.modified), (7, None));
        // A `.` in the path names the same buffer.
        let dotted = tree.0.join("./src/./App.wf");
        assert_eq!(vfs.read_to_string(&dotted).unwrap(), "unsaved");

        assert!(vfs.clear(tree.0.join("src/App.wf")));
        assert_eq!(
            vfs.read_to_string(&tree.0.join("src/App.wf")).unwrap(),
            "app"
        );
    }

    #[test]
    fn a_buffer_not_on_disk_yet_is_listed_where_it_will_be() {
        let tree = Tree::new("new");
        let mut vfs = OverlayVfs::over_disk();
        vfs.set(tree.0.join("src/pages/About.wf"), "about");
        vfs.set(tree.0.join("src/stores/deploys.wf"), "store");
        assert_eq!(
            names(&vfs, &tree.0.join("src/pages")),
            ["About.wf", "Home.wf"]
        );
        // A directory only a buffer is in is one, and lists it.
        assert_eq!(
            names(&vfs, &tree.0.join("src")),
            ["App.wf", "pages", "stores"]
        );
        assert!(vfs.is_dir(&tree.0.join("src/stores")));
        assert!(!vfs.is_file(&tree.0.join("src/stores")));
        assert!(vfs.is_file(&tree.0.join("src/stores/deploys.wf")));
        assert_eq!(names(&vfs, &tree.0.join("src/stores")), ["deploys.wf"]);
        // The disk is untouched.
        assert!(!tree.0.join("src/stores").exists());
    }

    #[test]
    fn a_deleted_path_is_gone_until_a_buffer_brings_it_back() {
        let tree = Tree::new("deleted");
        let mut vfs = OverlayVfs::over_disk();
        vfs.delete(tree.0.join("src/pages/Home.wf"));
        assert!(!vfs.exists(&tree.0.join("src/pages/Home.wf")));
        assert!(
            vfs.read_to_string(&tree.0.join("src/pages/Home.wf"))
                .is_err()
        );
        assert!(names(&vfs, &tree.0.join("src/pages")).is_empty());

        // A deleted directory takes what is in it.
        vfs.delete(tree.0.join("public"));
        assert!(!vfs.is_dir(&tree.0.join("public")));
        assert!(!vfs.is_file(&tree.0.join("public/a.txt")));
        assert_eq!(names(&vfs, &tree.0), ["src"]);

        vfs.set(tree.0.join("src/pages/Home.wf"), "again");
        assert_eq!(
            vfs.read_to_string(&tree.0.join("src/pages/Home.wf"))
                .unwrap(),
            "again"
        );
        assert_eq!(names(&vfs, &tree.0.join("src/pages")), ["Home.wf"]);

        // A buffer in a directory deleted after it goes with it.
        vfs.set(tree.0.join("src/stores/new.wf"), "store");
        vfs.delete(tree.0.join("src/stores"));
        assert!(!vfs.is_file(&tree.0.join("src/stores/new.wf")));
        assert_eq!(names(&vfs, &tree.0.join("src")), ["App.wf", "pages"]);
    }

    #[test]
    fn an_overlay_copies_out_what_it_reads() {
        let tree = Tree::new("copy");
        let mut vfs = OverlayVfs::over_disk();
        vfs.set(tree.0.join("public/b.txt"), "b");
        let out = tree.0.join("out");
        std::fs::create_dir_all(&out).unwrap();
        vfs.copy_out(&tree.0.join("public/a.txt"), &out.join("a.txt"))
            .unwrap();
        vfs.copy_out(&tree.0.join("public/b.txt"), &out.join("b.txt"))
            .unwrap();
        assert_eq!(std::fs::read_to_string(out.join("a.txt")).unwrap(), "a");
        assert_eq!(std::fs::read_to_string(out.join("b.txt")).unwrap(), "b");
    }

    #[test]
    fn an_overlay_can_sit_on_another() {
        let tree = Tree::new("stack");
        let mut lower = OverlayVfs::over_disk();
        lower.set(tree.0.join("src/App.wf"), "lower");
        lower.set(tree.0.join("src/Extra.wf"), "extra");
        let mut upper = OverlayVfs::new(Arc::new(lower));
        upper.set(tree.0.join("src/App.wf"), "upper");
        assert_eq!(
            upper.read_to_string(&tree.0.join("src/App.wf")).unwrap(),
            "upper"
        );
        assert_eq!(
            upper.read_to_string(&tree.0.join("src/Extra.wf")).unwrap(),
            "extra"
        );
        assert_eq!(
            names(&upper, &tree.0.join("src")),
            ["App.wf", "Extra.wf", "pages"]
        );
        assert_eq!(upper.buffers().count(), 1);
    }
}
