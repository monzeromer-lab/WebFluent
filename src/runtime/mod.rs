//! Embedded JavaScript runtime for WebFluent applications.
//!
//! Provides signal-based reactivity, DOM helpers, conditional/list rendering,
//! client-side routing, store management, i18n, animations, and toast notifications.

/// The full JavaScript runtime source, embedded at compile time.
///
/// This is included in the generated `app.js` output and provides:
/// - `WF.signal()`, `WF.effect()`, `WF.computed()` — reactivity
/// - `WF.h()` — DOM element creation with reactive attributes
/// - `WF.condRender()`, `WF.listRender()`, `WF.showRender()` — control flow
/// - `WF.createRouter()`, `WF.navigate()` — client-side routing
/// - `WF.createStore()` — shared state management
/// - `WF.createI18n()` — internationalization with RTL support
/// - `WF.animateIn()`, `WF.animateOut()` — CSS animations
/// - `WF.showToast()` — toast notifications
pub const RUNTIME_JS: &str = include_str!("runtime.js");
