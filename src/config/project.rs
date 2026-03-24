use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::error::{WebFluentError, Result};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    #[serde(default = "default_theme_name")]
    pub name: String,
    #[serde(default = "default_theme_mode")]
    pub mode: String,
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub tokens: HashMap<String, String>,
}

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

fn default_locale() -> String { "en".to_string() }
fn default_locales() -> Vec<String> { vec!["en".to_string()] }
fn default_translations_dir() -> String { "src/translations".to_string() }

fn default_version() -> String { "0.1.0".to_string() }
fn default_theme_name() -> String { "default".to_string() }
fn default_theme_mode() -> String { "light".to_string() }
fn default_output_dir() -> String { "./build".to_string() }
fn default_port() -> u16 { 3000 }
fn default_true() -> bool { true }
fn default_lang() -> String { "en".to_string() }

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: default_theme_name(),
            mode: default_theme_mode(),
            extends: None,
            tokens: HashMap::new(),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output: default_output_dir(),
            minify: true,
            sourcemap: false,
            ssg: false,
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
        }
    }
}

impl ProjectConfig {
    pub fn load(project_dir: &Path) -> Result<Self> {
        let config_path = project_dir.join("webfluent.app.json");
        if !config_path.exists() {
            return Err(WebFluentError::ConfigError(
                "webfluent.app.json not found. Run 'webfluent init' to create a project.".to_string()
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
