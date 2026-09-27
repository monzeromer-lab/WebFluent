//! Where a project is read from.
//!
//! Everything that reads a project — `wf build`, the language server, an
//! editor that builds what its buffers say before they are saved — reads it
//! through a [`Vfs`]. [`FsVfs`] is the disk, and what every command uses.
//! [`OverlayVfs`] is another file system with an editor's open buffers laid
//! over it: a buffer is read instead of the file it names, a buffer for a
//! file that is not on disk yet is listed in its directory, and a file can
//! be hidden as though it were deleted.
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
        std::fs::write(to, self.read(from)?)
    }
}

/// The disk.
#[derive(Clone, Copy, Debug, Default)]
pub struct FsVfs;

impl Vfs for FsVfs {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
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
        std::fs::read_dir(path)?
            .map(|entry| entry.map(|e| e.path()))
            .collect()
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let meta = std::fs::metadata(path)?;
        Ok(Metadata {
            len: meta.len(),
            modified: meta.modified().ok(),
        })
    }

    fn copy_out(&self, from: &Path, to: &Path) -> io::Result<()> {
        // `fs::copy` keeps the file's permissions, as `wf build` always has.
        std::fs::copy(from, to).map(|_| ())
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
    Deleted,
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

    /// Hide `path` — and, for a directory, everything under it — as though
    /// it were deleted. A buffer set under it later shows.
    pub fn delete(&mut self, path: impl AsRef<Path>) {
        self.layers.insert(key(path.as_ref()), Layer::Deleted);
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
            Layer::Deleted => None,
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
            .any(|(at, layer)| matches!(layer, Layer::Text(_)) && at != key && at.starts_with(key))
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
        match self.layers.get(&at) {
            Some(Layer::Text(text)) => Ok(text.as_bytes().to_vec()),
            _ if self.hidden(&at) => Err(Self::gone(path)),
            _ => self.base.read(path),
        }
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        let at = key(path);
        match self.layers.get(&at) {
            Some(Layer::Text(text)) => Ok(text.to_string()),
            _ if self.hidden(&at) => Err(Self::gone(path)),
            _ => self.base.read_to_string(path),
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        let at = key(path);
        match self.layers.get(&at) {
            Some(Layer::Text(_)) => true,
            _ if self.hidden(&at) => false,
            _ => self.base.is_file(path),
        }
    }

    fn is_dir(&self, path: &Path) -> bool {
        let at = key(path);
        if matches!(self.layers.get(&at), Some(Layer::Text(_))) {
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
                || matches!(self.layers.get(&entry), Some(Layer::Text(_)))
                || self.holds_buffer(&entry)
        });
        // Each buffer under `path` is listed by the name of its first
        // component below it: the file itself, or the directory it is in.
        for (buffered, layer) in &self.layers {
            if !matches!(layer, Layer::Text(_)) {
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
            Some(Layer::Text(text)) => Ok(Metadata {
                len: text.len() as u64,
                modified: None,
            }),
            _ if self.hidden(&at) => Err(Self::gone(path)),
            _ => self.base.metadata(path),
        }
    }

    fn copy_out(&self, from: &Path, to: &Path) -> io::Result<()> {
        let at = key(from);
        match self.layers.get(&at) {
            Some(Layer::Text(text)) => std::fs::write(to, text.as_bytes()),
            _ if self.hidden(&at) => Err(Self::gone(from)),
            _ => self.base.copy_out(from, to),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
