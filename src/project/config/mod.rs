//! Project configuration — loads and manages `webfluent.app.json`.

pub mod json_edit;
pub mod project;
pub mod schema;

pub use project::{
    CleanUrls, MotionConfig, OFFLINE_STRATEGIES, OfflineConfig, OutputType, Owner, ProjectConfig,
    RuntimeMode,
};
