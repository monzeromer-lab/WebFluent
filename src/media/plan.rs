//! What the pipeline will make of an image, before it makes it: each file
//! it would write, at each width and in each format, with its size — so
//! the cost of a 4 MB photograph is seen where it is added, not in a build
//! log later.
//!
//! A size is exact when the build has already written that file (it is in
//! `.wf-cache/media/` under the name the build gives it) or when the image
//! is small enough to encode outright; otherwise it is estimated: the image
//! is resized to that width, a few strips across it are encoded, and what
//! they come to is scaled by the height they stand for.
//! Every size says which it is.

use std::path::Path;

use super::probe::{self, Probed};
use super::{Settings, encode, widths_for};
use crate::error::Result;

/// Below this many pixels, across every width of a format, the files are
/// encoded outright: a small image costs less to encode than to explain.
const EXACT_PIXELS: u64 = 400_000;

/// How many strips an estimate encodes, and how tall each is.
const STRIPS: u32 = 4;
const STRIP_HEIGHT: u32 = 32;

/// One file the pipeline would write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFile {
    pub width: u32,
    pub height: u32,
    /// Its name under `img/`, as the build names it: `hero.1a2b3c4d.640.webp`.
    pub name: String,
    pub bytes: u64,
    /// Whether `bytes` is an estimate rather than the file's own size.
    pub estimated: bool,
}

/// What one format would come to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatPlan {
    pub format: String,
    pub files: Vec<PlannedFile>,
    /// Why nothing is written in it — a format the build cannot write — as
    /// the build itself would say it.
    pub error: Option<String>,
    /// The widths it would come to no less than the original at, which the
    /// build does not write: the original serves there.
    pub dropped: Vec<u32>,
}

impl FormatPlan {
    pub fn bytes(&self) -> u64 {
        self.files.iter().map(|f| f.bytes).sum()
    }
}

/// What the pipeline will make of one image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub width: u32,
    pub height: u32,
    /// The original, under its hashed name: the build writes it as well,
    /// for the `<img>` a browser without `<picture>` reads.
    pub original: PlannedFile,
    /// Off, the image is copied as it is and nothing else is made.
    pub pipeline: bool,
    pub widths: Vec<u32>,
    pub quality: u8,
    pub formats: Vec<FormatPlan>,
}

impl Plan {
    /// Every file made from the original, in every format.
    pub fn made(&self) -> impl Iterator<Item = &PlannedFile> {
        self.formats.iter().flat_map(|f| f.files.iter())
    }

    /// What those files come to.
    pub fn bytes(&self) -> u64 {
        self.made().map(|f| f.bytes).sum()
    }

    /// Whether any size is an estimate.
    pub fn estimated(&self) -> bool {
        self.made().any(|f| f.estimated)
    }

    /// In a line: `3 widths × 2 formats · 6 files · 84 kB`.
    pub fn summary(&self) -> String {
        if !self.pipeline {
            return format!("copied as it is · {}", size_text(self.original.bytes));
        }
        let formats = self.formats.iter().filter(|f| !f.files.is_empty()).count();
        if formats == 0 {
            return format!(
                "nothing smaller than the original · {}",
                size_text(self.original.bytes)
            );
        }
        let n = self.made().count();
        format!(
            "{} × {} · {} · {}{}",
            plural(self.widths.len(), "width", "widths"),
            plural(formats, "format", "formats"),
            plural(n, "file", "files"),
            if self.estimated() { "about " } else { "" },
            size_text(self.bytes())
        )
    }

    /// What `wf assets --plan` says after the files: what is written
    /// beside them, what is not and why, and whether a size is a guess.
    pub fn said(&self) -> Vec<String> {
        let original = format!(
            "{} ({})",
            self.original.name,
            size_text(self.original.bytes)
        );
        if !self.pipeline {
            return vec![format!(
                "`build.media.pipeline` is off: the original is copied as it is, as {original}."
            )];
        }
        let mut out = vec![format!("The original is written too, as {original}.")];
        for f in &self.formats {
            if f.dropped.is_empty() || f.error.is_some() {
                continue;
            }
            let widths: Vec<String> = f.dropped.iter().map(u32::to_string).collect();
            out.push(if f.files.is_empty() {
                format!(
                    "No {} is written: at every width it would be no smaller than the original, which serves instead.",
                    f.format
                )
            } else {
                format!(
                    "{} at {} is not written: it would be no smaller than the original, which serves there.",
                    f.format,
                    widths.join(" · ")
                )
            });
        }
        if self.estimated() {
            out.push(
                "A size marked `about` is estimated; `wf build` writes the files and makes it exact."
                    .to_string(),
            );
        }
        out
    }

    pub fn as_json(&self) -> serde_json::Value {
        let file = |f: &PlannedFile| {
            serde_json::json!({
                "width": f.width,
                "height": f.height,
                "name": f.name,
                "bytes": f.bytes,
                "estimated": f.estimated,
            })
        };
        serde_json::json!({
            "width": self.width,
            "height": self.height,
            "pipeline": self.pipeline,
            "widths": self.widths,
            "quality": self.quality,
            "original": file(&self.original),
            "formats": self.formats.iter().map(|f| serde_json::json!({
                "format": f.format,
                "files": f.files.iter().map(file).collect::<Vec<_>>(),
                "bytes": f.bytes(),
                "error": f.error,
                "dropped": f.dropped,
            })).collect::<Vec<_>>(),
            "said": self.said(),
            "files": self.made().count(),
            "bytes": self.bytes(),
            "estimated": self.estimated(),
            "summary": self.summary(),
        })
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// A size as a person reads it: `812 B`, `84 kB`, `4.2 MB`.
pub fn size_text(bytes: u64) -> String {
    if bytes < 1000 {
        format!("{bytes} B")
    } else if bytes < 1_000_000 {
        format!("{} kB", (bytes as f64 / 1000.0).round() as u64)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    }
}

/// What the pipeline will make of the image in `bytes`, read from `source`
/// (whose name the files are named after). `cache` is the project's
/// `.wf-cache/media/`, where a file the build already wrote is found and
/// its size read exactly.
pub fn plan(
    bytes: &[u8],
    source: &Path,
    settings: &Settings,
    cache: Option<&Path>,
) -> Result<Plan> {
    let probed = probe::probe(bytes, source)?;
    Ok(plan_probed(&probed, bytes, source, settings, cache))
}

/// [`plan`], for an image already decoded.
pub fn plan_probed(
    probed: &Probed,
    bytes: &[u8],
    source: &Path,
    settings: &Settings,
    cache: Option<&Path>,
) -> Plan {
    let hash = encode::hash(bytes, settings);
    let stem = source
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = source
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "png".into());
    let original = PlannedFile {
        width: probed.width,
        height: probed.height,
        name: format!("{stem}.{hash}.{extension}"),
        bytes: bytes.len() as u64,
        estimated: false,
    };
    let widths = widths_for(settings, probed.width);
    let height_at = |w: u32| -> u32 {
        if w >= probed.width {
            probed.height
        } else {
            (((w as f64) * probed.height as f64 / probed.width as f64).round() as u32).max(1)
        }
    };
    let mut formats = Vec::new();
    if settings.pipeline {
        for format in &settings.formats {
            if !super::FORMATS.contains(&format.as_str()) {
                formats.push(FormatPlan {
                    format: format.clone(),
                    files: Vec::new(),
                    dropped: Vec::new(),
                    error: Some(format!(
                        "`{format}` is not a format the build can write; it writes webp, png and jpeg"
                    )),
                });
                continue;
            }
            let pixels: u64 = widths
                .iter()
                .map(|w| u64::from(*w) * u64::from(height_at(*w)))
                .sum();
            let exact = pixels <= EXACT_PIXELS;
            let mut files = Vec::new();
            let mut dropped = Vec::new();
            let mut error = None;
            for w in &widths {
                let h = height_at(*w);
                let name = format!("{stem}.{hash}.{w}.{format}");
                let cached = cache
                    .map(|c| c.join(&name))
                    .and_then(|p| std::fs::metadata(p).ok())
                    .map(|m| m.len());
                let (size, estimated) = match cached {
                    Some(n) => (n, false),
                    None if exact => match encode::resized(probed, *w, format, settings.quality) {
                        Ok(out) => (out.len() as u64, false),
                        Err(e) => {
                            error = Some(e.to_string());
                            break;
                        }
                    },
                    None => match estimate(probed, *w, h, format, settings.quality) {
                        Some(n) => (n, true),
                        None => {
                            error = Some(format!("the image could not be written as {format}"));
                            break;
                        }
                    },
                };
                // As the build decides: no saving, not written.
                if super::no_saving(size, original.bytes) {
                    dropped.push(*w);
                    continue;
                }
                files.push(PlannedFile {
                    width: *w,
                    height: h,
                    name,
                    bytes: size,
                    estimated,
                });
            }
            if error.is_some() {
                files.clear();
                dropped.clear();
            }
            formats.push(FormatPlan {
                format: format.clone(),
                files,
                dropped,
                error,
            });
        }
    }
    Plan {
        width: probed.width,
        height: probed.height,
        original,
        pipeline: settings.pipeline,
        widths: if settings.pipeline {
            widths
        } else {
            Vec::new()
        },
        quality: settings.quality,
        formats,
    }
}

/// About what `probed` at `width` comes to in `format`: a few strips across
/// the resized image, encoded, scaled to the whole height. A strip is as
/// wide as the file, so a column's detail is counted at the scale the file
/// has it.
fn estimate(probed: &Probed, width: u32, height: u32, format: &str, quality: u8) -> Option<u64> {
    use image::GenericImageView;
    let scaled = if width >= probed.width {
        probed.image.clone()
    } else {
        probed
            .image
            .resize_exact(width, height, image::imageops::FilterType::Lanczos3)
    };
    let strip = STRIP_HEIGHT.min(height);
    let n = STRIPS.min(height / strip).max(1);
    let mut bytes = 0u64;
    for i in 0..n {
        // Spread down the image, the first at the top and the last at the
        // bottom.
        let y = if n == 1 {
            0
        } else {
            (height - strip) * i / (n - 1)
        };
        let piece = scaled.view(0, y, width, strip).to_image();
        let piece = Probed {
            image: image::DynamicImage::ImageRgba8(piece),
            width,
            height: strip,
            color: String::new(),
            placeholder: String::new(),
        };
        bytes += encode::resized(&piece, width, format, quality).ok()?.len() as u64;
    }
    // Each strip is a whole file, header and all; the file the build
    // writes has one header, not one per strip.
    let blank = Probed {
        image: image::DynamicImage::new_rgb8(1, 1),
        width: 1,
        height: 1,
        color: String::new(),
        placeholder: String::new(),
    };
    let header = encode::resized(&blank, 1, format, quality).ok()?.len() as u64;
    let body = bytes.saturating_sub(header * u64::from(n));
    Some(header + (body as f64 * f64::from(height) / f64::from(n * strip)).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_fn(w, h, |x, y| {
            image::Rgb([
                (x * 7 % 256) as u8,
                (y * 5 % 256) as u8,
                ((x ^ y) % 256) as u8,
            ])
        });
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn a_size_reads_as_a_person_says_it() {
        assert_eq!(size_text(812), "812 B");
        assert_eq!(size_text(84_400), "84 kB");
        assert_eq!(size_text(4_210_000), "4.2 MB");
    }

    #[test]
    fn the_widths_are_the_settings_order_never_up_scaled() {
        let s = Settings {
            widths: vec![960, 480, 3000],
            ..Settings::default()
        };
        assert_eq!(widths_for(&s, 1200), [960, 480, 1200]);
        assert_eq!(widths_for(&s, 400), [400]);
    }

    /// The names the plan gives are the names `process` writes, so a size
    /// read from the cache is the size of that very file.
    #[test]
    fn the_plan_names_what_process_writes() {
        let bytes = png(680, 200);
        let dir = std::env::temp_dir().join(format!("wf-plan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("pic.png");
        std::fs::write(&source, &bytes).unwrap();
        let settings = Settings {
            formats: vec!["webp".into(), "jpeg".into()],
            widths: vec![320, 640],
            quality: 70,
            pipeline: true,
            literal_src: true,
        };
        let before = plan(&bytes, &source, &settings, Some(&dir.join("cache"))).unwrap();
        // Small enough to encode outright.
        assert!(!before.estimated());
        assert_eq!(before.widths, [320, 640, 680]);
        let asset = crate::media::process(
            "pic",
            &source,
            &dir.join("out"),
            &dir.join("cache"),
            &settings,
            "",
        )
        .unwrap();
        let mut written: Vec<String> = asset
            .sources
            .values()
            .flatten()
            .map(|(_, url)| url.trim_start_matches("/img/").to_string())
            .collect();
        written.sort();
        let mut named: Vec<String> = before.made().map(|f| f.name.clone()).collect();
        named.sort();
        assert_eq!(written, named);
        assert_eq!(asset.url, format!("/img/{}", before.original.name));
        // Encoded outright, the size is the file's to the byte.
        for f in before.made() {
            let real = std::fs::metadata(dir.join("out/img").join(&f.name))
                .unwrap()
                .len();
            assert_eq!(f.bytes, real, "{}", f.name);
            let height = (f64::from(f.width) * 200.0 / 680.0).round() as u32;
            assert_eq!(f.height, height, "{}", f.name);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_large_image_is_estimated_and_says_so() {
        let bytes = png(1600, 900);
        let settings = Settings::default();
        let p = plan(&bytes, Path::new("big.png"), &settings, None).unwrap();
        assert!(p.estimated());
        assert_eq!(p.widths, [480, 960, 1440, 1600]);
        assert!(
            p.summary()
                .starts_with("4 widths × 1 format · 4 files · about "),
            "{}",
            p.summary()
        );
        let json = p.as_json();
        assert_eq!(json["files"], 4);
        assert_eq!(json["estimated"], true);
        assert_eq!(json["formats"][0]["bytes"], p.bytes());
    }

    /// A smooth picture, as a photograph's sky or a design's gradient is.
    fn gradient(w: u32, h: u32) -> image::DynamicImage {
        image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |x, y| {
            let t = (f64::from(x) / f64::from(w) + f64::from(y) / f64::from(h)) / 2.0;
            let mix = |a: f64, b: f64| (a * (1.0 - t) + b * t) as u8;
            image::Rgb([mix(40.0, 100.0), mix(80.0, 200.0), mix(120.0, 220.0)])
        }))
    }

    fn probed(image: image::DynamicImage) -> Probed {
        Probed {
            width: image.width(),
            height: image.height(),
            image,
            color: String::new(),
            placeholder: String::new(),
        }
    }

    /// WebP is written lossily, at the quality the config gives: a
    /// gradient comes to a fraction of its PNG (lossless, it came to eight
    /// times it), and a lower quality to fewer bytes.
    #[test]
    fn webp_is_lossy_at_the_quality_asked_and_smaller_than_the_picture() {
        let img = gradient(1440, 900);
        let mut png = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let p = probed(img);
        let q78 = encode::resized(&p, 1440, "webp", 78).unwrap();
        assert_eq!(&q78[..4], b"RIFF");
        assert_eq!(&q78[12..16], b"VP8 ", "a lossy WebP, not VP8L");
        assert!(
            q78.len() * 2 < png.len(),
            "{} against a {}-byte PNG",
            q78.len(),
            png.len()
        );
        let grain = probed(image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(
            640,
            400,
            |x, y| {
                image::Rgb([
                    (x * 7 % 256) as u8,
                    (y * 5 % 256) as u8,
                    ((x ^ y) % 256) as u8,
                ])
            },
        )));
        let low = encode::resized(&grain, 640, "webp", 30).unwrap();
        let high = encode::resized(&grain, 640, "webp", 95).unwrap();
        assert!(
            low.len() < high.len(),
            "quality 30: {}, 95: {}",
            low.len(),
            high.len()
        );
    }

    /// A picture's transparency survives its WebP.
    #[test]
    fn webp_keeps_alpha() {
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(64, 64, |x, _| {
            if x < 32 {
                image::Rgba([200, 40, 40, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        }));
        let out = encode::resized(&probed(img), 64, "webp", 78).unwrap();
        let back = image::load_from_memory_with_format(&out, image::ImageFormat::WebP)
            .unwrap()
            .to_rgba8();
        assert!(back.get_pixel(8, 8)[3] > 240, "the opaque half is opaque");
        assert!(back.get_pixel(56, 8)[3] < 16, "the clear half is clear");
    }

    /// A file of an older encoder is never taken from the cache for one
    /// this encoder writes: the encoder is part of the name.
    #[test]
    fn the_encoder_is_part_of_a_file_s_name() {
        let s = Settings::default();
        assert_eq!(encode::hash(b"pic", &s), encode::hash(b"pic", &s));
        assert_ne!(encode::hash(b"pic", &s), encode::hash(b"pid", &s));
        assert_eq!(encode::hash(b"pic", &s).len(), 8);
    }
}
