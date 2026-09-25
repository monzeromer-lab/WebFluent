//! What an image is, read from its bytes.
//!
//! The dimensions are what `width` and `height` are written from, so the
//! box the image will fill is the right size before a single byte of it
//! has arrived and the page never jumps. The average colour is what stands
//! in the meantime, and the placeholder is a very small blurred copy of
//! the thing itself — twenty-odd bytes of it, inlined.

use std::path::Path;

use image::{DynamicImage, GenericImageView, imageops::FilterType};

use crate::error::{Result, WebFluentError};

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

/// Decode `bytes`, and read what the build needs from them.
pub fn probe(bytes: &[u8], source: &Path) -> Result<Probed> {
    let image = image::load_from_memory(bytes).map_err(|e| {
        WebFluentError::IoError(format!("{} could not be read: {e}", source.display()))
    })?;
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
        (16u32, ((16.0 * h as f64 / w as f64).round() as u32).max(1))
    } else {
        (((16.0 * w as f64 / h as f64).round() as u32).max(1), 16u32)
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
