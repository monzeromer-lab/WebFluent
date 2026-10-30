//! One image, written again at another width and in another format.

use image::imageops::FilterType;

use super::probe::Probed;
use crate::error::{Result, WebFluentError};
use crate::media::Settings;

/// Which encoders made a file: part of its hash, so a file an older
/// encoder wrote (a lossless WebP) is never taken from the cache for one
/// this encoder would write.
const ENCODERS: &[u8] = b"libwebp-lossy";

/// A short hash of an image's bytes, the settings that shaped it and the
/// encoders that write it.
///
/// It goes in the file's name, which is what lets a host cache the file
/// forever: change the image, a setting or an encoder, and the name
/// changes with it.
pub fn hash(bytes: &[u8], settings: &Settings) -> String {
    // FNV-1a over the bytes and the settings: short, fast, and it only has
    // to tell two versions of one file apart.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |b: u8| {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x1000_0000_01b3);
    };
    for b in bytes {
        eat(*b);
    }
    for f in &settings.formats {
        for b in f.as_bytes() {
            eat(*b);
        }
    }
    for w in &settings.widths {
        for b in w.to_le_bytes() {
            eat(b);
        }
    }
    eat(settings.quality);
    for b in ENCODERS {
        eat(*b);
    }
    format!("{h:x}")[..8].to_string()
}

/// `image` at `width`, encoded as `format`.
pub fn resized(probed: &Probed, width: u32, format: &str, quality: u8) -> Result<Vec<u8>> {
    let height = ((width as f64) * probed.height as f64 / probed.width as f64).round() as u32;
    let scaled = if width >= probed.width {
        probed.image.clone()
    } else {
        // Lanczos3 for a photograph: the cheaper filters leave a downscale
        // looking soft, which is exactly what a reader notices.
        probed
            .image
            .resize_exact(width, height.max(1), FilterType::Lanczos3)
    };
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    let written = match format {
        "webp" => return Ok(webp_of(&scaled, quality)),
        "png" => scaled.write_to(&mut cursor, image::ImageFormat::Png),
        "jpeg" | "jpg" => {
            // The quality is the author's, and the encoder wants it here
            // rather than in the format.
            let rgb = scaled.to_rgb8();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality).encode(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )
        }
        other => {
            return Err(WebFluentError::IoError(format!(
                "`{other}` is not a format the build can write; it writes webp, png and jpeg"
            )));
        }
    };
    written.map_err(|e| WebFluentError::IoError(format!("the image could not be written: {e}")))?;
    Ok(out)
}

/// `image` as a lossy WebP at `quality`, its alpha kept where it has one.
fn webp_of(image: &image::DynamicImage, quality: u8) -> Vec<u8> {
    let (width, height) = (image.width(), image.height());
    let quality = f32::from(quality.clamp(1, 100));
    if image.color().has_alpha() {
        let rgba = image.to_rgba8();
        webp::Encoder::from_rgba(rgba.as_raw(), width, height)
            .encode(quality)
            .to_vec()
    } else {
        let rgb = image.to_rgb8();
        webp::Encoder::from_rgb(rgb.as_raw(), width, height)
            .encode(quality)
            .to_vec()
    }
}
