//! `wf build`: the library's build, printed as it goes.

use crate::build::{BuildLine, BuildOptions, Stream};
use crate::error::Result;
use crate::vfs::FsVfs;
use std::path::Path;

pub use crate::build::read_project;

pub fn run_build(project_dir: &Path) -> Result<()> {
    run_build_with(project_dir, false)
}

/// Build, and with `stats` print what the output weighs and which runtime
/// modules it carries.
pub fn run_build_with(project_dir: &Path, stats: bool) -> Result<()> {
    let print = |line: &BuildLine| match line.stream {
        Stream::Stdout => println!("{}", line.text),
        Stream::Stderr => eprintln!("{}", line.text),
    };
    let options = BuildOptions {
        stats,
        on_line: Some(&print),
        ..BuildOptions::new(project_dir, &FsVfs)
    };
    crate::build::build(&options)
        .map(|_| ())
        .map_err(|failure| failure.error)
}
