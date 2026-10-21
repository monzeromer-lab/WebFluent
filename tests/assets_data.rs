//! `public/` as data, over a project that names its files every way a
//! project can — and over the site's own: every file listed with its kind,
//! size and dimensions; every use found, at a line that does name it; the
//! unused exactly the files nothing names; a picker offering only files
//! that exist and fit; and the media pipeline's plan agreeing, file for
//! file, with what `wf build` then writes.

use std::path::{Path, PathBuf};
use std::process::Command;

use webfluent::assets::{self, Kind, Listing};
use webfluent::registry::FileKind;
use webfluent::vfs::{OverlayVfs, Vfs};

fn wf(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wf"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .expect("wf runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// A picture with the grain of a photograph: gradients under noise.
fn photo(w: u32, h: u32, format: image::ImageFormat) -> Vec<u8> {
    let mut img = image::RgbImage::new(w, h);
    let mut s: u32 = w ^ (h << 8);
    for (x, y, p) in img.enumerate_pixels_mut() {
        s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let n = (s >> 24) as i32 - 128;
        let fx = f64::from(x) / f64::from(w);
        let fy = f64::from(y) / f64::from(h);
        let c = |v: f64| ((v * 255.0) as i32 + n / 6).clamp(0, 255) as u8;
        *p = image::Rgb([c(fx), c(fy), c((fx * 12.0).sin() * 0.5 + 0.5)]);
    }
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut std::io::Cursor::new(&mut out), format)
        .unwrap();
    out
}

fn put(dir: &Path, path: &str, bytes: impl AsRef<[u8]>) {
    let at = dir.join(path);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, bytes).unwrap();
}

const HOME: &str = r#"const LOGO = "/logo.svg"

image card = "card.png"

data docs = "docs.json"

page Home(path: "/", title: "Home", description: "The home page.", image: "/og.png") {
    Image(src: "/hero.png", alt: "The harbour at dawn")
    Image(card, alt: "A card")
    Image(src: LOGO, alt: "Halyard")
    Video(src: "/talk.mp4", captions: "/talk.vtt", poster: "/poster.png?v=2").controls
    Image(src: "https://example.com/elsewhere.png", alt: "Elsewhere")
    Avatar(src: ["/icons/icon-192.png"][0] ?? "/og.png", alt: "Us")
}
"#;

/// A project that names its files every way it can.
fn project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "wf-assets-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    put(
        &dir,
        "webfluent.app.json",
        r#"{
  "name": "assets",
  "meta": {
    "favicon": "favicon.svg",
    "preload": ["/fonts/a.woff2"]
  }
}
"#,
    );
    put(&dir, "src/pages/Home.wf", HOME);
    put(
        &dir,
        "src/docs.json",
        r#"{ "file": "/guide.pdf", "title": "The guide" }"#,
    );
    put(
        &dir,
        "src/site.css",
        "@font-face {\n  font-family: A;\n  src: url(/fonts/a.woff2) format(\"woff2\");\n}\n",
    );
    put(
        &dir,
        "src/load.js",
        "function loadItems() {\n  return fetch(\"/data/items.json\");\n}\n",
    );
    put(
        &dir,
        "public/hero.png",
        photo(1440, 900, image::ImageFormat::Png),
    );
    put(
        &dir,
        "public/card.png",
        photo(1200, 630, image::ImageFormat::Png),
    );
    put(
        &dir,
        "public/og.png",
        photo(600, 315, image::ImageFormat::Png),
    );
    put(
        &dir,
        "public/poster.png",
        photo(320, 180, image::ImageFormat::Png),
    );
    put(
        &dir,
        "public/old-hero.jpg",
        photo(800, 600, image::ImageFormat::Jpeg),
    );
    put(
        &dir,
        "public/icons/icon-192.png",
        photo(192, 192, image::ImageFormat::Png),
    );
    put(
        &dir,
        "public/logo.svg",
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 40"><rect width="120" height="40"/></svg>"#,
    );
    put(
        &dir,
        "public/favicon.svg",
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"></svg>"#,
    );
    put(&dir, "public/favicon.ico", [0u8, 0, 1, 0]);
    put(&dir, "public/fonts/a.woff2", b"wOF2 not really");
    put(&dir, "public/guide.pdf", b"%PDF-1.7 not really");
    put(&dir, "public/talk.mp4", b"not really a video");
    put(
        &dir,
        "public/talk.vtt",
        "WEBVTT\n\n00:00.000 --> 00:01.000\nHello\n",
    );
    put(&dir, "public/data/items.json", "[]");
    put(&dir, "public/robots.txt", "User-agent: *\n");
    put(
        &dir,
        "public/manifest.webmanifest",
        r#"{ "name": "Assets", "icons": [{ "src": "icons/icon-192.png", "sizes": "192x192" }] }"#,
    );
    dir
}

/// Every use is at a line that names the file — by its path, or by the
/// name an `image` declaration gives it — so an editor's jump lands on it.
fn uses_are_where_they_say(root: &Path, listing: &Listing) -> usize {
    let mut checked = 0;
    for a in &listing.assets {
        let name = a.path.rsplit('/').next().unwrap();
        for u in &a.uses {
            let text = std::fs::read_to_string(root.join(&u.file))
                .unwrap_or_else(|e| panic!("{}: {e}", u.file));
            let line = text
                .lines()
                .nth(u.line - 1)
                .unwrap_or_else(|| panic!("{}:{} is past the end", u.file, u.line));
            let named = line.contains(name)
                || u.how
                    .strip_prefix("const ")
                    .is_some_and(|c| line.contains(c))
                || u.how
                    .strip_prefix("image ")
                    .is_some_and(|c| line.contains(c));
            assert!(
                named,
                "{} is named at {}:{} ({}), but that line is `{line}`",
                a.path, u.file, u.line, u.how
            );
            checked += 1;
        }
    }
    checked
}

#[test]
fn every_file_is_listed_with_its_kind_size_and_uses() {
    let dir = project();
    let listing = assets::list(&dir).unwrap();
    let paths: Vec<&str> = listing.assets.iter().map(|a| a.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "/card.png",
            "/data/items.json",
            "/favicon.ico",
            "/favicon.svg",
            "/fonts/a.woff2",
            "/guide.pdf",
            "/hero.png",
            "/icons/icon-192.png",
            "/logo.svg",
            "/manifest.webmanifest",
            "/og.png",
            "/old-hero.jpg",
            "/poster.png",
            "/robots.txt",
            "/talk.mp4",
            "/talk.vtt",
        ]
    );
    let get = |p: &str| listing.get(p).unwrap();
    let hows = |p: &str| -> Vec<(String, String)> {
        get(p)
            .uses
            .iter()
            .map(|u| (u.file.clone(), u.how.clone()))
            .collect()
    };
    let pair = |f: &str, h: &str| (f.to_string(), h.to_string());

    // Each way of naming a file.
    assert_eq!(hows("/hero.png"), [pair("src/pages/Home.wf", "Image src")]);
    assert_eq!(hows("/card.png"), [pair("src/pages/Home.wf", "image card")]);
    assert_eq!(
        hows("/og.png"),
        [
            pair("src/pages/Home.wf", "page image"),
            pair("src/pages/Home.wf", "Avatar src")
        ]
    );
    assert_eq!(hows("/logo.svg"), [pair("src/pages/Home.wf", "const LOGO")]);
    assert_eq!(hows("/talk.mp4"), [pair("src/pages/Home.wf", "Video src")]);
    assert_eq!(
        hows("/talk.vtt"),
        [pair("src/pages/Home.wf", "Video captions")]
    );
    assert_eq!(
        hows("/poster.png"),
        [pair("src/pages/Home.wf", "Video poster")]
    );
    assert_eq!(hows("/guide.pdf"), [pair("src/docs.json", "data docs")]);
    assert_eq!(hows("/data/items.json"), [pair("src/load.js", "a string")]);
    assert_eq!(
        hows("/fonts/a.woff2"),
        [
            pair("src/site.css", "url()"),
            pair("webfluent.app.json", "meta.preload")
        ]
    );
    assert_eq!(
        hows("/favicon.svg"),
        [pair("webfluent.app.json", "meta.favicon")]
    );
    // Inside a value, a path is still named by its prop.
    assert_eq!(
        hows("/icons/icon-192.png"),
        [
            pair("public/manifest.webmanifest", "manifest"),
            pair("src/pages/Home.wf", "Avatar src")
        ]
    );
    // Where: the line and the column of the argument.
    let hero = &get("/hero.png").uses[0];
    assert_eq!((hero.line, hero.column), (8, 11));
    let font = &get("/fonts/a.woff2").uses;
    assert_eq!((font[0].line, font[1].line), (3, 5));

    // Only what the pipeline processes says so.
    // What the pipeline processes says so: an `image`, and an `Image`'s
    // literal `src:`; a picture named any other way is copied.
    assert!(get("/card.png").processed());
    assert!(get("/hero.png").processed());
    assert!(!get("/og.png").processed(), "a page's image is copied");
    assert!(!get("/poster.png").processed(), "a poster is copied");
    assert!(!get("/logo.svg").processed(), "an SVG is copied");

    // Asked for by name, so not unused; and the one file nothing names.
    for p in ["/robots.txt", "/favicon.ico", "/manifest.webmanifest"] {
        assert!(get(p).convention.is_some() && !get(p).unused(), "{p}");
    }
    let unused: Vec<&str> = listing
        .assets
        .iter()
        .filter(|a| a.unused())
        .map(|a| a.path.as_str())
        .collect();
    assert_eq!(unused, ["/old-hero.jpg"]);

    // Kinds, icons, sizes.
    assert_eq!(get("/hero.png").kind, Kind::Image);
    assert_eq!(get("/fonts/a.woff2").kind, Kind::Font);
    assert_eq!(get("/talk.vtt").kind, Kind::Document);
    assert_eq!(
        (get("/hero.png").width, get("/hero.png").height),
        (Some(1440), Some(900))
    );
    assert_eq!(
        (get("/old-hero.jpg").width, get("/old-hero.jpg").height),
        (Some(800), Some(600))
    );
    assert_eq!(
        (get("/logo.svg").width, get("/logo.svg").height),
        (Some(120), Some(40))
    );
    assert!(get("/logo.svg").vector);
    assert_eq!(get("/talk.mp4").width, None);
    let icons: Vec<&str> = listing
        .assets
        .iter()
        .filter(|a| a.icon)
        .map(|a| a.path.as_str())
        .collect();
    assert_eq!(
        icons,
        ["/favicon.ico", "/favicon.svg", "/icons/icon-192.png"]
    );
    for a in &listing.assets {
        assert_eq!(
            a.bytes,
            std::fs::metadata(dir.join(&a.file)).unwrap().len(),
            "{}",
            a.path
        );
    }
    let counts = listing.counts();
    assert_eq!(counts["image"], 9);
    assert_eq!(counts["icon"], 3);
    assert_eq!(counts["unused"], 1);
    assert_eq!(
        counts.values().sum::<usize>() - counts["icon"] - counts["unused"],
        16
    );

    assert_eq!(uses_are_where_they_say(&dir, &listing), 15);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_picker_offers_only_files_that_exist_and_fit() {
    let dir = project();
    let listing = assets::list(&dir).unwrap();
    let offered =
        |k: FileKind| -> Vec<&str> { listing.offered(k).map(|a| a.path.as_str()).collect() };
    assert_eq!(
        offered(FileKind::Image),
        [
            "/card.png",
            "/favicon.svg",
            "/hero.png",
            "/icons/icon-192.png",
            "/logo.svg",
            "/og.png",
            "/old-hero.jpg",
            "/poster.png",
        ]
    );
    assert!(offered(FileKind::Icon).contains(&"/favicon.ico"));
    assert_eq!(offered(FileKind::Video), ["/talk.mp4"]);
    assert_eq!(offered(FileKind::Captions), ["/talk.vtt"]);
    assert_eq!(offered(FileKind::Audio), Vec::<&str>::new());
    assert_eq!(
        offered(FileKind::Document),
        ["/guide.pdf", "/robots.txt", "/talk.vtt"]
    );
    // Every file the registry types as a file has a picker of a kind, and
    // what it offers is there.
    let mut props = 0;
    for (_, _, kind) in webfluent::registry::file_props() {
        for a in listing.offered(kind) {
            assert!(dir.join(&a.file).is_file(), "{}", a.path);
        }
        props += 1;
    }
    assert!(props >= 8, "{props} file props");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_registry_and_the_config_say_which_props_name_a_file() {
    let props: Vec<(String, String, &str)> = webfluent::registry::file_props()
        .map(|(c, p, k)| (c.name.to_string(), p.name.to_string(), k.word()))
        .collect();
    let has = |c: &str, p: &str, k: &str| props.iter().any(|x| x.0 == c && x.1 == p && x.2 == k);
    assert!(has("Image", "src", "image"));
    assert!(has("Avatar", "src", "image"));
    assert!(has("ImageSlide", "src", "image"));
    assert!(has("Video", "src", "video"));
    assert!(has("Video", "poster", "image"));
    assert!(has("Video", "captions", "captions"));
    assert!(has("Audio", "src", "audio"));
    assert!(has("Audio", "transcript", "document"));
    // A frame shows a page, not a file.
    assert!(!props.iter().any(|x| x.0 == "Frame"));
    // The registry's JSON says it, for an editor's picker.
    let registry = webfluent::registry::json::registry_json();
    let image = registry["components"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Image")
        .unwrap();
    let src = image["props"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "src")
        .unwrap();
    assert_eq!(src["file"], "image");
    assert_eq!(src["type"], "String");
    // So does the config's schema.
    let schema = webfluent::config::schema::to_json();
    let row = |p: &str| {
        schema
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["path"] == p)
            .unwrap()
            .clone()
    };
    assert_eq!(row("meta.favicon")["type"], "file");
    assert_eq!(row("meta.favicon")["file"], "icon");
    assert_eq!(row("meta.image")["file"], "image");
    assert_eq!(row("meta.touch_icon")["file"], "image");
}

#[test]
fn a_missing_file_a_file_prop_names_is_d01_and_not_listed() {
    let dir = project();
    put(
        &dir,
        "src/pages/Home.wf",
        HOME.replace("/talk.vtt", "/talk.en.vtt"),
    );
    let (code, out, err) = wf(&dir, &["check", "--format", "json"]);
    let all = format!("{out}{err}");
    assert!(
        all.contains("D01") && all.contains("/talk.en.vtt"),
        "{code}: {all}"
    );
    let listing = assets::list(&dir).unwrap();
    assert!(listing.get("/talk.en.vtt").is_none());
    assert!(listing.get("/talk.vtt").unwrap().unused());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_unsaved_buffer_is_read_as_the_editor_holds_it() {
    let dir = project();
    let mut vfs = OverlayVfs::over_disk();
    vfs.set(
        dir.join("src/pages/More.wf"),
        "page More(path: \"/more\", title: \"More\", description: \"More.\") {\n    Image(src: \"/old-hero.jpg\", alt: \"Before\")\n}\n",
    );
    let listing = assets::list_via(&vfs, &dir).unwrap();
    let old = listing.get("/old-hero.jpg").unwrap();
    assert!(!old.unused());
    assert_eq!(old.uses[0].file, "src/pages/More.wf");
    assert_eq!(old.uses[0].line, 2);
    // Nothing reached the disk.
    assert!(!dir.join("src/pages/More.wf").exists());
    assert!(
        assets::list(&dir)
            .unwrap()
            .get("/old-hero.jpg")
            .unwrap()
            .unused()
    );
    assert!(vfs.is_file(&dir.join("src/pages/More.wf")));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_plan_is_what_the_build_then_writes() {
    let dir = project();
    let plan = assets::plan(&dir, "/card.png").unwrap();
    assert!(plan.pipeline);
    assert_eq!((plan.width, plan.height), (1200, 630));
    // Never up-scaled: the default widths below 1200, then its own.
    assert_eq!(plan.widths, [480, 960, 1200]);
    assert_eq!(plan.formats.len(), 1);
    assert_eq!(plan.formats[0].format, "webp");
    assert!(plan.estimated(), "nothing is built yet");
    let estimates: Vec<u64> = plan.made().map(|f| f.bytes).collect();

    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    let mut written: Vec<String> = std::fs::read_dir(dir.join("build/img"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("card."))
        .collect();
    written.sort();
    let mut named: Vec<String> = plan.made().map(|f| f.name.clone()).collect();
    named.push(plan.original.name.clone());
    named.sort();
    assert_eq!(written, named, "the plan names every file the build writes");

    // Built, the plan reads the cache and is exact.
    let again = assets::plan(&dir, "/card.png").unwrap();
    assert!(!again.estimated());
    for (f, estimate) in again.made().zip(&estimates) {
        let real = std::fs::metadata(dir.join("build/img").join(&f.name))
            .unwrap()
            .len();
        assert_eq!(f.bytes, real, "{}", f.name);
        let ratio = *estimate as f64 / real as f64;
        assert!(
            (0.67..1.5).contains(&ratio),
            "{}: estimated {estimate}, written {real}",
            f.name
        );
    }
    assert_eq!(
        again.original.bytes,
        std::fs::metadata(dir.join("public/card.png"))
            .unwrap()
            .len()
    );
    assert!(
        again
            .summary()
            .starts_with("3 widths × 1 format · 3 files · "),
        "{}",
        again.summary()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_plan_follows_the_settings_and_says_what_it_cannot_do() {
    let dir = project();
    let (code, _, err) = wf(
        &dir,
        &[
            "config",
            "set",
            "build.media.formats",
            r#"["avif", "jpeg", "webp"]"#,
        ],
    );
    assert_eq!(code, 0, "{err}");
    wf(&dir, &["config", "set", "build.media.widths", "[320, 640]"]);
    let plan = assets::plan(&dir, "/poster.png").unwrap();
    // 320 is not smaller than the poster: its own width stands for it.
    assert_eq!(plan.widths, [320]);
    let formats: Vec<&str> = plan.formats.iter().map(|f| f.format.as_str()).collect();
    assert_eq!(formats, ["avif", "jpeg", "webp"]);
    assert!(
        plan.formats[0]
            .error
            .as_deref()
            .unwrap()
            .contains("not a format the build can write")
    );
    assert!(plan.formats[0].files.is_empty());
    // Small enough to encode outright: exact before any build.
    assert!(!plan.estimated());
    assert_eq!(plan.made().count(), 2);
    assert!(
        plan.summary()
            .starts_with("1 width × 2 formats · 2 files · "),
        "{}",
        plan.summary()
    );

    // A PDF embeds a picture named by its path as it is; an `image` is
    // still read.
    let (code, _, err) = wf(&dir, &["config", "set", "build.output_type", "pdf"]);
    assert_eq!(code, 0, "{err}");
    let pdf = assets::list(&dir).unwrap();
    assert!(!pdf.get("/hero.png").unwrap().processed());
    assert!(pdf.get("/card.png").unwrap().processed());
    wf(&dir, &["config", "unset", "build.output_type"]);
    assert!(
        assets::list(&dir)
            .unwrap()
            .get("/hero.png")
            .unwrap()
            .processed()
    );

    wf(&dir, &["config", "set", "build.media.pipeline", "false"]);
    let off = assets::plan(&dir, "/hero.png").unwrap();
    assert!(!off.pipeline);
    assert_eq!(off.made().count(), 0);
    assert!(off.summary().starts_with("copied as it is"));
    // Off, nothing is processed.
    assert!(
        !assets::list(&dir)
            .unwrap()
            .get("/card.png")
            .unwrap()
            .processed()
    );

    for (path, says) in [
        ("/logo.svg", "copied as it is"),
        ("/favicon.ico", "copied as it is"),
        ("/guide.pdf", "copied as it is"),
        ("/nothing.png", "does not exist"),
        ("https://example.com/a.png", "not a path of the site"),
    ] {
        let e = assets::plan(&dir, path).unwrap_err().to_string();
        assert!(e.contains(says), "{path}: {e}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn wf_assets_says_what_the_library_holds() {
    let dir = project();
    let (code, out, err) = wf(&dir, &["assets", "--json"]);
    assert_eq!(code, 0, "{err}");
    let said: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(said, assets::list(&dir).unwrap().as_json());

    let (_, out, _) = wf(&dir, &["assets"]);
    assert!(out.contains("16 files"), "{out}");
    assert!(
        out.contains("/hero.png") && out.contains("1440×900"),
        "{out}"
    );
    assert!(out.contains("used in 1 file, processed"), "{out}");
    assert!(
        out.contains("/og.png") && out.contains("used in 1 file\n"),
        "{out}"
    );
    assert!(out.contains("by convention"), "{out}");
    let (_, out, _) = wf(&dir, &["assets", "--unused"]);
    let rows: Vec<&str> = out.lines().filter(|l| l.starts_with("  /")).collect();
    assert_eq!(rows.len(), 1, "{out}");
    assert!(rows[0].contains("/old-hero.jpg"));

    let (code, out, err) = wf(&dir, &["assets", "--plan", "/hero.png", "--json"]);
    assert_eq!(code, 0, "{err}");
    let mut said: serde_json::Value = serde_json::from_str(&out).unwrap();
    let mut held = assets::plan(&dir, "/hero.png").unwrap().as_json();
    // 1440 is its own width: three files, not four.
    assert_eq!(said["files"], 3);
    // Estimates are the same estimate both times; nothing else may differ.
    said["summary"] = serde_json::Value::Null;
    held["summary"] = serde_json::Value::Null;
    assert_eq!(said, held);
    let (_, out, _) = wf(&dir, &["assets", "--plan", "/hero.png"]);
    assert!(
        out.contains("webp") && out.contains("480 · 960 · 1440"),
        "{out}"
    );
    assert!(out.contains("The original is written too"), "{out}");

    let (code, _, err) = wf(&dir, &["assets", "--plan", "/logo.svg"]);
    assert_ne!(code, 0);
    assert!(err.contains("copied as it is"), "{err}");

    let empty = std::env::temp_dir().join(format!("wf-assets-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let (code, out, _) = wf(&empty, &["assets"]);
    assert_eq!(code, 0);
    assert!(out.contains("public/ is empty"), "{out}");
    let _ = std::fs::remove_dir_all(&empty);
    let _ = std::fs::remove_dir_all(&dir);
}

/// The site's own project: every use found is at a line that names it,
/// and nothing it shows is counted unused that it names.
#[test]
fn over_the_site_every_use_is_where_it_says() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("site");
    let listing = assets::list(&root).unwrap();
    assert!(listing.assets.len() >= 20);
    let checked = uses_are_where_they_say(&root, &listing);
    assert!(checked >= 30, "{checked} uses");
    // A file the site names in any source is not unused.
    for a in listing.assets.iter().filter(|a| a.unused()) {
        let name = a.path.trim_start_matches('/');
        let named = Command::new("grep")
            .args(["-rlF", name, "src", "webfluent.app.json"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            named.stdout.is_empty(),
            "{} is counted unused, but {} name it",
            a.path,
            String::from_utf8_lossy(&named.stdout)
        );
    }
}

/// `Image(src: "/hero.png")` is the picture an `image` declaration makes:
/// every width, in every format, the box the right size — in the static
/// page and in the bundle — and the files are the ones the plan named.
#[test]
fn a_literal_image_src_is_processed_as_an_image_is() {
    let dir = project();
    put(
        &dir,
        "webfluent.app.json",
        r#"{ "name": "assets", "build": { "ssg": true, "base_path": "/Site" } }"#,
    );
    put(
        &dir,
        "src/pages/Home.wf",
        "page Home(path: \"/\", title: \"Home\", description: \"The home page.\") {\n    Heading(\"Home\").h1\n    Image(src: \"/hero.png?v=2\", alt: \"The harbour at dawn\", sizes: \"100vw\")\n    Image(src: \"/hero.png\", alt: \"Again\")\n    Image(src: \"/logo.svg\", alt: \"Halyard\")\n    Image(src: \"https://example.com/elsewhere.png\", alt: \"Elsewhere\")\n}\n",
    );
    let plan = assets::plan(&dir, "/hero.png").unwrap();
    let (_, before, _) = wf(&dir, &["check", "--format", "json"]);
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    // A build that checks adds no finding of its own.
    let (_, after, _) = wf(&dir, &["check", "--format", "json"]);
    assert_eq!(before, after);

    let mut written: Vec<String> = std::fs::read_dir(dir.join("build/img"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("hero."))
        .collect();
    written.sort();
    let mut named: Vec<String> = plan.made().map(|f| f.name.clone()).collect();
    named.push(plan.original.name.clone());
    named.sort();
    assert_eq!(written, named, "once for both, as the plan named them");

    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    let pictures = html.matches("<picture>").count();
    assert_eq!(pictures, 2, "{html}");
    let widest = plan.made().last().unwrap();
    assert!(
        html.contains(&format!("/Site/img/{} 1440w", widest.name)),
        "{html}"
    );
    assert!(html.contains("width=\"1440\" height=\"900\""), "{html}");
    assert!(html.contains("sizes=\"100vw\""));
    assert!(html.contains("alt=\"The harbour at dawn\""));
    // What the pipeline does not re-encode is as it was.
    assert!(
        html.contains("<img class=\"wf-image\" src=\"/logo.svg\""),
        "{html}"
    );
    assert!(html.contains("https://example.com/elsewhere.png"));
    let bundle = std::fs::read_to_string(dir.join("build/pages/Home.js"))
        .unwrap_or_else(|_| std::fs::read_to_string(dir.join("build/app.js")).unwrap());
    assert_eq!(bundle.matches("WF.picture(").count(), 2, "{bundle}");

    // Exact now: the build wrote what the plan estimated.
    assert!(!assets::plan(&dir, "/hero.png").unwrap().estimated());

    // The pipeline off, the file is copied and named as it was written.
    wf(&dir, &["config", "set", "build.media.pipeline", "false"]);
    let (code, _, err) = wf(&dir, &["build"]);
    assert_eq!(code, 0, "{err}");
    let html = std::fs::read_to_string(dir.join("build/index.html")).unwrap();
    assert_eq!(html.matches("<picture>").count(), 0, "{html}");
    assert!(html.contains("/hero.png"), "{html}");
    assert!(dir.join("build/hero.png").is_file());
    let _ = std::fs::remove_dir_all(&dir);
}
