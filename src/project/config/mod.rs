//! Project configuration — loads and manages `webfluent.app.json`.
//!
//! The [`ProjectConfig`] struct holds all project settings: theme, build options,
//! dev server config, meta tags, and i18n settings.
//!
//! Project configuration — loads and manages `webfluent.app.json`.

#[doc(hidden)]
pub mod json_edit;
pub mod project;
#[doc(hidden)]
pub mod schema;

pub use project::{
    CleanUrls, MotionConfig, OFFLINE_STRATEGIES, OfflineConfig, OutputType, Owner, ProjectConfig,
    RuntimeMode,
};
