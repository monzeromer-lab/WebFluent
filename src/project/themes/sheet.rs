//! The engine's built-in stylesheet, written once for both of its forms.
//!
//! [`full`] lays every built-in out and paints it — the engine's own look,
//! what `wf build` ships unless a project asks otherwise. [`structural`] is
//! the same layout and mechanics with that look left out, for a project
//! that brings its own design (`"theme": { "builtin": "structural" }`); see
//! [`super::structural`] for where the line between the two runs.
//!
//! Most of the sheet is the same in both, and is written here once. Where
//! the two differ, a line `/*@full*/` starts what only the full sheet has,
//! `/*@structural*/` what only the structural one has, and `/*@both*/` goes
//! back to what both share. A fix to a shared rule is made once; a rule
//! that differs shows both of its forms side by side.

use std::sync::OnceLock;

/// The two sheets, interleaved by the markers above.
const SOURCE: &str = r#"
/*@full*/
/* ─── Reset ─────────────────────────────────────────── */
/*@structural*/
/* ─── Normalize ─────────────────────────────────────── */
/*@both*/
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html { scroll-behavior: smooth; }
/*@full*/
body {
  font-family: var(--font-family);
  font-size: var(--font-size-base);
  line-height: var(--line-height-normal);
  color: var(--color-text);
  background: var(--color-background);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  min-height: 100vh;
}
/*@structural*/
body { min-height: 100vh; }
/*@both*/
#app { min-height: 100vh; display: flex; flex-direction: column; }
/*@full*/
a { color: var(--color-primary); text-decoration: none; transition: color var(--transition-fast); }
a:hover { text-decoration: underline; opacity: 0.85; }
/*@structural*/
a { color: inherit; text-decoration: none; }
button, input, select, textarea { font: inherit; color: inherit; }
/*@both*/

/* ─── Layout ────────────────────────────────────────── */
.wf-container { width: 100%; max-width: 1200px; margin: 0 auto; padding: 0 var(--spacing-md); }
.wf-container--fluid { max-width: 100%; }
/* A `show` whose condition is false in the static paint. The runtime
   sets `display` on the wrapper, which beats this. */
.wf-hidden { display: none; }
.wf-visually-hidden { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }
.wf-row { --wf-gap: var(--spacing-md); display: flex; flex-wrap: wrap; gap: var(--wf-gap); }
.wf-row--center { align-items: center; }
.wf-row--between { justify-content: space-between; }
.wf-row--end { justify-content: flex-end; }
.wf-align--start { align-items: flex-start; }
.wf-align--center { align-items: center; }
.wf-align--end { align-items: flex-end; }
.wf-align--stretch { align-items: stretch; }
.wf-align--baseline { align-items: baseline; }
.wf-justify--start { justify-content: flex-start; }
.wf-justify--center { justify-content: center; }
.wf-justify--end { justify-content: flex-end; }
.wf-justify--between { justify-content: space-between; }
.wf-justify--around { justify-content: space-around; }
.wf-justify--evenly { justify-content: space-evenly; }
/* ─── Column ────────────────────────────────────────── */
.wf-col { flex: 1 1 0%; min-width: 0; }
/* A span is its share of the row less its share of the gaps, so spans
   that add to 12 sit on one line: `8` and `4` in a row with a gap
   used to come to 100% plus the gap, and the second always wrapped. */
.wf-col--1 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 1 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 1 / 12 - var(--wf-gap, 0px)); }
.wf-col--2 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 2 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 2 / 12 - var(--wf-gap, 0px)); }
.wf-col--3 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 3 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 3 / 12 - var(--wf-gap, 0px)); }
.wf-col--4 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 4 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 4 / 12 - var(--wf-gap, 0px)); }
.wf-col--5 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 5 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 5 / 12 - var(--wf-gap, 0px)); }
.wf-col--6 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 6 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 6 / 12 - var(--wf-gap, 0px)); }
.wf-col--7 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 7 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 7 / 12 - var(--wf-gap, 0px)); }
.wf-col--8 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 8 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 8 / 12 - var(--wf-gap, 0px)); }
.wf-col--9 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 9 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 9 / 12 - var(--wf-gap, 0px)); }
.wf-col--10 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 10 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 10 / 12 - var(--wf-gap, 0px)); }
.wf-col--11 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 11 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 11 / 12 - var(--wf-gap, 0px)); }
.wf-col--12 { flex: 0 0 calc((100% + var(--wf-gap, 0px)) * 12 / 12 - var(--wf-gap, 0px)); max-width: calc((100% + var(--wf-gap, 0px)) * 12 / 12 - var(--wf-gap, 0px)); }
/* ─── Grid ──────────────────────────────────────────── */
.wf-grid { display: grid; gap: var(--spacing-md); }
.wf-grid[data-cols="1"] { grid-template-columns: repeat(1, 1fr); }
.wf-grid[data-cols="2"] { grid-template-columns: repeat(2, 1fr); }
.wf-grid[data-cols="3"] { grid-template-columns: repeat(3, 1fr); }
.wf-grid[data-cols="4"] { grid-template-columns: repeat(4, 1fr); }
.wf-grid[data-cols="5"] { grid-template-columns: repeat(5, 1fr); }
.wf-grid[data-cols="6"] { grid-template-columns: repeat(6, 1fr); }
.wf-grid[data-cols="7"] { grid-template-columns: repeat(7, 1fr); }
.wf-grid[data-cols="8"] { grid-template-columns: repeat(8, 1fr); }
.wf-grid[data-cols="9"] { grid-template-columns: repeat(9, 1fr); }
.wf-grid[data-cols="10"] { grid-template-columns: repeat(10, 1fr); }
.wf-grid[data-cols="11"] { grid-template-columns: repeat(11, 1fr); }
.wf-grid[data-cols="12"] { grid-template-columns: repeat(12, 1fr); }
/* ─── Layout (stacks and gaps) ──────────────────────── */
.wf-stack { display: flex; flex-direction: column; gap: var(--spacing-md); }
/* A gap a layout names wins over its default, so it comes after them. */
.wf-gap--xs { --wf-gap: var(--spacing-xs); gap: var(--wf-gap); }
.wf-gap--sm { --wf-gap: var(--spacing-sm); gap: var(--wf-gap); }
.wf-gap--md { --wf-gap: var(--spacing-md); gap: var(--wf-gap); }
.wf-gap--lg { --wf-gap: var(--spacing-lg); gap: var(--wf-gap); }
.wf-gap--xl { --wf-gap: var(--spacing-xl); gap: var(--wf-gap); }
/* ─── Spacer ────────────────────────────────────────── */
.wf-spacer { height: var(--spacing-md); }
.wf-spacer--xs { height: var(--spacing-xs); }
.wf-spacer--sm { height: var(--spacing-sm); }
.wf-spacer--lg { height: var(--spacing-lg); }
.wf-spacer--xl { height: var(--spacing-xl); }
/* ─── Divider ───────────────────────────────────────── */
/*@structural*/
/* A Divider's whole meaning is "draw a line" — the rule is the element, not its skin. */
/*@both*/
.wf-divider { border: none; border-top: 1px solid var(--color-border); margin: var(--spacing-md) 0; }
.wf-divider--label { display: flex; align-items: center; gap: var(--spacing-md); border: none; margin: var(--spacing-md) 0; }
.wf-divider--label::before, .wf-divider--label::after { content: ""; flex: 1; border-top: 1px solid var(--color-border); }

/* ─── Navbar ────────────────────────────────────────── */
/*@full*/
.wf-navbar { display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-lg); background: color-mix(in srgb, var(--color-surface) 85%, transparent); color: var(--color-text); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); border-bottom: 1px solid var(--color-border); position: sticky; top: 0; z-index: var(--z-sticky); }
.wf-navbar__brand { font-weight: var(--font-weight-bold); font-size: var(--font-size-lg); flex-shrink: 0; }
.wf-navbar__brand a { color: var(--color-text); }
.wf-navbar__brand a:hover { text-decoration: none; }
/*@structural*/
/* Opacity is mechanics here, not decoration: the bar is `position: sticky`, so
   without a fill the page scrolls straight through it. */
.wf-navbar { display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-lg); background: var(--color-background); position: sticky; top: 0; z-index: var(--z-sticky); }
.wf-navbar__brand { flex-shrink: 0; }
/*@both*/
.wf-navbar__links { display: flex; gap: var(--spacing-sm); align-items: center; flex-wrap: wrap; overflow-x: auto; }
/*@full*/
.wf-navbar__links a { color: var(--color-text-muted); transition: color var(--transition-fast); font-size: var(--font-size-sm); padding: var(--spacing-xs) var(--spacing-sm); border-radius: var(--radius-md); white-space: nowrap; }
.wf-navbar__links a:hover { color: var(--color-primary); text-decoration: none; background: var(--color-surface); }
.wf-navbar__links a.active { color: var(--color-primary); font-weight: var(--font-weight-medium); }
/*@structural*/
.wf-navbar__links a { padding: var(--spacing-xs) var(--spacing-sm); white-space: nowrap; }
/*@both*/
.wf-navbar__actions { display: flex; gap: var(--spacing-sm); align-items: center; margin-inline-start: auto; flex-shrink: 0; }

/* ─── Sidebar ───────────────────────────────────────── */
/*@full*/
/* `top` and `height` key off --wf-header-height rather than a hardcoded 56px:
   the navbar wraps to two rows on a narrow screen, and the old constant left the
   sidebar overlapping it. */
.wf-sidebar { width: 260px; max-width: 85vw; background: var(--color-surface); border-inline-end: 1px solid var(--color-border); box-shadow: inset -1px 0 0 rgba(0,0,0,0.04); padding: var(--spacing-md); display: flex; flex-direction: column; gap: var(--spacing-xs); flex-shrink: 0; position: sticky; top: var(--wf-header-height); height: calc(100vh - var(--wf-header-height)); overflow-y: auto; }
/*@structural*/
/* The background is mechanics, not decoration: a sidebar that slides over the
   page has to be opaque or the content reads straight through it. Structural
   mode used to omit it entirely, which is why it looked transparent. */
.wf-sidebar { width: 260px; max-width: 85vw; background: var(--color-surface); padding: var(--spacing-md); display: flex; flex-direction: column; gap: var(--spacing-xs); flex-shrink: 0; position: sticky; top: var(--wf-header-height); height: calc(100vh - var(--wf-header-height)); overflow-y: auto; }
/*@both*/
/* A sidebar is a column of fixed height that scrolls: what is in it keeps
   its size and the column scrolls, instead of every item being squeezed. */
.wf-sidebar > * { flex-shrink: 0; }
/*@full*/
.wf-sidebar__header { padding: var(--spacing-sm) 0; font-weight: var(--font-weight-bold); font-size: var(--font-size-lg); }
.wf-sidebar__item { display: flex; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-sm) var(--spacing-md); border-radius: var(--radius-md); color: var(--color-text); cursor: pointer; transition: background var(--transition-fast); }
.wf-sidebar__item:hover { background: var(--color-border); text-decoration: none; }
.wf-sidebar__item.active { background: var(--color-primary); color: #fff; }
/*@structural*/
.wf-sidebar__header { padding: var(--spacing-sm) 0; }
.wf-sidebar__toggle, .wf-navbar__toggle { display: none; align-items: center; justify-content: center; width: 40px; height: 40px; cursor: pointer; }
.wf-sidebar__scrim { display: none; position: fixed; inset: 0; z-index: calc(var(--z-drawer) - 1); background: rgba(0,0,0,0.4); }
.wf-sidebar__scrim[hidden] { display: none; }
.wf-sidebar__item { display: flex; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; }
/*@both*/
.wf-sidebar__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-sm) 0; }
/*@full*/
.wf-sidebar > .wf-text { padding: var(--spacing-xs) var(--spacing-md); letter-spacing: 0.05em; font-size: var(--font-size-xs); margin-top: var(--spacing-xs); }

/* ─── Off-canvas controls ───────────────────────────── */
/* Hidden until a breakpoint asks for them, so a wide layout is unchanged. */
.wf-sidebar__toggle, .wf-navbar__toggle {
  display: none;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-background);
  color: var(--color-text);
  cursor: pointer;
  font-size: var(--font-size-lg);
  line-height: 1;
}
.wf-sidebar__scrim {
  display: none;
  position: fixed;
  inset: 0;
  z-index: calc(var(--z-drawer) - 1);
  background: rgba(0,0,0,0.4);
}
.wf-sidebar__scrim[hidden] { display: none; }
/*@structural*/
.wf-sidebar > .wf-text { padding: var(--spacing-xs) var(--spacing-md); margin-top: var(--spacing-xs); }
/*@both*/

/* ─── Breadcrumb ────────────────────────────────────── */
/*@full*/
.wf-breadcrumb { display: flex; align-items: center; gap: var(--spacing-xs); font-size: var(--font-size-sm); color: var(--color-text-muted); }
.wf-breadcrumb__item + .wf-breadcrumb__item::before { content: "/"; margin-inline-end: var(--spacing-xs); color: var(--color-border); }
.wf-breadcrumb__item:last-child { color: var(--color-text); font-weight: var(--font-weight-medium); }
/*@structural*/
.wf-breadcrumb { display: flex; align-items: center; gap: var(--spacing-xs); }
.wf-breadcrumb__item + .wf-breadcrumb__item::before { content: "/"; margin-inline-end: var(--spacing-xs); }
/*@both*/

/* ─── Link ──────────────────────────────────────────── */
/*@full*/
.wf-link { color: var(--color-primary); cursor: pointer; }
/* A bare inline link keeps its text hit area; one used as navigation gets a
   target a thumb can hit. */
.wf-navbar__links .wf-link, .wf-sidebar .wf-link { min-height: var(--target-min); display: inline-flex; align-items: center; }
.wf-link:hover { text-decoration: underline; }
.wf-link.active { font-weight: var(--font-weight-bold); }
/*@structural*/
.wf-link { cursor: pointer; }
/*@both*/

/* ─── Menu ──────────────────────────────────────────── */
.wf-menu { position: relative; display: inline-block; }
/*@full*/
.wf-menu__trigger { cursor: pointer; min-height: var(--target-min); display: inline-flex; align-items: center; }
.wf-menu__items { list-style: none; margin: 0; min-width: 180px; padding: var(--spacing-xs) 0; color: var(--color-text); background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius-md); box-shadow: var(--shadow-lg); }
/*@structural*/
.wf-menu__trigger { cursor: pointer; }
.wf-menu__items { list-style: none; margin: 0; min-width: 180px; padding: var(--spacing-xs) 0; color: var(--color-text); background: var(--color-surface); border: 1px solid var(--color-border); }
/*@both*/
.wf-menu__items:not([data-open]) { display: none; }
/*@full*/
.wf-menu__item { display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; transition: background var(--transition-fast); }
/*@structural*/
.wf-menu__item { display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; }
/*@both*/
.wf-menu__item:hover:not([aria-disabled="true"]), .wf-menu__item:focus-visible, .wf-menu__item[aria-expanded="true"] { background: var(--color-border); }
.wf-menu__item > .wf-text { margin: 0; }
.wf-menu__item:focus-visible { outline-offset: -2px; }
.wf-menu__item[aria-disabled="true"] { opacity: 0.5; cursor: default; }
.wf-menu__items:has(> [role="menuitemcheckbox"]) > .wf-menu__item::before { content: ""; width: 1em; flex: none; text-align: center; }
.wf-menu__items > .wf-menu__item[aria-checked="true"]::before { content: "\2713"; }
.wf-menu__shortcut, .wf-menu__chevron { margin-inline-start: auto; padding-inline-start: var(--spacing-lg); color: var(--color-text-muted); font: inherit; font-size: var(--font-size-xs); }
.wf-menu__item--danger { color: var(--color-danger); }
.wf-menu__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-xs) 0; }

/*@full*/
/* ─── Popover ─────────────────────────────────────── */
.wf-popover { background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); border-radius: var(--radius-md); box-shadow: var(--shadow-lg); padding: var(--spacing-sm); min-width: 160px; max-width: min(360px, calc(100vw - 16px)); overflow: visible; }
.wf-popover--arrow::before { content: ""; position: absolute; width: 10px; height: 10px; background: var(--color-surface); border: 1px solid var(--color-border); border-width: 1px 0 0 1px; }
.wf-popover--arrow[data-placement^="bottom"]::before { top: -6px; left: 16px; transform: rotate(45deg); }
.wf-popover--arrow[data-placement^="top"]::before { bottom: -6px; left: 16px; transform: rotate(225deg); }
.wf-popover--arrow[data-placement^="right"]::before { left: -6px; top: 12px; transform: rotate(-45deg); }
.wf-popover--arrow[data-placement^="left"]::before { right: -6px; top: 12px; transform: rotate(135deg); }
.wf-popover--arrow[data-placement$="-center"][data-placement^="bottom"]::before, .wf-popover--arrow[data-placement$="-center"][data-placement^="top"]::before { left: calc(50% - 5px); }
.wf-popover--arrow[data-placement$="-end"][data-placement^="bottom"]::before, .wf-popover--arrow[data-placement$="-end"][data-placement^="top"]::before { left: auto; right: 16px; }

/*@both*/
/* ─── Tabs ──────────────────────────────────────────── */
.wf-tabs { width: 100%; }
/*@full*/
.wf-tabs__nav { display: flex; border-bottom: 2px solid var(--color-border); gap: 0; }
.wf-tabs__tab { padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; border-bottom: 2px solid transparent; margin-bottom: -2px; color: var(--color-text-muted); transition: all var(--transition-fast); background: none; border-top: none; border-inline-start: none; border-inline-end: none; font-family: var(--font-family); font-size: var(--font-size-base); }
.wf-tabs__tab:hover { color: var(--color-text); }
.wf-tabs__tab.active { color: var(--color-primary); border-bottom-color: var(--color-primary); font-weight: var(--font-weight-medium); }
/*@structural*/
.wf-tabs__nav { display: flex; gap: 0; }
.wf-tabs__tab { padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; background: none; border: none; }
/*@both*/
.wf-tab-page { padding: var(--spacing-md) 0; display: none; }
.wf-tab-page.active { display: block; }

/* ─── Card ──────────────────────────────────────────── */
/*@full*/
.wf-card { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius-md); overflow: hidden; }
.wf-card--elevated { border: none; box-shadow: var(--shadow-md); }
.wf-card--outlined { background: transparent; }
.wf-card--flat { border: none; box-shadow: none; }
.wf-card__header { padding: var(--spacing-md); border-bottom: 1px solid var(--color-border); }
/*@structural*/
.wf-card { overflow: hidden; }
.wf-card__header { padding: var(--spacing-md); }
/*@both*/
.wf-card__body { padding: var(--spacing-md); }
/*@full*/
.wf-card__footer { padding: var(--spacing-md); border-top: 1px solid var(--color-border); display: flex; gap: var(--spacing-sm); justify-content: flex-end; }
/*@structural*/
.wf-card__footer { padding: var(--spacing-md); display: flex; gap: var(--spacing-sm); justify-content: flex-end; }
/*@both*/

/* ─── Table ─────────────────────────────────────────── */
.wf-table { width: 100%; border-collapse: collapse; }
/*@full*/
.wf-table th, .wf-table td { padding: var(--spacing-sm) var(--spacing-md); text-align: start; border-bottom: 1px solid var(--color-border); }
.wf-table th { font-weight: var(--font-weight-bold); color: var(--color-text-muted); font-size: var(--font-size-sm); text-transform: uppercase; letter-spacing: 0.05em; background: var(--color-surface); }
.wf-table tr:hover td { background: var(--color-surface); }
/*@structural*/
.wf-table th, .wf-table td { padding: var(--spacing-sm) var(--spacing-md); text-align: start; }
/*@both*/

/* ─── List ──────────────────────────────────────────── */
.wf-list { list-style: none; }
ol.wf-list { list-style: decimal; padding-left: var(--spacing-lg); }
/*@full*/
.wf-list__item { padding: var(--spacing-sm) var(--spacing-md); border-bottom: 1px solid var(--color-border); display: flex; align-items: center; gap: var(--spacing-sm); }
.wf-list__item:last-child { border-bottom: none; }
/*@structural*/
.wf-list__item { padding: var(--spacing-sm) var(--spacing-md); display: flex; align-items: center; gap: var(--spacing-sm); }
/*@both*/

/* ─── Badge ─────────────────────────────────────────── */
/*@full*/
.wf-badge { display: inline-flex; align-items: center; padding: 0.125rem var(--spacing-sm); font-size: var(--font-size-xs); font-weight: var(--font-weight-medium); border-radius: var(--radius-full); background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.wf-badge--primary { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }
.wf-badge--success { background: var(--color-success); color: #fff; border-color: var(--color-success); }
.wf-badge--danger { background: var(--color-danger); color: #fff; border-color: var(--color-danger); }
.wf-badge--warning { background: var(--color-warning); color: #000; border-color: var(--color-warning); }
.wf-badge--info { background: var(--color-info); color: #fff; border-color: var(--color-info); }
.wf-badge--secondary { background: var(--color-secondary); color: #fff; border-color: var(--color-secondary); }
/*@structural*/
.wf-badge { display: inline-flex; align-items: center; padding: 0.125rem var(--spacing-sm); }
.wf-badge--primary { background: var(--color-primary); color: #fff; }
.wf-badge--success { background: var(--color-success); color: #fff; }
.wf-badge--danger { background: var(--color-danger); color: #fff; }
.wf-badge--warning { background: var(--color-warning); color: #000; }
.wf-badge--info { background: var(--color-info); color: #fff; }
.wf-badge--secondary { background: var(--color-secondary); color: #fff; }
/*@both*/
.wf-badge--pill { border-radius: var(--radius-full); }

/* ─── Avatar ────────────────────────────────────────── */
/*@full*/
.wf-avatar { width: 40px; height: 40px; border-radius: var(--radius-full); overflow: hidden; display: inline-flex; align-items: center; justify-content: center; background: var(--color-primary); color: #fff; font-weight: var(--font-weight-bold); font-size: var(--font-size-sm); }
/*@structural*/
.wf-avatar { width: 40px; height: 40px; overflow: hidden; display: inline-flex; align-items: center; justify-content: center; }
/*@both*/
.wf-avatar img { width: 100%; height: 100%; object-fit: cover; }
.wf-avatar--primary { background: var(--color-primary); color: #fff; }
/*@full*/
.wf-avatar--small { width: 32px; height: 32px; font-size: var(--font-size-xs); }
.wf-avatar--large { width: 56px; height: 56px; font-size: var(--font-size-lg); }
/*@structural*/
.wf-avatar--small { width: 32px; height: 32px; }
.wf-avatar--large { width: 56px; height: 56px; }
/*@both*/

/* ─── Tooltip ───────────────────────────────────────── */
.wf-tooltip { display: inline-block; }
/*@full*/
.wf-tooltip__text { display: inline-flex; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-xs) var(--spacing-sm); border: 0; background: var(--color-text); color: var(--color-background); white-space: nowrap; pointer-events: none; border-radius: var(--radius-sm); font-size: var(--font-size-xs); }
/*@structural*/
.wf-tooltip__text { display: inline-flex; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-xs) var(--spacing-sm); border: 0; background: var(--color-text); color: var(--color-background); white-space: nowrap; pointer-events: none; }
/*@both*/
.wf-tooltip__text:not([data-open]) { display: none; }
.wf-tooltip__shortcut { font: inherit; padding: 0 var(--spacing-xs); border-radius: var(--radius-sm); background: color-mix(in srgb, var(--color-background) 20%, transparent); }

/* ─── Tag ───────────────────────────────────────────── */
/*@full*/
.wf-tag { display: inline-flex; align-items: center; gap: var(--spacing-xs); padding: 0.125rem var(--spacing-sm); font-size: var(--font-size-sm); border-radius: var(--radius-md); background: var(--color-surface); border: 1px solid var(--color-border); }
.wf-tag__remove { cursor: pointer; opacity: 0.6; font-size: var(--font-size-xs); min-width: var(--target-min); min-height: var(--target-min); display: inline-flex; align-items: center; justify-content: center; }
.wf-tag__remove:hover { opacity: 1; }
/*@structural*/
.wf-tag { display: inline-flex; align-items: center; gap: var(--spacing-xs); padding: 0.125rem var(--spacing-sm); }
.wf-tag__remove { cursor: pointer; }
/*@both*/

/* ─── Button ────────────────────────────────────────── */
/*@full*/
.wf-btn { display: inline-flex; align-items: center; justify-content: center; gap: var(--spacing-xs); padding: var(--spacing-sm) var(--spacing-md); font-family: var(--font-family); font-size: var(--font-size-base); font-weight: var(--font-weight-medium); border: 1px solid var(--color-border); border-radius: var(--radius-md); background: var(--color-surface); color: var(--color-text); cursor: pointer; transition: all var(--transition-fast); line-height: 1; }
.wf-btn:hover { background: var(--color-border); }
.wf-btn:active { transform: scale(0.98); }
.wf-btn--primary { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }
.wf-btn--primary:hover { filter: brightness(1.1); background: var(--color-primary); }
.wf-btn--secondary { background: var(--color-secondary); color: #fff; border-color: var(--color-secondary); }
.wf-btn--success { background: var(--color-success); color: #fff; border-color: var(--color-success); }
.wf-btn--danger { background: var(--color-danger); color: #fff; border-color: var(--color-danger); }
.wf-btn--warning { background: var(--color-warning); color: #000; border-color: var(--color-warning); }
.wf-btn--info { background: var(--color-info); color: #fff; border-color: var(--color-info); }
.wf-btn--small { padding: var(--spacing-xs) var(--spacing-sm); font-size: var(--font-size-sm); }
.wf-btn--large { padding: var(--spacing-md) var(--spacing-lg); font-size: var(--font-size-lg); }
/*@structural*/
.wf-btn { display: inline-flex; align-items: center; justify-content: center; gap: var(--spacing-xs); padding: var(--spacing-sm) var(--spacing-md); background: none; border: none; cursor: pointer; line-height: 1; }
.wf-btn--primary { background: var(--color-primary); color: #fff; }
.wf-btn--secondary { background: var(--color-secondary); color: #fff; }
.wf-btn--success { background: var(--color-success); color: #fff; }
.wf-btn--danger { background: var(--color-danger); color: #fff; }
.wf-btn--warning { background: var(--color-warning); color: #000; }
.wf-btn--info { background: var(--color-info); color: #fff; }
.wf-btn--small { padding: var(--spacing-xs) var(--spacing-sm); }
.wf-btn--large { padding: var(--spacing-md) var(--spacing-lg); }
/*@both*/
.wf-btn--full { width: 100%; }
.wf-btn--rounded { border-radius: var(--radius-full); }
.wf-btn--pill { border-radius: var(--radius-full); }
/*@full*/
.wf-btn--outlined { background: transparent; }
/*@structural*/
.wf-btn--outlined { background: transparent; border: 1px solid currentColor; }
/*@both*/
.wf-btn--outlined.wf-btn--primary { color: var(--color-primary); background: transparent; }
.wf-btn--outlined.wf-btn--danger { color: var(--color-danger); background: transparent; }
/* Busy: its work is under way (`loading:`, or a handler's action still
   running). Past a moment — so a quick save does not flicker — its label
   gives way to a spinner, and the button keeps its width. */
.wf-btn[aria-busy="true"] { position: relative; cursor: progress; animation: wf-busy-label 0s linear 300ms forwards; }
.wf-btn[aria-busy="true"]::after { content: ""; position: absolute; inset: 0; margin: auto; width: 1em; height: 1em; box-sizing: border-box; border: 2px solid currentColor; border-inline-end-color: transparent; border-radius: 50%; opacity: 0; animation: wf-busy-show 0s linear 300ms forwards, wf-busy-spin 0.6s linear infinite; }
@keyframes wf-busy-label { to { -webkit-text-fill-color: transparent; } }
@keyframes wf-busy-show { to { opacity: 1; } }
@keyframes wf-busy-spin { to { transform: rotate(360deg); } }
.wf-form[aria-busy="true"] { cursor: progress; }

/* ─── Icon Button ───────────────────────────────────── */
/*@full*/
.wf-icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 36px; height: 36px; padding: 0; border: none; border-radius: var(--radius-md); background: transparent; color: var(--color-text); cursor: pointer; transition: background var(--transition-fast); font-size: var(--font-size-lg); }
.wf-icon-btn:hover { background: var(--color-surface); }
.wf-icon-btn--small { width: 28px; height: 28px; font-size: var(--font-size-base); }
.wf-icon-btn--large { width: 48px; height: 48px; font-size: var(--font-size-xl); }
/*@structural*/
.wf-icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 36px; height: 36px; padding: 0; background: transparent; border: none; cursor: pointer; }
.wf-icon-btn--small { width: 28px; height: 28px; }
.wf-icon-btn--large { width: 48px; height: 48px; }
/*@both*/
.wf-icon-btn--primary { color: var(--color-primary); }
.wf-icon-btn--danger { color: var(--color-danger); }

/* ─── Button Group ──────────────────────────────────── */
.wf-btn-group { display: inline-flex; }
/*@full*/
.wf-btn-group .wf-btn { border-radius: 0; margin-inline-start: -1px; }
.wf-btn-group .wf-btn:first-child { border-radius: var(--radius-md) 0 0 var(--radius-md); margin-inline-start: 0; }
.wf-btn-group .wf-btn:last-child { border-radius: 0 var(--radius-md) var(--radius-md) 0; }
/*@structural*/
.wf-btn-group .wf-btn { margin-inline-start: -1px; }
.wf-btn-group .wf-btn:first-child { margin-inline-start: 0; }
/*@both*/

/* ─── Dropdown ──────────────────────────────────────── */
.wf-dropdown { position: relative; display: inline-block; }
/*@full*/
.wf-dropdown__items { list-style: none; margin: 0; min-width: 180px; padding: var(--spacing-xs) 0; color: var(--color-text); background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius-md); box-shadow: var(--shadow-lg); }
/*@structural*/
.wf-dropdown__items { list-style: none; margin: 0; min-width: 180px; padding: var(--spacing-xs) 0; color: var(--color-text); background: var(--color-surface); border: 1px solid var(--color-border); }
/*@both*/
.wf-dropdown__items:not([data-open]) { display: none; }
/*@full*/
.wf-dropdown__item { display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; transition: background var(--transition-fast); }
/*@structural*/
.wf-dropdown__item { display: flex; align-items: center; gap: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; }
/*@both*/
.wf-dropdown__item:hover:not([aria-disabled="true"]), .wf-dropdown__item:focus-visible, .wf-dropdown__item[aria-expanded="true"] { background: var(--color-border); }
.wf-dropdown__item > .wf-text { margin: 0; }
.wf-dropdown__item:focus-visible { outline-offset: -2px; }
.wf-dropdown__item[aria-disabled="true"] { opacity: 0.5; cursor: default; }
.wf-dropdown__items:has(> [role="menuitemcheckbox"]) > .wf-dropdown__item::before { content: ""; width: 1em; flex: none; text-align: center; }
.wf-dropdown__items > .wf-dropdown__item[aria-checked="true"]::before { content: "\2713"; }
.wf-dropdown__shortcut, .wf-dropdown__chevron { margin-inline-start: auto; padding-inline-start: var(--spacing-lg); color: var(--color-text-muted); font: inherit; font-size: var(--font-size-xs); }
.wf-dropdown__item--danger { color: var(--color-danger); }
.wf-dropdown__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-xs) 0; }

/* ─── Input ─────────────────────────────────────────── */
.wf-field { display: flex; flex-direction: column; gap: var(--spacing-xs); }
.wf-field .wf-label { font-weight: var(--font-weight-medium); }
.wf-field__hint { margin: 0; font-size: var(--font-size-sm); color: var(--color-text-muted); }
.wf-field__error { margin: 0; font-size: var(--font-size-sm); color: var(--color-danger); }
.wf-field__error[hidden] { display: none; }
.wf-input[aria-invalid="true"], .wf-select[aria-invalid="true"] { border-color: var(--color-danger); }
/*@full*/
.wf-input { display: block; width: 100%; padding: var(--spacing-sm) var(--spacing-md); font-family: var(--font-family); font-size: var(--font-size-base); color: var(--color-text); background: var(--color-background); border: 1px solid var(--color-border); border-radius: var(--radius-md); transition: border-color var(--transition-fast), box-shadow var(--transition-fast); }
.wf-input:focus-visible { border-color: var(--color-primary); box-shadow: 0 0 0 3px rgba(59,130,246,0.15); }
.wf-input::placeholder { color: var(--color-text-muted); }
.wf-textarea { min-height: 6rem; resize: vertical; }
.wf-field__count { display: block; margin-top: var(--spacing-xs); font-size: var(--font-size-sm); color: var(--color-text-muted); text-align: right; }
.wf-input--small { padding: var(--spacing-xs) var(--spacing-sm); font-size: var(--font-size-sm); }
.wf-input--large { padding: var(--spacing-md); font-size: var(--font-size-lg); }
/*@structural*/
.wf-input { display: block; width: 100%; padding: var(--spacing-sm) var(--spacing-md); }
.wf-input--small { padding: var(--spacing-xs) var(--spacing-sm); }
.wf-input--large { padding: var(--spacing-md); }
/*@both*/
.wf-input--rounded { border-radius: var(--radius-full); }
.wf-input--full { width: 100%; }

/* ─── Select ────────────────────────────────────────── */
/*@full*/
.wf-select { display: block; width: 100%; padding: var(--spacing-sm) var(--spacing-md); font-family: var(--font-family); font-size: var(--font-size-base); color: var(--color-text); background: var(--color-background); border: 1px solid var(--color-border); border-radius: var(--radius-md); appearance: none; cursor: pointer; }
.wf-select:focus-visible { border-color: var(--color-primary); box-shadow: 0 0 0 3px rgba(59,130,246,0.15); }
/*@structural*/
.wf-select { display: block; width: 100%; padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; }
/*@both*/

/* ─── Checkbox ──────────────────────────────────────── */
/*@full*/
.wf-checkbox { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; font-size: var(--font-size-base); min-height: var(--target-min); padding: var(--spacing-xs) 0; }
.wf-checkbox input[type="checkbox"] { width: 20px; height: 20px; accent-color: var(--color-primary); cursor: pointer; }
/*@structural*/
.wf-checkbox { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; }
.wf-checkbox input[type="checkbox"] { width: 18px; height: 18px; cursor: pointer; }
/*@both*/

/* ─── Radio ─────────────────────────────────────────── */
/*@full*/
.wf-radio { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; font-size: var(--font-size-base); min-height: var(--target-min); padding: var(--spacing-xs) 0; }
.wf-radio input[type="radio"] { width: 20px; height: 20px; accent-color: var(--color-primary); cursor: pointer; }
/*@structural*/
.wf-radio { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; }
.wf-radio input[type="radio"] { width: 18px; height: 18px; cursor: pointer; }
/*@both*/

/*@full*/
/* ─── Switch ────────────────────────────────────────── */
.wf-switch { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; position: relative; min-height: var(--target-min); }
.wf-switch__track { width: 44px; height: 24px; background: var(--color-border); border-radius: var(--radius-full); position: relative; transition: background var(--transition-fast); }
.wf-switch__thumb { width: 20px; height: 20px; background: #fff; border-radius: var(--radius-full); position: absolute; top: 2px; inset-inline-start: 2px; transition: transform var(--transition-fast); box-shadow: var(--shadow-sm); }
/*@structural*/
/* ─── Switch (mechanism paint) ──────────────────────── */
.wf-switch { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; position: relative; }
.wf-switch__track { width: 44px; height: 24px; position: relative; background: var(--color-border); border-radius: var(--radius-full); }
.wf-switch__thumb { width: 20px; height: 20px; position: absolute; top: 2px; inset-inline-start: 2px; background: #fff; border-radius: var(--radius-full); }
/*@both*/
.wf-switch input:checked + .wf-switch__track { background: var(--color-primary); }
.wf-switch input:checked + .wf-switch__track .wf-switch__thumb { transform: translateX(20px); }
/* The input is the switch to a keyboard and a screen reader (`role="switch"`,
   `aria-checked`): laid over the whole control, unseen, never `display: none`. */
.wf-switch input { position: absolute; inset: 0; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: pointer; z-index: 1; }
.wf-switch input:focus-visible + .wf-switch__track { outline: 2px solid var(--color-primary); outline-offset: 2px; }
.wf-switch:dir(rtl) input:checked + .wf-switch__track .wf-switch__thumb { transform: translateX(-20px); }

/* ─── Slider ────────────────────────────────────────── */
.wf-slider { display: flex; align-items: center; gap: var(--spacing-sm); flex-wrap: wrap; }
/*@full*/
.wf-slider input[type="range"] { flex: 1; accent-color: var(--color-primary); min-width: 120px; }
.wf-slider__value { font-size: var(--font-size-sm); color: var(--color-text-muted); min-width: 2em; text-align: end; }
.wf-form-label { font-size: var(--font-size-sm); font-weight: var(--font-weight-medium); color: var(--color-text); margin-bottom: var(--spacing-xs); display: block; }
/*@structural*/
.wf-slider input[type="range"] { flex: 1; min-width: 120px; accent-color: var(--color-primary); }
.wf-slider__value { min-width: 2em; text-align: end; }
.wf-form-label { display: block; margin-bottom: var(--spacing-xs); }
/*@both*/
.wf-form-group { display: flex; flex-direction: column; gap: var(--spacing-xs); }
/*@full*/

/* ─── Date Picker ───────────────────────────────────── */
/*@both*/
.wf-datepicker { display: flex; flex-direction: column; gap: var(--spacing-xs); }

/* ─── File Upload ──────────────────────────────────── */
.wf-file-upload { display: flex; flex-direction: column; gap: var(--spacing-xs); }
/*@full*/
.wf-file-upload input[type="file"] { padding: var(--spacing-sm); border: 2px dashed var(--color-border); border-radius: var(--radius-md); background: var(--color-surface); cursor: pointer; transition: border-color var(--transition-fast); }
.wf-file-upload input[type="file"]:hover { border-color: var(--color-primary); }
.wf-file-upload input[type="file"]::file-selector-button { padding: var(--spacing-xs) var(--spacing-md); border: 1px solid var(--color-border); border-radius: var(--radius-sm); background: var(--color-background); cursor: pointer; margin-inline-end: var(--spacing-sm); }
/*@structural*/
.wf-file-upload input[type="file"] { padding: var(--spacing-sm); cursor: pointer; }
.wf-file-upload input[type="file"]::file-selector-button { padding: var(--spacing-xs) var(--spacing-md); cursor: pointer; margin-inline-end: var(--spacing-sm); }
/*@both*/

/* ─── Form ──────────────────────────────────────────── */
.wf-form { display: flex; flex-direction: column; gap: var(--spacing-md); }

/* ─── Alert ─────────────────────────────────────────── */
/*@full*/
.wf-alert { padding: var(--spacing-md); border-radius: var(--radius-md); border: 1px solid var(--color-border); background: var(--color-surface); display: flex; align-items: center; gap: var(--spacing-sm); }
.wf-alert--success { background: #f0fdf4; border-color: var(--color-success); color: #166534; }
.wf-alert--danger { background: #fef2f2; border-color: var(--color-danger); color: #991b1b; }
.wf-alert--warning { background: #fffbeb; border-color: var(--color-warning); color: #92400e; }
.wf-alert--info { background: #ecfeff; border-color: var(--color-info); color: #155e75; }
.wf-alert__dismiss { margin-inline-start: auto; cursor: pointer; opacity: 0.6; background: none; border: none; font-size: var(--font-size-lg); color: inherit; min-width: var(--target-min); min-height: var(--target-min); }
.wf-alert__dismiss:hover { opacity: 1; }
/*@structural*/
.wf-alert { padding: var(--spacing-md); display: flex; align-items: center; gap: var(--spacing-sm); }
.wf-alert--success { color: var(--color-success); }
.wf-alert--danger { color: var(--color-danger); }
.wf-alert--warning { color: var(--color-warning); }
.wf-alert--info { color: var(--color-info); }
.wf-alert__dismiss { margin-inline-start: auto; cursor: pointer; background: none; border: none; color: inherit; }
/*@both*/

/*@full*/
/* ─── Toast ─────────────────────────────────────────── */
/*@structural*/
/* ─── Toast (overlay) ───────────────────────────────── */
/*@both*/
.wf-toast-container { position: fixed; top: var(--spacing-md); inset-inline-end: var(--spacing-md); z-index: var(--z-toast); display: flex; flex-direction: column; gap: var(--spacing-sm); }
/* Drawn in the top layer as a popover: none of the box a popover is given. */
.wf-toast-container[popover] { inset: auto; top: var(--spacing-md); inset-inline-end: var(--spacing-md); margin: 0; padding: 0; border: 0; background: transparent; color: inherit; overflow: visible; width: auto; height: auto; }
.wf-toast-container[popover]:not(:popover-open) { display: none; }
/*@full*/
.wf-toast { padding: var(--spacing-sm) var(--spacing-md); border-radius: var(--radius-md); color: #fff; font-size: var(--font-size-sm); box-shadow: var(--shadow-lg); animation: wf-toast-in 0.3s ease; min-width: 200px; }
.wf-toast--success { background: var(--color-success); }
.wf-toast--danger { background: var(--color-danger); }
/*@structural*/
.wf-toast { padding: var(--spacing-sm) var(--spacing-md); min-width: 200px; background: var(--color-surface); color: var(--color-text); animation: wf-toast-in 0.3s ease; }
.wf-toast--success { background: var(--color-success); color: #fff; }
.wf-toast--danger { background: var(--color-danger); color: #fff; }
/*@both*/
.wf-toast--warning { background: var(--color-warning); color: #000; }
/*@full*/
.wf-toast--info { background: var(--color-info); }
/*@structural*/
.wf-toast--info { background: var(--color-info); color: #fff; }
/*@both*/
.wf-toast--exit { animation: wf-toast-out 0.3s ease forwards; }
.wf-toast { display: flex; align-items: center; gap: var(--spacing-sm); }
.wf-toast__text { flex: 1; }
.wf-toast__action, .wf-toast__close { flex-shrink: 0; min-height: var(--target-min); min-width: var(--target-min); border: 0; background: none; color: inherit; font: inherit; cursor: pointer; padding: 0 var(--spacing-xs); }
.wf-toast__action { font-weight: var(--font-weight-bold); text-decoration: underline; }
.wf-toast__close { opacity: 0.8; font-size: var(--font-size-lg); line-height: 1; }
.wf-toast__close:hover { opacity: 1; }
.wf-toast { --wf-toast-from: 100%; }
.wf-toast:dir(rtl) { --wf-toast-from: -100%; }
@keyframes wf-toast-in { from { transform: translateX(var(--wf-toast-from)); opacity: 0; } to { transform: none; opacity: 1; } }
@keyframes wf-toast-out { from { opacity: 1; } to { opacity: 0; transform: translateX(var(--wf-toast-from)); } }

/*@full*/
/* ─── Modal ─────────────────────────────────────────── */
/* A native <dialog>: the browser centres it (with the `margin: auto` the reset
   took away), renders the backdrop, traps focus
   and hides it while closed, so none of that is done here. */
/*@structural*/
/* ─── Modal (overlay) ───────────────────────────────── */
/*@both*/
.wf-modal { margin: auto; border: none; padding: 0; background: transparent; max-width: 90vw; max-height: 90vh; overflow: visible; }
.wf-modal::backdrop { background: rgba(0,0,0,0.5); }
/* It comes in and goes out: the browser keeps a closing dialog in the top
   layer, and shown, until its transition has played (`allow-discrete`), so
   Escape, the backdrop and its state all play the same exit. */
.wf-modal, .wf-modal::backdrop { transition: opacity var(--animation-duration-fast) var(--ease-standard), transform var(--animation-duration-fast) var(--ease-standard), overlay var(--animation-duration-fast) allow-discrete, display var(--animation-duration-fast) allow-discrete; }
.wf-modal:not([open]), .wf-modal:not([open])::backdrop { opacity: 0; }
.wf-modal:not([open]) { transform: scale(0.97); }
@starting-style { .wf-modal[open], .wf-modal[open]::backdrop { opacity: 0; } .wf-modal[open] { transform: scale(0.97); } }
/*@full*/
.wf-modal__content { background: var(--color-background); border-radius: var(--radius-lg); box-shadow: var(--shadow-xl); max-width: 500px; width: 90vw; max-height: 90vh; overflow-y: auto; }
.wf-modal__header { padding: var(--spacing-md); border-bottom: 1px solid var(--color-border); display: flex; justify-content: space-between; align-items: center; }
/*@structural*/
.wf-modal__content { background: var(--color-background); max-width: 500px; width: 90%; max-height: 90vh; overflow-y: auto; }
.wf-modal__header { padding: var(--spacing-md); display: flex; justify-content: space-between; align-items: center; }
/*@both*/
.wf-modal__header h3 { margin: 0; }
.wf-modal__close, .wf-dialog__close { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; padding: 0; border: 0; border-radius: var(--radius-sm, 4px); background: none; color: inherit; cursor: pointer; opacity: 0.7; }
.wf-modal__close:hover, .wf-dialog__close:hover { opacity: 1; }
.wf-modal__body { padding: var(--spacing-md); }
/*@full*/
.wf-modal__footer { padding: var(--spacing-md); border-top: 1px solid var(--color-border); display: flex; justify-content: flex-end; gap: var(--spacing-sm); }
/*@structural*/
.wf-modal__footer { padding: var(--spacing-md); display: flex; justify-content: flex-end; gap: var(--spacing-sm); }
/*@both*/

/*@full*/
/* ─── Dialog ────────────────────────────────────────── */
/*@structural*/
/* ─── Dialog (overlay) ──────────────────────────────── */
/*@both*/
.wf-dialog { margin: auto; border: none; padding: 0; background: transparent; max-width: 90vw; max-height: 90vh; overflow: visible; }
.wf-dialog::backdrop { background: rgba(0,0,0,0.5); }
.wf-dialog, .wf-dialog::backdrop { transition: opacity var(--animation-duration-fast) var(--ease-standard), transform var(--animation-duration-fast) var(--ease-standard), overlay var(--animation-duration-fast) allow-discrete, display var(--animation-duration-fast) allow-discrete; }
.wf-dialog:not([open]), .wf-dialog:not([open])::backdrop { opacity: 0; }
.wf-dialog:not([open]) { transform: scale(0.97); }
@starting-style { .wf-dialog[open], .wf-dialog[open]::backdrop { opacity: 0; } .wf-dialog[open] { transform: scale(0.97); } }
/*@full*/
.wf-dialog__content { background: var(--color-background); border-radius: var(--radius-lg); box-shadow: var(--shadow-xl); padding: var(--spacing-lg); max-width: 400px; width: 90vw; display: flex; flex-direction: column; gap: var(--spacing-md); }
/*@structural*/
.wf-dialog__content { background: var(--color-background); padding: var(--spacing-lg); max-width: 400px; width: 90%; display: flex; flex-direction: column; gap: var(--spacing-md); }
/*@both*/
.wf-dialog__header { display: flex; justify-content: space-between; align-items: center; gap: var(--spacing-sm); }
.wf-dialog__header h3 { margin: 0; }
.wf-dialog__detail { margin: 0; }
.wf-dialog__footer { display: flex; justify-content: flex-end; flex-wrap: wrap; gap: var(--spacing-sm); }

/*@full*/
/* ─── Sheet ─────────────────────────────────────────── */
/* A modal <dialog> drawn against an edge: the bottom, or the start or end
   of the line. `--wf-sheet-drag` is how far a finger has pulled it toward
   that edge. */
/*@structural*/
/* ─── Sheet (overlay) ───────────────────────────────── */
/*@both*/
.wf-sheet { position: fixed; border: none; padding: 0; margin: 0; max-width: 100vw; max-height: 100dvh; overflow: visible; background: transparent; display: flex; flex-direction: column; --wf-sheet-drag: 0px; }
.wf-sheet:not([open]) { display: none; }
.wf-sheet::backdrop { background: rgba(0,0,0,0.5); }
.wf-sheet[data-side="bottom"] { inset: auto 0 0 0; width: 100%; max-height: 90dvh; transform: translateY(var(--wf-sheet-drag)); }
.wf-sheet[data-side="start"] { inset-block: 0; inset-inline-start: 0; inset-inline-end: auto; height: 100dvh; width: min(360px, 85vw); }
.wf-sheet[data-side="end"] { inset-block: 0; inset-inline-end: 0; inset-inline-start: auto; height: 100dvh; width: min(360px, 85vw); }
.wf-sheet[data-side="start"] { transform: translateX(calc(-1 * var(--wf-sheet-drag))); }
.wf-sheet[data-side="end"] { transform: translateX(var(--wf-sheet-drag)); }
.wf-sheet[data-side="start"]:dir(rtl) { transform: translateX(var(--wf-sheet-drag)); }
.wf-sheet[data-side="end"]:dir(rtl) { transform: translateX(calc(-1 * var(--wf-sheet-drag))); }
.wf-sheet, .wf-sheet::backdrop { transition: transform var(--transition-normal), height var(--transition-normal), opacity var(--transition-normal), overlay var(--transition-normal) allow-discrete, display var(--transition-normal) allow-discrete; }
.wf-sheet[data-dragging] { transition: none; }
.wf-sheet:not([open])::backdrop { opacity: 0; }
.wf-sheet[data-side="bottom"]:not([open]) { transform: translateY(100%); }
.wf-sheet[data-side="start"]:not([open]), .wf-sheet[data-side="end"]:dir(rtl):not([open]) { transform: translateX(-100%); }
.wf-sheet[data-side="end"]:not([open]), .wf-sheet[data-side="start"]:dir(rtl):not([open]) { transform: translateX(100%); }
@starting-style {
  .wf-sheet[open]::backdrop { opacity: 0; }
  .wf-sheet[data-side="bottom"][open] { transform: translateY(100%); }
  .wf-sheet[data-side="start"][open], .wf-sheet[data-side="end"]:dir(rtl)[open] { transform: translateX(-100%); }
  .wf-sheet[data-side="end"][open], .wf-sheet[data-side="start"]:dir(rtl)[open] { transform: translateX(100%); }
}
.wf-sheet__content { display: flex; flex-direction: column; flex: 1; min-height: 0; background: var(--color-background); color: var(--color-text); overflow: hidden; }
.wf-sheet__handle { align-self: center; width: 44px; min-height: 24px; display: flex; align-items: center; justify-content: center; cursor: grab; touch-action: none; flex-shrink: 0; }
.wf-sheet__handle::before { content: ""; width: 36px; height: 4px; border-radius: 2px; background: var(--color-border); }
.wf-sheet__header { display: flex; justify-content: space-between; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-md); touch-action: none; flex-shrink: 0; }
.wf-sheet__header h3 { margin: 0; }
.wf-sheet__close { display: inline-flex; align-items: center; justify-content: center; min-width: var(--target-min); min-height: var(--target-min); padding: 0; border: 0; background: none; color: inherit; cursor: pointer; }
.wf-sheet__body { padding: 0 var(--spacing-md) var(--spacing-md); overflow-y: auto; flex: 1; min-height: 0; }
.wf-sheet__footer { padding: var(--spacing-md); display: flex; justify-content: flex-end; gap: var(--spacing-sm); flex-shrink: 0; padding-bottom: max(var(--spacing-md), env(safe-area-inset-bottom)); }
/*@full*/
.wf-sheet__content { box-shadow: var(--shadow-xl); }
.wf-sheet[data-side="bottom"] .wf-sheet__content { border-radius: var(--radius-lg) var(--radius-lg) 0 0; }
.wf-sheet__footer { border-top: 1px solid var(--color-border); }
/*@both*/

/*@full*/
/* ─── Spinner ───────────────────────────────────────── */
/*@structural*/
/* ─── Spinner (mechanism paint) ─────────────────────── */
/*@both*/
.wf-spinner { width: 24px; height: 24px; border: 3px solid var(--color-border); border-top-color: var(--color-primary); border-radius: 50%; animation: wf-spin 0.6s linear infinite; display: inline-block; }
.wf-spinner--large { width: 40px; height: 40px; border-width: 4px; }
.wf-spinner--small { width: 16px; height: 16px; border-width: 2px; }
@keyframes wf-spin { to { transform: rotate(360deg); } }

/*@full*/
/* ─── Progress ──────────────────────────────────────── */
.wf-progress { width: 100%; height: 8px; background: var(--color-border); border-radius: var(--radius-full); overflow: hidden; appearance: none; }
/*@structural*/
/* ─── Progress (mechanism paint) ────────────────────── */
.wf-progress { width: 100%; height: 8px; background: var(--color-border); border-radius: var(--radius-full); overflow: hidden; }
/*@both*/
.wf-progress::-webkit-progress-bar { background: var(--color-border); border-radius: var(--radius-full); }
/*@full*/
.wf-progress::-webkit-progress-value { background: var(--color-primary); border-radius: var(--radius-full); transition: width var(--transition-normal); }
/*@structural*/
.wf-progress::-webkit-progress-value { background: var(--color-primary); border-radius: var(--radius-full); }
/*@both*/
.wf-progress::-moz-progress-bar { background: var(--color-primary); border-radius: var(--radius-full); }

/*@full*/
/* ─── Skeleton ──────────────────────────────────────── */
.wf-skeleton { background: linear-gradient(90deg, var(--color-surface) 25%, var(--color-border) 50%, var(--color-surface) 75%); background-size: 200% 100%; animation: wf-shimmer 1.5s ease-in-out infinite; border-radius: var(--radius-md); }
/*@structural*/
/* ─── Skeleton (mechanism paint) ────────────────────── */
.wf-skeleton { background: linear-gradient(90deg, var(--color-surface) 25%, var(--color-border) 50%, var(--color-surface) 75%); background-size: 200% 100%; animation: wf-shimmer 1.5s ease-in-out infinite; }
/*@both*/
.wf-skeleton--circle { border-radius: 50%; }
@keyframes wf-shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }

/* ─── Image ─────────────────────────────────────────── */
.wf-image { max-width: 100%; height: auto; display: block; }
.wf-image--rounded { border-radius: var(--radius-md); }
.wf-image--circle { border-radius: 50%; }

/* ─── Frame ─────────────────────────────────────────── */
.wf-frame { display: block; width: 100%; border: 0; }

/* ─── Virtual List ──────────────────────────────────── */
.wf-virtual-list { display: block; overflow-y: auto; overflow-anchor: none; contain: content; min-height: 0; }
.wf-virtual-list__space { flex: none; }

/* ─── Portal ────────────────────────────────────────── */
.wf-portal { display: contents; }

/* ─── Splitter ──────────────────────────────────────── */
/*@full*/
.wf-splitter { flex: none; align-self: stretch; position: relative; width: 4px; cursor: col-resize; touch-action: none; background: transparent; transition: background var(--transition-fast); }
/*@structural*/
.wf-splitter { flex: none; align-self: stretch; position: relative; width: 4px; cursor: col-resize; touch-action: none; background: transparent; }
/*@both*/
/* A 4px line that reaches as far as a target must (`target-min`). */
.wf-splitter::before { content: ""; position: absolute; inset: 0 calc((4px - var(--target-min)) / 2); }
.wf-splitter--horizontal { width: auto; height: 4px; cursor: row-resize; }
.wf-splitter--horizontal::before { inset: calc((4px - var(--target-min)) / 2) 0; }
.wf-splitter:hover, .wf-splitter:focus-visible { background: var(--color-border); }
.wf-splitter[data-dragging] { background: var(--color-primary); }
:root[data-wf-resizing] { user-select: none; }
:root[data-wf-resizing="x"], :root[data-wf-resizing="x"] * { cursor: col-resize !important; }
:root[data-wf-resizing="y"], :root[data-wf-resizing="y"] * { cursor: row-resize !important; }

/* ─── Drag ──────────────────────────────────────────── */
[data-wf-drag] { touch-action: pan-y; -webkit-user-select: none; user-select: none; -webkit-user-drag: none; }
[data-dragging] { opacity: 0.4; }
/*@full*/
.wf-drag-ghost { position: fixed; left: 0; top: 0; margin: 0; pointer-events: none; z-index: 2147483647; opacity: 0.9; box-shadow: var(--shadow-lg); }
/*@structural*/
.wf-drag-ghost { position: fixed; left: 0; top: 0; margin: 0; pointer-events: none; z-index: 2147483647; opacity: 0.9; }
/*@both*/
:root[data-wf-dragging] { user-select: none; }
:root[data-wf-dragging], :root[data-wf-dragging] * { cursor: grabbing !important; }
[data-drop="before"] { box-shadow: inset 0 2px 0 var(--color-primary); }
[data-drop="after"] { box-shadow: inset 0 -2px 0 var(--color-primary); }
[data-drop-axis="x"][data-drop="before"] { box-shadow: inset 2px 0 0 var(--color-primary); }
[data-drop-axis="x"][data-drop="after"] { box-shadow: inset -2px 0 0 var(--color-primary); }
[data-drop="inside"] { outline: 2px solid var(--color-primary); outline-offset: -2px; background: color-mix(in srgb, var(--color-primary) 10%, transparent); }

/* ─── Layer ─────────────────────────────────────────── */
.wf-layer { position: fixed; inset: auto; margin: 0; }
.wf-layer:not([data-open]) { display: none; }
/* A layer drawn as a popover fades out, kept in the top layer until it has. */
.wf-layer[popover] { transition: opacity var(--animation-duration-fast) var(--ease-standard), overlay var(--animation-duration-fast) allow-discrete, display var(--animation-duration-fast) allow-discrete; }
.wf-layer[popover]:not([data-open]) { opacity: 0; }
@starting-style { .wf-layer[popover][data-open] { opacity: 0; } }

/* ─── Icon ─────────────────────────────────────────── */
.wf-icon { display: inline-flex; align-items: center; justify-content: center; width: 1em; height: 1em; vertical-align: middle; }
.wf-icon svg { width: 100%; height: 100%; }
.wf-icon--small { font-size: var(--font-size-sm); }
.wf-icon--large { font-size: var(--font-size-xl); }
.wf-icon--primary { color: var(--color-primary); }
.wf-icon--danger { color: var(--color-danger); }
.wf-icon--success { color: var(--color-success); }
.wf-icon--secondary { color: var(--color-secondary); }
.wf-icon--warning { color: var(--color-warning); }
.wf-icon--info { color: var(--color-info); }

/*@full*/
/* ─── Carousel ──────────────────────────────────────── */
.wf-carousel { position: relative; overflow: hidden; border-radius: var(--radius-md); }
.wf-carousel__track { display: flex; transition: transform var(--transition-normal); }
/*@structural*/
/* ─── Carousel (mechanism paint on the dots) ────────── */
.wf-carousel { position: relative; overflow: hidden; }
.wf-carousel__track { display: flex; }
/*@both*/
.wf-carousel__slide { flex: 0 0 100%; min-width: 100%; }
.wf-carousel__nav { position: absolute; bottom: var(--spacing-sm); left: 50%; transform: translateX(-50%); display: flex; align-items: center; gap: var(--spacing-xs); }
.wf-carousel__dots { display: flex; gap: var(--spacing-xs); }
/*@full*/
.wf-carousel__control { min-width: var(--target-min); height: var(--target-min); padding: 0 6px; border: none; border-radius: var(--radius-sm); background: rgba(0,0,0,0.45); color: #fff; font: inherit; font-size: 14px; line-height: 1; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; }
.wf-carousel__control:hover { background: rgba(0,0,0,0.65); }
.wf-carousel__dot { width: var(--target-min); height: var(--target-min); padding: calc((var(--target-min) - 8px) / 2); border-radius: 50%; background: transparent; border: none; cursor: pointer; display: inline-flex; box-sizing: border-box; }
.wf-carousel__dot::after { content: ""; width: 100%; height: 100%; border-radius: 50%; background: rgba(255,255,255,0.5); }
.wf-carousel__dot.active::after { background: #fff; }
/*@structural*/
.wf-carousel__control { min-width: var(--target-min); height: var(--target-min); padding: 0 6px; border: none; cursor: pointer; display: inline-flex; align-items: center; justify-content: center; font: inherit; }
.wf-carousel__dot { width: var(--target-min); height: var(--target-min); padding: calc((var(--target-min) - 8px) / 2); border: none; cursor: pointer; display: inline-flex; box-sizing: border-box; }
.wf-carousel__dot.active { background: #fff; }
/*@both*/

/*@full*/
/* ─── Typography ────────────────────────────────────── */
/*@structural*/
/* ─── Typography: requested modifiers only ──────────── */
/*@both*/
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
.wf-text--secondary { color: var(--color-secondary); }
.wf-text--info { color: var(--color-info); }

/*@full*/
.wf-heading { font-weight: var(--font-weight-bold); line-height: var(--line-height-tight); margin: 0; letter-spacing: -0.02em; }
h1.wf-heading { font-size: var(--font-size-3xl); letter-spacing: -0.03em; }
h2.wf-heading { font-size: var(--font-size-2xl); }
h3.wf-heading { font-size: var(--font-size-xl); }
h4.wf-heading { font-size: var(--font-size-lg); }
h5.wf-heading { font-size: var(--font-size-base); }
h6.wf-heading { font-size: var(--font-size-sm); }

/*@structural*/
/* No h1–h6 scale here: the level the author wrote carries the UA's own sizing. */
.wf-heading { margin: 0; line-height: var(--line-height-tight); }
/*@both*/
/* ─── Code ──────────────────────────────────────────── */
/*@full*/
.wf-code { font-family: var(--font-family-mono); font-size: var(--font-size-sm); background: var(--color-surface); padding: 0.125rem var(--spacing-xs); border-radius: var(--radius-sm); }
pre.wf-code, .wf-code--block { display: block; padding: var(--spacing-md); overflow-x: auto; background: var(--term-bg); color: var(--term-ink); border: none; border-radius: var(--radius-md); font-size: var(--font-size-sm); line-height: 1.7; white-space: pre; }
/*@structural*/
.wf-code { font-family: var(--font-family-mono); }
pre.wf-code, .wf-code--block { display: block; padding: var(--spacing-md); overflow-x: auto; white-space: pre; }
/*@both*/
/* ─── Code Block ────────────────────────────────────── */
/* A bar over the code (its file name, Copy), and the line numbers beside it. */
.wf-code-block { display: grid; grid-template-columns: auto minmax(0, 1fr); min-width: 0; }
.wf-code-block > .wf-code { grid-column: 2; margin: 0; min-width: 0; }
.wf-code-block:not(:has(> .wf-code-block__lines)) > .wf-code { grid-column: 1 / -1; }
.wf-code-block__bar { grid-column: 1 / -1; display: flex; align-items: center; gap: var(--spacing-sm); min-height: var(--target-min); padding: var(--spacing-xs) var(--spacing-sm); font-family: var(--font-family-mono); font-size: var(--font-size-sm); }
.wf-code-block__bar > .wf-copy-button { order: 1; margin-inline-start: auto; }
.wf-code-block__lines { margin: 0; padding: var(--spacing-md) var(--spacing-sm); text-align: end; font-family: var(--font-family-mono); font-size: var(--font-size-sm); line-height: 1.7; user-select: none; }
/*@full*/
.wf-code-block { border-radius: var(--radius-md); overflow: hidden; background: var(--term-bg); color: var(--term-ink); }
.wf-code-block > .wf-code { border-radius: 0; }
.wf-code-block__bar { color: var(--term-dim); border-bottom: 1px solid rgba(255,255,255,0.08); }
.wf-code-block__lines { color: var(--term-dim); background: var(--term-bg); }
/*@both*/

/* ─── Copy Button ───────────────────────────────────── */
.wf-copy-button { flex-shrink: 0; min-height: var(--target-min); }
.wf-copy-button[data-copied] { cursor: default; }
/*@full*/
.wf-code-block .wf-copy-button { padding: var(--spacing-xs) var(--spacing-sm); font-size: var(--font-size-sm); background: transparent; color: inherit; border-color: rgba(255,255,255,0.2); }
/*@both*/

/* ─── Callout ───────────────────────────────────────── */
/* A note set apart from its text, as GitHub's alerts are: a rule down its
   start, its kind's icon and word, then what it says. */
.wf-callout { display: flex; flex-direction: column; gap: var(--spacing-xs); padding: var(--spacing-sm) var(--spacing-md); border-inline-start: 3px solid currentColor; }
.wf-callout > * { margin: 0; color: var(--color-text); }
.wf-callout__title { display: flex; align-items: center; gap: var(--spacing-xs); font-weight: var(--font-weight-bold); color: inherit; }
.wf-callout > .wf-callout__title { color: inherit; }
.wf-callout { color: var(--color-info); }
.wf-callout[data-kind="tip"] { color: var(--color-success); }
.wf-callout[data-kind="important"] { color: var(--color-primary); }
.wf-callout[data-kind="warning"] { color: var(--color-warning); }
.wf-callout[data-kind="caution"] { color: var(--color-danger); }
/*@full*/
.wf-callout { background: var(--color-surface); border-radius: 0 var(--radius-md) var(--radius-md) 0; }
.wf-callout:dir(rtl) { border-radius: var(--radius-md) 0 0 var(--radius-md); }
/*@both*/

/* ─── Kbd ───────────────────────────────────────────── */
.wf-kbd { display: inline-block; font-family: var(--font-family-mono); font-size: 0.85em; line-height: 1.4; white-space: nowrap; }
/*@full*/
.wf-kbd { padding: 0 0.4em; border: 1px solid var(--color-border); border-bottom-width: 2px; border-radius: var(--radius-sm); background: var(--color-surface); }
/*@both*/

/* ─── Empty State ───────────────────────────────────── */
.wf-empty-state { display: flex; flex-direction: column; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-xl) var(--spacing-md); text-align: center; }
.wf-empty-state > .wf-icon { font-size: 2.5rem; color: var(--color-text-muted); }
.wf-empty-state__title { margin: 0; font-size: var(--font-size-lg); font-weight: var(--font-weight-bold); }
.wf-empty-state__text { margin: 0; max-width: 40ch; color: var(--color-text-muted); }
.wf-empty-state__actions { display: flex; flex-wrap: wrap; justify-content: center; gap: var(--spacing-sm); margin-top: var(--spacing-sm); }

/* ─── Stat ──────────────────────────────────────────── */
.wf-stat { display: flex; flex-direction: column; gap: var(--spacing-xs); }
.wf-stat > p { margin: 0; }
.wf-stat__label { font-size: var(--font-size-sm); color: var(--color-text-muted); }
.wf-stat__value { font-size: 1.75rem; font-weight: var(--font-weight-bold); line-height: 1.2; font-variant-numeric: tabular-nums; }
.wf-stat__change { font-size: var(--font-size-sm); font-variant-numeric: tabular-nums; }
.wf-stat__change[data-trend="up"] { color: var(--color-success); }
.wf-stat__change[data-trend="down"] { color: var(--color-danger); }
.wf-stat__change[data-trend="up"]::before { content: "▲ " / ""; }
.wf-stat__change[data-trend="down"]::before { content: "▼ " / ""; }

/* ─── Chip ──────────────────────────────────────────── */
.wf-chip { display: inline-flex; align-items: center; max-width: 100%; vertical-align: middle; }
.wf-chip__label, .wf-chip__action { display: inline-flex; align-items: center; gap: var(--spacing-xs); min-height: var(--target-min); padding: 0 var(--spacing-sm); font: inherit; font-size: var(--font-size-sm); color: inherit; background: none; border: 0; }
.wf-chip__action { cursor: pointer; }
.wf-chip__remove { display: inline-flex; align-items: center; justify-content: center; min-width: var(--target-min); min-height: var(--target-min); padding: 0; font-size: var(--font-size-xs); color: inherit; background: none; border: 0; cursor: pointer; opacity: 0.7; }
.wf-chip__remove:hover { opacity: 1; }
/*@full*/
.wf-chip { border: 1px solid var(--color-border); border-radius: var(--radius-full); background: var(--color-surface); }
.wf-chip:has(> .wf-chip__action[aria-pressed="true"]) { background: var(--color-primary); border-color: var(--color-primary); color: #fff; }
/*@both*/

/* ─── Theme Toggle ──────────────────────────────────── */
/* A moon in a light page, a sun in a dark one: chosen here, so the static
   paint shows the right one before any script runs. */
.wf-theme-toggle > [data-shows="dark"] { display: none; }
:root[data-theme="dark"] .wf-theme-toggle > [data-shows="dark"] { display: inline-flex; }
:root[data-theme="dark"] .wf-theme-toggle > [data-shows="light"] { display: none; }
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) .wf-theme-toggle > [data-shows="dark"] { display: inline-flex; }
  :root:not([data-theme="light"]) .wf-theme-toggle > [data-shows="light"] { display: none; }
}

/* ─── Locale Switch ─────────────────────────────────── */
.wf-locale-switch { display: inline-flex; flex-wrap: wrap; gap: var(--spacing-xs); }
.wf-locale-switch__option[aria-pressed="true"] { font-weight: var(--font-weight-bold); }
/*@full*/
.wf-locale-switch__option[aria-pressed="true"] { border-color: var(--color-primary); }
/*@both*/

/* ─── Accordion ─────────────────────────────────────── */
.wf-accordion { display: flex; flex-direction: column; }
.wf-accordion__summary { display: flex; align-items: center; justify-content: space-between; gap: var(--spacing-sm); min-height: var(--target-min); padding: var(--spacing-sm) 0; cursor: pointer; list-style: none; font-weight: var(--font-weight-medium); }
.wf-accordion__summary::-webkit-details-marker { display: none; }
.wf-accordion__summary::after { content: "" / ""; width: 0.5em; height: 0.5em; border-inline-end: 2px solid currentColor; border-bottom: 2px solid currentColor; transform: rotate(45deg); transition: transform var(--transition-fast); flex-shrink: 0; }
.wf-accordion__item[open] > .wf-accordion__summary::after { transform: rotate(225deg); }
.wf-accordion__panel { padding: 0 0 var(--spacing-md); }
/* It opens to its height where the browser can say so. */
.wf-accordion__item { interpolate-size: allow-keywords; }
.wf-accordion__item::details-content { block-size: 0; overflow: hidden; transition: block-size var(--transition-normal), content-visibility var(--transition-normal) allow-discrete; }
.wf-accordion__item[open]::details-content { block-size: auto; }
/*@full*/
.wf-accordion__item { border-bottom: 1px solid var(--color-border); }
/*@both*/

/* ─── Field ─────────────────────────────────────────── */
.wf-fieldset { display: flex; flex-direction: column; gap: var(--spacing-sm); min-width: 0; margin: 0; padding: var(--spacing-md); }
.wf-fieldset__legend { padding: 0 var(--spacing-xs); font-weight: var(--font-weight-bold); }
/*@full*/
.wf-fieldset { border: 1px solid var(--color-border); border-radius: var(--radius-md); }
/*@structural*/
.wf-fieldset { border: 0; }
/*@both*/

/* ─── Input Group ───────────────────────────────────── */
/* An input and what goes beside it, drawn as one box. */
.wf-input-group { display: flex; align-items: center; gap: var(--spacing-xs); padding: 0 var(--spacing-sm); }
.wf-input-group > .wf-input, .wf-input-group > .wf-field { flex: 1; min-width: 0; }
.wf-input-group > .wf-input { border: 0; background: none; box-shadow: none; padding-inline: var(--spacing-xs); }
.wf-input-group > .wf-icon { color: var(--color-text-muted); flex-shrink: 0; }
.wf-input-group__addon { flex-shrink: 0; color: var(--color-text-muted); font-size: var(--font-size-sm); }
.wf-input-group__clear { display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; min-width: var(--target-min); min-height: var(--target-min); padding: 0; color: var(--color-text-muted); background: none; border: 0; cursor: pointer; }
.wf-input-group__clear[hidden] { display: none; }
/*@full*/
.wf-input-group { border: 1px solid var(--color-border); border-radius: var(--radius-md); background: var(--color-background); }
.wf-input-group:focus-within { border-color: var(--color-primary); box-shadow: 0 0 0 3px rgba(59,130,246,0.15); }
.wf-input-group > .wf-input:focus-visible { box-shadow: none; }
/*@both*/

/* ─── On This Page ──────────────────────────────────── */
.wf-on-this-page { display: flex; flex-direction: column; gap: var(--spacing-xs); font-size: var(--font-size-sm); }
.wf-on-this-page[hidden] { display: none; }
.wf-on-this-page__title { margin: 0; font-weight: var(--font-weight-bold); }
.wf-on-this-page__list { margin: 0; padding: 0; list-style: none; display: flex; flex-direction: column; }
.wf-on-this-page__link { display: block; padding: 2px 0; padding-inline-start: var(--spacing-sm); color: var(--color-text-muted); text-decoration: none; border-inline-start: 2px solid transparent; }
.wf-on-this-page__link[data-level="3"] { padding-inline-start: var(--spacing-lg); }
.wf-on-this-page__link[data-level="4"] { padding-inline-start: var(--spacing-xl); }
.wf-on-this-page__link:hover { color: var(--color-text); }
.wf-on-this-page__link[aria-current="location"] { color: var(--color-text); border-inline-start-color: var(--color-primary); font-weight: var(--font-weight-medium); }

/* ─── Toggle ────────────────────────────────────────── */
/* A button that stays pressed, and a group of them: the icon before the
   words, the pressed one marked. */
.wf-toggle, .wf-toggle-group__item { display: inline-flex; align-items: center; gap: var(--spacing-xs); }
.wf-toggle > .wf-icon, .wf-toggle-group__item > .wf-icon { order: -1; }
.wf-toggle-group { display: inline-flex; flex-wrap: wrap; }
.wf-toggle-group__item[aria-pressed="true"], .wf-toggle[aria-pressed="true"] { font-weight: var(--font-weight-medium); }
/*@full*/
.wf-toggle[aria-pressed="true"], .wf-toggle-group__item[aria-pressed="true"] { background: var(--color-primary); border-color: var(--color-primary); color: #fff; }
.wf-toggle-group > .wf-toggle-group__item { border-radius: 0; margin-inline-start: -1px; }
.wf-toggle-group > .wf-toggle-group__item:first-child { border-start-start-radius: var(--radius-md); border-end-start-radius: var(--radius-md); margin-inline-start: 0; }
.wf-toggle-group > .wf-toggle-group__item:last-child { border-start-end-radius: var(--radius-md); border-end-end-radius: var(--radius-md); }
/*@both*/

/* ─── Radio Group ───────────────────────────────────── */
.wf-radio-group { display: flex; flex-direction: column; gap: var(--spacing-xs); margin: 0; padding: 0; border: 0; min-width: 0; }
.wf-radio-group__legend { padding: 0; margin-bottom: var(--spacing-xs); font-weight: var(--font-weight-medium); }
.wf-radio-group__option { display: flex; flex-direction: column; }
.wf-radio-group__hint { margin: 0; padding-inline-start: 1.6em; font-size: var(--font-size-sm); color: var(--color-text-muted); }
.wf-radio-group[data-look="cards"] { display: grid; grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr)); gap: var(--spacing-sm); }
.wf-radio-group[data-look="cards"] > .wf-radio-group__legend { grid-column: 1 / -1; }
.wf-radio-group[data-look="cards"] .wf-radio-group__option { position: relative; padding: var(--spacing-md); }
/*@full*/
.wf-radio-group[data-look="cards"] .wf-radio-group__option { border: 1px solid var(--color-border); border-radius: var(--radius-md); }
.wf-radio-group[data-look="cards"] .wf-radio-group__option:has(input:checked) { border-color: var(--color-primary); box-shadow: 0 0 0 1px var(--color-primary); }
/*@both*/

/* ─── Pagination ────────────────────────────────────── */
.wf-pagination { display: flex; flex-wrap: wrap; align-items: center; gap: var(--spacing-xs); }
.wf-pagination__page, .wf-pagination__prev, .wf-pagination__next { min-width: var(--target-min); min-height: var(--target-min); justify-content: center; }
.wf-pagination__gap { padding: 0 var(--spacing-xs); color: var(--color-text-muted); }
/*@full*/
.wf-pagination__page[aria-current="page"] { background: var(--color-primary); border-color: var(--color-primary); color: #fff; }
/*@structural*/
.wf-pagination__page[aria-current="page"] { font-weight: var(--font-weight-bold); }
/*@both*/

/* ─── Stepper ───────────────────────────────────────── */
.wf-stepper { display: flex; flex-direction: column; gap: var(--spacing-md); }
.wf-stepper__list { display: flex; flex-wrap: wrap; gap: var(--spacing-sm); margin: 0; padding: 0; list-style: none; counter-reset: wf-step; }
.wf-stepper__item { counter-increment: wf-step; }
.wf-stepper__item > .wf-btn { gap: var(--spacing-xs); }
.wf-stepper__item > .wf-btn::before { content: counter(wf-step) / ""; display: inline-flex; align-items: center; justify-content: center; width: 1.5em; height: 1.5em; border-radius: 50%; border: 1px solid currentColor; font-size: var(--font-size-sm); }
.wf-stepper__item > .wf-btn[data-state="done"]::before { content: "✓" / ""; }
.wf-stepper__item > .wf-btn[data-state="error"]::before { content: "!" / ""; color: var(--color-danger); }
.wf-stepper__item > .wf-btn[aria-current="step"] { font-weight: var(--font-weight-bold); }
/*@full*/
.wf-stepper__item > .wf-btn { border-color: transparent; background: none; }
.wf-stepper__item > .wf-btn[aria-current="step"]::before { background: var(--color-primary); border-color: var(--color-primary); color: #fff; }
.wf-stepper__item > .wf-btn[data-state="done"]::before { background: var(--color-success); border-color: var(--color-success); color: #fff; }
/*@both*/

/* ─── Number Input ──────────────────────────────────── */
.wf-number-input .wf-input-group > .wf-input { text-align: center; }
/*@full*/
/* The − and + beside it are its buttons: the browser's own go. */
.wf-number-input .wf-input-group > .wf-input { -moz-appearance: textfield; }
.wf-number-input .wf-input-group > .wf-input::-webkit-inner-spin-button, .wf-number-input .wf-input-group > .wf-input::-webkit-outer-spin-button { -webkit-appearance: none; margin: 0; }
/*@both*/
.wf-number-input__step { display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; min-width: var(--target-min); min-height: var(--target-min); padding: 0; border: 0; background: none; color: inherit; cursor: pointer; }
.wf-number-input__step:disabled { opacity: 0.4; cursor: default; }
.wf-number-input .wf-input-group > .wf-number-input__step:first-of-type { order: -1; }

/* ─── Calendar ──────────────────────────────────────── */
.wf-calendar { display: inline-flex; flex-direction: column; gap: var(--spacing-xs); }
.wf-calendar__head { display: flex; align-items: center; justify-content: space-between; gap: var(--spacing-sm); }
.wf-calendar__title { margin: 0; font-weight: var(--font-weight-medium); }
.wf-calendar__nav { min-width: var(--target-min); min-height: var(--target-min); border: 0; background: none; color: inherit; font-size: var(--font-size-lg); cursor: pointer; }
.wf-calendar__nav:dir(rtl) { transform: scaleX(-1); }
.wf-calendar__grid { border-collapse: collapse; }
.wf-calendar__grid th { font-size: var(--font-size-xs); font-weight: var(--font-weight-medium); color: var(--color-text-muted); padding: var(--spacing-xs) 0; }
.wf-calendar__grid td { padding: 1px; text-align: center; }
.wf-calendar__day { width: 2.25rem; height: 2.25rem; min-width: var(--target-min); min-height: var(--target-min); border: 0; border-radius: var(--radius-md); background: none; color: inherit; font: inherit; font-variant-numeric: tabular-nums; cursor: pointer; }
.wf-calendar__day[data-outside] { color: var(--color-text-muted); }
.wf-calendar__day[aria-disabled="true"] { opacity: 0.35; cursor: default; text-decoration: line-through; }
.wf-calendar__day[aria-current="date"] { font-weight: var(--font-weight-bold); text-decoration: underline; }
/*@full*/
.wf-calendar__day:hover { background: var(--color-surface); }
.wf-calendar__grid td[aria-selected="true"] > .wf-calendar__day { background: var(--color-primary); color: #fff; }
/*@structural*/
.wf-calendar__grid td[aria-selected="true"] > .wf-calendar__day { outline: 2px solid currentColor; }
/*@both*/

/* ─── Highlight ─────────────────────────────────────── */
.wf-tok-kw { color: var(--syntax-keyword); }
.wf-tok-name { color: var(--syntax-function); }
.wf-tok-str { color: var(--syntax-string); }
.wf-tok-num { color: var(--syntax-number); }
.wf-tok-prop { color: var(--syntax-punct); }
.wf-tok-tok { color: var(--syntax-function); }
.wf-tok-cmt { color: var(--syntax-comment); }
.wf-tok-prompt { color: var(--term-dim); }

/* ─── Blockquote ────────────────────────────────────── */
/*@full*/
.wf-blockquote { border-inline-start: 4px solid var(--color-primary); padding: var(--spacing-sm) var(--spacing-md); margin: 0; color: var(--color-text-muted); font-style: italic; }
/* ─── Markdown ──────────────────────────────────────── */
.wf-markdown { line-height: 1.65; }
.wf-markdown > * + * { margin-top: var(--spacing-md); }
.wf-markdown h1, .wf-markdown h2, .wf-markdown h3, .wf-markdown h4 { line-height: 1.25; }
.wf-markdown pre { padding: var(--spacing-md); overflow-x: auto; background: var(--term-bg); color: var(--term-ink); border-radius: var(--radius-md); font-size: var(--font-size-sm); }
.wf-markdown code { font-family: var(--font-family-mono); font-size: 0.925em; }
.wf-markdown blockquote { border-inline-start: 4px solid var(--color-primary); padding: var(--spacing-sm) var(--spacing-md); margin: 0; color: var(--color-text-muted); }
.wf-markdown ul, .wf-markdown ol { padding-inline-start: 1.5em; }
.wf-markdown img { max-width: 100%; }
/*@structural*/
.wf-blockquote { border-inline-start: 4px solid currentColor; padding: var(--spacing-sm) var(--spacing-md); margin: 0; }
/*@both*/

/*@full*/
/* ─── Variant Colors (shared) ───────────────────────── */
/*@structural*/
/* ─── Variant Colors (shared, requested) ────────────── */
/*@both*/
.wf-primary { color: var(--color-primary); }
.wf-secondary { color: var(--color-secondary); }
.wf-success { color: var(--color-success); }
.wf-danger { color: var(--color-danger); }
.wf-warning { color: var(--color-warning); }
.wf-info { color: var(--color-info); }

/*@full*/
/* ─── Responsive ────────────────────────────────────── */
/* A layout is the author's. `.stacks` is how a Row, a Grid or a Column
   asks to reflow on a narrow screen; a responsive value —
   `Grid(columns: { base: 1, md: 3 })` — says exactly what it should do,
   at every width. The engine used to do this to every layout whatever
   the author wrote, with `!important`, which took twelve overrides and
   five `!important`s to undo on one site. */
/*@structural*/
/* ─── Responsive: reflow only, no type scale ────────── */
/*@both*/
@media (max-width: 1024px) {
  .wf-grid.wf-stacks { grid-template-columns: repeat(2, 1fr); }
  .wf-col--4.wf-stacks, .wf-col--3.wf-stacks { flex: 0 0 50%; max-width: 50%; }
}
@media (max-width: 768px) {
/*@structural*/
  /* Every reflow here is asked for: `.stacks` on a Row, a Grid or a Column,
     or a responsive value that says what to do at each width. */
/*@both*/
  .wf-row.wf-stacks { flex-direction: column; }
  .wf-col.wf-stacks, .wf-col--1.wf-stacks, .wf-col--2.wf-stacks, .wf-col--3.wf-stacks,
  .wf-col--4.wf-stacks, .wf-col--5.wf-stacks, .wf-col--6.wf-stacks, .wf-col--7.wf-stacks,
  .wf-col--8.wf-stacks, .wf-col--9.wf-stacks, .wf-col--10.wf-stacks, .wf-col--11.wf-stacks,
  .wf-col--12.wf-stacks {
    flex: 0 0 100%; max-width: 100%;
  }
  .wf-grid.wf-stacks { grid-template-columns: 1fr; }
  .wf-navbar { flex-wrap: wrap; padding: var(--spacing-sm); }
/*@full*/
  .wf-navbar__links { flex-wrap: wrap; gap: var(--spacing-xs); font-size: var(--font-size-sm); }
/*@structural*/
  .wf-navbar__links { flex-wrap: wrap; gap: var(--spacing-xs); }
/*@both*/
  .wf-navbar__brand { width: 100%; }
  .wf-navbar__actions { width: 100%; justify-content: flex-start; }
/*@full*/
  /* Off-canvas, not gone: `.wf-sidebar` used to be `display: none` here, which
     removed a site's entire navigation on a phone with no way to reach it. */
/*@both*/
  .wf-sidebar {
    position: fixed;
    inset-inline-start: 0;
    top: 0;
    height: 100vh;
    z-index: var(--z-drawer);
    transform: translateX(-100%);
    /* Off the page, it is out of the Tab order and the accessibility tree
       too. */
    visibility: hidden;
    transition: transform var(--transition-normal), visibility var(--transition-normal);
/*@full*/
    box-shadow: var(--shadow-lg);
/*@both*/
  }
  /* It waits off its own edge: the left in a left-to-right page, the right
     in a right-to-left one. */
  [dir="rtl"] .wf-sidebar { transform: translateX(100%); }
  /* Opening, it is visible at once (so the focus can go into it); shutting,
     only once it has slid out. */
  .wf-sidebar[data-open="true"], [dir="rtl"] .wf-sidebar[data-open="true"] { transform: none; visibility: visible; transition: transform var(--transition-normal); }
  .wf-sidebar__toggle { display: inline-flex; position: fixed; inset-inline-start: var(--spacing-sm); top: var(--spacing-sm); z-index: calc(var(--z-drawer) + 1); }
  .wf-sidebar__scrim { display: block; }
  .wf-container { padding: 0 var(--spacing-sm); }
/*@full*/
  .wf-table { font-size: var(--font-size-sm); }
/*@both*/
  .wf-table th, .wf-table td { padding: var(--spacing-xs) var(--spacing-sm); }
  .wf-btn-group { flex-wrap: wrap; }
  .wf-modal__content, .wf-dialog__content { width: 95%; }
/*@full*/
  pre.wf-code { font-size: var(--font-size-xs); }
/*@both*/
}
@media (max-width: 480px) {
/*@full*/
  /* The links used to be `display: none` with no menu behind them, so a phone
     got a navbar with nothing in it. They collapse behind the navbar's own
     toggle instead. */
/*@both*/
  .wf-navbar__toggle { display: inline-flex; }
/*@full*/
  .wf-navbar__links {
    display: none;
    order: 3;
    width: 100%;
    flex-direction: column;
    align-items: stretch;
  }
/*@structural*/
  .wf-navbar__links { display: none; order: 3; width: 100%; flex-direction: column; align-items: stretch; }
/*@both*/
  .wf-navbar[data-open="true"] .wf-navbar__links { display: flex; }
  .wf-navbar__actions { margin-inline-start: auto; }
/*@full*/
  body { font-size: var(--font-size-sm); }
/*@both*/
}

/* ─── Animation Keyframes ───────────────────────────── */
@keyframes wf-fadeIn { from { opacity: 0; } to { opacity: 1; } }
@keyframes wf-fadeOut { from { opacity: 1; } to { opacity: 0; } }
@keyframes wf-slideUp { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: none; } }
@keyframes wf-slideDown { from { opacity: 0; transform: translateY(-20px); } to { opacity: 1; transform: none; } }
@keyframes wf-slideLeft { from { opacity: 0; transform: translateX(20px); } to { opacity: 1; transform: none; } }
/*@full*/
@keyframes wf-slideOutLeft { from { opacity: 1; transform: none; } to { opacity: 0; transform: translateX(-20px); } }
/*@both*/
@keyframes wf-slideRight { from { opacity: 0; transform: translateX(-20px); } to { opacity: 1; transform: none; } }
@keyframes wf-scaleIn { from { opacity: 0; transform: scale(0.9); } to { opacity: 1; transform: none; } }
@keyframes wf-scaleOut { from { opacity: 1; transform: none; } to { opacity: 0; transform: scale(0.9); } }
@keyframes wf-bounce { 0% { opacity: 0; transform: scale(0.3); } 50% { transform: scale(1.05); } 70% { transform: scale(0.9); } 100% { opacity: 1; transform: none; } }
@keyframes wf-shake { 0%,100% { transform: none; } 10%,30%,50%,70%,90% { transform: translateX(-4px); } 20%,40%,60%,80% { transform: translateX(4px); } }
@keyframes wf-pulse { 0%,100% { transform: scale(1); } 50% { transform: scale(1.05); } }
/*@full*/
@keyframes wf-expand { from { block-size: 0; opacity: 0; } to { block-size: auto; opacity: 1; } }
@keyframes wf-collapse { from { block-size: auto; opacity: 1; } to { block-size: 0; opacity: 0; } }
/*@both*/

/* ─── Animation Utility Classes ─────────────────────── */
.wf-animate-fadeIn { animation: wf-fadeIn var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-fadeOut { animation: wf-fadeOut var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideUp { animation: wf-slideUp var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideDown { animation: wf-slideDown var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-slideLeft { animation: wf-slideLeft var(--animation-duration-normal) var(--animation-easing-default) both; }
/*@full*/
/* A route change through the View Transitions API: the browser crossfades
   the old page into the new by default; `Router(transition: .slide)` slides. */
::view-transition-old(root), ::view-transition-new(root) { animation-duration: var(--animation-duration-normal); animation-timing-function: var(--animation-easing-default); }
[data-wf-transition="slide"]::view-transition-old(root) { animation-name: wf-slideOutLeft; }
[data-wf-transition="slide"]::view-transition-new(root) { animation-name: wf-slideLeft; }
/*@both*/
.wf-animate-slideRight { animation: wf-slideRight var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-scaleIn { animation: wf-scaleIn var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-scaleOut { animation: wf-scaleOut var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-bounce { animation: wf-bounce var(--animation-duration-slow) var(--animation-easing-bounce) both; }
.wf-animate-shake { animation: wf-shake var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-pulse { animation: wf-pulse var(--animation-duration-slow) var(--animation-easing-default) infinite; }
.wf-animate-spin { animation: wf-spin 0.6s linear infinite; }
/*@full*/
/* Opening to the height of the content. A browser that understands
   `interpolate-size` animates the height; one that does not drops the
   `auto` keyframe and fades instead. `WF.expand` measures and is exact
   either way, and is what a `show`, an `if` or a list plays. */
.wf-animate-expand { interpolate-size: allow-keywords; overflow: hidden; animation: wf-expand var(--animation-duration-normal) var(--animation-easing-default) both; }
.wf-animate-collapse { interpolate-size: allow-keywords; overflow: hidden; animation: wf-collapse var(--animation-duration-normal) var(--animation-easing-default) both; }
/*@both*/
.wf-animate--fast { animation-duration: var(--animation-duration-fast) !important; }
.wf-animate--slow { animation-duration: var(--animation-duration-slow) !important; }

/*@full*/
/* ─── Hover Replay: cards with inline animation style replay on hover ── */
/*@structural*/
/* ─── Hover Replay: inline-animated elements replay on hover ── */
/*@both*/
.wf-card[style*="animation"]:hover { animation-iteration-count: 1 !important; animation-play-state: running; }
.wf-anim-hover { transition: transform 0.15s ease; }
.wf-anim-hover:hover { animation-iteration-count: 1 !important; }

/*@structural*/
/* ─── Focus Visibility ──────────────────────────────── */
/* Structural mode ships no design, but a keyboard user still has to be able to
   see where they are — this sheet had no focus rule at all, which left the
   indicator to whatever the browser happened to do under the author's CSS. */
:focus-visible { outline: 2px solid var(--color-primary); outline-offset: 2px; }

/* ─── Hit Areas ─────────────────────────────────────── */
/* WCAG 2.2 §2.5.8 sets 24×24 as the floor for a pointer target (`target-min`). */
.wf-checkbox, .wf-radio, .wf-switch, .wf-menu__trigger { min-height: var(--target-min); }
.wf-tag__remove, .wf-alert__dismiss { min-width: var(--target-min); min-height: var(--target-min); }

/*@both*/
/* ─── Touch Targets ─────────────────────────────────── */
/* Every control reaches at least `target-min` — 24px, WCAG 2.2's floor for a
   pointer — and `target-min-touch` (44px) where the pointer is a finger. A
   theme sets either. */
@media (pointer: coarse) { :root { --target-min: var(--target-min-touch); } }
.wf-btn, .wf-icon-btn, .wf-tabs__tab, .wf-menu__item, .wf-dropdown__item, .wf-sidebar__item { min-block-size: var(--target-min); }
.wf-icon-btn { min-inline-size: var(--target-min); }

/* ─── Reduced Motion ────────────────────────────────── */
/*@full*/
/* A user who has asked their system for less motion has usually done so because
   motion makes them ill. Honour it: keep the end state, drop the movement. */
/*@both*/
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
/*@full*/
  html { scroll-behavior: auto; }
/*@both*/
}
/* The same, asked for by the page itself: an editor previewing what a
   reader who wants less motion sees sets `data-wf-motion="reduce"` on the
   root. */
:root[data-wf-motion="reduce"] *, :root[data-wf-motion="reduce"] *::before, :root[data-wf-motion="reduce"] *::after {
  animation-duration: 0.01ms !important;
  animation-iteration-count: 1 !important;
  transition-duration: 0.01ms !important;
  scroll-behavior: auto !important;
}

/*@full*/
/* ─── Focus Visibility ──────────────────────────────── */
/* Every focusable control needs a focus indicator a keyboard user can see. The
   engine styled the input ring and left everything else to whatever the browser
   happened to do. */
:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}
.wf-skip-link {
  position: absolute;
  inset-inline-start: var(--spacing-sm);
  top: var(--spacing-sm);
  transform: translateY(-200%);
  z-index: var(--z-skip-link);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-background);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}
/*@structural*/
/* ─── Skip Link ─────────────────────────────────────── */
/* Opaque: the link appears over page content when focused, and a transparent
   one is unreadable exactly when it is needed. */
.wf-skip-link { position: absolute; inset-inline-start: 0.5rem; top: 0.5rem; transform: translateY(-200%); z-index: var(--z-skip-link); padding: 0.5rem 1rem; background: var(--color-background); color: var(--color-text); }
/*@both*/
.wf-skip-link:focus { transform: none; }

"#;

/// The lines of [`SOURCE`] that belong to `which` (`"full"` or
/// `"structural"`), with the markers left out.
fn select(which: &str) -> String {
    let mut mode = "both";
    let mut keep: Vec<&str> = Vec::new();
    for line in SOURCE.split('\n') {
        if let Some(m) = line
            .strip_prefix("/*@")
            .and_then(|rest| rest.strip_suffix("*/"))
            .filter(|m| matches!(*m, "both" | "full" | "structural"))
        {
            mode = m;
            continue;
        }
        if mode == "both" || mode == which {
            keep.push(line);
        }
    }
    keep.join("\n")
}

/// The full sheet: layout, mechanics and the engine's look.
pub(super) fn full() -> &'static str {
    static SHEET: OnceLock<String> = OnceLock::new();
    SHEET.get_or_init(|| format!("{}{}", select("full"), crate::paged_rules!()))
}

/// The structural sheet: layout and mechanics, without the engine's look.
pub(super) fn structural() -> &'static str {
    static SHEET: OnceLock<String> = OnceLock::new();
    SHEET.get_or_init(|| format!("{}{}", select("structural"), crate::paged_rules!()))
}
