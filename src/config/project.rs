use crate::error::{Result, WebFluentError};
use crate::themes::BuiltinCss;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// The output format for the build pipeline.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    /// Single-page application with client-side routing.
    Spa,
    /// Static site with pre-rendered HTML pages.
    Static,
    /// PDF document output.
    Pdf,
    /// PDF slide deck output — one Slide = one page, no flow pagination.
    Slides,
    /// Standards-based custom elements: each component `build.elements`
    /// names becomes a `<kebab-case>` tag, usable from React, Vue,
    /// Svelte, Rails, WordPress or a plain HTML page.
    ///
    /// The shared interface between frameworks is the platform, so this is
    /// one output rather than an adapter per framework.
    Elements,
}

fn default_output_type() -> OutputType {
    OutputType::Spa
}

/// Root project configuration, loaded from `webfluent.app.json`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub dev: DevConfig,
    #[serde(default)]
    pub meta: MetaConfig,
    #[serde(default)]
    pub i18n: Option<I18nConfig>,
    /// What every animation does when nothing says otherwise.
    #[serde(default)]
    pub motion: MotionConfig,
    /// Values the program reads as `env.NAME`: an API base, a public key,
    /// a feature switch — set per build, never hard-coded in a page.
    ///
    /// **A value here that the page reads is in the bundle.** Anyone who
    /// opens the site can read it, so only the names this build calls
    /// public may be read from a page, a component, a store or an `api`
    /// block; the rest reach `wf render` and nothing the browser is sent.
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, serde_json::Value>,
    /// The `env` names a page may read, beyond every name that begins
    /// `PUBLIC_`. Anything else is a compile error where it is written,
    /// rather than a key in the bundle nobody noticed.
    #[serde(default)]
    pub public_env: Vec<String>,
    /// A service worker, and what it keeps for when the network is gone.
    /// Absent, nothing is registered and no `sw.js` is written.
    #[serde(default)]
    pub offline: Option<OfflineConfig>,
}

/// `"offline": { … }` — the site working without the network.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OfflineConfig {
    /// The routes a first visit stores, as globs over page paths:
    /// `"/"`, `"/docs/*"`, `"*"` for every route a static build writes.
    #[serde(default = "default_precache")]
    pub precache: Vec<String>,
    /// The page shown for a route that is not stored, when the network is
    /// gone: a page's `path`, stored with the rest.
    #[serde(default)]
    pub fallback: Option<String>,
    /// How a path the build did not write is fetched, by glob:
    /// `network-first`, `cache-first`, `stale-while-revalidate` or
    /// `network-only`. A path none of these name passes through untouched.
    #[serde(default)]
    pub cache: std::collections::BTreeMap<String, String>,
    /// Queue a write made while offline — any method but `GET` and `HEAD`,
    /// through the request engine — and send it when the connection returns.
    #[serde(default)]
    pub sync: bool,
}

fn default_precache() -> Vec<String> {
    vec!["/".to_string()]
}

impl Default for OfflineConfig {
    fn default() -> Self {
        OfflineConfig {
            precache: default_precache(),
            fallback: None,
            cache: Default::default(),
            sync: false,
        }
    }
}

/// The ways a path `offline.cache` names may be fetched.
pub const OFFLINE_STRATEGIES: &[&str] = &[
    "network-first",
    "cache-first",
    "stale-while-revalidate",
    "network-only",
];

/// How long an animation runs and how it is paced, when the element does
/// not say.
///
/// It is shorthand for two design tokens, so a project that would rather
/// write them in its theme still can — and so a style block reads the same
/// values as `$animation-duration-normal` and `$animation-easing-default`.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MotionConfig {
    /// `"180ms"` — the length of an animation with no `duration:` of its own.
    #[serde(default)]
    pub duration: Option<String>,
    /// `"$ease-standard"`, or any CSS timing function — the pacing of an
    /// animation with no `easing:` of its own.
    #[serde(default)]
    pub easing: Option<String>,
}

impl MotionConfig {
    /// The token overrides this asks for, if any.
    ///
    /// `$name` is written as a token, as it is in a style value, and
    /// resolved against the tokens already settled.
    pub fn tokens(&self, resolved: &HashMap<String, String>) -> Vec<(String, String)> {
        let value = |v: &String| match v.strip_prefix('$') {
            Some(name) => resolved
                .get(name)
                .cloned()
                .unwrap_or_else(|| format!("var(--{name})")),
            None => v.clone(),
        };
        let mut out = Vec::new();
        if let Some(d) = &self.duration {
            out.push(("animation-duration-normal".to_string(), value(d)));
        }
        if let Some(e) = &self.easing {
            out.push(("animation-easing-default".to_string(), value(e)));
        }
        out
    }
}

/// Theme configuration — name, mode, custom design tokens, and how much of the
/// engine's built-in stylesheet to emit.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ThemeConfig {
    /// The `Theme` declaration this build uses, by name.
    ///
    /// Optional: a project that declares exactly one theme does not need to name
    /// it, and one that declares none gets the baseline tokens. `"default"` is
    /// the baseline. This used to select one of four palettes the engine carried
    /// in Rust; those are now example `.wf` files you copy and edit.
    #[serde(default)]
    pub name: Option<String>,
    /// Token overrides applied after the theme, for values a machine supplies —
    /// a deploy pipeline injecting a brand colour, or the studio's inspector.
    #[serde(default)]
    pub tokens: HashMap<String, String>,
    /// `full` (default) ships the engine's baseline design; `structural` ships only
    /// layout and language mechanics, for projects that supply their own design.
    /// Absent from existing `wf.json` files, so they keep building as before.
    #[serde(default)]
    pub builtin: BuiltinCss,
    /// The `theme` declaration that stands in under `prefers-color-scheme:
    /// dark`, and whenever `setTheme("dark")` was called: its tokens
    /// override the build's theme.
    #[serde(default)]
    pub dark: Option<String>,
}

/// Build pipeline configuration — output directory, minification, SSG, and PDF settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildConfig {
    /// The components `output_type: "elements"` publishes, by name. Each
    /// becomes a custom element whose tag is its name in kebab-case:
    /// `PriceTag` is `<price-tag>`.
    #[serde(default)]
    pub elements: Vec<String>,
    /// Whether this build's pages carry a `style=` attribute — a style
    /// value that reads state, which has nowhere else to go.
    ///
    /// Not a setting: it is answered from the program before anything is
    /// rendered (`codegen::csp::writes_inline_styles`) and read by the
    /// policy, so what a page ships says what that page contains. It is
    /// skipped by serde for that reason.
    #[serde(skip)]
    pub inline_styles: bool,
    /// The origins this build's `external` modules are imported from.
    ///
    /// Not a setting either: it is read from the program, and the policy
    /// names them under `script-src` so a declared import is never blocked
    /// by the policy shipped beside it.
    #[serde(skip)]
    pub script_origins: Vec<String>,
    #[serde(default = "default_output_dir")]
    pub output: String,
    #[serde(default = "default_true")]
    pub minify: bool,
    #[serde(default)]
    pub sourcemap: bool,
    #[serde(default)]
    pub ssg: bool,
    /// Base path for deployment (e.g., "/WebFluent" for GitHub Pages project sites)
    #[serde(default)]
    pub base_path: String,
    /// Emit a strict `Content-Security-Policy` meta tag, and a `_headers` file
    /// for hosts that read one.
    ///
    /// Off by default: the generated output already satisfies the policy, but a
    /// site that later embeds a third-party script would find it blocked, and
    /// that is a decision to make deliberately rather than inherit.
    #[serde(default)]
    pub csp: bool,
    /// Whether each page is written as its own `pages/<Name>.js`, loaded when
    /// its route is shown, rather than every page shipping in `app.js`.
    #[serde(default = "default_true")]
    pub split: bool,
    /// Whether every text output — HTML, JavaScript, CSS, SVG, JSON, XML —
    /// is also written gzipped beside itself, as `<file>.gz`, for hosts that
    /// serve a precompressed file when they have one (nginx `gzip_static`,
    /// Apache `MultiViews`, most CDNs). `wf serve` sends it too.
    #[serde(default = "default_true")]
    pub compress: bool,
    /// A size a named output must stay under, gzipped — `{ "app.js": "40 kB" }`.
    ///
    /// Advisory: a build over budget prints a warning and goes on. A build
    /// that fails on size is a decision for a project's own CI, which can read
    /// the same numbers from `wf build --stats`.
    #[serde(default)]
    pub budget: std::collections::BTreeMap<String, String>,
    /// Which runtime modules ship: `"auto"` (the default) carries only the
    /// features the program reaches — `each` for a `for`, `router` for more
    /// than one page, `icons` for an `Icon` — and `"full"` carries every one.
    ///
    /// `"full"` is for a site whose own `<script>` calls `WF` in ways a build
    /// cannot see; `wf build --stats` prints what `"auto"` kept and dropped.
    #[serde(default)]
    pub runtime: RuntimeMode,
    /// What the build does with the images the program names.
    #[serde(default)]
    pub media: MediaConfig,
    /// Output type: "spa" (default), "static", "pdf", or "slides"
    #[serde(default = "default_output_type")]
    pub output_type: OutputType,
    /// PDF-specific configuration
    #[serde(default)]
    pub pdf: PdfConfig,
    /// Slides-specific configuration
    #[serde(default)]
    pub slides: SlidesConfig,
}

/// What the build does with the images a program names.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaConfig {
    /// The formats to write, best first.
    #[serde(default = "default_formats")]
    pub formats: Vec<String>,
    /// The widths to write, for a page that does not say.
    #[serde(default = "default_widths")]
    pub widths: Vec<u32>,
    /// 1–100, for the formats that take one.
    #[serde(default = "default_quality")]
    pub quality: u8,
    /// Off, an image is copied as it always was.
    #[serde(default = "default_true")]
    pub pipeline: bool,
}

fn default_formats() -> Vec<String> {
    vec!["webp".to_string()]
}

fn default_widths() -> Vec<u32> {
    vec![480, 960, 1440, 1920]
}

fn default_quality() -> u8 {
    78
}

impl Default for MediaConfig {
    fn default() -> Self {
        MediaConfig {
            formats: default_formats(),
            widths: default_widths(),
            quality: default_quality(),
            pipeline: true,
        }
    }
}

impl MediaConfig {
    /// The settings, as the pipeline takes them.
    pub fn settings(&self) -> crate::media::Settings {
        crate::media::Settings {
            formats: self.formats.clone(),
            widths: self.widths.clone(),
            quality: self.quality,
            pipeline: self.pipeline,
        }
    }
}

/// How much of the JavaScript runtime a build ships.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeMode {
    /// Only the modules the program reaches.
    #[default]
    Auto,
    /// Every module.
    Full,
}

/// PDF output configuration — page size, margins, fonts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfConfig {
    #[serde(default = "default_page_size")]
    pub page_size: String,
    #[serde(default = "default_margins")]
    pub margins: PdfMargins,
    #[serde(default = "default_font")]
    pub default_font: String,
    #[serde(default = "default_font_size")]
    pub default_font_size: f64,
    #[serde(default)]
    pub output_filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdfMargins {
    #[serde(default = "default_margin")]
    pub top: f64,
    #[serde(default = "default_margin")]
    pub bottom: f64,
    #[serde(default = "default_margin")]
    pub left: f64,
    #[serde(default = "default_margin")]
    pub right: f64,
}

fn default_page_size() -> String {
    "A4".to_string()
}
fn default_margins() -> PdfMargins {
    PdfMargins::default()
}
fn default_font() -> String {
    "Helvetica".to_string()
}
fn default_font_size() -> f64 {
    12.0
}
fn default_margin() -> f64 {
    72.0
}

/// Slides output configuration — page size, fonts, slide chrome.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlidesConfig {
    /// Named size ("16:9", "4:3", "A4-landscape") or "WIDTHxHEIGHT" in points.
    #[serde(default = "default_slide_size")]
    pub size: String,
    /// Explicit width override in points (wins over `size`).
    #[serde(default)]
    pub width: Option<f64>,
    /// Explicit height override in points (wins over `size`).
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default = "default_slide_font")]
    pub default_font: String,
    #[serde(default = "default_slide_font_size")]
    pub default_font_size: f64,
    #[serde(default = "default_slide_margin")]
    pub margin: f64,
    /// Show "n / total" page numbers in the bottom-right corner.
    #[serde(default)]
    pub show_slide_numbers: bool,
    /// Footer text shown bottom-left of every slide.
    #[serde(default)]
    pub footer_text: Option<String>,
    /// Deck-wide page background color (e.g. "#1A1A19"). Per-slide
    /// `style { background }` overrides this for individual slides.
    #[serde(default)]
    pub background_color: Option<String>,
    /// Override slide-chrome (page numbers + footer text) color. If None,
    /// the chrome color is auto-derived from the slide's background luminance.
    #[serde(default)]
    pub chrome_color: Option<String>,
    #[serde(default)]
    pub output_filename: Option<String>,
}

fn default_slide_size() -> String {
    "16:9".to_string()
}
fn default_slide_font() -> String {
    "Helvetica".to_string()
}
fn default_slide_font_size() -> f64 {
    24.0
}
fn default_slide_margin() -> f64 {
    60.0
}

impl Default for SlidesConfig {
    fn default() -> Self {
        Self {
            size: default_slide_size(),
            width: None,
            height: None,
            default_font: default_slide_font(),
            default_font_size: default_slide_font_size(),
            margin: default_slide_margin(),
            show_slide_numbers: false,
            footer_text: None,
            background_color: None,
            chrome_color: None,
            output_filename: None,
        }
    }
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            page_size: default_page_size(),
            margins: PdfMargins::default(),
            default_font: default_font(),
            default_font_size: default_font_size(),
            output_filename: None,
        }
    }
}

impl Default for PdfMargins {
    fn default() -> Self {
        Self {
            top: default_margin(),
            bottom: default_margin(),
            left: default_margin(),
            right: default_margin(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DevConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub hot_reload: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaConfig {
    /// The subresource-integrity hash of each external asset, by its URL:
    /// `"https://cdn/x.css": "sha384-…"`.
    ///
    /// A stylesheet or font from somewhere else is code that origin can
    /// change after you have read it. A hash makes the browser check that
    /// it has not. The build never fetches the file to work the hash out —
    /// that would make a build depend on the network — so it is declared,
    /// and a declared asset without one draws a warning.
    #[serde(default)]
    pub integrity: HashMap<String, String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub favicon: String,
    #[serde(default = "default_lang")]
    pub lang: String,

    /// The site's own origin, e.g. `https://example.com`.
    ///
    /// A canonical link, an `og:url`, a sitemap entry and an hreflang alternate
    /// all have to be absolute — Google's guidance is explicit that a relative
    /// canonical "can cause problems in the long run". Without this the engine
    /// emits none of them rather than emitting a wrong one.
    #[serde(default)]
    pub site_url: String,

    /// The organisation or person behind the site, for `Organization`
    /// structured data and `og:site_name`. Defaults to the project name.
    #[serde(default)]
    pub site_name: String,

    /// A default sharing image for pages that do not name their own.
    #[serde(default)]
    pub image: String,

    /// Emit `sitemap.xml` and `robots.txt`. On by default for static builds,
    /// which are the ones a crawler can read.
    #[serde(default = "default_true")]
    pub sitemap: bool,

    /// Web-font stylesheets to load, by URL — a Google Fonts `css2?family=…`
    /// link, or a stylesheet on the site's own origin. Each is linked in the
    /// head with a `preconnect` to its origin, ahead of `styles.css`, so the
    /// fonts a theme names are actually fetched. The baseline theme names
    /// system fonts on purpose; a theme that names a web font lists it here.
    #[serde(default)]
    pub fonts: Vec<String>,

    /// Extra stylesheets to link before `styles.css`, by URL or site-relative
    /// path (a file in `public/`) — a sheet the build does not own. A sheet
    /// the project writes itself belongs under `src/` as a `.css` file,
    /// where it is bundled into `styles.css`.
    #[serde(default)]
    pub stylesheets: Vec<String>,
}

/// The origin (`scheme://host`) of an absolute URL, or `None` for a
/// site-relative path.
pub fn url_origin(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let host = rest.split('/').next()?;
    if host.is_empty() {
        return None;
    }
    Some(format!("{}://{}", scheme, host))
}

/// The origins a font or stylesheet URL needs the policy to allow: its own,
/// plus the font origin Google Fonts serves files from, which its stylesheet
/// references.
fn asset_origins(url: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(origin) = url_origin(url) {
        if origin == "https://fonts.googleapis.com" {
            out.push("https://fonts.gstatic.com".to_string());
        }
        out.push(origin);
    }
    out
}

/// The `Content-Security-Policy` for a project: [`CSP_POLICY`], widened by
/// exactly the origins its declared fonts and stylesheets are served from.
///
/// A policy that ignored `meta.fonts` would block the very stylesheet the
/// config asked for, and the failure would show up as a silent fallback font.
/// The policy a `<meta>` tag may carry.
///
/// `frame-ancestors` is **ignored** in a meta tag — a browser only honours
/// it in a response header — so leaving it there is a policy that says one
/// thing and delivers another. It is written to `_headers` instead, where
/// it works, and left out here so nothing claims otherwise.
pub fn csp_meta_policy(config: &ProjectConfig) -> String {
    csp_policy(config)
        .split("; ")
        .filter(|d| !d.starts_with("frame-ancestors"))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn csp_policy(config: &ProjectConfig) -> String {
    let meta = &config.meta;
    let mut style: Vec<String> = Vec::new();
    // A style value that reads state is written on the element; a policy
    // that forbade it would be one the pages break.
    if config.build.inline_styles {
        style.push("'unsafe-inline'".to_string());
    }
    let mut font: Vec<String> = Vec::new();
    for url in meta.fonts.iter().chain(meta.stylesheets.iter()) {
        for origin in asset_origins(url) {
            let is_font_files = origin == "https://fonts.gstatic.com";
            if !is_font_files && !style.contains(&origin) {
                style.push(origin.clone());
            }
            if !font.contains(&origin) {
                font.push(origin);
            }
        }
    }
    let mut policy = CSP_POLICY.to_string();
    if !style.is_empty() {
        policy = policy.replace(
            "style-src 'self';",
            &format!("style-src 'self' {};", style.join(" ")),
        );
    }
    if !font.is_empty() {
        policy = policy.replace(
            "font-src 'self';",
            &format!("font-src 'self' {};", font.join(" ")),
        );
    }
    // And the origins a program's `external` modules are imported from,
    // which the config cannot know because they are in the source.
    let mut origins: Vec<&str> = config
        .build
        .script_origins
        .iter()
        .map(String::as_str)
        .collect();
    origins.sort();
    origins.dedup();
    if !origins.is_empty() {
        policy = policy.replace(
            "script-src 'self';",
            &format!("script-src 'self' {};", origins.join(" ")),
        );
    }
    policy
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct I18nConfig {
    #[serde(default = "default_locale")]
    pub default_locale: String,
    #[serde(default = "default_locales")]
    pub locales: Vec<String>,
    #[serde(default = "default_translations_dir")]
    pub dir: String,
}

fn default_locale() -> String {
    "en".to_string()
}
fn default_locales() -> Vec<String> {
    vec!["en".to_string()]
}
fn default_translations_dir() -> String {
    "src/translations".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}
/// The policy the generated output satisfies without exception.
///
/// `script-src 'self'` and `style-src 'self'` are only possible because the
/// compiler emits external files and binds events with `addEventListener`
/// rather than inline `on*` attributes — the refactor most codebases have to
/// make before a strict policy is even reachable.
pub const CSP_POLICY: &str = "default-src 'self'; script-src 'self'; style-src 'self'; \
img-src 'self' data:; font-src 'self'; object-src 'none'; base-uri 'none'; \
frame-ancestors 'none'; form-action 'self'";

fn default_output_dir() -> String {
    "./build".to_string()
}
fn default_port() -> u16 {
    3000
}
fn default_true() -> bool {
    true
}
fn default_lang() -> String {
    "en".to_string()
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            inline_styles: false,
            script_origins: Vec::new(),
            output: default_output_dir(),
            minify: true,
            sourcemap: false,
            ssg: false,
            base_path: String::new(),
            csp: false,
            split: true,
            compress: true,
            budget: Default::default(),
            media: MediaConfig::default(),
            runtime: RuntimeMode::Auto,
            output_type: OutputType::Spa,
            pdf: PdfConfig::default(),
            slides: SlidesConfig::default(),
        }
    }
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            hot_reload: true,
        }
    }
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            integrity: HashMap::new(),
            title: String::new(),
            description: String::new(),
            favicon: String::new(),
            lang: default_lang(),
            site_url: String::new(),
            site_name: String::new(),
            image: String::new(),
            sitemap: true,
            fonts: Vec::new(),
            stylesheets: Vec::new(),
        }
    }
}

impl ProjectConfig {
    /// Whether a page may read `env.NAME`.
    ///
    /// Everything a page reads is in the bundle, so the rule is a list,
    /// not a judgement: a name that begins `PUBLIC_` says so in its own
    /// spelling, and `public_env` names the rest.
    pub fn env_is_public(&self, name: &str) -> bool {
        name.starts_with("PUBLIC_") || self.public_env.iter().any(|n| n == name)
    }

    /// The `env` values a page may read, and so the only ones the bundle may
    /// carry. The rest reach `wf render`, which runs on a server.
    ///
    /// The whole map used to be written into `app.js` as `const env = {…}`:
    /// the compiler refused a page that read `env.STRIPE_SECRET`, and then
    /// shipped the secret to every reader anyway.
    pub fn public_env_values(&self) -> std::collections::BTreeMap<String, serde_json::Value> {
        self.env
            .iter()
            .filter(|(name, _)| self.env_is_public(name))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn load(project_dir: &Path) -> Result<Self> {
        let config_path = project_dir.join("webfluent.app.json");
        if !config_path.exists() {
            return Err(WebFluentError::ConfigError(
                "webfluent.app.json not found. Run 'wf init' to create a project.".to_string(),
            ));
        }
        let content = fs::read_to_string(&config_path)?;
        let config: ProjectConfig = serde_json::from_str(&content).map_err(|e| {
            WebFluentError::ConfigError(format!("Failed to parse webfluent.app.json: {}", e))
        })?;
        Ok(config)
    }

    /// The keys of `webfluent.app.json` that nothing reads, each with the
    /// path to it and the key it most likely meant.
    ///
    /// A key the loader does not know was dropped without a word — the
    /// documentation site's own config carried `"defaultLocale"` for months,
    /// and the build read `default_locale` and never said so. The parsed
    /// config, written back out, holds every key there is (a free-form map
    /// such as `env` or `theme.tokens` holds exactly what it was given), so a
    /// key in the file that is not in that is one nothing reads.
    pub fn unknown_keys(project_dir: &Path) -> Vec<String> {
        let Ok(content) = fs::read_to_string(project_dir.join("webfluent.app.json")) else {
            return Vec::new();
        };
        let Ok(given) = serde_json::from_str::<serde_json::Value>(&content) else {
            return Vec::new();
        };
        let Ok(parsed) = serde_json::from_value::<ProjectConfig>(given.clone()) else {
            return Vec::new();
        };
        let Ok(known) = serde_json::to_value(&parsed) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        unknown_in(&given, &known, "", &mut out);
        out
    }

    /// Adds to `env` what the build's surroundings say: a `.env` file beside
    /// the config, then the shell's own variables — so a pipeline can give
    /// staging and production different values without editing a file.
    ///
    /// The later source wins: the config, then `.env`, then the shell. The
    /// shell supplies a name the config or `.env` already declares, or one
    /// that is public (`PUBLIC_…`, or listed in `public_env`) — not every
    /// variable the build happens to run with, which would put `PATH` and
    /// `HOME` in `wf audit` and in reach of `wf render`.
    pub fn resolve_env(&mut self, project_dir: &Path) {
        self.resolve_env_from(project_dir, std::env::vars());
    }

    /// [`resolve_env`](Self::resolve_env) with the shell's variables given.
    pub fn resolve_env_from(
        &mut self,
        project_dir: &Path,
        shell: impl IntoIterator<Item = (String, String)>,
    ) {
        if let Ok(text) = fs::read_to_string(project_dir.join(".env")) {
            for (name, value) in parse_dotenv(&text) {
                self.env.insert(name, serde_json::Value::String(value));
            }
        }
        for (name, value) in shell {
            if self.env.contains_key(&name) || self.env_is_public(&name) {
                self.env.insert(name, serde_json::Value::String(value));
            }
        }
    }

    pub fn default_config(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: default_version(),
            author: String::new(),
            theme: ThemeConfig::default(),
            build: BuildConfig::default(),
            dev: DevConfig::default(),
            i18n: None,
            public_env: Vec::new(),
            motion: MotionConfig::default(),
            meta: MetaConfig {
                title: name.to_string(),
                ..Default::default()
            },
            env: Default::default(),
            offline: None,
        }
    }
}

/// Every key of `given` that `known` lacks, as a path (`i18n.defaultLocale`),
/// with the key at that level it most likely meant.
fn unknown_in(
    given: &serde_json::Value,
    known: &serde_json::Value,
    at: &str,
    out: &mut Vec<String>,
) {
    let (Some(given), Some(known)) = (given.as_object(), known.as_object()) else {
        return;
    };
    for (key, value) in given {
        let path = if at.is_empty() {
            key.clone()
        } else {
            format!("{at}.{key}")
        };
        match known.get(key) {
            Some(expected) => unknown_in(value, expected, &path, out),
            None => {
                let hint = known
                    .keys()
                    .map(|k| {
                        (
                            crate::linter::vocabulary::levenshtein(&normalise(key), &normalise(k)),
                            k,
                        )
                    })
                    .filter(|(d, _)| *d <= 3)
                    .min()
                    .map(|(_, k)| format!(" — did you mean `{k}`?"))
                    .unwrap_or_default();
                out.push(format!(
                    "`{path}` is not a setting, and nothing reads it{hint}"
                ));
            }
        }
    }
}

/// A key without its case or separators, so `defaultLocale` is one step
/// from `default_locale` rather than four.
fn normalise(key: &str) -> String {
    key.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

/// The `NAME=value` lines of a `.env` file. A `#` line and a blank one are
/// skipped, an `export ` before the name is allowed, and a value in quotes
/// loses them (in double quotes, `\n` is a line break).
fn parse_dotenv(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let value = value.trim();
        let value = if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value[1..value.len() - 1].replace("\\n", "\n")
        } else if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
            value[1..value.len() - 1].to_string()
        } else {
            // An unquoted value ends at a ` #` comment.
            value
                .split(" #")
                .next()
                .unwrap_or("")
                .trim_end()
                .to_string()
        };
        out.push((name.to_string(), value));
    }
    out
}

#[cfg(test)]
mod env_and_key_tests {
    use super::*;

    struct Dir(std::path::PathBuf);
    impl Dir {
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn project(config: &str, dotenv: Option<&str>) -> Dir {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "wf-config-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("webfluent.app.json"), config).unwrap();
        if let Some(text) = dotenv {
            fs::write(dir.join(".env"), text).unwrap();
        }
        Dir(dir)
    }

    #[test]
    fn the_shell_beats_dotenv_which_beats_the_config() {
        let dir = project(
            r#"{ "name": "t", "env": { "PUBLIC_API": "/config", "SECRET": "from-config" } }"#,
            Some(
                "# a comment\nexport PUBLIC_API=\"/dotenv\"\nPUBLIC_MODE='quiet'\nOTHER=x # trailing\n",
            ),
        );
        let mut config = ProjectConfig::load(dir.path()).unwrap();
        config.resolve_env_from(
            dir.path(),
            [
                ("PUBLIC_API".to_string(), "/shell".to_string()),
                ("SECRET".to_string(), "from-shell".to_string()),
                ("HOME".to_string(), "/home/someone".to_string()),
            ],
        );
        assert_eq!(config.env["PUBLIC_API"], "/shell");
        assert_eq!(config.env["PUBLIC_MODE"], "quiet");
        assert_eq!(config.env["OTHER"], "x");
        assert_eq!(
            config.env["SECRET"], "from-shell",
            "a declared name is overridden"
        );
        assert!(
            !config.env.contains_key("HOME"),
            "an undeclared, non-public shell variable stays out"
        );
    }

    #[test]
    fn a_key_nothing_reads_is_named_with_the_one_it_meant() {
        let dir = project(
            r#"{ "name": "t", "i18n": { "defaultLocale": "en", "locales": ["en"] },
                 "build": { "minfy": true }, "env": { "ANYTHING": 1 },
                 "theme": { "tokens": { "brand-new-token": "red" } } }"#,
            None,
        );
        let found = ProjectConfig::unknown_keys(dir.path());
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(
            found
                .iter()
                .any(|f| f.contains("`i18n.defaultLocale`") && f.contains("`default_locale`")),
            "{found:?}"
        );
        assert!(
            found
                .iter()
                .any(|f| f.contains("`build.minfy`") && f.contains("`minify`")),
            "{found:?}"
        );
    }

    #[test]
    fn a_config_that_is_all_known_keys_draws_nothing() {
        let dir = project(
            r#"{ "name": "t", "version": "1", "build": { "ssg": true, "budget": { "app.js": "40 kB" } },
                 "offline": { "precache": ["/"], "cache": { "/api/*": "network-first" } } }"#,
            None,
        );
        assert!(ProjectConfig::unknown_keys(dir.path()).is_empty());
    }
}

#[cfg(test)]
mod head_asset_tests {
    use super::*;

    fn meta(fonts: &[&str], sheets: &[&str]) -> ProjectConfig {
        let mut config = ProjectConfig::default_config("t");
        config.meta = MetaConfig {
            fonts: fonts.iter().map(|s| s.to_string()).collect(),
            stylesheets: sheets.iter().map(|s| s.to_string()).collect(),
            ..MetaConfig::default()
        };
        config
    }

    #[test]
    fn no_assets_leaves_the_baseline_policy_alone() {
        assert_eq!(csp_policy(&meta(&[], &[])), CSP_POLICY);
        assert_eq!(csp_policy(&meta(&[], &["/base.css"])), CSP_POLICY);
    }

    #[test]
    fn a_google_font_widens_style_and_font_sources() {
        let policy = csp_policy(&meta(
            &["https://fonts.googleapis.com/css2?family=Manrope&display=swap"],
            &[],
        ));
        assert!(
            policy.contains("style-src 'self' https://fonts.googleapis.com;"),
            "{policy}"
        );
        assert!(
            policy.contains(
                "font-src 'self' https://fonts.gstatic.com https://fonts.googleapis.com;"
            ),
            "{policy}"
        );
    }

    #[test]
    fn url_origin_reads_scheme_and_host_only() {
        assert_eq!(
            url_origin("https://cdn.example.com/a/b.css").as_deref(),
            Some("https://cdn.example.com")
        );
        assert_eq!(url_origin("/base.css"), None);
    }
}
