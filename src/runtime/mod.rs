//! Embedded JavaScript runtime for WebFluent applications.
//!
//! Provides signal-based reactivity, DOM helpers, conditional/list rendering,
//! client-side routing, store management, i18n, animations, and toast notifications.

/// The full JavaScript runtime source, embedded at compile time.
///
/// This is included in the generated `app.js` output and provides:
/// - `WF.signal()`, `WF.effect()`, `WF.computed()` — reactivity
/// - `WF.el()` — DOM element creation with reactive attributes
/// - `WF.when()`, `WF.each()`, `WF.show()` — control flow
/// - `WF.router()`, `WF.navigate()` — client-side routing
/// - `WF.store()` — shared state management
/// - `WF.locales()` — internationalization with RTL support
/// - `WF.animateIn()`, `WF.animateOut()` — CSS animations
/// - `WF.toast()` — toast notifications
pub const RUNTIME_JS: &str = include_str!("runtime.js");
