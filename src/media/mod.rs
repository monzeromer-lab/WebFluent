//! What the build does with an image.
//!
//! `public/` used to be copied byte for byte: a 4 MB photograph reached the
//! reader as a 4 MB photograph, at whatever size the page happened to show
//! it, and the layout jumped when it arrived because nothing knew how tall
//! it was.
//!
//! Now an image the program names is read at build time — its real
//! dimensions, its average colour, a hash of its bytes — and re-encoded at
//! the widths the page asks for, in the formats the config lists. The page
//! gets a `<picture>` with a `srcset`, `width` and `height` from the file
//! itself (so nothing shifts), and a tiny blurred placeholder inlined as a
//! data URI to stand in until the real one lands.
//!
//! The work is cached under `.wf-cache/media/` by content hash, so a build
//! that changes no image does no image work.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::Result;

pub mod encode;
pub mod probe;

/// What the config says to make.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The formats to write, best first: `avif`, `webp`, and the original.
    pub formats: Vec<String>,
    /// The widths to write, when the page does not say.
    pub widths: Vec<u32>,
    /// 1–100, for the formats that take one.
    pub quality: u8,
    /// Whether to do any of it. Off, an image is copied as it always was.
    pub pipeline: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            // WebP is smaller than JPEG everywhere that matters and is
            // understood by every browser that is still shipped.
            formats: vec!["webp".into()],
            widths: vec![480, 960, 1440, 1920],
            quality: 78,
            pipeline: true,
        }
    }
}

/// One image, as the build knows it.
#[derive(Debug, Clone)]
pub struct Asset {
    /// The name the program gave it.
    pub name: String,
    /// Where it was read from.
    pub source: PathBuf,
    /// Its path in the output, with the content hash in the name.
    pub url: String,
    pub width: u32,
    pub height: u32,
    /// The average colour, as `#RRGGBB` — what a placeholder box shows.
    pub color: String,
    /// A very small blurred copy, inlined as a data URI.
    pub placeholder: String,
    /// The widths written, by format: `webp` → `[(480, "/img/hero.a1b2.480.webp"), …]`.
    pub sources: BTreeMap<String, Vec<(u32, String)>>,
}

impl Asset {
    /// The `srcset` for one format.
    pub fn srcset(&self, format: &str) -> Option<String> {
        let widths = self.sources.get(format)?;
        if widths.is_empty() {
            return None;
        }
        Some(
            widths
                .iter()
                .map(|(w, url)| format!("{url} {w}w"))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }

    /// What it is as a value the program can read: `hero.width`,
    /// `hero.color`, and `Image(hero)` reading `hero.src`.
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "src": self.url,
            "width": self.width,
            "height": self.height,
            "color": self.color,
            "placeholder": self.placeholder,
            "srcset": self.srcset(self.sources.keys().next().map(String::as_str).unwrap_or("")).unwrap_or_default(),
            "sources": self.sources.iter().map(|(format, widths)| {
                serde_json::json!({
                    "type": format!("image/{format}"),
                    "srcset": widths.iter().map(|(w, url)| format!("{url} {w}w")).collect::<Vec<_>>().join(", "),
                })
            }).collect::<Vec<_>>(),
        })
    }
}

/// Read an image and write every size and format of it the settings ask
/// for, into `out_dir`, under `img/`.
///
/// What is already in the cache is not made again: the hash is of the
/// file's bytes and the settings, so a build that changes no image and no
/// setting does no work.
pub fn process(
    name: &str,
    source: &Path,
    out_dir: &Path,
    cache_dir: &Path,
    settings: &Settings,
    base_path: &str,
) -> Result<Asset> {
    let bytes = std::fs::read(source)
        .map_err(|e| crate::error::WebFluentError::IoError(format!("{}: {e}", source.display())))?;
    let probed = probe::probe(&bytes, source)?;
    let hash = encode::hash(&bytes, settings);
    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let extension = source
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "png".into());

    let img_dir = out_dir.join("img");
    std::fs::create_dir_all(&img_dir).ok();
    let url_of = |file: &str| format!("{base_path}/img/{file}");

    // The original, under its hashed name, so it can be cached forever.
    let original = format!("{stem}.{hash}.{extension}");
    std::fs::write(img_dir.join(&original), &bytes)
        .map_err(|e| crate::error::WebFluentError::IoError(e.to_string()))?;

    let mut sources: BTreeMap<String, Vec<(u32, String)>> = BTreeMap::new();
    if settings.pipeline {
        // Never up-scale: a 600px photograph has no 1920px version.
        let widths: Vec<u32> = settings
            .widths
            .iter()
            .copied()
            .filter(|w| *w < probed.width)
            .chain(std::iter::once(probed.width))
            .collect();
        for format in &settings.formats {
            let mut made = Vec::new();
            for width in &widths {
                let file = format!("{stem}.{hash}.{width}.{format}");
                let at = img_dir.join(&file);
                let cached = cache_dir.join(&file);
                if !at.exists() {
                    if cached.exists() {
                        std::fs::copy(&cached, &at).ok();
                    } else {
                        let encoded = encode::resized(&probed, *width, format, settings.quality)?;
                        std::fs::write(&at, &encoded)
                            .map_err(|e| crate::error::WebFluentError::IoError(e.to_string()))?;
                        std::fs::create_dir_all(cache_dir).ok();
                        std::fs::write(&cached, &encoded).ok();
                    }
                }
                made.push((*width, url_of(&file)));
            }
            sources.insert(format.clone(), made);
        }
    }

    Ok(Asset {
        name: name.to_string(),
        source: source.to_path_buf(),
        url: url_of(&original),
        width: probed.width,
        height: probed.height,
        color: probed.color.clone(),
        placeholder: probed.placeholder.clone(),
        sources,
    })
}
