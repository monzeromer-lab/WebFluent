//! What an image is, read from its bytes.
//!
//! The dimensions are what `width` and `height` are written from, so the
//! box the image will fill is the right size before a single byte of it
//! has arrived and the page never jumps. The average colour is what stands
//! in the meantime, and the placeholder is a very small blurred copy of
//! the thing itself — twenty-odd bytes of it, inlined.

use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageReader, Limits, imageops::FilterType};

use crate::diagnostics::error::{Result, WebFluentError};

/// The most pixels a picture the build reads may have: 50 megapixels,
/// more than any screen shows (an 8K frame is 33). A picture is decoded
/// whole, at four bytes a pixel and more, so a small file of a vast
/// picture — an 11000×11000 PNG is 470 KB — would otherwise take gigabytes
/// and minutes; past this the build says so instead.
pub const MAX_PIXELS: u64 = 50_000_000;

/// An image, decoded once.
pub struct Probed {
    pub image: DynamicImage,
    pub width: u32,
    pub height: u32,
    /// `#RRGGBB`: the average of every pixel.
    pub color: String,
    /// A data URI of a tiny blurred copy.
    pub placeholder: String,
}

/// A reader over `bytes`, its format guessed from them.
fn reader<'a>(bytes: &'a [u8], source: &Path) -> Result<ImageReader<std::io::Cursor<&'a [u8]>>> {
    ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| WebFluentError::io(format!("{} could not be read: {e}", source.display())))
}

/// The width and height `bytes` say they hold, read from the header alone.
///
/// # Errors
///
/// [`WebFluentError::IoError`] when the header is no picture's the build
/// reads.
pub fn dimensions(bytes: &[u8], source: &Path) -> Result<(u32, u32)> {
    reader(bytes, source)?
        .into_dimensions()
        .map_err(|e| WebFluentError::io(format!("{} could not be read: {e}", source.display())))
}

/// Decode `bytes`, and read what the build needs from them.
///
/// # Errors
///
/// [`WebFluentError::IoError`] when the bytes are no picture the build
/// reads, or one of more than [`MAX_PIXELS`], which is said before
/// anything is decoded.
pub fn probe(bytes: &[u8], source: &Path) -> Result<Probed> {
    let (w, h) = dimensions(bytes, source)?;
    let pixels = u64::from(w) * u64::from(h);
    if pixels > MAX_PIXELS {
        return Err(WebFluentError::io(format!(
            "{} is {w}×{h}, {:.0} megapixels; the build reads pictures of at most {} — make it smaller, or serve it from public/ as a file the build does not read",
            source.display(),
            pixels as f64 / 1e6,
            MAX_PIXELS / 1_000_000
        )));
    }
    let mut reader = reader(bytes, source)?;
    // The decoder is held to what the header said, and to what that many
    // pixels take at the widest a pixel is (16-bit RGBA, eight bytes), so a
    // file that lies about its size cannot ask for more.
    let mut limits = Limits::default();
    limits.max_image_width = Some(w);
    limits.max_image_height = Some(h);
    limits.max_alloc = Some((MAX_PIXELS * 8).max(64 * 1024 * 1024));
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| WebFluentError::io(format!("{} could not be read: {e}", source.display())))?;
    let (width, height) = image.dimensions();
    Ok(Probed {
        color: average_color(&image),
        placeholder: placeholder(&image),
        image,
        width,
        height,
    })
}

/// The average of every pixel, which is what an empty box shows while the
/// real image is on its way.
fn average_color(image: &DynamicImage) -> String {
    // One pixel of a thumbnail is the average of the lot, and costs far
    // less than walking every pixel of a photograph.
    let small = image.resize_exact(1, 1, FilterType::Triangle).to_rgb8();
    let pixel = small.get_pixel(0, 0);
    format!("#{:02X}{:02X}{:02X}", pixel[0], pixel[1], pixel[2])
}

/// A very small blurred copy, as a data URI: what the reader sees in the
/// shape of the image until the image itself arrives.
fn placeholder(image: &DynamicImage) -> String {
    let (w, h) = image.dimensions();
    let (tw, th) = if w >= h {
        (
            16u32,
            ((16.0 * f64::from(h) / f64::from(w)).round() as u32).max(1),
        )
    } else {
        (
            ((16.0 * f64::from(w) / f64::from(h)).round() as u32).max(1),
            16u32,
        )
    };
    let small = image.resize_exact(tw, th, FilterType::Triangle);
    let mut out = Vec::new();
    if small
        .write_to(
            &mut std::io::Cursor::new(&mut out),
            image::ImageFormat::WebP,
        )
        .is_err()
    {
        return String::new();
    }
    format!("data:image/webp;base64,{}", base64(&out))
}

/// Base64, written here because one function is less than a dependency.
pub fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}
