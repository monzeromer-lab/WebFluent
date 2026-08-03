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
/// [`generate_css_with`](crate::codegen::generate_css_with) under
/// [`BuiltinCss::Structural`](super::BuiltinCss::Structural).
pub fn structural_css() -> &'static str {
    r#"
/* ─── Normalize ─────────────────────────────────────── */
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html { scroll-behavior: smooth; }
body { min-height: 100vh; }
#app { min-height: 100vh; display: flex; flex-direction: column; }
a { color: inherit; text-decoration: none; }
button, input, select, textarea { font: inherit; color: inherit; }

/* ─── Layout ────────────────────────────────────────── */
.wf-container { width: 100%; max-width: 1200px; margin: 0 auto; padding: 0 var(--spacing-md); }
.wf-container--fluid { max-width: 100%; }
.wf-row { display: flex; flex-wrap: wrap; gap: var(--spacing-md); }
.wf-row--center { align-items: center; }
.wf-row--between { justify-content: space-between; }
.wf-row--end { justify-content: flex-end; }
.wf-col { flex: 1 1 0%; min-width: 0; }
.wf-col--1 { flex: 0 0 calc(100%/12*1); max-width: calc(100%/12*1); }
.wf-col--2 { flex: 0 0 calc(100%/12*2); max-width: calc(100%/12*2); }
.wf-col--3 { flex: 0 0 25%; max-width: 25%; }
.wf-col--4 { flex: 0 0 calc(100%/3); max-width: calc(100%/3); }
.wf-col--5 { flex: 0 0 calc(100%/12*5); max-width: calc(100%/12*5); }
.wf-col--6 { flex: 0 0 50%; max-width: 50%; }
.wf-col--7 { flex: 0 0 calc(100%/12*7); max-width: calc(100%/12*7); }
.wf-col--8 { flex: 0 0 calc(100%/12*8); max-width: calc(100%/12*8); }
.wf-col--9 { flex: 0 0 75%; max-width: 75%; }
.wf-col--10 { flex: 0 0 calc(100%/12*10); max-width: calc(100%/12*10); }
.wf-col--11 { flex: 0 0 calc(100%/12*11); max-width: calc(100%/12*11); }
.wf-col--12 { flex: 0 0 100%; max-width: 100%; }
.wf-grid { display: grid; gap: var(--spacing-md); }
.wf-stack { display: flex; flex-direction: column; gap: var(--spacing-md); }
.wf-spacer { height: var(--spacing-md); }
.wf-spacer--xs { height: var(--spacing-xs); }
.wf-spacer--sm { height: var(--spacing-sm); }
.wf-spacer--lg { height: var(--spacing-lg); }
.wf-spacer--xl { height: var(--spacing-xl); }
/* A Divider's whole meaning is "draw a line" — the rule is the element, not its skin. */
.wf-divider { border: none; border-top: 1px solid var(--color-border); margin: var(--spacing-md) 0; }
.wf-divider--label { display: flex; align-items: center; gap: var(--spacing-md); border: none; margin: var(--spacing-md) 0; }
.wf-divider--label::before, .wf-divider--label::after { content: ""; flex: 1; border-top: 1px solid var(--color-border); }

/* ─── Navbar ────────────────────────────────────────── */
.wf-navbar { display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-lg); position: sticky; top: 0; z-index: 100; }
.wf-navbar__brand { flex-shrink: 0; }
.wf-navbar__links { display: flex; gap: var(--spacing-sm); align-items: center; flex-wrap: wrap; overflow-x: auto; }
.wf-navbar__links a { padding: var(--spacing-xs) var(--spacing-sm); white-space: nowrap; }
.wf-navbar__actions { display: flex; gap: var(--spacing-sm); align-items: center; margin-inline-start: auto; flex-shrink: 0; }

/* ─── Sidebar ───────────────────────────────────────── */
.wf-sidebar { width: 260px; padding: var(--spacing-md); display: flex; flex-direction: column; gap: var(--spacing-xs); flex-shrink: 0; position: sticky; top: 56px; height: calc(100vh - 56px); overflow-y: auto; }
.wf-sidebar__header { padding: var(--spacing-sm) 0; }
.wf-sidebar__item { display: flex; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; }
.wf-sidebar__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-sm) 0; }
.wf-sidebar > .wf-text { padding: var(--spacing-xs) var(--spacing-md); margin-top: var(--spacing-xs); }

/* ─── Breadcrumb ────────────────────────────────────── */
.wf-breadcrumb { display: flex; align-items: center; gap: var(--spacing-xs); }
.wf-breadcrumb__item + .wf-breadcrumb__item::before { content: "/"; margin-inline-end: var(--spacing-xs); }

/* ─── Link ──────────────────────────────────────────── */
.wf-link { cursor: pointer; }

/* ─── Menu ──────────────────────────────────────────── */
.wf-menu { position: relative; display: inline-block; }
.wf-menu__trigger { cursor: pointer; }
.wf-menu__items { position: absolute; top: 100%; left: 0; min-width: 180px; z-index: 50; display: none; padding: var(--spacing-xs) 0; background: var(--color-surface); }
.wf-menu.open .wf-menu__items { display: block; }
.wf-menu__item { padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; }
.wf-menu__item--danger { color: var(--color-danger); }
.wf-menu__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-xs) 0; }

/* ─── Tabs ──────────────────────────────────────────── */
.wf-tabs { width: 100%; }
.wf-tabs__nav { display: flex; gap: 0; }
.wf-tabs__tab { padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; background: none; border: none; }
.wf-tab-page { padding: var(--spacing-md) 0; display: none; }
.wf-tab-page.active { display: block; }

/* ─── Card ──────────────────────────────────────────── */
.wf-card { overflow: hidden; }
.wf-card__header { padding: var(--spacing-md); }
.wf-card__body { padding: var(--spacing-md); }
.wf-card__footer { padding: var(--spacing-md); display: flex; gap: var(--spacing-sm); justify-content: flex-end; }

/* ─── Table ─────────────────────────────────────────── */
.wf-table { width: 100%; border-collapse: collapse; }
.wf-table th, .wf-table td { padding: var(--spacing-sm) var(--spacing-md); text-align: start; }

/* ─── List ──────────────────────────────────────────── */
.wf-list { list-style: none; }
.wf-list-item { padding: var(--spacing-sm) var(--spacing-md); display: flex; align-items: center; gap: var(--spacing-sm); }

/* ─── Badge ─────────────────────────────────────────── */
.wf-badge { display: inline-flex; align-items: center; padding: 0.125rem var(--spacing-sm); }
.wf-badge--primary { background: var(--color-primary); color: #fff; }
.wf-badge--success { background: var(--color-success); color: #fff; }
.wf-badge--danger { background: var(--color-danger); color: #fff; }
.wf-badge--warning { background: var(--color-warning); color: #000; }
.wf-badge--info { background: var(--color-info); color: #fff; }
.wf-badge--secondary { background: var(--color-secondary); color: #fff; }
.wf-badge--pill { border-radius: var(--radius-full); }

/* ─── Avatar ────────────────────────────────────────── */
.wf-avatar { width: 40px; height: 40px; overflow: hidden; display: inline-flex; align-items: center; justify-content: center; }
.wf-avatar img { width: 100%; height: 100%; object-fit: cover; }
.wf-avatar--small { width: 32px; height: 32px; }
.wf-avatar--large { width: 56px; height: 56px; }

/* ─── Tooltip ───────────────────────────────────────── */
.wf-tooltip { position: relative; display: inline-block; }
.wf-tooltip__text { visibility: hidden; position: absolute; bottom: 125%; left: 50%; transform: translateX(-50%); padding: var(--spacing-xs) var(--spacing-sm); white-space: nowrap; z-index: 99; opacity: 0; background: var(--color-text); color: var(--color-background); }
.wf-tooltip:hover .wf-tooltip__text { visibility: visible; opacity: 1; }

/* ─── Tag ───────────────────────────────────────────── */
.wf-tag { display: inline-flex; align-items: center; gap: var(--spacing-xs); padding: 0.125rem var(--spacing-sm); }
.wf-tag__remove { cursor: pointer; }

/* ─── Button ────────────────────────────────────────── */
.wf-btn { display: inline-flex; align-items: center; justify-content: center; gap: var(--spacing-xs); padding: var(--spacing-sm) var(--spacing-md); background: none; border: none; cursor: pointer; line-height: 1; }
.wf-btn--primary { background: var(--color-primary); color: #fff; }
.wf-btn--secondary { background: var(--color-secondary); color: #fff; }
.wf-btn--success { background: var(--color-success); color: #fff; }
.wf-btn--danger { background: var(--color-danger); color: #fff; }
.wf-btn--warning { background: var(--color-warning); color: #000; }
.wf-btn--info { background: var(--color-info); color: #fff; }
.wf-btn--small { padding: var(--spacing-xs) var(--spacing-sm); }
.wf-btn--large { padding: var(--spacing-md) var(--spacing-lg); }
.wf-btn--full { width: 100%; }
.wf-btn--rounded { border-radius: var(--radius-full); }
.wf-btn--pill { border-radius: var(--radius-full); }
.wf-btn--outlined { background: transparent; border: 1px solid currentColor; }
.wf-btn--outlined.wf-btn--primary { color: var(--color-primary); background: transparent; }
.wf-btn--outlined.wf-btn--danger { color: var(--color-danger); background: transparent; }

/* ─── Icon Button ───────────────────────────────────── */
.wf-icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 36px; height: 36px; padding: 0; background: transparent; border: none; cursor: pointer; }
.wf-icon-btn--small { width: 28px; height: 28px; }
.wf-icon-btn--large { width: 48px; height: 48px; }
.wf-icon-btn--primary { color: var(--color-primary); }
.wf-icon-btn--danger { color: var(--color-danger); }

/* ─── Button Group ──────────────────────────────────── */
.wf-btn-group { display: inline-flex; }
.wf-btn-group .wf-btn { margin-inline-start: -1px; }
.wf-btn-group .wf-btn:first-child { margin-inline-start: 0; }

/* ─── Dropdown ──────────────────────────────────────── */
.wf-dropdown { position: relative; display: inline-block; }
.wf-dropdown__items { position: absolute; top: 100%; left: 0; min-width: 180px; z-index: 50; display: none; margin-top: var(--spacing-xs); background: var(--color-surface); }
.wf-dropdown.open .wf-dropdown__items { display: block; }
.wf-dropdown__item { padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; }
.wf-dropdown__item--danger { color: var(--color-danger); }
.wf-dropdown__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-xs) 0; }

/* ─── Input ─────────────────────────────────────────── */
.wf-input { display: block; width: 100%; padding: var(--spacing-sm) var(--spacing-md); }
.wf-input--small { padding: var(--spacing-xs) var(--spacing-sm); }
.wf-input--large { padding: var(--spacing-md); }
.wf-input--rounded { border-radius: var(--radius-full); }
.wf-input--full { width: 100%; }

/* ─── Select ────────────────────────────────────────── */
.wf-select { display: block; width: 100%; padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; }

/* ─── Checkbox ──────────────────────────────────────── */
.wf-checkbox { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; }
.wf-checkbox input[type="checkbox"] { width: 18px; height: 18px; cursor: pointer; }

/* ─── Radio ─────────────────────────────────────────── */
.wf-radio { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; }
.wf-radio input[type="radio"] { width: 18px; height: 18px; cursor: pointer; }

/* ─── Switch (mechanism paint) ──────────────────────── */
.wf-switch { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; }
.wf-switch__track { width: 44px; height: 24px; position: relative; background: var(--color-border); border-radius: var(--radius-full); }
.wf-switch__thumb { width: 20px; height: 20px; position: absolute; top: 2px; left: 2px; background: #fff; border-radius: var(--radius-full); }
.wf-switch input:checked + .wf-switch__track { background: var(--color-primary); }
.wf-switch input:checked + .wf-switch__track .wf-switch__thumb { transform: translateX(20px); }
.wf-switch input { display: none; }

/* ─── Slider ────────────────────────────────────────── */
.wf-slider { display: flex; align-items: center; gap: var(--spacing-sm); flex-wrap: wrap; }
.wf-slider input[type="range"] { flex: 1; min-width: 120px; }
.wf-slider__value { min-width: 2em; text-align: end; }
.wf-form-label { display: block; margin-bottom: var(--spacing-xs); }
.wf-form-group { display: flex; flex-direction: column; gap: var(--spacing-xs); }

/* ─── File Upload ──────────────────────────────────── */
.wf-file-upload { display: flex; flex-direction: column; gap: var(--spacing-xs); }
.wf-file-upload input[type="file"] { padding: var(--spacing-sm); cursor: pointer; }
.wf-file-upload input[type="file"]::file-selector-button { padding: var(--spacing-xs) var(--spacing-md); cursor: pointer; margin-inline-end: var(--spacing-sm); }

/* ─── Form ──────────────────────────────────────────── */
.wf-form { display: flex; flex-direction: column; gap: var(--spacing-md); }

/* ─── Alert ─────────────────────────────────────────── */
.wf-alert { padding: var(--spacing-md); display: flex; align-items: center; gap: var(--spacing-sm); }
.wf-alert--success { color: var(--color-success); }
.wf-alert--danger { color: var(--color-danger); }
.wf-alert--warning { color: var(--color-warning); }
.wf-alert--info { color: var(--color-info); }
.wf-alert__dismiss { margin-inline-start: auto; cursor: pointer; background: none; border: none; color: inherit; }

/* ─── Toast (overlay) ───────────────────────────────── */
.wf-toast-container { position: fixed; top: var(--spacing-md); right: var(--spacing-md); z-index: 9999; display: flex; flex-direction: column; gap: var(--spacing-sm); }
.wf-toast { padding: var(--spacing-sm) var(--spacing-md); min-width: 200px; background: var(--color-surface); color: var(--color-text); animation: wf-toast-in 0.3s ease; }
.wf-toast--success { background: var(--color-success); color: #fff; }
.wf-toast--danger { background: var(--color-danger); color: #fff; }
.wf-toast--warning { background: var(--color-warning); color: #000; }
.wf-toast--info { background: var(--color-info); color: #fff; }
.wf-toast--exit { animation: wf-toast-out 0.3s ease forwards; }
@keyframes wf-toast-in { from { transform: translateX(100%); opacity: 0; } to { transform: none; opacity: 1; } }
@keyframes wf-toast-out { from { opacity: 1; } to { opacity: 0; transform: translateX(100%); } }

/* ─── Modal (overlay) ───────────────────────────────── */
.wf-modal { position: fixed; inset: 0; z-index: 1000; display: none; align-items: center; justify-content: center; background: rgba(0,0,0,0.5); }
.wf-modal.open { display: flex; }
.wf-modal__content { background: var(--color-background); max-width: 500px; width: 90%; max-height: 90vh; overflow-y: auto; }
.wf-modal__header { padding: var(--spacing-md); display: flex; justify-content: space-between; align-items: center; }
.wf-modal__header h3 { margin: 0; }
.wf-modal__body { padding: var(--spacing-md); }
.wf-modal__footer { padding: var(--spacing-md); display: flex; justify-content: flex-end; gap: var(--spacing-sm); }

/* ─── Dialog (overlay) ──────────────────────────────── */
.wf-dialog { position: fixed; inset: 0; z-index: 1000; display: none; align-items: center; justify-content: center; background: rgba(0,0,0,0.5); }
.wf-dialog.open { display: flex; }
.wf-dialog__content { background: var(--color-background); padding: var(--spacing-lg); max-width: 400px; width: 90%; display: flex; flex-direction: column; gap: var(--spacing-md); }

/* ─── Spinner (mechanism paint) ─────────────────────── */
.wf-spinner { width: 24px; height: 24px; border: 3px solid var(--color-border); border-top-color: var(--color-primary); border-radius: 50%; animation: wf-spin 0.6s linear infinite; display: inline-block; }
.wf-spinner--large { width: 40px; height: 40px; border-width: 4px; }
@keyframes wf-spin { to { transform: rotate(360deg); } }

/* ─── Progress (mechanism paint) ────────────────────── */
.wf-progress { width: 100%; height: 8px; background: var(--color-border); border-radius: var(--radius-full); overflow: hidden; }
.wf-progress::-webkit-progress-bar { background: var(--color-border); border-radius: var(--radius-full); }
.wf-progress::-webkit-progress-value { background: var(--color-primary); border-radius: var(--radius-full); }
.wf-progress::-moz-progress-bar { background: var(--color-primary); border-radius: var(--radius-full); }

/* ─── Skeleton (mechanism paint) ────────────────────── */
.wf-skeleton { background: linear-gradient(90deg, var(--color-surface) 25%, var(--color-border) 50%, var(--color-surface) 75%); background-size: 200% 100%; animation: wf-shimmer 1.5s ease-in-out infinite; }
.wf-skeleton--circle { border-radius: 50%; }
@keyframes wf-shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }

/* ─── Image ─────────────────────────────────────────── */
.wf-image { max-width: 100%; height: auto; display: block; }
.wf-image--rounded { border-radius: var(--radius-md); }
.wf-image--circle { border-radius: 50%; }

/* ─── Icon ─────────────────────────────────────────── */
.wf-icon { display: inline-flex; align-items: center; justify-content: center; width: 1em; height: 1em; vertical-align: middle; }
.wf-icon svg { width: 100%; height: 100%; }
.wf-icon--small { font-size: var(--font-size-sm); }
.wf-icon--large { font-size: var(--font-size-xl); }
.wf-icon--primary { color: var(--color-primary); }
.wf-icon--danger { color: var(--color-danger); }
.wf-icon--success { color: var(--color-success); }

/* ─── Carousel (mechanism paint on the dots) ────────── */
.wf-carousel { position: relative; overflow: hidden; }
.wf-carousel__track { display: flex; }
.wf-carousel__slide { flex: 0 0 100%; min-width: 100%; }
.wf-carousel__nav { position: absolute; bottom: var(--spacing-sm); left: 50%; transform: translateX(-50%); display: flex; gap: var(--spacing-xs); }
.wf-carousel__dot { width: 8px; height: 8px; border-radius: 50%; background: rgba(255,255,255,0.5); border: none; cursor: pointer; }
.wf-carousel__dot.active { background: #fff; }

/* ─── Typography: requested modifiers only ──────────── */
.wf-text { margin: 0; }
.wf-text--heading { font-weight: var(--font-weight-bold); font-size: var(--font-size-lg); }
.wf-text--subtitle { font-size: var(--font-size-sm); color: var(--color-text-muted); }
.wf-text--muted { color: var(--color-text-muted); }
.wf-text--bold { font-weight: var(--font-weight-bold); }
.wf-text--italic { font-style: italic; }
.wf-text--underline { text-decoration: underline; }
.wf-text--uppercase { text-transform: uppercase; }
.wf-text--lowercase { text-transform: lowercase; }
.wf-text--center { text-align: center; }
.wf-text--left { text-align: start; }
.wf-text--right { text-align: end; }
.wf-text--small { font-size: var(--font-size-sm); }
.wf-text--large { font-size: var(--font-size-lg); }
.wf-text--primary { color: var(--color-primary); }
.wf-text--danger { color: var(--color-danger); }
.wf-text--success { color: var(--color-success); }
.wf-text--warning { color: var(--color-warning); }

/* No h1–h6 scale here: the level the author wrote carries the UA's own sizing. */
.wf-heading { margin: 0; line-height: var(--line-height-tight); }
.wf-code { font-family: var(--font-family-mono); }
pre.wf-code, .wf-code--block { display: block; padding: var(--spacing-md); overflow-x: auto; white-space: pre-wrap; word-break: break-word; }
.wf-blockquote { border-inline-start: 4px solid currentColor; padding: var(--spacing-sm) var(--spacing-md); margin: 0; }

/* ─── Variant Colors (shared, requested) ────────────── */
.wf-primary { color: var(--color-primary); }
.wf-secondary { color: var(--color-secondary); }
.wf-success { color: var(--color-success); }
.wf-danger { color: var(--color-danger); }
.wf-warning { color: var(--color-warning); }
.wf-info { color: var(--color-info); }

/* ─── Responsive: reflow only, no type scale ────────── */
@media (max-width: 1024px) {
  .wf-grid { grid-template-columns: repeat(2, 1fr) !important; }
  .wf-col--4, .wf-col--3 { flex: 0 0 50%; max-width: 50%; }
}
@media (max-width: 768px) {
  .wf-row { flex-direction: column; }
  .wf-col, .wf-col--1, .wf-col--2, .wf-col--3, .wf-col--4, .wf-col--5, .wf-col--6,
  .wf-col--7, .wf-col--8, .wf-col--9, .wf-col--10, .wf-col--11, .wf-col--12 {
    flex: 0 0 100%; max-width: 100%;
  }
  .wf-grid { grid-template-columns: 1fr !important; }
  .wf-navbar { flex-wrap: wrap; padding: var(--spacing-sm); }
  .wf-navbar__links { flex-wrap: wrap; gap: var(--spacing-xs); }
  .wf-navbar__brand { width: 100%; }
  .wf-navbar__actions { width: 100%; justify-content: flex-start; }
  .wf-sidebar { display: none; }
  .wf-container { padding: 0 var(--spacing-sm); }
  .wf-table th, .wf-table td { padding: var(--spacing-xs) var(--spacing-sm); }
  .wf-btn-group { flex-wrap: wrap; }
  .wf-modal__content, .wf-dialog__content { width: 95%; }
}
@media (max-width: 480px) {
  .wf-navbar__links { display: none; }
  .wf-navbar__actions { margin-inline-start: auto; }
}

/* ─── Animation Keyframes ───────────────────────────── */
@keyframes wf-fadeIn { from { opacity: 0; } to { opacity: 1; } }
@keyframes wf-fadeOut { from { opacity: 1; } to { opacity: 0; } }
@keyframes wf-slideUp { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: none; } }
@keyframes wf-slideDown { from { opacity: 0; transform: translateY(-20px); } to { opacity: 1; transform: none; } }
@keyframes wf-slideLeft { from { opacity: 0; transform: translateX(20px); } to { opacity: 1; transform: none; } }
@keyframes wf-slideRight { from { opacity: 0; transform: translateX(-20px); } to { opacity: 1; transform: none; } }
@keyframes wf-scaleIn { from { opacity: 0; transform: scale(0.9); } to { opacity: 1; transform: none; } }
@keyframes wf-scaleOut { from { opacity: 1; transform: none; } to { opacity: 0; transform: scale(0.9); } }
@keyframes wf-bounce { 0% { opacity: 0; transform: scale(0.3); } 50% { transform: scale(1.05); } 70% { transform: scale(0.9); } 100% { opacity: 1; transform: none; } }
@keyframes wf-shake { 0%,100% { transform: none; } 10%,30%,50%,70%,90% { transform: translateX(-4px); } 20%,40%,60%,80% { transform: translateX(4px); } }
@keyframes wf-pulse { 0%,100% { transform: scale(1); } 50% { transform: scale(1.05); } }

/* ─── Animation Utility Classes ─────────────────────── */
.wf-animate-fadeIn { animation: wf-fadeIn var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-fadeOut { animation: wf-fadeOut var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideUp { animation: wf-slideUp var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideDown { animation: wf-slideDown var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideLeft { animation: wf-slideLeft var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideRight { animation: wf-slideRight var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-scaleIn { animation: wf-scaleIn var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-scaleOut { animation: wf-scaleOut var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-bounce { animation: wf-bounce var(--animation-duration-slow) var(--animation-easing-bounce) both; }
.wf-animate-shake { animation: wf-shake var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-pulse { animation: wf-pulse var(--animation-duration-slow) var(--animation-easing-default) infinite; }
.wf-animate-spin { animation: wf-spin 0.6s linear infinite; }
.wf-animate--fast { animation-duration: var(--animation-duration-fast) !important; }
.wf-animate--slow { animation-duration: var(--animation-duration-slow) !important; }

/* ─── Hover Replay: inline-animated elements replay on hover ── */
.wf-card[style*="animation"]:hover { animation-iteration-count: 1 !important; animation-play-state: running; }
.wf-anim-hover { transition: transform 0.15s ease; }
.wf-anim-hover:hover { animation-iteration-count: 1 !important; }
"#
}
