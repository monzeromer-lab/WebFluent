//! Design system — theme tokens and the built-in stylesheets.
//!
//! - [`resolve_tokens`] — the design tokens a build ships, from its `Theme`
//!   declaration, the baseline, and any config overrides
//! - [`component_css`] — the original sheet: layout *and* the engine's baseline design
//! - [`structural_css`] — the same layout with that baseline design removed
//!
//! Which one a build gets is [`BuiltinCss`], carried on `ThemeConfig::builtin`.

/// The rules a paged document and a slide deck need, in both of the engine's
/// sheets: a PDF is the page's markup laid out on paper, styled as the web
/// styles everything else. Sizes are in `em` of the deck's base size, so
/// `slides.default_font_size` scales them.
#[macro_export]
#[doc(hidden)]
macro_rules! paged_rules {
    () => {
        r#"
/* ─── Paged documents and slide decks ───────────────── */
/* A PDF is this markup laid out on paper, so a document's own elements and a
   deck's slides are styled here, as the web styles everything else. Sizes are
   in `em` of the deck's base size, so `slides.default_font_size` scales them. */
.wf-page-break { break-after: page; }
.wf-chart, .wf-qr { margin: 0; }
.wf-chart svg { width: 100%; height: auto; display: block; }
.wf-qr svg { width: 100%; height: auto; display: block; }
.wf-toc { display: block; }
.wf-toc__title { margin-bottom: 0.75em; }
.wf-toc__entry { display: flex; align-items: baseline; gap: 0.4em; padding: 0.2em 0; color: inherit; text-decoration: none; }
.wf-toc__entry--2 { padding-left: 1.25em; }
.wf-toc__entry--3 { padding-left: 2.5em; font-size: 0.92em; }
.wf-toc__leader { flex: 1; border-bottom: 1px dotted currentColor; opacity: 0.35; min-width: 1em; }
.wf-toc__page { font-variant-numeric: tabular-nums; }
.wf-watermark { font-size: 72pt; font-weight: 700; color: rgba(0, 0, 0, 0.08); white-space: nowrap; }
.wf-slide, .wf-title-slide, .wf-section-slide, .wf-two-column, .wf-image-slide { display: flex; flex-direction: column; gap: 0.5em; }
.wf-title-slide { justify-content: center; align-items: center; text-align: center; }
.wf-slide__title { font-size: 2.33em; font-weight: 700; line-height: 1.15; }
.wf-slide__subtitle { font-size: 1.17em; color: var(--color-text-muted); }
.wf-section-slide { justify-content: center; align-items: center; text-align: center; color: #fff; background: var(--color-primary); }
.wf-section-slide--primary { background: var(--color-primary); }
.wf-section-slide--success { background: var(--color-success); }
.wf-section-slide--danger { background: var(--color-danger); }
.wf-section-slide--warning { background: var(--color-warning); color: #000; }
.wf-section-slide--info { background: var(--color-info); }
.wf-slide__label { font-size: 2em; font-weight: 700; }
.wf-two-column { flex-direction: row; gap: 1em; }
.wf-two-column > * { flex: 1 1 0; min-width: 0; }
.wf-image-slide { align-items: center; justify-content: center; }
.wf-slide__image { max-width: 100%; max-height: 85%; object-fit: contain; }
.wf-slide__caption { color: var(--color-text-muted); font-size: 0.75em; }
.wf-slide h1, .wf-two-column h1 { font-size: 2em; }
.wf-slide h2, .wf-two-column h2 { font-size: 1.6em; }
.wf-slide h3, .wf-two-column h3 { font-size: 1.3em; }
"#
    };
}

use serde::{Deserialize, Serialize};

pub mod components;
pub mod prune;
pub mod resolve;
pub mod structural;
pub mod tokens;

pub use components::component_css;
pub use prune::{Usage, prune_css};
pub use resolve::{apply_motion, resolve_dark_tokens, resolve_tokens};
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
