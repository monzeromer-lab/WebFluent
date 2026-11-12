//! The bottom layer: spans, text helpers, the file systems a build reads
//! and writes, and the platform's threads and clocks. It uses nothing else
//! in the crate.

/// Where a build writes: the disk, or — inside [`outfs::capture`] — memory,
/// which is how the compiler builds as WebAssembly in a browser.
pub mod outfs;
/// Threads and clocks, and their stand-ins where WebAssembly has none.
pub mod platform;
/// Where a node of a program was written.
pub(crate) mod span;
/// Small text utilities the rest of the crate shares.
pub(crate) mod text;
/// Where a project is read from: the disk ([`vfs::FsVfs`]), or the disk with
/// an editor's unsaved buffers over it ([`vfs::OverlayVfs`]).
pub mod vfs;
