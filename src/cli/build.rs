//! `wf build` and `wf check`: the library's build, printed as it goes.

use crate::build::{BuildFailure, BuildLine, BuildOptions, BuildReport, Stream};
use crate::diagnostics::format::Format;
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
    run_build_opts(
        project_dir,
        Options {
            stats,
            ..Options::default()
        },
    )
}

/// How a build, or a check, reports.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    /// Print what the output weighs.
    pub stats: bool,
    /// How the findings are written: for a person, or for a tool.
    pub format: Format,
    /// A warning stops the build as an error does — for CI.
    pub deny_warnings: bool,
    /// A build for `wf serve`: responses are held to their declared types.
    pub dev: bool,
}

/// Build with `options`. With a format for a tool, the findings are written
/// to standard output — once, as one document for JSON and SARIF, whether
/// the build finished or stopped.
pub fn run_build_opts(project_dir: &Path, options: Options) -> Result<()> {
    run(project_dir, options, crate::build::build)
}

/// `wf check`: every finding of a build, with nothing written.
pub fn run_check(project_dir: &Path, options: Options) -> Result<()> {
    run(project_dir, options, crate::build::check)
}

fn run(
    project_dir: &Path,
    options: Options,
    go: fn(&BuildOptions) -> std::result::Result<BuildReport, BuildFailure>,
) -> Result<()> {
    let print = |line: &BuildLine| match line.stream {
        Stream::Stdout => println!("{}", line.text),
        Stream::Stderr => eprintln!("{}", line.text),
    };
    let build = BuildOptions {
        stats: options.stats,
        format: options.format,
        deny_warnings: options.deny_warnings,
        dev: options.dev,
        color: crate::diagnostics::render::stderr_wants_color(),
        on_line: Some(&print),
        ..BuildOptions::new(project_dir, &FsVfs)
    };
    go(&build).map(|_| ()).map_err(|failure| failure.error)
}
