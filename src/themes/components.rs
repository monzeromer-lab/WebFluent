/// Returns CSS for all built-in component styles.
pub fn component_css() -> &'static str {
    r#"
/* ─── Reset ─────────────────────────────────────────── */
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: var(--font-family);
  font-size: var(--font-size-base);
  line-height: var(--line-height-normal);
  color: var(--color-text);
  background: var(--color-background);
  -webkit-font-smoothing: antialiased;
}
a { color: var(--color-primary); text-decoration: none; }
a:hover { text-decoration: underline; }

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
.wf-divider { border: none; border-top: 1px solid var(--color-border); margin: var(--spacing-md) 0; }
.wf-divider--label { display: flex; align-items: center; gap: var(--spacing-md); border: none; margin: var(--spacing-md) 0; }
.wf-divider--label::before, .wf-divider--label::after { content: ""; flex: 1; border-top: 1px solid var(--color-border); }

/* ─── Navbar ────────────────────────────────────────── */
.wf-navbar { display: flex; align-items: center; justify-content: space-between; padding: var(--spacing-sm) var(--spacing-lg); background: var(--color-surface); border-bottom: 1px solid var(--color-border); position: sticky; top: 0; z-index: 100; }
.wf-navbar__brand { font-weight: var(--font-weight-bold); font-size: var(--font-size-lg); }
.wf-navbar__links { display: flex; gap: var(--spacing-md); align-items: center; }
.wf-navbar__links a { color: var(--color-text); transition: color var(--transition-fast); }
.wf-navbar__links a:hover, .wf-navbar__links a.active { color: var(--color-primary); text-decoration: none; }
.wf-navbar__actions { display: flex; gap: var(--spacing-sm); align-items: center; }

/* ─── Sidebar ───────────────────────────────────────── */
.wf-sidebar { width: 260px; min-height: 100vh; background: var(--color-surface); border-right: 1px solid var(--color-border); padding: var(--spacing-md); display: flex; flex-direction: column; gap: var(--spacing-xs); }
.wf-sidebar__header { padding: var(--spacing-sm) 0; font-weight: var(--font-weight-bold); font-size: var(--font-size-lg); }
.wf-sidebar__item { display: flex; align-items: center; gap: var(--spacing-sm); padding: var(--spacing-sm) var(--spacing-md); border-radius: var(--radius-md); color: var(--color-text); cursor: pointer; transition: background var(--transition-fast); }
.wf-sidebar__item:hover { background: var(--color-border); text-decoration: none; }
.wf-sidebar__item.active { background: var(--color-primary); color: #fff; }
.wf-sidebar__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-sm) 0; }

/* ─── Breadcrumb ────────────────────────────────────── */
.wf-breadcrumb { display: flex; align-items: center; gap: var(--spacing-xs); font-size: var(--font-size-sm); color: var(--color-text-muted); }
.wf-breadcrumb__item + .wf-breadcrumb__item::before { content: "/"; margin-right: var(--spacing-xs); color: var(--color-border); }
.wf-breadcrumb__item:last-child { color: var(--color-text); font-weight: var(--font-weight-medium); }

/* ─── Link ──────────────────────────────────────────── */
.wf-link { color: var(--color-primary); cursor: pointer; }
.wf-link:hover { text-decoration: underline; }
.wf-link.active { font-weight: var(--font-weight-bold); }

/* ─── Menu ──────────────────────────────────────────── */
.wf-menu { position: relative; display: inline-block; }
.wf-menu__trigger { cursor: pointer; }
.wf-menu__items { position: absolute; top: 100%; left: 0; min-width: 180px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius-md); box-shadow: var(--shadow-lg); z-index: 50; display: none; padding: var(--spacing-xs) 0; }
.wf-menu.open .wf-menu__items { display: block; }
.wf-menu__item { padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; transition: background var(--transition-fast); }
.wf-menu__item:hover { background: var(--color-border); }
.wf-menu__item--danger { color: var(--color-danger); }
.wf-menu__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-xs) 0; }

/* ─── Tabs ──────────────────────────────────────────── */
.wf-tabs { width: 100%; }
.wf-tabs__nav { display: flex; border-bottom: 2px solid var(--color-border); gap: 0; }
.wf-tabs__tab { padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; border-bottom: 2px solid transparent; margin-bottom: -2px; color: var(--color-text-muted); transition: all var(--transition-fast); background: none; border-top: none; border-left: none; border-right: none; font-family: var(--font-family); font-size: var(--font-size-base); }
.wf-tabs__tab:hover { color: var(--color-text); }
.wf-tabs__tab.active { color: var(--color-primary); border-bottom-color: var(--color-primary); font-weight: var(--font-weight-medium); }
.wf-tab-page { padding: var(--spacing-md) 0; display: none; }
.wf-tab-page.active { display: block; }

/* ─── Card ──────────────────────────────────────────── */
.wf-card { background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius-md); overflow: hidden; }
.wf-card--elevated { border: none; box-shadow: var(--shadow-md); }
.wf-card--outlined { background: transparent; }
.wf-card--flat { border: none; box-shadow: none; }
.wf-card__header { padding: var(--spacing-md); border-bottom: 1px solid var(--color-border); }
.wf-card__body { padding: var(--spacing-md); }
.wf-card__footer { padding: var(--spacing-md); border-top: 1px solid var(--color-border); display: flex; gap: var(--spacing-sm); justify-content: flex-end; }

/* ─── Table ─────────────────────────────────────────── */
.wf-table { width: 100%; border-collapse: collapse; }
.wf-table th, .wf-table td { padding: var(--spacing-sm) var(--spacing-md); text-align: left; border-bottom: 1px solid var(--color-border); }
.wf-table th { font-weight: var(--font-weight-bold); color: var(--color-text-muted); font-size: var(--font-size-sm); text-transform: uppercase; letter-spacing: 0.05em; background: var(--color-surface); }
.wf-table tr:hover td { background: var(--color-surface); }

/* ─── List ──────────────────────────────────────────── */
.wf-list { list-style: none; }
.wf-list-item { padding: var(--spacing-sm) var(--spacing-md); border-bottom: 1px solid var(--color-border); display: flex; align-items: center; gap: var(--spacing-sm); }
.wf-list-item:last-child { border-bottom: none; }

/* ─── Badge ─────────────────────────────────────────── */
.wf-badge { display: inline-flex; align-items: center; padding: 0.125rem var(--spacing-sm); font-size: var(--font-size-xs); font-weight: var(--font-weight-medium); border-radius: var(--radius-full); background: var(--color-surface); color: var(--color-text); border: 1px solid var(--color-border); }
.wf-badge--primary { background: var(--color-primary); color: #fff; border-color: var(--color-primary); }
.wf-badge--success { background: var(--color-success); color: #fff; border-color: var(--color-success); }
.wf-badge--danger { background: var(--color-danger); color: #fff; border-color: var(--color-danger); }
.wf-badge--warning { background: var(--color-warning); color: #000; border-color: var(--color-warning); }
.wf-badge--info { background: var(--color-info); color: #fff; border-color: var(--color-info); }
.wf-badge--secondary { background: var(--color-secondary); color: #fff; border-color: var(--color-secondary); }
.wf-badge--pill { border-radius: var(--radius-full); }

/* ─── Avatar ────────────────────────────────────────── */
.wf-avatar { width: 40px; height: 40px; border-radius: var(--radius-full); overflow: hidden; display: inline-flex; align-items: center; justify-content: center; background: var(--color-primary); color: #fff; font-weight: var(--font-weight-bold); font-size: var(--font-size-sm); }
.wf-avatar img { width: 100%; height: 100%; object-fit: cover; }
.wf-avatar--small { width: 32px; height: 32px; font-size: var(--font-size-xs); }
.wf-avatar--large { width: 56px; height: 56px; font-size: var(--font-size-lg); }

/* ─── Tooltip ───────────────────────────────────────── */
.wf-tooltip { position: relative; display: inline-block; }
.wf-tooltip__text { visibility: hidden; position: absolute; bottom: 125%; left: 50%; transform: translateX(-50%); padding: var(--spacing-xs) var(--spacing-sm); background: var(--color-text); color: var(--color-background); border-radius: var(--radius-sm); font-size: var(--font-size-xs); white-space: nowrap; z-index: 99; opacity: 0; transition: opacity var(--transition-fast); }
.wf-tooltip:hover .wf-tooltip__text { visibility: visible; opacity: 1; }

/* ─── Tag ───────────────────────────────────────────── */
.wf-tag { display: inline-flex; align-items: center; gap: var(--spacing-xs); padding: 0.125rem var(--spacing-sm); font-size: var(--font-size-sm); border-radius: var(--radius-md); background: var(--color-surface); border: 1px solid var(--color-border); }
.wf-tag__remove { cursor: pointer; opacity: 0.6; font-size: var(--font-size-xs); }
.wf-tag__remove:hover { opacity: 1; }

/* ─── Button ────────────────────────────────────────── */
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
.wf-btn--full { width: 100%; }
.wf-btn--rounded { border-radius: var(--radius-full); }
.wf-btn--pill { border-radius: var(--radius-full); }
.wf-btn--outlined { background: transparent; }
.wf-btn--outlined.wf-btn--primary { color: var(--color-primary); background: transparent; }
.wf-btn--outlined.wf-btn--danger { color: var(--color-danger); background: transparent; }

/* ─── Icon Button ───────────────────────────────────── */
.wf-icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 36px; height: 36px; padding: 0; border: none; border-radius: var(--radius-md); background: transparent; color: var(--color-text); cursor: pointer; transition: background var(--transition-fast); font-size: var(--font-size-lg); }
.wf-icon-btn:hover { background: var(--color-surface); }
.wf-icon-btn--small { width: 28px; height: 28px; font-size: var(--font-size-base); }
.wf-icon-btn--large { width: 48px; height: 48px; font-size: var(--font-size-xl); }
.wf-icon-btn--primary { color: var(--color-primary); }
.wf-icon-btn--danger { color: var(--color-danger); }

/* ─── Button Group ──────────────────────────────────── */
.wf-btn-group { display: inline-flex; }
.wf-btn-group .wf-btn { border-radius: 0; margin-left: -1px; }
.wf-btn-group .wf-btn:first-child { border-radius: var(--radius-md) 0 0 var(--radius-md); margin-left: 0; }
.wf-btn-group .wf-btn:last-child { border-radius: 0 var(--radius-md) var(--radius-md) 0; }

/* ─── Dropdown ──────────────────────────────────────── */
.wf-dropdown { position: relative; display: inline-block; }
.wf-dropdown__items { position: absolute; top: 100%; left: 0; min-width: 180px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius-md); box-shadow: var(--shadow-lg); z-index: 50; display: none; margin-top: var(--spacing-xs); }
.wf-dropdown.open .wf-dropdown__items { display: block; }
.wf-dropdown__item { padding: var(--spacing-sm) var(--spacing-md); cursor: pointer; transition: background var(--transition-fast); }
.wf-dropdown__item:hover { background: var(--color-border); }
.wf-dropdown__item--danger { color: var(--color-danger); }
.wf-dropdown__divider { border-top: 1px solid var(--color-border); margin: var(--spacing-xs) 0; }

/* ─── Input ─────────────────────────────────────────── */
.wf-input { display: block; width: 100%; padding: var(--spacing-sm) var(--spacing-md); font-family: var(--font-family); font-size: var(--font-size-base); color: var(--color-text); background: var(--color-background); border: 1px solid var(--color-border); border-radius: var(--radius-md); outline: none; transition: border-color var(--transition-fast), box-shadow var(--transition-fast); }
.wf-input:focus { border-color: var(--color-primary); box-shadow: 0 0 0 3px rgba(59,130,246,0.15); }
.wf-input::placeholder { color: var(--color-text-muted); }
.wf-input--small { padding: var(--spacing-xs) var(--spacing-sm); font-size: var(--font-size-sm); }
.wf-input--large { padding: var(--spacing-md); font-size: var(--font-size-lg); }
.wf-input--rounded { border-radius: var(--radius-full); }
.wf-input--full { width: 100%; }

/* ─── Select ────────────────────────────────────────── */
.wf-select { display: block; width: 100%; padding: var(--spacing-sm) var(--spacing-md); font-family: var(--font-family); font-size: var(--font-size-base); color: var(--color-text); background: var(--color-background); border: 1px solid var(--color-border); border-radius: var(--radius-md); outline: none; appearance: none; cursor: pointer; }
.wf-select:focus { border-color: var(--color-primary); box-shadow: 0 0 0 3px rgba(59,130,246,0.15); }

/* ─── Checkbox ──────────────────────────────────────── */
.wf-checkbox { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; font-size: var(--font-size-base); }
.wf-checkbox input[type="checkbox"] { width: 18px; height: 18px; accent-color: var(--color-primary); cursor: pointer; }

/* ─── Radio ─────────────────────────────────────────── */
.wf-radio { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; font-size: var(--font-size-base); }
.wf-radio input[type="radio"] { width: 18px; height: 18px; accent-color: var(--color-primary); cursor: pointer; }

/* ─── Switch ────────────────────────────────────────── */
.wf-switch { display: inline-flex; align-items: center; gap: var(--spacing-sm); cursor: pointer; }
.wf-switch__track { width: 44px; height: 24px; background: var(--color-border); border-radius: var(--radius-full); position: relative; transition: background var(--transition-fast); }
.wf-switch__thumb { width: 20px; height: 20px; background: #fff; border-radius: var(--radius-full); position: absolute; top: 2px; left: 2px; transition: transform var(--transition-fast); box-shadow: var(--shadow-sm); }
.wf-switch input:checked + .wf-switch__track { background: var(--color-primary); }
.wf-switch input:checked + .wf-switch__track .wf-switch__thumb { transform: translateX(20px); }
.wf-switch input { display: none; }

/* ─── Slider ────────────────────────────────────────── */
.wf-slider { display: flex; align-items: center; gap: var(--spacing-sm); }
.wf-slider input[type="range"] { flex: 1; accent-color: var(--color-primary); }

/* ─── Form ──────────────────────────────────────────── */
.wf-form { display: flex; flex-direction: column; gap: var(--spacing-md); }

/* ─── Alert ─────────────────────────────────────────── */
.wf-alert { padding: var(--spacing-md); border-radius: var(--radius-md); border: 1px solid var(--color-border); background: var(--color-surface); display: flex; align-items: center; gap: var(--spacing-sm); }
.wf-alert--success { background: #f0fdf4; border-color: var(--color-success); color: #166534; }
.wf-alert--danger { background: #fef2f2; border-color: var(--color-danger); color: #991b1b; }
.wf-alert--warning { background: #fffbeb; border-color: var(--color-warning); color: #92400e; }
.wf-alert--info { background: #ecfeff; border-color: var(--color-info); color: #155e75; }
.wf-alert__dismiss { margin-left: auto; cursor: pointer; opacity: 0.6; background: none; border: none; font-size: var(--font-size-lg); color: inherit; }
.wf-alert__dismiss:hover { opacity: 1; }

/* ─── Toast ─────────────────────────────────────────── */
.wf-toast-container { position: fixed; top: var(--spacing-md); right: var(--spacing-md); z-index: 9999; display: flex; flex-direction: column; gap: var(--spacing-sm); }
.wf-toast { padding: var(--spacing-sm) var(--spacing-md); border-radius: var(--radius-md); color: #fff; font-size: var(--font-size-sm); box-shadow: var(--shadow-lg); animation: wf-toast-in 0.3s ease; min-width: 200px; }
.wf-toast--success { background: var(--color-success); }
.wf-toast--danger { background: var(--color-danger); }
.wf-toast--warning { background: var(--color-warning); color: #000; }
.wf-toast--info { background: var(--color-info); }
.wf-toast--exit { animation: wf-toast-out 0.3s ease forwards; }
@keyframes wf-toast-in { from { transform: translateX(100%); opacity: 0; } to { transform: none; opacity: 1; } }
@keyframes wf-toast-out { from { opacity: 1; } to { opacity: 0; transform: translateX(100%); } }

/* ─── Modal ─────────────────────────────────────────── */
.wf-modal { position: fixed; inset: 0; z-index: 1000; display: none; align-items: center; justify-content: center; background: rgba(0,0,0,0.5); }
.wf-modal.open { display: flex; }
.wf-modal__content { background: var(--color-background); border-radius: var(--radius-lg); box-shadow: var(--shadow-xl); max-width: 500px; width: 90%; max-height: 90vh; overflow-y: auto; }
.wf-modal__header { padding: var(--spacing-md); border-bottom: 1px solid var(--color-border); display: flex; justify-content: space-between; align-items: center; }
.wf-modal__header h3 { margin: 0; }
.wf-modal__body { padding: var(--spacing-md); }
.wf-modal__footer { padding: var(--spacing-md); border-top: 1px solid var(--color-border); display: flex; justify-content: flex-end; gap: var(--spacing-sm); }

/* ─── Dialog ────────────────────────────────────────── */
.wf-dialog { position: fixed; inset: 0; z-index: 1000; display: none; align-items: center; justify-content: center; background: rgba(0,0,0,0.5); }
.wf-dialog.open { display: flex; }
.wf-dialog__content { background: var(--color-background); border-radius: var(--radius-lg); box-shadow: var(--shadow-xl); padding: var(--spacing-lg); max-width: 400px; width: 90%; display: flex; flex-direction: column; gap: var(--spacing-md); }

/* ─── Spinner ───────────────────────────────────────── */
.wf-spinner { width: 24px; height: 24px; border: 3px solid var(--color-border); border-top-color: var(--color-primary); border-radius: 50%; animation: wf-spin 0.6s linear infinite; display: inline-block; }
.wf-spinner--large { width: 40px; height: 40px; border-width: 4px; }
@keyframes wf-spin { to { transform: rotate(360deg); } }

/* ─── Progress ──────────────────────────────────────── */
.wf-progress { width: 100%; height: 8px; background: var(--color-border); border-radius: var(--radius-full); overflow: hidden; appearance: none; }
.wf-progress::-webkit-progress-bar { background: var(--color-border); border-radius: var(--radius-full); }
.wf-progress::-webkit-progress-value { background: var(--color-primary); border-radius: var(--radius-full); transition: width var(--transition-normal); }
.wf-progress::-moz-progress-bar { background: var(--color-primary); border-radius: var(--radius-full); }

/* ─── Skeleton ──────────────────────────────────────── */
.wf-skeleton { background: linear-gradient(90deg, var(--color-surface) 25%, var(--color-border) 50%, var(--color-surface) 75%); background-size: 200% 100%; animation: wf-shimmer 1.5s ease-in-out infinite; border-radius: var(--radius-md); }
.wf-skeleton--circle { border-radius: 50%; }
@keyframes wf-shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }

/* ─── Image ─────────────────────────────────────────── */
.wf-image { max-width: 100%; height: auto; display: block; }
.wf-image--rounded { border-radius: var(--radius-md); }
.wf-image--circle { border-radius: 50%; }

/* ─── Carousel ──────────────────────────────────────── */
.wf-carousel { position: relative; overflow: hidden; border-radius: var(--radius-md); }
.wf-carousel__track { display: flex; transition: transform var(--transition-normal); }
.wf-carousel__slide { flex: 0 0 100%; min-width: 100%; }
.wf-carousel__nav { position: absolute; bottom: var(--spacing-sm); left: 50%; transform: translateX(-50%); display: flex; gap: var(--spacing-xs); }
.wf-carousel__dot { width: 8px; height: 8px; border-radius: 50%; background: rgba(255,255,255,0.5); border: none; cursor: pointer; }
.wf-carousel__dot.active { background: #fff; }

/* ─── Typography ────────────────────────────────────── */
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
.wf-text--left { text-align: left; }
.wf-text--right { text-align: right; }
.wf-text--small { font-size: var(--font-size-sm); }
.wf-text--large { font-size: var(--font-size-lg); }
.wf-text--primary { color: var(--color-primary); }
.wf-text--danger { color: var(--color-danger); }
.wf-text--success { color: var(--color-success); }
.wf-text--warning { color: var(--color-warning); }

.wf-heading { font-weight: var(--font-weight-bold); line-height: var(--line-height-tight); margin: 0; }
h1.wf-heading { font-size: var(--font-size-3xl); }
h2.wf-heading { font-size: var(--font-size-2xl); }
h3.wf-heading { font-size: var(--font-size-xl); }
h4.wf-heading { font-size: var(--font-size-lg); }
h5.wf-heading { font-size: var(--font-size-base); }
h6.wf-heading { font-size: var(--font-size-sm); }

.wf-code { font-family: var(--font-family-mono); font-size: var(--font-size-sm); background: var(--color-surface); padding: 0.125rem var(--spacing-xs); border-radius: var(--radius-sm); }
pre.wf-code { padding: var(--spacing-md); overflow-x: auto; display: block; border: 1px solid var(--color-border); border-radius: var(--radius-md); }
.wf-blockquote { border-left: 4px solid var(--color-primary); padding: var(--spacing-sm) var(--spacing-md); margin: 0; color: var(--color-text-muted); font-style: italic; }

/* ─── Variant Colors (shared) ───────────────────────── */
.wf-primary { color: var(--color-primary); }
.wf-secondary { color: var(--color-secondary); }
.wf-success { color: var(--color-success); }
.wf-danger { color: var(--color-danger); }
.wf-warning { color: var(--color-warning); }
.wf-info { color: var(--color-info); }

/* ─── Responsive ────────────────────────────────────── */
@media (max-width: 768px) {
  .wf-row { flex-direction: column; }
  .wf-col--md-12 { flex: 0 0 100%; max-width: 100%; }
  .wf-navbar { flex-wrap: wrap; gap: var(--spacing-sm); }
  .wf-sidebar { width: 100%; min-height: auto; }
}
"#
}
