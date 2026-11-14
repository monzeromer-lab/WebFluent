//! The **structural** stylesheet — layout and language mechanics, no baseline look.
//!
//! [`components::component_css`](super::components::component_css) is the engine's
//! original opinionated sheet: it lays elements out *and* paints them, so every site
//! built with it inherits the same blue-on-Inter identity whether or not the author
//! asked for one. This sheet is the same layout with the assumed design removed, for
//! authors (the studio among them) who supply their own.
//!
//! The line between the two is *what the author asked for*:
//!
//! - **Layout** — `display`, flex/grid, `gap`, sizing, the box's own padding,
//!   `position`/`inset`/`z-index`, `overflow`. Kept: it is not a look, it is a
//!   structure, and dropping it turns a `Row` into a stack of divs.
//! - **Mechanics** — the `.open` / `.active` display toggles the runtime drives, the
//!   `:checked` switch transform, keyframes and `.wf-animate-*`. Kept: these implement
//!   language features, and removing them breaks `Modal`, `Tabs` and `Animate`.
//! - **Requested variants** — `.wf-btn--primary`, `.wf-text--center`, `.wf-badge--pill`.
//!   Kept, colour included: `Button("Go", primary)` is the author asking for the primary
//!   colour, not the engine deciding one.
//! - **Baseline paint** — what applied when the author asked for *nothing*: the button's
//!   chrome, the card's border and radius, the navbar's frosted background, `body`'s
//!   font and colour, the `h1`–`h6` type scale, hover and transition states. Dropped.
//!   This is the design the engine was making on the author's behalf.
//!
//! Two deliberate exceptions, both about function rather than taste:
//!
//! - **Overlays get an opaque surface.** A dropdown or modal panel that does not occlude
//!   the content beneath it is unreadable — a defect, not a minimal style. Those panels
//!   keep a `--color-surface`/`--color-background` fill and the modal keeps its scrim.
//! - **Mechanism paint.** A switch, spinner, progress bar, skeleton and carousel dot
//!   *are* their geometry — with no ink they are invisible and inoperable. They keep the
//!   minimum required to be perceived, drawn from tokens so a palette change retunes them.
//!
//! Accessibility defaults are left alone: this sheet sets no `outline: none` and no
//! `appearance: none`, so focus rings and native control affordances survive. The
//! original sheet suppressed both and drew its own replacements.
//!
//! Tokens are unaffected — [`tokens::default_tokens`](super::tokens::default_tokens)
//! still populates `:root`, so `var(--spacing-md)` here and a `style { background:
//! surface }` block in `.wf` source both resolve exactly as before.

/// Returns the layout-and-mechanics CSS, without the engine's baseline design.
///
/// Paired with the `:root` token block by
/// [`generate_css_with`](crate::web::codegen::generate_css_with) under
/// [`BuiltinCss::Structural`](super::BuiltinCss::Structural).
pub fn structural_css() -> &'static str {
    super::sheet::structural()
}
