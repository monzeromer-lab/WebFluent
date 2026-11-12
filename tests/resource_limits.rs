//! What one input may cost: an expression worked out at build time, a
//! picture the build reads, a picture a PDF shows many times. Each is
//! bounded, and says so, rather than taking the machine with it.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use webfluent::codegen::static_eval::{Scope, Static, eval, eval_metered};
use webfluent::parser::v2::parse_v2;
use webfluent::parser::{Arg, Declaration, Expr, StatementKind};

/// Run `f` on its own thread and end the process if it does not finish in
/// `secs`: a runaway allocation must fail the test, not the machine.
fn within<T: Send + 'static>(secs: u64, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(move || {
            let _ = tx.send(f());
        })
        .unwrap();
    match rx.recv_timeout(Duration::from_secs(secs)) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("did not finish within {secs}s");
            std::process::exit(1);
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wf-limits-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `src` parsed as the expression of a page's one `Text`.
fn expr_of(src: &str) -> Expr {
    let program = parse_v2(&format!("page P(path: \"/\") {{ Text({src}) }}"), "x.wf").unwrap();
    let Declaration::Page(page) = &program.declarations[0] else {
        unreachable!()
    };
    let StatementKind::UIElement(ui) = &page.body[0].kind else {
        unreachable!()
    };
    let Arg::Positional(e) = &ui.args[0] else {
        unreachable!()
    };
    e.clone()
}

// ─── M2: the build-time evaluator's fuel bounds what it makes ────────

#[test]
fn an_expression_that_asks_for_billions_of_values_stops() {
    // The reviewers': 10^10 values. Each item is paid for as it is made,
    // so it stops after a few hundred thousand.
    for src in [
        "(1..=1000000).map(i => 1..=1000000)",
        "(1..=1000).map(i => 1..=1000)",
        "(1..=1000).flatMap(i => 1..=1000)",
        "(1..=600).map(i => big).length",
        "big.split(\"\").join(big)",
        "big.split(\"\").map(c => big).length",
        "[...big.split(\"\"), ...big.split(\"\")].concat(big.split(\"\")).length",
        "big.replace(/(x)/g, \"$&$&$&$&$&$&$&$&$&$&\").replace(/x/g, big).length",
    ] {
        let e = expr_of(src);
        let metered = within(60, move || {
            let scope = Scope::of([("big".to_string(), Static::Str("x".repeat(100_000)))]);
            eval_metered(&e, &scope)
        });
        assert!(metered.exhausted, "{src}: {metered:?}");
        assert_eq!(metered.value, None, "{src}");
    }
}

#[test]
fn what_the_fuel_allows_is_still_worked_out() {
    for (src, want) in [
        ("(1..=1000).map(i => i * 2).length", Static::Num(1000.0)),
        ("(1..=200000).length", Static::Num(200_000.0)),
        ("(1..=100).flatMap(i => [i, i]).length", Static::Num(200.0)),
        ("(1..=300).map(i => \"ab\").join(\"\").length", Static::Num(600.0)),
        ("[1, 2, 3][1.5] ?? \"none\"", Static::Str("none".into())),
        ("[1, 2, 3][-1] ?? \"none\"", Static::Str("none".into())),
    ] {
        assert_eq!(eval(&expr_of(src), &Scope::default()), Some(want), "{src}");
    }
}

#[test]
fn to_fixed_takes_what_javascript_takes() {
    let at = |src: &str| eval(&expr_of(src), &Scope::default());
    assert_eq!(at("(1.5).toFixed(2)"), Some(Static::Str("1.50".into())));
    assert_eq!(
        at("(1.5).toFixed(100)"),
        Some(Static::Str(format!("1.5{}", "0".repeat(99))))
    );
    // A RangeError in the browser: the build leaves it to the page.
    for digits in ["101", "70000", "1000000000", "-1"] {
        assert_eq!(at(&format!("(1.5).toFixed({digits})")), None, "{digits}");
    }
}

#[test]
fn a_format_precision_past_what_intl_takes_is_held_to_it() {
    let at = |src: &str| eval(&expr_of(src), &Scope::default());
    let Some(Static::Str(text)) = at("format(1.5, .decimal, 70000)") else {
        panic!("format gave nothing");
    };
    assert!(text.len() < 200, "{text}");
}

// ─── M4, M11: pictures the build reads ───────────────────────────────

/// A PNG of `w`×`h` noise: a file no smaller format would make for free.
fn noise_png(w: u32, h: u32) -> Vec<u8> {
    let mut seed: u32 = 0x1234_5678;
    let img = image::RgbImage::from_fn(w, h, |_, _| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        image::Rgb([seed as u8, (seed >> 8) as u8, (seed >> 16) as u8])
    });
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn process(dir: &Path, file: &str, bytes: &[u8]) -> webfluent::Result<webfluent::media::Asset> {
    let source = dir.join(file);
    std::fs::write(&source, bytes).unwrap();
    webfluent::media::process(
        "x",
        &source,
        &dir.join("build"),
        &dir.join("cache"),
        &webfluent::media::Settings::default(),
        "",
    )
}

#[test]
fn a_picture_wider_than_webp_holds_is_served_as_it_is_at_that_width() {
    // The reviewers': 16400×8, one past what libwebp writes, panicked the
    // build. The narrower widths are still made; the full width is the
    // original's.
    let dir = scratch("wide");
    let asset = within(120, move || process(&dir, "wide.png", &noise_png(16_400, 8)).unwrap());
    let widths: Vec<u32> = asset
        .sources
        .get("webp")
        .map(|s| s.iter().map(|(w, _)| *w).collect())
        .unwrap_or_default();
    assert!(!widths.contains(&16_400), "{widths:?}");
    assert!(widths.contains(&480), "{widths:?}");
    assert_eq!((asset.width, asset.height), (16_400, 8));
    // And a tall one, which has no narrower width to make.
    let dir = scratch("tall");
    let asset = within(120, move || process(&dir, "tall.png", &noise_png(8, 16_400)).unwrap());
    assert!(asset.sources.get("webp").is_none_or(|s| s.is_empty()), "{asset:?}");
}

#[test]
fn a_picture_at_webp_s_limit_is_still_made() {
    let dir = scratch("edge");
    let asset = within(120, move || process(&dir, "edge.png", &noise_png(16_383, 8)).unwrap());
    let widths: Vec<u32> = asset.sources["webp"].iter().map(|(w, _)| *w).collect();
    assert!(widths.contains(&16_383), "{widths:?}");
}

/// A PNG that says it is `w`×`h` and holds no pixels: what a header
/// claims is all the build may read before deciding.
fn header_only_png(w: u32, h: u32) -> Vec<u8> {
    fn crc(bytes: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for b in bytes {
            c ^= u32::from(*b);
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
        !c
    }
    let chunk = |kind: &[u8], data: &[u8]| {
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc(&body).to_be_bytes());
        out
    };
    let mut ihdr = w.to_be_bytes().to_vec();
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend(chunk(b"IHDR", &ihdr));
    // An empty zlib stream: the header is followed by data, which holds
    // nothing.
    png.extend(chunk(b"IDAT", &[0x78, 0x9C, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01]));
    png.extend(chunk(b"IEND", &[]));
    png
}

#[test]
fn a_picture_of_too_many_pixels_is_refused_before_it_is_decoded() {
    // 11000×11000 is 121 megapixels: the reviewers' 470 KB file took 3 GB
    // and seven minutes. Its header alone now decides.
    let dir = scratch("vast");
    let err = within(30, move || process(&dir, "vast.png", &header_only_png(11_000, 11_000)).unwrap_err());
    let message = err.to_string();
    assert!(message.contains("11000×11000"), "{message}");
    assert!(message.contains("at most 50"), "{message}");
    // Just under the limit, the header is read and the (empty) picture is
    // then refused as unreadable, not as too large.
    let dir = scratch("under");
    let err = within(30, move || process(&dir, "under.png", &header_only_png(7_000, 7_000)).unwrap_err());
    assert!(!err.to_string().contains("megapixels"), "{err}");
}

// ─── M13: a picture a document shows many times is read once ─────────

#[test]
fn a_pdf_reads_a_picture_it_shows_many_times_once() {
    let png = noise_png(64, 64);
    let reads = Cell::new(0usize);
    let reader = |src: &str| -> Option<Vec<u8>> {
        if src == "/a.png" {
            reads.set(reads.get() + 1);
            Some(png.clone())
        } else {
            None
        }
    };
    let body: String = (0..300)
        .map(|_| "<img src=\"/a.png\" style=\"width: 8px; height: 8px\">")
        .collect();
    let html = format!("<!DOCTYPE html><html><body>{body}<p>end</p></body></html>");
    let config = webfluent::PdfConfig::default();
    let options = webfluent::paged::Options::from_pdf(&config, &reader, None);
    let out = webfluent::paged::render(&html, "", &options).unwrap();
    assert!(out.bytes.starts_with(b"%PDF"));
    assert_eq!(reads.get(), 1);
}
