//! One image, written again at another width and in another format.

use image::imageops::FilterType;

use super::probe::Probed;
use crate::diagnostics::error::{Result, WebFluentError};
use crate::project::media::Settings;

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

/// The longest side libwebp writes. A wider or taller WebP is not made:
/// the original serves at that width instead.
pub const WEBP_MAX_SIDE: u32 = 16_383;

/// Whether a `width`×`height` file can be written as `format`.
pub fn fits(format: &str, width: u32, height: u32) -> bool {
    format != "webp" || (width <= WEBP_MAX_SIDE && height <= WEBP_MAX_SIDE)
}

/// Whether this build of the compiler can write `format`. Without the
/// `native-media` feature (a WebAssembly build) there is no WebP encoder,
/// and the original picture serves where a WebP would have.
pub fn can_write(format: &str) -> bool {
    format != "webp" || cfg!(wf_webp)
}

/// `image` at `width`, encoded as `format`.
///
/// # Errors
///
/// [`WebFluentError::IoError`] when the encoder refuses the picture — a
/// WebP with a side past [`WEBP_MAX_SIDE`] among them — or `format` is not
/// one the build writes.
pub fn resized(probed: &Probed, width: u32, format: &str, quality: u8) -> Result<Vec<u8>> {
    let height = ((width as f64) * probed.height as f64 / probed.width as f64).round() as u32;
    // At its own width the picture is encoded where it is, not copied.
    let scaled: std::borrow::Cow<'_, image::DynamicImage> = if width >= probed.width {
        std::borrow::Cow::Borrowed(&probed.image)
    } else {
        // Lanczos3 for a photograph: the cheaper filters leave a downscale
        // looking soft, which is exactly what a reader notices.
        std::borrow::Cow::Owned(probed.image.resize_exact(
            width,
            height.max(1),
            FilterType::Lanczos3,
        ))
    };
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    let written = match format {
        #[cfg(wf_webp)]
        "webp" => return webp_of(&scaled, quality),
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
            return Err(WebFluentError::io(format!(
                "`{other}` is not a format the build can write; it writes webp, png and jpeg"
            )));
        }
    };
    written
        .map_err(|e| WebFluentError::io_with(format!("the image could not be written: {e}"), e))?;
    Ok(out)
}

/// `image` as a lossy WebP at `quality`, its alpha kept where it has one.
/// Pixels already laid out as the encoder reads them are handed over as
/// they are; anything else is converted once.
#[cfg(wf_webp)]
fn webp_of(image: &image::DynamicImage, quality: u8) -> Result<Vec<u8>> {
    use image::DynamicImage;
    let (width, height) = (image.width(), image.height());
    if !fits("webp", width, height) {
        return Err(WebFluentError::io(format!(
            "a WebP is at most {WEBP_MAX_SIDE} pixels on a side, and this one would be {width}×{height}"
        )));
    }
    let quality = f32::from(quality.clamp(1, 100));
    let converted;
    let encoder = match image {
        DynamicImage::ImageRgba8(rgba) => webp::Encoder::from_rgba(rgba.as_raw(), width, height),
        DynamicImage::ImageRgb8(rgb) => webp::Encoder::from_rgb(rgb.as_raw(), width, height),
        other if other.color().has_alpha() => {
            converted = DynamicImage::ImageRgba8(other.to_rgba8());
            webp::Encoder::from_rgba(converted.as_bytes(), width, height)
        }
        other => {
            converted = DynamicImage::ImageRgb8(other.to_rgb8());
            webp::Encoder::from_rgb(converted.as_bytes(), width, height)
        }
    };
    encoder
        .encode_simple(false, quality)
        .map(|m| m.to_vec())
        .map_err(|e| WebFluentError::io(format!("the image could not be written as WebP: {e:?}")))
}
