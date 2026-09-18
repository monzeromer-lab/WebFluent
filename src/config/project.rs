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
}

/// Build pipeline configuration — output directory, minification, SSG, and PDF settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildConfig {
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
    /// path (a file in `public/`). For the few things no element-level
    /// `style { }` can say — `html { background }`, `::selection` — not for
    /// component styling, which belongs in `.wf` source.
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
pub fn csp_policy(meta: &MetaConfig) -> String {
    let mut style: Vec<String> = Vec::new();
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
            output: default_output_dir(),
            minify: true,
            sourcemap: false,
            ssg: false,
            base_path: String::new(),
            csp: false,
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

    pub fn default_config(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: default_version(),
            author: String::new(),
            theme: ThemeConfig::default(),
            build: BuildConfig::default(),
            dev: DevConfig::default(),
            i18n: None,
            meta: MetaConfig {
                title: name.to_string(),
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod head_asset_tests {
    use super::*;

    fn meta(fonts: &[&str], sheets: &[&str]) -> MetaConfig {
        MetaConfig {
            fonts: fonts.iter().map(|s| s.to_string()).collect(),
            stylesheets: sheets.iter().map(|s| s.to_string()).collect(),
            ..MetaConfig::default()
        }
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
