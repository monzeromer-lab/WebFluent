//! A project as data: its config, theme, translations, scripts, data
//! files, media and API descriptions, and the helpers they share.

pub mod builtin;
pub mod config;
pub mod data;
pub mod format;
pub mod i18n;
pub mod media;
pub mod openapi;
pub mod project_js;
pub mod sanitize;
pub mod scoped_css;
pub(crate) mod sources;
pub mod static_eval;
pub mod themes;
pub mod url;
