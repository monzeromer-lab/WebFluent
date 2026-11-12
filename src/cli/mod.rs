pub(crate) mod act;
pub(crate) mod assets;
pub(crate) mod audit;
pub(crate) mod build;
pub(crate) mod components;
pub(crate) mod config;
pub(crate) mod context;
pub(crate) mod describe;
pub(crate) mod docs;
pub(crate) mod explain;
pub(crate) mod fix;
pub(crate) mod fmt;
pub(crate) mod generate;
pub(crate) mod init;
pub(crate) mod migrate;
pub(crate) mod motion;
pub(crate) mod preview;
pub(crate) mod render;
pub(crate) mod rules;
pub(crate) mod scope;
pub(crate) mod serve;
pub(crate) mod test;
pub(crate) mod theme;
pub(crate) mod translations;

/// How a command reports: as text for a reader, or as JSON for a tool
/// (`--json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Report {
    Text,
    Json,
}

impl Report {
    /// The report `--json` asks for.
    pub(crate) fn from_json_flag(json: bool) -> Self {
        if json { Report::Json } else { Report::Text }
    }
}

/// The layout a migrated file is written in: braces (`.wf`) or
/// indentation (`.wfx`, `--wfx`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layout {
    Braces,
    Indented,
}
pub(crate) mod verify;

mod app;
pub use app::main;
