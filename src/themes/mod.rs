//! Design system — theme tokens and component CSS.
//!
//! - [`get_theme_tokens`] — returns CSS custom properties for a given theme name
//! - [`component_css`] — returns all component-level CSS rules

pub mod tokens;
pub mod components;

pub use tokens::get_theme_tokens;
pub use components::component_css;
