//! A project as data: its config, theme, translations, scripts, data
//! files, media and API descriptions, and the helpers they share.

pub mod builtin;
/// Project configuration — loads and manages `webfluent.app.json`.
///
/// The [`config::ProjectConfig`] struct holds all project settings: theme, build options,
/// dev server config, meta tags, and i18n settings.
pub mod config;
/// `data` declarations: the JSON files and pictures a program names, read
/// at build time and made constants.
pub mod data;
pub mod format;
pub mod i18n;
pub mod media;
pub mod openapi;
pub mod project_js;
pub mod sanitize;
pub mod scoped_css;
/// A project's sources, read and merged into one program.
pub(crate) mod sources;
pub mod static_eval;
pub mod themes;
pub mod url;
