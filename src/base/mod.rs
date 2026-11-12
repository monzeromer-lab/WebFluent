//! The bottom layer: spans, text helpers, the file systems a build reads
//! and writes, and the platform's threads and clocks. It uses nothing else
//! in the crate.

pub mod outfs;
pub mod platform;
pub(crate) mod span;
pub(crate) mod text;
pub mod vfs;
