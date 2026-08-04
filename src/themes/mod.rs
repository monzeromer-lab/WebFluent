//! Design system — theme tokens and the built-in stylesheets.
//!
//! - [`resolve_tokens`] — the design tokens a build ships, from its `Theme`
//!   declaration, the baseline, and any config overrides
//! - [`component_css`] — the original sheet: layout *and* the engine's baseline design
//! - [`structural_css`] — the same layout with that baseline design removed
//!
//! Which one a build gets is [`BuiltinCss`], carried on `ThemeConfig::builtin`.

use serde::{Deserialize, Serialize};

pub mod components;
pub mod resolve;
pub mod structural;
pub mod tokens;

pub use components::component_css;
pub use resolve::resolve_tokens;
pub use structural::structural_css;

/// How much of the engine's built-in stylesheet a build emits.
///
/// The engine has always shipped one opinionated sheet, so every site inherited the
/// same look whether or not its author wanted one. That is the right default for
/// `wf build` — an author who writes no styles still gets a finished-looking site —
/// and the wrong one for a tool that generates its own design, which then has to
/// fight the baseline it never asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BuiltinCss {
    /// Layout, mechanics *and* the engine's baseline design. The historical
    /// behaviour, and the default — existing projects build byte-for-byte as before.
    #[default]
    Full,
    /// Layout and language mechanics only; the author supplies the design. Named
    /// themes do not apply their overrides in this mode — with no baseline to
    /// restyle, a canned palette is just another opinion. See [`structural`].
    Structural,
}
