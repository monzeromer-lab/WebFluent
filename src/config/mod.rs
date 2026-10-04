//! Project configuration — loads and manages `webfluent.app.json`.

pub mod project;

pub use project::{
    CleanUrls, MotionConfig, OFFLINE_STRATEGIES, OfflineConfig, OutputType, Owner, ProjectConfig,
    RuntimeMode,
};
