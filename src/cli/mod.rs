pub mod act;
pub mod assets;
pub mod audit;
pub mod build;
pub mod components;
pub mod config;
pub mod context;
pub mod describe;
pub mod docs;
pub mod explain;
pub mod fix;
pub mod fmt;
pub mod generate;
pub mod init;
pub mod migrate;
pub mod motion;
pub mod preview;
pub mod render;
pub mod rules;
pub mod scope;
pub mod serve;
pub mod test;
pub mod theme;
pub mod translations;

/// How a command reports: as text for a reader, or as JSON for a tool
/// (`--json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    Text,
    Json,
}

impl Report {
    /// The report `--json` asks for.
    pub fn from_json_flag(json: bool) -> Self {
        if json { Report::Json } else { Report::Text }
    }
}

/// The layout a migrated file is written in: braces (`.wf`) or
/// indentation (`.wfx`, `--wfx`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Braces,
    Indented,
}
pub mod verify;
